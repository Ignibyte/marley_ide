//! Agent reports (#652): an agent in a Marley terminal reports its state through
//! `$MARLEY_BIN report`, as the Claude Code plugin Marley shares with rustal-harness does.
//!
//! At start Marley binds its agent socket (`marley_mcp::agent_socket`) under the runtime directory,
//! named by its data directory's digest, and writes `marley-agent` with the socket's path beside
//! it into `<data_dir>/mcp/`; local interactive terminals then name the program as `MARLEY_BIN`.
//! A request's process chain is read on its connection's thread; on the main thread the chain is
//! matched to the terminal whose shell it passes through, and the terminal's `MARLEY_TERMINAL_ID`
//! must be the one sent. The first accepted report makes its source and the reporter's parent (the
//! agent) the terminal's authority until a release, or until that process ends. While it stands,
//! the report sets the seat's state and question, the hook frames keep their labels, and a
//! restart resumes the session it named.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use futures::StreamExt as _;
use futures::channel::mpsc;
use gpui::{AnyWindowHandle, App, AppContext as _, Entity, EntityId, Global};
use marley_agent::AgentKind;
use marley_agent::report::{self, Refusal, Report, Request};
use marley_fleet::{Session, State};
use sha2::{Digest as _, Sha256};
use terminal_view::TerminalView;
use workspace::Workspace;

/// The program, as Marley ships it.
const PROGRAM: &str = include_str!("../bin/marley-agent");

/// Its file in Marley's `mcp` folder.
const PROGRAM_FILE: &str = "marley-agent";

/// The file beside it naming this Marley's socket.
const SOCKET_FILE: &str = "agent-socket";

/// The socket, and each terminal's authority.
#[derive(Default)]
struct Reports {
    socket: Option<marley_mcp::agent_socket::AgentSocket>,
    held: HashMap<EntityId, Held>,
    /// The last `seq` accepted per terminal view and source, which keeps rising across releases.
    seqs: HashMap<(EntityId, String), u64>,
}

impl Global for Reports {}

/// A terminal's authority: its source, the agent process with its start time, its terminal and
/// the last report.
struct Held {
    source: String,
    agent: (u32, u64),
    terminal: EntityId,
    report: Report,
}

/// A request's terminal: its workspace, its view, and the agent process with its start time.
type Found = (Entity<Workspace>, Entity<TerminalView>, (u32, u64));

/// A request with what its connection's thread read: the reporter's process chain, nearest first,
/// and its parent's working directory.
struct Incoming {
    request: marley_mcp::agent_socket::AgentRequest,
    chain: Vec<(u32, u64)>,
    folder: Option<PathBuf>,
}

/// Binds the agent socket and writes the program for this Marley's `data_dir`; terminals name the
/// program once both are in place. On any failure terminals name none, and the hook frames go on
/// as before.
pub(crate) fn start(data_dir: PathBuf, cx: &mut App) {
    cx.set_global(Reports::default());
    let socket = socket_path(&data_dir);
    if !marley_browser::service::socket_fits(&socket) {
        log::warn!(
            "agent reports: {} is too long for a socket; terminals name no MARLEY_BIN",
            socket.display()
        );
        return;
    }
    let (incoming, mut requests) = mpsc::unbounded::<Incoming>();
    let handler: marley_mcp::agent_socket::AgentHandler = std::sync::Arc::new(move |request| {
        let chain = process_chain(request.pid);
        let folder = chain
            .get(1)
            .and_then(|(agent, _)| std::fs::read_link(format!("/proc/{agent}/cwd")).ok());
        if let Err(error) = incoming.unbounded_send(Incoming {
            request,
            chain,
            folder,
        }) {
            log::debug!("agent reports: a request came as Marley shut down: {error}");
        }
    });
    let bound = socket
        .parent()
        .ok_or_else(|| std::io::Error::other("the socket has no folder"))
        .and_then(private_folder)
        .and_then(|()| marley_mcp::agent_socket::spawn(&socket, handler));
    match bound {
        Ok(bound) => cx.default_global::<Reports>().socket = Some(bound),
        Err(error) => {
            log::warn!("agent reports: binding {}: {error}", socket.display());
            return;
        }
    }
    cx.spawn(async move |cx| {
        while let Some(incoming) = requests.next().await {
            cx.update(|cx| take(incoming, cx));
        }
    })
    .detach();
    cx.background_spawn(futures::future::lazy(move |_| {
        let written =
            crate::mcp::write_program_in(&data_dir, PROGRAM_FILE, PROGRAM).and_then(|program| {
                let beside = program.with_file_name(SOCKET_FILE);
                std::fs::write(&beside, format!("{}\n", socket.display())).map(|()| program)
            });
        match written {
            Ok(program) => marley_terminal::identity::set_agent_program(Some(program)),
            Err(error) => log::warn!("agent reports: writing {PROGRAM_FILE}: {error}"),
        }
    }))
    .detach();
}

