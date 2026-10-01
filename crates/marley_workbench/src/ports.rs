//! The ports each project's processes listen on (#521).
//!
//! While a rail is open, Marley reads `/proc` every three seconds off the main thread
//! (`marley_browser::ports::listeners_in`) and gives each of the user's listeners to the project
//! group whose folder is the deepest one holding the process's working directory, across every
//! window. Marley's own listeners are left out: its MCP server, another Marley's, and a project
//! Chromium's relay, which runs Marley's own executable, all named `marley`, and a Chromium an
//! earlier build started, whose command line names Marley's `browser/` folder.
//!
//! Stop stops a listener's systemd service with `systemctl`, and signals any other process
//! (#603): a service's restart policy would start a signalled process again.
//!
//! A port a container publishes (#614) is found from `docker-proxy`'s command line or a rootless
//! Podman helper's socket, named by its container where the engine answers, given to the project
//! its Compose folder is in or else listed apart, and stopped with the engine's `stop`, never
//! through the engine's own systemd unit.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Context as _;
use gpui::{App, AppContext as _, Global, Task};
use marley_browser::containers::{
    self, Container, ENGINE_UNITS, Engine, HELPERS, ProxiedPort, proxied_ports_in,
};
use marley_browser::ports::{Listener, Service, Stopped, listeners_in, own_service_in, stop_in};
use project::ProjectGroupKey;
use workspace::MultiWorkspace;

/// Where the machine's processes are.
const PROC: &str = "/proc";

/// How often the rails' ports are read while one is open.
const SCAN_EVERY: Duration = Duration::from_secs(3);

/// How often after a scan that took longer than [`SLOW_SCAN`], on a machine with many processes.
const SCAN_SLOWLY: Duration = Duration::from_secs(30);

/// A scan past this reads the machine too hard to run every three seconds.
const SLOW_SCAN: Duration = Duration::from_millis(500);

/// How long an engine's list of containers is kept while its ports stay the same.
const ENGINE_KEPT: Duration = Duration::from_secs(30);

/// How long a restarted service's row is kept while its port does not listen again (#615).
const RESTART_KEPT: Duration = Duration::from_mins(10);

/// A listener, with the project folder that holds its working directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectListener {
    /// The folder, one of the project's, that holds the process's working directory.
    pub folder: PathBuf,
    /// The listener, as the scan found it.
    pub listener: Listener,
    /// The container that publishes the port, when a container does (#614).
    pub container: Option<ContainerRef>,
}

/// The container that publishes a port (#614).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerRef {
    /// Its engine.
    pub engine: Engine,
    /// Its name, when the engine said it.
    pub name: Option<String>,
    /// The container's address and port, when its proxy's command line says them.
    pub target: Option<String>,
    /// Why the engine did not say which container it is, when it did not.
    pub refusal: Option<String>,
}

/// What an engine last said about its containers, and for which of its ports.
#[derive(Debug, Clone)]
struct EngineAnswer {
    ports: Vec<u16>,
    asked: Instant,
    containers: Result<Vec<Container>, String>,
}

/// A systemd unit's state, as `systemctl show` gives it (#615).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitState {
    /// Its `ActiveState`: `active`, `activating`, `failed`, ….
    pub active: String,
    /// Its `SubState`: `running`, `auto-restart`, `dead`, ….
    pub sub: String,
}

/// A service the user restarted, whose port is kept as a row until it listens again (#615).
#[derive(Debug, Clone)]
struct Restarting {
    key: ProjectGroupKey,
    port: u16,
    service: Service,
    folder: PathBuf,
    since: Instant,
}

/// A port a container publishes, as the scan found it, before the engine names it.
#[derive(Debug, Clone)]
struct ContainerPort {
    engine: Engine,
    pid: u32,
    address: std::net::SocketAddr,
    target: Option<String>,
}

/// The project groups' listeners, as the last scan found them.
#[derive(Debug, Default)]
pub struct Ports {
    by_group: HashMap<ProjectGroupKey, Vec<ProjectListener>>,
    /// Container ports no project's folder holds (#614).
    containers: Vec<ProjectListener>,
    /// Each engine's last answer, asked again when its ports change or it grows old.
    engines: HashMap<Engine, EngineAnswer>,
    /// The state of each listed service's unit, by its name and whether it is the user's (#615).
    units: HashMap<(String, bool), UnitState>,
    /// The services restarted from a row, kept until their ports listen again (#615).
    restarting: Vec<Restarting>,
    /// The rails that are open, which keep the scan running.
    watchers: usize,
    scanning: bool,
    /// Where the scan reads the sockets in place of `/proc`: a test's folder, so a test never
    /// reads the machine's ports or asks its container engines.
    pub(crate) proc_root: Option<PathBuf>,
}

