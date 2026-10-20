//! The tmux sessions on the hosts `marley.tmux_hosts` names, in the rail (#808).
//!
//! A host is `local`, this machine, or an `ssh_connections` entry by its nickname or host. Every
//! ten seconds Marley lists the panes of each host's default tmux server, here or behind a
//! `BatchMode` ssh, and the rail shows each session as a row under the project whose folder holds
//! the folder it started in, or under Home (#821). A click on a session attaches it in a task
//! terminal of that project. On a host the attach runs behind `ssh -t` under the remote terminal's
//! id base, so a dropped link is checked and the session attached again (#641); in a remote
//! project on that host it runs there as the project's own task. Closing an attach's tab asks
//! whether to end the session or only hide it (`close_guard`).

use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::Duration;

use gpui::{App, AsyncApp, BackgroundExecutor, Context, Global, SharedString, Task, Window};
use marley_agent::AgentKind;
use marley_remote::SessionName;
use settings::SettingsStore;
use workspace::Workspace;
use workspace::notifications::DetachAndPromptErr as _;

use crate::harness_hosts::shell_word;
use crate::remote::Host;

/// The name `marley.tmux_hosts` gives this machine.
const LOCAL: &str = "local";

/// How often each host's sessions are listed.
const LIST_EVERY: Duration = Duration::from_secs(10);

/// How long a listing may take before its host shows that it did not answer.
const LIST_TIMEOUT: Duration = Duration::from_secs(20);

/// How long after a + its host is listed again, so the new session's row comes before the next
/// listing.
const LIST_AFTER_NEW: Duration = Duration::from_secs(1);

/// What `list-panes` prints for each pane: its session's name, windows and clients, the pane's
/// foreground program and the folder the session started in, a tab between each.
const PANE_FORMAT: &str = "#{session_name}\t#{session_windows}\t#{session_attached}\t#{pane_current_command}\t#{session_path}";

/// The id base of a tmux attach on this machine.
const LOCAL_ID_BASE: &str = "marley-tmux";

/// One tmux session, as its host's listing gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TmuxSession {
    pub(crate) name: String,
    pub(crate) windows: u32,
    /// Whether a client has it attached.
    pub(crate) attached: bool,
    /// The agent CLI one of its panes runs.
    pub(crate) agent: Option<AgentKind>,
    /// The folder it started in, which places its row under a project (#821).
    pub(crate) folder: PathBuf,
}

/// What Marley last read of a host's sessions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Listing {
    /// The first listing has not come back.
    Waiting,
    Sessions(Vec<TmuxSession>),
    /// Why the host could not be listed.
    Failed(String),
}

/// Where a named host's tmux runs.
#[derive(Clone)]
enum Place {
    Here,
    Host(Host),
    /// The name matches no saved host.
    Unknown,
}

impl Place {
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Here, Self::Here) | (Self::Unknown, Self::Unknown) => true,
            (Self::Host(one), Self::Host(other)) => {
                one.label == other.label && one.target == other.target
            }
            _ => false,
        }
    }
}

/// A host `marley.tmux_hosts` names, and what the rail shows of it.
pub(crate) struct TmuxHost {
    /// The name as the setting gives it.
    pub(crate) name: SharedString,
    place: Place,
    pub(crate) listing: Listing,
}

impl TmuxHost {
    /// The name its heading and its tabs show: `this machine`, or the saved host's.
    pub(crate) fn label(&self) -> SharedString {
        match &self.place {
            Place::Here => SharedString::new_static("this machine"),
            Place::Host(host) => host.label.clone(),
            Place::Unknown => self.name.clone(),
        }
    }

    /// Whether its sessions are this machine's.
    pub(crate) const fn is_here(&self) -> bool {
        matches!(self.place, Place::Here)
    }

    /// Whether `connection` (a remote project's) reaches this host: an ssh link to the host its
    /// saved entry names.
    pub(crate) fn reached_by(&self, connection_type: &str, connection_host: &str) -> bool {
        match &self.place {
            Place::Host(host) => connection_type == "ssh" && host.target.host == connection_host,
            Place::Here | Place::Unknown => false,
        }
    }
}

/// A tab Marley opened attached to a session: its task's label, the host's name and the session's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Attach {
    pub(crate) label: String,
    pub(crate) host: SharedString,
    pub(crate) session: String,
}

