//! The ports each project's processes listen on (#521).
//!
//! While a rail is open, Marley reads `/proc` every three seconds off the main thread
//! (`marley_browser::ports::listeners_in`) and gives each of the user's listeners to the project
//! group whose folder is the deepest one holding the process's working directory, across every
//! window. Marley's own listeners are left out: its MCP server, another Marley's, and a project
//! Chromium's relay, which runs Marley's own executable, all named `marley`, and a Chromium an
//! earlier build started, whose command line names Marley's `browser/` folder.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gpui::{App, AppContext as _, Global, Task};
use marley_browser::ports::{Listener, Stopped, listeners_in, stop_in};
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

/// A listener, with the project folder that holds its working directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectListener {
    /// The folder, one of the project's, that holds the process's working directory.
    pub folder: PathBuf,
    /// The listener, as the scan found it.
    pub listener: Listener,
}

/// The project groups' listeners, as the last scan found them.
#[derive(Debug, Default)]
pub struct Ports {
    by_group: HashMap<ProjectGroupKey, Vec<ProjectListener>>,
    /// The rails that are open, which keep the scan running.
    watchers: usize,
    scanning: bool,
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
            let started = Instant::now();
            let scanned = cx
                .background_spawn(futures::future::lazy(move |_| {
                    attribute(listeners_in(Path::new(PROC)), &folders)
                }))
                .await;
            let took = started.elapsed();
            // Reading the global leaves the rails alone; a mutable access tells them it changed,
            // so it happens only when it did.
            let watched = cx.update(|cx| {
                let known = cx.try_global::<Ports>();
                if known.is_none_or(|ports| ports.watchers == 0) {
                    cx.default_global::<Ports>().scanning = false;
                    return false;
                }
                let changed = matches!((&scanned, known), (Ok(by_group), Some(ports)) if *by_group != ports.by_group);
                match scanned {
                    Ok(by_group) if changed => cx.default_global::<Ports>().by_group = by_group,
                    Ok(_) => {}
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

/// Every window's project groups, each with the folders of all its member workspaces, local
/// ones only.
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
            match folders.iter_mut().find(|(key, _)| *key == group.key) {
                Some((_, known)) => known.extend(roots),
                None => folders.push((group.key.clone(), roots.collect())),
            }
        }
    }
    folders
}

/// Each listener of `scanned` given to the project group whose folder is the deepest holding the
/// process's working directory; Marley's own, and those in no project, left out.
fn attribute(
    scanned: anyhow::Result<Vec<Listener>>,
    folders: &[(ProjectGroupKey, Vec<PathBuf>)],
) -> anyhow::Result<HashMap<ProjectGroupKey, Vec<ProjectListener>>> {
    let own = std::process::id();
    let browser_dir = paths::data_dir().join("browser");
    let browser_dir = browser_dir.to_string_lossy();
    let mut by_group: HashMap<ProjectGroupKey, Vec<ProjectListener>> = HashMap::new();
    for listener in scanned? {
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
        let groups: Vec<_> = multi_workspace
            .project_groups(cx)
            .into_iter()
            .filter(|group| !group.workspaces.is_empty())
            .collect();
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

/// Stops the process `pid` that listens on `port`, after a scan made now finds it listening
/// there still.
pub fn stop(port: u16, pid: u32, cx: &App) -> Task<anyhow::Result<Stopped>> {
    cx.background_spawn(futures::future::lazy(move |_| {
        stop_in(Path::new(PROC), port, pid)
    }))
}