impl Global for Ports {}

impl Ports {
    /// The listeners of the project group `key`, by port.
    pub fn of(key: &ProjectGroupKey, cx: &App) -> Vec<ProjectListener> {
        cx.try_global::<Self>()
            .and_then(|ports| ports.by_group.get(key))
            .cloned()
            .unwrap_or_default()
    }

    /// The container ports no project's folder holds (#614).
    pub fn containers(cx: &App) -> Vec<ProjectListener> {
        cx.try_global::<Self>()
            .map(|ports| ports.containers.clone())
            .unwrap_or_default()
    }
}

/// Keeps the scan running for an open rail, which calls [`unwatch`] when it closes or goes.
pub fn watch(cx: &mut App) {
    let ports = cx.default_global::<Ports>();
    ports.watchers += 1;
    if !ports.scanning {
        ports.scanning = true;
        scan_while_watched(cx).detach();
    }
}

/// Lets the scan stop once no rail is open.
pub fn unwatch(cx: &mut App) {
    if cx.has_global::<Ports>() {
        let ports = cx.global_mut::<Ports>();
        ports.watchers = ports.watchers.saturating_sub(1);
    }
}

/// The scan's loop: read, give each listener its project, tell the rails when that changed, and
/// read again after [`SCAN_EVERY`], until no rail is open.
fn scan_while_watched(cx: &App) -> Task<()> {
    cx.spawn(async move |cx| {
        loop {
            let folders = cx.update(|cx| project_folders(cx));
            let proc_root = cx.update(|cx| {
                cx.try_global::<Ports>()
                    .and_then(|ports| ports.proc_root.clone())
                    .unwrap_or_else(|| PathBuf::from(PROC))
            });
            let started = Instant::now();
            let (listened, proxied) = cx
                .background_spawn(futures::future::lazy(move |_| {
                    (listeners_in(&proc_root), proxied_ports_in(&proc_root))
                }))
                .await;
            let took = started.elapsed();
            let found = container_ports(listened.as_deref().unwrap_or_default(), &proxied);
            let known = cx.update(|cx| {
                cx.try_global::<Ports>()
                    .map(|ports| ports.engines.clone())
                    .unwrap_or_default()
            });
            let engines = ask_engines(&found, known).await;
            let scanned = attribute(listened, &folders).map(|mut by_group| {
                let apart = attribute_containers(found, &engines, &folders, &mut by_group);
                (by_group, apart)
            });
            let restarting = cx.update(|cx| {
                cx.try_global::<Ports>()
                    .map(|ports| ports.restarting.clone())
                    .unwrap_or_default()
            });
            let mut services: Vec<(String, bool)> = scanned
                .iter()
                .flat_map(|(by_group, _)| by_group.values().flatten())
                .filter_map(|found| found.listener.service.as_ref())
                .chain(restarting.iter().map(|kept| &kept.service))
                .map(|service| (service.unit.clone(), service.user))
                .collect();
            services.sort();
            services.dedup();
            let units = unit_states(&services).await;
            let (scanned, restarting) = match scanned {
                Ok((mut by_group, apart)) => {
                    let restarting = keep_restarting(restarting, &mut by_group, &units);
                    (Ok((by_group, apart)), restarting)
                }
                Err(error) => (Err(error), restarting),
            };
            // Reading the global leaves the rails alone; a mutable access tells them it changed,
            // so it happens only when it did.
            let watched = cx.update(|cx| {
                let known = cx.try_global::<Ports>();
                if known.is_none_or(|ports| ports.watchers == 0) {
                    cx.default_global::<Ports>().scanning = false;
                    return false;
                }
                let changed = matches!((&scanned, known), (Ok((by_group, apart)), Some(ports)) if *by_group != ports.by_group || *apart != ports.containers || units != ports.units);
                match scanned {
                    Ok((by_group, apart)) if changed => {
                        let ports = cx.default_global::<Ports>();
                        ports.by_group = by_group;
                        ports.containers = apart;
                        ports.engines = engines;
                        ports.units = units;
                        ports.restarting = restarting;
                    }
                    Ok(_) => {
                        // The engines' answers carry their age, which is not news to the rails.
                        if cx.has_global::<Ports>() {
                            let ports = cx.global_mut::<Ports>();
                            ports.engines = engines;
                            ports.restarting = restarting;
                        }
                    }
                    Err(error) => log::warn!("ports: reading the listening ports: {error:#}"),
                }
                true
            });
            if !watched {
                break;
            }
            let wait = if took > SLOW_SCAN {
                log::info!("ports: a scan took {took:?}; the next in {SCAN_SLOWLY:?}");
                SCAN_SLOWLY
            } else {
                SCAN_EVERY
            };
            cx.background_executor().timer(wait).await;
        }
    })
}