/// The hosts the rail lists tmux sessions for, in the setting's order, and the listings' tasks.
#[derive(Default)]
pub(crate) struct TmuxHosts {
    pub(crate) hosts: Vec<TmuxHost>,
    /// The attaches Marley opened, by their tasks' labels, which tell a tab's terminal as a
    /// session's (#821).
    pub(crate) attaches: Vec<Attach>,
    /// The listing every [`LIST_EVERY`], while a host is named.
    poll: Option<Task<()>>,
    /// The listing a + asks for.
    soon: Option<Task<()>>,
}

impl Global for TmuxHosts {}

impl TmuxHosts {
    /// The session a task labelled `label` attaches, if Marley opened it.
    pub(crate) fn attach_of<'a>(&'a self, label: &str) -> Option<&'a Attach> {
        self.attaches.iter().find(|attach| attach.label == label)
    }

    fn remember(attach: Attach, cx: &mut App) {
        let attaches = &mut cx.global_mut::<Self>().attaches;
        if !attaches.contains(&attach) {
            attaches.push(attach);
        }
    }
}

/// Follows `marley.tmux_hosts`, now and on every settings change.
pub fn init(cx: &mut App) {
    cx.set_global(TmuxHosts::default());
    follow(cx);
    cx.observe_global::<SettingsStore>(follow).detach();
}

/// Whether tmux is on the search path Marley finds agent CLIs on (#819).
pub(crate) fn installed(cx: &App) -> bool {
    let search_path = crate::agents::launcher(cx).search_path;
    which::which_in("tmux", search_path.as_deref(), "/").is_ok()
}

/// Reads the named hosts: one whose name and place are unchanged keeps its listing and fold, and
/// the poll runs only while a host is named. An unset list names this machine while tmux is
/// installed (#819); a list the user wrote, an empty one too, is kept.
fn follow(cx: &mut App) {
    let set = cx
        .global::<SettingsStore>()
        .merged_settings()
        .marley
        .as_ref()
        .and_then(|marley| marley.tmux_hosts.clone());
    let names = set.unwrap_or_else(|| {
        if installed(cx) {
            vec![LOCAL.to_string()]
        } else {
            Vec::new()
        }
    });
    let saved = crate::remote::saved_hosts(cx);
    let mut wanted: Vec<(SharedString, Place)> = Vec::new();
    for name in &names {
        let name = name.trim();
        if name.is_empty() || wanted.iter().any(|(known, _)| known == name) {
            continue;
        }
        wanted.push((SharedString::from(name.to_string()), place_of(name, &saved)));
    }
    let tmux = cx.global::<TmuxHosts>();
    let unchanged = tmux.hosts.len() == wanted.len()
        && tmux
            .hosts
            .iter()
            .zip(&wanted)
            .all(|(host, (name, place))| &host.name == name && host.place.same(place));
    if unchanged {
        return;
    }
    let mut kept = std::mem::take(&mut cx.global_mut::<TmuxHosts>().hosts);
    let hosts: Vec<TmuxHost> = wanted
        .into_iter()
        .map(|(name, place)| {
            if let Some(index) = kept
                .iter()
                .position(|old| old.name == name && old.place.same(&place))
            {
                return kept.remove(index);
            }
            let listing = match &place {
                Place::Unknown => Listing::Failed(format!("no saved host is named {name}")),
                Place::Here | Place::Host(_) => Listing::Waiting,
            };
            TmuxHost {
                name,
                place,
                listing,
            }
        })
        .collect();
    let poll = (!hosts.is_empty() && cx.global::<TmuxHosts>().poll.is_none()).then(|| {
        cx.spawn(async move |cx| {
            loop {
                list_all(cx).await;
                cx.background_executor().timer(LIST_EVERY).await;
            }
        })
    });
    let tmux = cx.global_mut::<TmuxHosts>();
    if hosts.is_empty() {
        tmux.poll = None;
        tmux.soon = None;
    } else if poll.is_some() {
        tmux.poll = poll;
    }
    tmux.hosts = hosts;
}

/// Where `name` is: this machine for `local`, else the saved host whose nickname, then whose
/// host, it is.
fn place_of(name: &str, saved: &[Host]) -> Place {
    if name == LOCAL {
        return Place::Here;
    }
    saved
        .iter()
        .find(|host| host.label == name)
        .or_else(|| saved.iter().find(|host| host.target.host == name))
        .map_or(Place::Unknown, |host| Place::Host(host.clone()))
}