/// The socket's path: `marley/<16 hex of the data directory's SHA-256>.sock` under the runtime
/// directory, else under Zed's temporary folder, so each Marley and each e2e profile has its own.
fn socket_path(data_dir: &Path) -> PathBuf {
    let digest = Sha256::digest(data_dir.as_os_str().as_encoded_bytes());
    let mut first = [0_u8; 8];
    first.copy_from_slice(&digest[..8]);
    let name = format!("{:016x}", u64::from_be_bytes(first));
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|folder| folder.is_absolute())
        .unwrap_or_else(|| paths::temp_dir().clone())
        .join("marley")
        .join(format!("{name}.sock"))
}

/// Makes `folder` and leaves it the user's alone.
fn private_folder(folder: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::{DirBuilderExt as _, PermissionsExt as _};
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(folder)?;
    std::fs::set_permissions(folder, std::fs::Permissions::from_mode(0o700))
}

/// `pid` and its parents, nearest first, each with its start time, at most
/// [`report::MAX_ANCESTRY`] of them. It reads `/proc`, so callers run it off the main thread.
pub(crate) fn process_chain(pid: u32) -> Vec<(u32, u64)> {
    let mut chain = Vec::new();
    let mut next = pid;
    while next > 1 && chain.len() < report::MAX_ANCESTRY {
        let Some((parent, started)) = std::fs::read_to_string(format!("/proc/{next}/stat"))
            .ok()
            .as_deref()
            .and_then(report::stat_fields)
        else {
            break;
        };
        chain.push((next, started));
        next = parent;
    }
    chain
}

/// Whether the process `pid` that started at `started` still runs.
fn alive((pid, started): (u32, u64)) -> bool {
    std::fs::read_to_string(format!("/proc/{pid}/stat"))
        .ok()
        .as_deref()
        .and_then(report::stat_fields)
        .is_some_and(|(_, now)| now == started)
}

/// Answers one request on the main thread.
fn take(incoming: Incoming, cx: &mut App) {
    let Incoming {
        request,
        chain,
        folder,
    } = incoming;
    let reply = match serde_json::from_str::<Request>(&request.line) {
        Err(error) => report::answer_refused(Refusal::Shape, &error.to_string()),
        Ok(parsed) => match accept(parsed, &chain, folder.as_deref(), cx) {
            Ok(()) => report::answer_ok(),
            Err((refusal, reason)) => report::answer_refused(refusal, &reason),
        },
    };
    request.answer(reply);
}

/// A local terminal the chain passes through: its workspace, its view, and the reporter's parent,
/// the agent; checked against the id the program sent.
fn terminal_of(
    chain: &[(u32, u64)],
    terminal_id: &str,
    cx: &App,
) -> Result<Found, (Refusal, String)> {
    let unknown = |reason: &str| Err((Refusal::Unknown, reason.to_string()));
    let Some((workspace, view)) = terminal_on_chain(chain, cx) else {
        return unknown("the caller runs in none of Marley's local terminals");
    };
    if view.read(cx).terminal().read(cx).marley_terminal_id() != Some(terminal_id) {
        return unknown("the caller's MARLEY_TERMINAL_ID is not its terminal's");
    }
    let Some(agent) = chain.get(1).copied() else {
        return unknown("the caller has no parent process");
    };
    Ok((workspace, view, agent))
}

/// The local terminal, of any window, whose shell is the nearest of `chain`'s processes, with its
/// workspace.
pub(crate) fn terminal_on_chain(
    chain: &[(u32, u64)],
    cx: &App,
) -> Option<(Entity<Workspace>, Entity<TerminalView>)> {
    let shells: HashMap<u32, (Entity<Workspace>, Entity<TerminalView>)> = crate::mcp::terminals(cx)
        .into_iter()
        .filter_map(|(workspace, view)| {
            let terminal = view.read(cx).terminal().read(cx);
            if crate::remote::is_remote(terminal) {
                return None;
            }
            let shell = terminal.pid_getter()?.fallback_pid().as_u32();
            Some((shell, (workspace, view)))
        })
        .collect();
    chain
        .iter()
        .find_map(|(pid, _)| shells.get(pid))
        .map(|(workspace, view)| (workspace.clone(), view.clone()))
}