/// The states of `services`' units: one `systemctl show` for the user's and one for the
/// system's, whose blocks come in the order the units were asked (#615). A unit `systemctl` does
/// not answer for has no state.
async fn unit_states(services: &[(String, bool)]) -> HashMap<(String, bool), UnitState> {
    let mut states = HashMap::new();
    for user in [true, false] {
        let units: Vec<&str> = services
            .iter()
            .filter(|(_, of_user)| *of_user == user)
            .map(|(unit, _)| unit.as_str())
            .collect();
        if units.is_empty() {
            continue;
        }
        let mut args = vec!["show", "-p", "ActiveState,SubState"];
        if user {
            args.insert(0, "--user");
        }
        args.extend(&units);
        let output = match crate::process::output("systemctl", &args, None, &[]).await {
            Ok(output) => output,
            Err(error) => {
                log::debug!("ports: asking systemctl for the units' states: {error}");
                continue;
            }
        };
        let text = String::from_utf8_lossy(&output.stdout);
        for (unit, block) in units.iter().zip(text.split("\n\n")) {
            let field = |name: &str| {
                block
                    .lines()
                    .find_map(|line| line.strip_prefix(name))
                    .unwrap_or_default()
                    .to_string()
            };
            states.insert(
                ((*unit).to_string(), user),
                UnitState {
                    active: field("ActiveState="),
                    sub: field("SubState="),
                },
            );
        }
    }
    states
}

/// The restarted services still kept: each whose project has no listener on its port yet gets a
/// row in `by_group` (its pid 0, its unit as its name), until the port listens again, its unit
/// stops, or [`RESTART_KEPT`] passes.
fn keep_restarting(
    restarting: Vec<Restarting>,
    by_group: &mut HashMap<ProjectGroupKey, Vec<ProjectListener>>,
    units: &HashMap<(String, bool), UnitState>,
) -> Vec<Restarting> {
    let kept: Vec<Restarting> = restarting
        .into_iter()
        .filter(|kept| {
            let listens = by_group.get(&kept.key).is_some_and(|found| {
                found
                    .iter()
                    .any(|found| found.listener.address.port() == kept.port)
            });
            let stopped = units
                .get(&(kept.service.unit.clone(), kept.service.user))
                .is_some_and(|state| state.active == "inactive");
            !listens && !stopped && kept.since.elapsed() < RESTART_KEPT
        })
        .collect();
    for kept in &kept {
        by_group
            .entry(kept.key.clone())
            .or_default()
            .push(ProjectListener {
                folder: kept.folder.clone(),
                listener: Listener {
                    address: std::net::SocketAddr::from(([127, 0, 0, 1], kept.port)),
                    pid: 0,
                    name: kept.service.unit.clone(),
                    command: format!("systemctl restart {}", kept.service.unit),
                    cwd: kept.folder.clone(),
                    service: Some(kept.service.clone()),
                },
                container: None,
            });
    }
    kept
}

/// The word a service's row shows for its unit's state, or none while it simply runs (#615).
/// `kept` is a row for a restarted service whose port does not listen yet.
pub fn unit_word(service: &Service, kept: bool, cx: &App) -> Option<String> {
    let state = cx
        .try_global::<Ports>()
        .and_then(|ports| ports.units.get(&(service.unit.clone(), service.user)));
    let word = match state.map(|state| (state.active.as_str(), state.sub.as_str())) {
        Some((_, "auto-restart")) => Some("restarting"),
        Some(("active", _)) if kept => Some("starting"),
        Some(("activating", _)) => Some("starting"),
        Some(("deactivating", _)) => Some("stopping"),
        Some(("failed", _)) => Some("failed"),
        Some(("inactive", _)) => Some("stopped"),
        None if kept => Some("restarting"),
        _ => None,
    };
    word.map(str::to_string)
}