/// Lists every named host at once, and keeps each listing that changed.
async fn list_all(cx: &AsyncApp) {
    let wanted: Vec<(SharedString, Place)> = cx.update(|cx| {
        cx.global::<TmuxHosts>()
            .hosts
            .iter()
            .map(|host| (host.name.clone(), host.place.clone()))
            .collect()
    });
    let executor = cx.background_executor().clone();
    let listings =
        futures::future::join_all(wanted.iter().map(|(_, place)| list(place, &executor))).await;
    // A host the settings moved while its listing ran keeps what it has.
    let listed_at = |host: &TmuxHost, name: &SharedString, place: &Place| {
        &host.name == name && host.place.same(place)
    };
    cx.update(|cx| {
        let changed: Vec<(SharedString, Place, Listing)> = wanted
            .into_iter()
            .zip(listings)
            .filter_map(|((name, place), listing)| Some((name, place, listing?)))
            .filter(|(name, place, listing)| {
                cx.global::<TmuxHosts>()
                    .hosts
                    .iter()
                    .any(|host| listed_at(host, name, place) && &host.listing != listing)
            })
            .collect();
        if changed.is_empty() {
            return;
        }
        let tmux = cx.global_mut::<TmuxHosts>();
        for (name, place, listing) in changed {
            if let Some(host) = tmux
                .hosts
                .iter_mut()
                .find(|host| listed_at(host, &name, &place))
            {
                host.listing = listing;
            }
        }
    });
}

/// Lists the sessions of `place`'s default tmux server; none for a name no saved host has.
async fn list(place: &Place, executor: &BackgroundExecutor) -> Option<Listing> {
    let list_words = ["tmux", "list-panes", "-a", "-F", PANE_FORMAT];
    let (program, args): (String, Vec<String>) = match place {
        Place::Unknown => return None,
        Place::Here => (
            "tmux".to_string(),
            list_words
                .iter()
                .skip(1)
                .map(|word| (*word).to_string())
                .collect(),
        ),
        Place::Host(host) => {
            let mut args = vec![
                "-o".to_string(),
                format!("ConnectTimeout={}", marley_remote::CONNECT_TIMEOUT_S),
            ];
            args.extend(crate::harness_hosts::ssh_arguments(&host.target));
            args.extend(list_words.iter().map(|word| shell_word(word)));
            // A scenario's fake stands in for ssh, as it does in the remote terminal's task.
            let ssh = std::env::var("MARLEY_SSH").unwrap_or_else(|_| "ssh".to_string());
            (ssh, args)
        }
    };
    let run = crate::process::output(&program, &args, None, &[]);
    let timer = executor.timer(LIST_TIMEOUT);
    let listing = match futures::future::select(Box::pin(run), timer).await {
        futures::future::Either::Left((Ok(output), _)) => read_listing(&output),
        futures::future::Either::Left((Err(error), _)) => {
            Listing::Failed(format!("{program} did not run: {error}"))
        }
        futures::future::Either::Right(_) => Listing::Failed(format!(
            "did not answer within {} s",
            LIST_TIMEOUT.as_secs()
        )),
    };
    Some(listing)
}

/// What a listing's run says: its sessions, none when no tmux server runs, or the first line it
/// printed on a failure.
fn read_listing(output: &Output) -> Listing {
    let stderr = String::from_utf8_lossy(&output.stderr);
    if output.status.success() {
        return Listing::Sessions(parse(&String::from_utf8_lossy(&output.stdout)));
    }
    if stderr.contains("no server running") || stderr.contains("error connecting to") {
        return Listing::Sessions(Vec::new());
    }
    let line = stderr.lines().map(str::trim).find(|line| !line.is_empty());
    Listing::Failed(line.map_or_else(
        || format!("tmux ended with {}", output.status),
        str::to_string,
    ))
}

/// The sessions in [`PANE_FORMAT`]'s lines, in tmux's order; a session is an agent's when one of
/// its panes runs an agent CLI.
fn parse(stdout: &str) -> Vec<TmuxSession> {
    let mut sessions: Vec<TmuxSession> = Vec::new();
    for line in stdout.lines() {
        let mut fields = line.splitn(5, '\t');
        let (Some(name), Some(windows), Some(attached), Some(command), Some(folder)) = (
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
        ) else {
            continue;
        };
        let agent = marley_agent::agent_kind_of(command);
        if let Some(session) = sessions.iter_mut().find(|session| session.name == name) {
            session.agent = session.agent.or(agent);
            continue;
        }
        sessions.push(TmuxSession {
            name: name.to_string(),
            windows: windows.parse().unwrap_or_default(),
            attached: attached.parse::<u32>().is_ok_and(|clients| clients > 0),
            agent,
            folder: PathBuf::from(folder),
        });
    }
    sessions
}