/// Takes a report or a release, or says why not.
fn accept(
    request: Request,
    chain: &[(u32, u64)],
    folder: Option<&Path>,
    cx: &mut App,
) -> Result<(), (Refusal, String)> {
    match request {
        Request::Report { terminal, report } => {
            let report = *report;
            let (workspace, view, agent) = terminal_of(chain, &terminal, cx)?;
            report.validate()?;
            let id = view.entity_id();
            let reports = cx.default_global::<Reports>();
            let last = reports.seqs.get(&(id, report.source.clone())).copied();
            if last.is_some_and(|last| report.seq <= last) {
                return Err((
                    Refusal::Stale,
                    format!(
                        "seq {} is not past {}",
                        report.seq,
                        last.unwrap_or_default()
                    ),
                ));
            }
            let holder = reports
                .held
                .get(&id)
                .map(|held| (alive(held.agent), held.source.clone(), held.agent));
            match holder {
                Some((false, _, _)) => {
                    let _lapsed = reports.held.remove(&id);
                }
                Some((true, source, holding)) if source != report.source || holding != agent => {
                    return Err((Refusal::Authority, format!("{source} holds this terminal")));
                }
                Some(_) | None => {}
            }
            let _previous = reports.seqs.insert((id, report.source.clone()), report.seq);
            let terminal_entity = view.read(cx).terminal().clone();
            let _replaced = cx.default_global::<Reports>().held.insert(
                id,
                Held {
                    source: report.source.clone(),
                    agent,
                    terminal: terminal_entity.entity_id(),
                    report: report.clone(),
                },
            );
            let kind = crate::agent_bar::agent_in(terminal_entity.read(cx));
            let moved = crate::agent_events::apply_report(id, &report, kind, false, cx);
            if let (Some(session), Some(folder)) = (&report.session_id, folder) {
                crate::resume::on_report(&terminal_entity, session, folder, cx);
            }
            if let Some((before, after)) = moved {
                announce(&workspace, &view, before, &after, cx);
            }
            Ok(())
        }
        Request::Release { terminal, source } => {
            let (_, view, agent) = terminal_of(chain, &terminal, cx)?;
            let id = view.entity_id();
            let reports = cx.default_global::<Reports>();
            let Some(held) = reports.held.get(&id) else {
                return Err((
                    Refusal::ReleaseNone,
                    "this terminal has no authority".to_string(),
                ));
            };
            if held.source != source || (held.agent != agent && alive(held.agent)) {
                return Err((
                    Refusal::ReleaseAuthority,
                    format!("{} holds this terminal", held.source),
                ));
            }
            let _released = reports.held.remove(&id);
            crate::agent_events::end(&[id.as_u64()], cx);
            let terminal_entity = view.read(cx).terminal().clone();
            crate::resume::on_release(&terminal_entity, cx);
            Ok(())
        }
    }
}

/// The banner (#538) and the push (#535) a reported change makes, as a frame's would.
fn announce(
    workspace: &Entity<Workspace>,
    view: &Entity<TerminalView>,
    before: State,
    after: &Session,
    cx: &mut App,
) {
    let Some(window) = crate::browser::window_of(workspace, cx) else {
        return;
    };
    let shown = AnyWindowHandle::from(window).update(cx, |_, window, cx| {
        view.update(cx, |view, cx| {
            crate::notifications::on_seat_change(view, before, after, window, cx);
            crate::push::on_change(view, before, after, window, cx);
        });
    });
    if shown.is_err() {
        log::debug!("agent reports: the terminal's window closed before its banner");
    }
}

/// The report of the authority holding the terminal `view`, while its agent runs; an authority
/// whose agent ended lapses here.
pub(crate) fn held_report(view: EntityId, cx: &mut App) -> Option<Report> {
    let reports = cx.try_global::<Reports>()?;
    let held = reports.held.get(&view)?;
    if alive(held.agent) {
        return Some(held.report.clone());
    }
    let _lapsed = cx.default_global::<Reports>().held.remove(&view);
    None
}

/// Whether an authority holds the terminal `terminal`, so its hook frames name no session to
/// resume (#652).
pub(crate) fn holds_terminal(terminal: EntityId, cx: &App) -> bool {
    cx.try_global::<Reports>().is_some_and(|reports| {
        reports
            .held
            .values()
            .any(|held| held.terminal == terminal && alive(held.agent))
    })
}

/// Drops the authority of the terminal view `view`, which closed or whose agent left.
pub(crate) fn forget(view: EntityId, cx: &mut App) {
    if cx
        .try_global::<Reports>()
        .is_some_and(|reports| reports.held.contains_key(&view))
    {
        let _dropped = cx.default_global::<Reports>().held.remove(&view);
    }
}

/// The agent a report's seat names when the rail recognizes the terminal's program.
pub(crate) const fn agent_label(kind: AgentKind) -> &'static str {
    match kind {
        AgentKind::Claude => "claude-code",
        AgentKind::Codex => marley_agent::codex_events::AGENT,
        AgentKind::Gemini => "gemini",
        AgentKind::OpenCode => "opencode",
    }
}