/// The ports containers publish: each `docker-proxy`'s, from its command line, and each port a
/// rootless Podman helper of the user's listens on.
fn container_ports(listened: &[Listener], proxied: &[ProxiedPort]) -> Vec<ContainerPort> {
    let docker = proxied.iter().map(|port| ContainerPort {
        engine: Engine::Docker,
        pid: port.pid,
        address: port.address,
        target: Some(port.target.clone()),
    });
    let podman = listened.iter().filter_map(|listener| {
        // A `docker-proxy` the user can read is the same port as its command line's.
        let engine =
            Engine::of_helper(&listener.name).filter(|engine| *engine == Engine::Podman)?;
        Some(ContainerPort {
            engine,
            pid: listener.pid,
            address: listener.address,
            target: None,
        })
    });
    let mut ports: Vec<ContainerPort> = docker.chain(podman).collect();
    ports.sort_by_key(|port| (port.address.port(), port.pid));
    ports.dedup_by_key(|port| port.address.port());
    ports
}

/// Each engine with ports in `found` asked for its containers, unless its answer in `known` is
/// for the same ports and young enough.
async fn ask_engines(
    found: &[ContainerPort],
    known: HashMap<Engine, EngineAnswer>,
) -> HashMap<Engine, EngineAnswer> {
    let mut answers = HashMap::new();
    for engine in [Engine::Docker, Engine::Podman] {
        let mut ports: Vec<u16> = found
            .iter()
            .filter(|port| port.engine == engine)
            .map(|port| port.address.port())
            .collect();
        if ports.is_empty() {
            continue;
        }
        ports.sort_unstable();
        let kept = known
            .get(&engine)
            .filter(|answer| answer.ports == ports && answer.asked.elapsed() < ENGINE_KEPT);
        let answer = match kept {
            Some(answer) => answer.clone(),
            None => EngineAnswer {
                ports,
                asked: Instant::now(),
                containers: ask_engine(engine).await,
            },
        };
        answers.insert(engine, answer);
    }
    answers
}