/// Attaches the session `session` of the host named `name` in a terminal tab of `workspace`, or
/// shows the tab where that attach still runs. In a remote project on that host the attach is the
/// project's own task; anywhere else it runs here, behind `ssh -t` for a host.
///
/// # Errors
///
/// When the host is no longer named or has no saved host, or the terminal cannot open.
pub(crate) fn attach(
    name: &str,
    session: &str,
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> anyhow::Result<()> {
    let target = format!("={session}");
    open(
        name,
        session,
        &["tmux", "attach-session", "-t", &target],
        workspace,
        window,
        cx,
    )
}

/// Starts a session of Marley's naming in `folder`, attached in a terminal tab of `workspace`; the
/// rail lists it soon (#819). In a remote project the session starts on the project's host (#821).
/// With `command`, tmux types it into the session's shell, so an agent CLI runs as in any terminal
/// of the user's, by the shell's own PATH, and the session stays when it exits.
///
/// # Errors
///
/// When the terminal cannot open.
pub(crate) fn new_session_in(
    folder: &Path,
    command: Option<&str>,
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> anyhow::Result<()> {
    let session = SessionName::from_bits(rand::random());
    let session = session.as_str();
    let mut tmux: Vec<String> = ["tmux", "new-session", "-A", "-s", session, "-c"]
        .iter()
        .map(|word| (*word).to_string())
        .collect();
    tmux.push(folder.to_string_lossy().into_owned());
    if let Some(command) = command {
        // tmux's own command separator: the keys go to the new session once it is made.
        tmux.extend(
            [";", "send-keys", "-t", session, command, "Enter"]
                .iter()
                .map(|word| (*word).to_string()),
        );
    }
    let remote = workspace.project().read(cx).remote_connection_options(cx);
    match remote {
        Some(connection) => {
            let host = cx
                .global::<TmuxHosts>()
                .hosts
                .iter()
                .find(|host| host.reached_by(connection.connection_type(), &connection.host()))
                .map(|host| (host.name.clone(), host.label()));
            let Some((name, label)) = host else {
                anyhow::bail!("marley.tmux_hosts names no host of this project");
            };
            open_in_project(&name, &label, session, tmux, workspace, window, cx)?;
        }
        None => open_here(session, tmux, workspace, window, cx)?,
    }
    list_soon(cx);
    Ok(())
}

/// Opens a terminal tab titled for this machine and `session` that runs the tmux words `tmux`.
fn open_here(
    session: &str,
    tmux: Vec<String>,
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> anyhow::Result<()> {
    let title = format!("this machine · {session}");
    TmuxHosts::remember(
        Attach {
            label: title.clone(),
            host: SharedString::new_static(LOCAL),
            session: session.to_string(),
        },
        cx,
    );
    // tmux refuses to attach from inside a session, as from a Marley started in one.
    let words = ["env", "-u", "TMUX"]
        .iter()
        .map(|word| (*word).to_string())
        .chain(tmux)
        .collect();
    crate::remote::open_words(None, title, LOCAL_ID_BASE, words, workspace, window, cx)
}

/// Opens a terminal tab of the remote project in `workspace` that runs the tmux words `tmux` on
/// the project's host, the host named `name`.
fn open_in_project(
    name: &SharedString,
    label: &SharedString,
    session: &str,
    tmux: Vec<String>,
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> anyhow::Result<()> {
    let title = format!("{label} · {session}");
    TmuxHosts::remember(
        Attach {
            label: title.clone(),
            host: name.clone(),
            session: session.to_string(),
        },
        cx,
    );
    crate::remote::open_on_project_host(
        title,
        crate::remote::TMUX_ID_BASE,
        tmux,
        workspace,
        window,
        cx,
    )
}

/// Opens a terminal tab named after the host and `session` that runs the tmux words `tmux` on the
/// host named `name`: in `workspace`'s own project when it is a remote one on that host, else here.
fn open(
    name: &str,
    session: &str,
    tmux: &[&str],
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> anyhow::Result<()> {
    let host = cx
        .global::<TmuxHosts>()
        .hosts
        .iter()
        .find(|host| host.name == name)
        .map(|host| (host.name.clone(), host.label(), host.place.clone()));
    let Some((name, label, place)) = host else {
        anyhow::bail!("marley.tmux_hosts no longer names {name}");
    };
    let words: Vec<String> = tmux.iter().map(|word| (*word).to_string()).collect();
    match place {
        Place::Here => open_here(session, words, workspace, window, cx),
        Place::Host(host) => {
            let in_project = workspace
                .project()
                .read(cx)
                .remote_connection_options(cx)
                .is_some_and(|connection| {
                    connection.connection_type() == "ssh" && connection.host() == host.target.host
                });
            if in_project {
                return open_in_project(&name, &label, session, words, workspace, window, cx);
            }
            let title = format!("{label} · {session}");
            TmuxHosts::remember(
                Attach {
                    label: title.clone(),
                    host: name,
                    session: session.to_string(),
                },
                cx,
            );
            let mut command = vec!["ssh".to_string(), "-t".to_string()];
            command.extend(marley_remote::keepalive_options());
            command.extend([
                "-o".to_string(),
                format!("ConnectTimeout={}", marley_remote::CONNECT_TIMEOUT_S),
            ]);
            // `ssh_command` gives `ssh`, the port, `--` and the destination.
            command.extend(marley_remote::ssh_command(&host.target).into_iter().skip(1));
            command.extend(words.iter().map(|word| shell_word(word)));
            crate::remote::open_words(
                Some(&host),
                title,
                crate::remote::TMUX_ID_BASE,
                command,
                workspace,
                window,
                cx,
            )
        }
        Place::Unknown => anyhow::bail!("no saved host is named {name}"),
    }
}

/// Ends the session `session` on the host named `name` (`tmux kill-session`), here or behind the
/// listing's ssh, then lists the hosts again soon.
pub(crate) fn kill(name: &str, session: &str, cx: &App) -> Task<anyhow::Result<()>> {
    let place = cx
        .global::<TmuxHosts>()
        .hosts
        .iter()
        .find(|host| host.name == name)
        .map(|host| host.place.clone());
    let target = format!("={session}");
    let kill_words = ["tmux", "kill-session", "-t", target.as_str()];
    let (program, args): (String, Vec<String>) = match place {
        Some(Place::Here) => (
            "tmux".to_string(),
            kill_words
                .iter()
                .skip(1)
                .map(|word| (*word).to_string())
                .collect(),
        ),
        Some(Place::Host(host)) => {
            let mut args = vec![
                "-o".to_string(),
                format!("ConnectTimeout={}", marley_remote::CONNECT_TIMEOUT_S),
            ];
            args.extend(crate::harness_hosts::ssh_arguments(&host.target));
            args.extend(kill_words.iter().map(|word| shell_word(word)));
            let ssh = std::env::var("MARLEY_SSH").unwrap_or_else(|_| "ssh".to_string());
            (ssh, args)
        }
        Some(Place::Unknown) | None => {
            return Task::ready(Err(anyhow::anyhow!(
                "marley.tmux_hosts no longer names {name}"
            )));
        }
    };
    let session = session.to_string();
    cx.spawn(async move |cx| {
        let output = crate::process::output(&program, &args, None, &[]).await?;
        cx.update(list_soon);
        if output.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        let line = stderr.lines().map(str::trim).find(|line| !line.is_empty());
        Err(anyhow::anyhow!(
            "tmux did not end {session}: {}",
            line.map_or_else(|| output.status.to_string(), str::to_string)
        ))
    })
}

/// Lists every host again after [`LIST_AFTER_NEW`].
fn list_soon(cx: &mut App) {
    let task = cx.spawn(async move |cx| {
        cx.background_executor().timer(LIST_AFTER_NEW).await;
        list_all(cx).await;
    });
    cx.global_mut::<TmuxHosts>().soon = Some(task);
}

/// The words a session's row shows under its name: `tmux`, the agent, its windows, and whether a
/// client has it.
pub(crate) fn session_words(session: &TmuxSession) -> String {
    let windows = match session.windows {
        1 => "1 window".to_string(),
        count => format!("{count} windows"),
    };
    std::iter::once("tmux".to_string())
        .chain(session.agent.map(|agent| agent.display_name().to_string()))
        .chain([windows])
        .chain(session.attached.then(|| "attached".to_string()))
        .collect::<Vec<_>>()
        .join(" · ")
}

/// What the error row of a host that could not be listed says (#821).
pub(crate) fn error_words(host: &TmuxHost) -> Option<String> {
    match &host.listing {
        Listing::Failed(reason) => Some(format!("tmux on {}: {reason}", host.label())),
        Listing::Waiting | Listing::Sessions(_) => None,
    }
}

/// Shows `result`'s error in a prompt: a click on a row or a + has nothing else to show it on.
pub(crate) fn report(result: anyhow::Result<()>, window: &Window, cx: &App) {
    if let Err(error) = result {
        Task::ready(Err::<(), _>(error)).detach_and_prompt_err(
            "Could not open the tmux session",
            window,
            cx,
            |_, _, _| None,
        );
    }
}