/// The engine's containers, or why it would not list them: its error's last line.
async fn ask_engine(engine: Engine) -> Result<Vec<Container>, String> {
    let args: &[&str] = match engine {
        Engine::Docker => &["ps", "--format", "{{json .}}"],
        Engine::Podman => &["ps", "--format", "json"],
    };
    let command = engine.command();
    let output = crate::process::output(command, args, None, &[])
        .await
        .map_err(|error| format!("`{command}` could not start: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr
            .lines()
            .map(str::trim)
            .rfind(|line| !line.is_empty())
            .map_or_else(
                || format!("`{command} ps` ended with {}", output.status),
                str::to_string,
            ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(match engine {
        Engine::Docker => containers::parse_docker_ps(&stdout),
        Engine::Podman => containers::parse_podman_ps(&stdout),
    })
}

/// Each container port given to the project whose folder holds its Compose folder, into
/// `by_group`; the rest returned, to be listed apart.
fn attribute_containers(
    found: Vec<ContainerPort>,
    engines: &HashMap<Engine, EngineAnswer>,
    folders: &[(ProjectGroupKey, Vec<PathBuf>)],
    by_group: &mut HashMap<ProjectGroupKey, Vec<ProjectListener>>,
) -> Vec<ProjectListener> {
    let mut apart = Vec::new();
    for port in found {
        let answer = engines.get(&port.engine).map(|answer| &answer.containers);
        let container = answer.and_then(|answer| {
            answer
                .as_ref()
                .ok()?
                .iter()
                .find(|container| container.host_ports.contains(&port.address.port()))
        });
        let refusal = answer.and_then(|answer| answer.as_ref().err().cloned());
        let name = container.map(|container| container.name.clone());
        let working_dir = container.and_then(|container| container.working_dir.clone());
        let command = match (&name, &port.target) {
            (Some(name), _) => format!("{} container {name}", port.engine.command()),
            (None, Some(target)) => format!("docker-proxy → {target}"),
            (None, None) => format!("a {} container", port.engine.command()),
        };
        let listed = ProjectListener {
            folder: working_dir.clone().unwrap_or_default(),
            listener: Listener {
                address: port.address,
                pid: port.pid,
                name: name
                    .clone()
                    .unwrap_or_else(|| port.engine.command().to_string()),
                command,
                cwd: working_dir.clone().unwrap_or_default(),
                service: None,
            },
            container: Some(ContainerRef {
                engine: port.engine,
                name,
                target: port.target,
                refusal,
            }),
        };
        let home = working_dir.and_then(|folder| deepest(folders, &folder));
        match home {
            Some((key, root)) => by_group.entry(key).or_default().push(ProjectListener {
                folder: root,
                ..listed
            }),
            None => apart.push(listed),
        }
    }
    apart
}

/// The project group whose folder is the deepest holding `path`, and that folder.
fn deepest(
    folders: &[(ProjectGroupKey, Vec<PathBuf>)],
    path: &Path,
) -> Option<(ProjectGroupKey, PathBuf)> {
    folders
        .iter()
        .flat_map(|(key, roots)| roots.iter().map(move |root| (key, root)))
        .filter(|(_, root)| path.starts_with(root))
        .max_by_key(|(_, root)| root.components().count())
        .map(|(key, root)| (key.clone(), root.clone()))
}

/// Every window's project groups, each with the folders of all its member workspaces, local
/// ones only. A local project the window lists closed (#606) has its key's folders (#617).
fn project_folders(cx: &App) -> Vec<(ProjectGroupKey, Vec<PathBuf>)> {
    let mut folders: Vec<(ProjectGroupKey, Vec<PathBuf>)> = Vec::new();
    for window in cx.windows() {
        let Some(multi_workspace) = window
            .downcast::<MultiWorkspace>()
            .and_then(|window| window.read(cx).ok())
        else {
            continue;
        };
        for group in multi_workspace.project_groups(cx) {
            let closed = if group.workspaces.is_empty() && group.key.host().is_none() {
                group.key.path_list().paths().to_vec()
            } else {
                Vec::new()
            };
            let roots = group.workspaces.iter().flat_map(|workspace| {
                let workspace = workspace.read(cx);
                if workspace.project().read(cx).is_local() {
                    workspace
                        .root_paths(cx)
                        .into_iter()
                        .map(|root| root.to_path_buf())
                        .collect()
                } else {
                    Vec::new()
                }
            });
            let roots = roots.chain(closed);
            match folders.iter_mut().find(|(key, _)| *key == group.key) {
                Some((_, known)) => known.extend(roots),
                None => folders.push((group.key.clone(), roots.collect())),
            }
        }
    }
    folders
}

/// Each listener of `scanned` given to the project group whose folder is the deepest holding the
/// process's working directory; Marley's own, and those in no project, left out. A listener in
/// the unit Marley itself runs in, a dev server in one of its terminals, is a process: stopping
/// the unit would stop Marley.
fn attribute(
    scanned: anyhow::Result<Vec<Listener>>,
    folders: &[(ProjectGroupKey, Vec<PathBuf>)],
) -> anyhow::Result<HashMap<ProjectGroupKey, Vec<ProjectListener>>> {
    let own = std::process::id();
    let own_service = own_service_in(Path::new(PROC));
    let browser_dir = paths::data_dir().join("browser");
    let browser_dir = browser_dir.to_string_lossy();
    let mut by_group: HashMap<ProjectGroupKey, Vec<ProjectListener>> = HashMap::new();
    for mut listener in scanned? {
        if listener.service.is_some() && listener.service == own_service {
            listener.service = None;
        }
        // A container's port is its container's (#614), and an engine's own unit is never what
        // a port's Stop stops.
        if HELPERS.contains(&listener.name.as_str()) {
            continue;
        }
        if listener
            .service
            .as_ref()
            .is_some_and(|service| ENGINE_UNITS.contains(&service.unit.as_str()))
        {
            listener.service = None;
        }
        if listener.pid == own
            || listener.name == "marley"
            || listener.command.contains(browser_dir.as_ref())
        {
            continue;
        }
        let deepest = folders
            .iter()
            .flat_map(|(key, roots)| roots.iter().map(move |root| (key, root)))
            .filter(|(_, root)| listener.cwd.starts_with(root))
            .fold(
                None,
                |best: Option<(&ProjectGroupKey, &PathBuf)>, candidate| match best {
                    Some(best)
                        if best.1.components().count() >= candidate.1.components().count() =>
                    {
                        Some(best)
                    }
                    _ => Some(candidate),
                },
            );
        if let Some((key, folder)) = deepest {
            by_group
                .entry(key.clone())
                .or_default()
                .push(ProjectListener {
                    folder: folder.clone(),
                    listener,
                    container: None,
                });
        }
    }
    Ok(by_group)
}

/// Each project group's name as the rail shows it, in every window.
pub fn project_names(cx: &App) -> HashMap<ProjectGroupKey, String> {
    let mut names = HashMap::new();
    for window in cx.windows() {
        let Some(multi_workspace) = window
            .downcast::<MultiWorkspace>()
            .and_then(|window| window.read(cx).ok())
        else {
            continue;
        };
        // A closed project's ports are listed too (#617), under the name its header shows.
        let groups = multi_workspace.project_groups(cx);
        for (group, name) in groups.iter().zip(crate::group_names(&groups)) {
            names.entry(group.key.clone()).or_insert(name);
        }
    }
    names
}

/// Every project group's listeners from a scan made now, for `ports_list`.
pub fn list(cx: &App) -> Task<anyhow::Result<HashMap<ProjectGroupKey, Vec<ProjectListener>>>> {
    let folders = project_folders(cx);
    cx.background_spawn(futures::future::lazy(move |_| {
        attribute(listeners_in(Path::new(PROC)), &folders)
    }))
}

/// How a Stop went (#603).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stop {
    /// The listener runs in no service, and was signalled as #521 does.
    Signalled(Stopped),
    /// `systemctl` stopped the listener's service.
    Unit(Service),
    /// `systemctl` could not stop it, for systemd's `reason`.
    Refused {
        /// The service.
        service: Service,
        /// What `systemctl` printed.
        reason: String,
    },
}

/// Stops what listens on `port` as `pid`, after a scan made now finds it listening there still.
///
/// A listener in a systemd service has its service stopped with `systemctl` (the system's through
/// polkit, which asks the desktop's agent); any other process gets SIGTERM.
pub fn stop(port: u16, pid: u32, cx: &App) -> Task<anyhow::Result<Stop>> {
    cx.background_spawn(async move {
        let proc_root = Path::new(PROC);
        let found = listeners_in(proc_root)?
            .into_iter()
            .find(|listener| listener.pid == pid && listener.address.port() == port);
        let Some(found) = found else {
            return Ok(Stop::Signalled(Stopped::NotListening));
        };
        let own_service = own_service_in(proc_root);
        match found.service.filter(|service| {
            Some(service) != own_service.as_ref() && !ENGINE_UNITS.contains(&service.unit.as_str())
        }) {
            Some(service) => stop_unit(service).await,
            None => Ok(Stop::Signalled(stop_in(proc_root, port, pid)?)),
        }
    })
}

/// Runs `systemctl stop` for `service`, with `--user` for the user's own.
async fn stop_unit(service: Service) -> anyhow::Result<Stop> {
    let mut args = vec!["stop", service.unit.as_str()];
    if service.user {
        args.insert(0, "--user");
    }
    let output = crate::process::output("systemctl", &args, None, &[])
        .await
        .context("running `systemctl`")?;
    if output.status.success() {
        return Ok(Stop::Unit(service));
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let reason = refusal_reason(&stderr, &service.unit)
        .unwrap_or_else(|| format!("`systemctl` ended with {}", output.status));
    Ok(Stop::Refused { service, reason })
}

/// systemd's reason in what `systemctl` printed: its first line, after the `Failed to stop
/// <unit>: ` it starts with, without the pointer to the logs that follows.
fn refusal_reason(stderr: &str, unit: &str) -> Option<String> {
    let line = stderr
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?;
    let reason = line
        .split_once(&format!("{unit}: "))
        .map_or(line, |(_, reason)| reason);
    Some(reason.trim_end_matches('.').to_string())
}

/// The container that publishes `port`, under a project or apart, as the last scan found it.
pub fn container_at(port: u16, cx: &App) -> Option<ContainerRef> {
    let ports = cx.try_global::<Ports>()?;
    ports
        .by_group
        .values()
        .flatten()
        .chain(&ports.containers)
        .find(|found| found.listener.address.port() == port && found.container.is_some())
        .and_then(|found| found.container.clone())
}

/// How a service's Restart went (#615).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceAction {
    /// `systemctl` did it.
    Done,
    /// It could not, for `reason`; `command` does it by hand.
    Refused {
        /// The command to copy.
        command: String,
        /// Why.
        reason: String,
    },
}

/// Keeps a row for the service that listens on `port` in the project `key` while it restarts, and
/// gives the service; none when no service listens there.
pub fn begin_restart(key: &ProjectGroupKey, port: u16, cx: &mut App) -> Option<Service> {
    let found = Ports::of(key, cx)
        .into_iter()
        .find(|found| found.listener.address.port() == port)?;
    let service = found.listener.service?;
    let ports = cx.default_global::<Ports>();
    ports
        .restarting
        .retain(|kept| !(kept.key == *key && kept.port == port));
    ports.restarting.push(Restarting {
        key: key.clone(),
        port,
        service: service.clone(),
        folder: found.folder,
        since: Instant::now(),
    });
    Some(service)
}

/// Restarts `service` with `systemctl`, with `--user` for the user's own. A container engine's
/// own unit is refused, as Stop refuses it.
pub fn restart_service(service: Service, cx: &App) -> Task<anyhow::Result<ServiceAction>> {
    let command = if service.user {
        format!("systemctl --user restart {}", service.unit)
    } else {
        format!("sudo systemctl restart {}", service.unit)
    };
    if ENGINE_UNITS.contains(&service.unit.as_str()) {
        return Task::ready(Ok(ServiceAction::Refused {
            command,
            reason: "Marley does not restart a container engine from a port".to_string(),
        }));
    }
    cx.background_spawn(async move {
        let mut args = vec!["restart", service.unit.as_str()];
        if service.user {
            args.insert(0, "--user");
        }
        let output = crate::process::output("systemctl", &args, None, &[])
            .await
            .context("running `systemctl`")?;
        if output.status.success() {
            return Ok(ServiceAction::Done);
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        let reason = refusal_reason(&stderr, &service.unit)
            .unwrap_or_else(|| format!("`systemctl` ended with {}", output.status));
        Ok(ServiceAction::Refused { command, reason })
    })
}

/// Stops `service` itself, for a row whose process is gone, such as a restarted one's (#615).
pub fn stop_service(service: Service, cx: &App) -> Task<anyhow::Result<Stop>> {
    if ENGINE_UNITS.contains(&service.unit.as_str()) {
        return Task::ready(Ok(Stop::Refused {
            service,
            reason: "Marley does not stop a container engine from a port".to_string(),
        }));
    }
    cx.background_spawn(stop_unit(service))
}

/// How a container's Stop went (#614).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerStop {
    /// The engine stopped it.
    Stopped,
    /// It could not be stopped, for `reason`; `command` stops it by hand.
    Refused {
        /// The command to copy.
        command: String,
        /// Why.
        reason: String,
    },
}

/// Stops the container that publishes `port`, with its engine's `stop`. A container the engine did
/// not name is refused at once, with a command that finds it by its port.
pub fn stop_container(container: &ContainerRef, port: u16, cx: &App) -> Task<ContainerStop> {
    let engine = container.engine.command();
    let Some(name) = container.name.clone() else {
        let reason = container
            .refusal
            .clone()
            .unwrap_or_else(|| format!("`{engine}` did not say which container it is"));
        return Task::ready(ContainerStop::Refused {
            command: format!("{engine} stop $({engine} ps -q --filter publish={port})"),
            reason,
        });
    };
    cx.background_spawn(async move {
        let command = format!("{engine} stop {name}");
        match crate::process::output(engine, ["stop", name.as_str()], None, &[]).await {
            Ok(output) if output.status.success() => ContainerStop::Stopped,
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let reason = stderr
                    .lines()
                    .map(str::trim)
                    .rfind(|line| !line.is_empty())
                    .map_or_else(
                        || format!("`{command}` ended with {}", output.status),
                        str::to_string,
                    );
                ContainerStop::Refused { command, reason }
            }
            Err(error) => ContainerStop::Refused {
                reason: format!("`{engine}` could not start: {error}"),
                command,
            },
        }
    })
}

/// The command that stops `service` by hand, which a refusal offers to copy.
#[must_use]
pub fn hand_command(service: &Service) -> String {
    if service.user {
        format!("systemctl --user stop {}", service.unit)
    } else {
        format!("sudo systemctl stop {}", service.unit)
    }
}
