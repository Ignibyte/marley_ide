//! Agent sessions resumed after a restart: Claude Code's (#540) and Codex's (#750).
//!
//! A Marley quit ends every agent running in its terminals, and Zed brings each terminal back as a
//! new shell. A terminal keeps its `MARLEY_TERMINAL_ID` across the restore (#575), so the session
//! each terminal runs is kept in a table of Marley's own under that id: Claude Code's from the
//! session events #519 receives (a `SessionStart` names the session and the folder it started in),
//! Codex's from the lead thread of the App Server Marley started for that terminal (#650). An exit
//! the user makes, or the agent leaving the foreground while Marley runs, drops it. When a
//! restored terminal's view appears and its id holds a session, its shell gets the agent's resume
//! line once it is ready, so the conversation comes back where it stopped; a Codex joins a new App
//! Server of its own when the switch allows one. One session resumes in one terminal per launch;
//! `marley.resume_agents` turns it off.
//!
//! The table also keeps whether each session's turn was open, written as the seat moves into or
//! out of working or waiting (#750). A session resumed with its turn cut off by the restart offers
//! Interrupted · Continue on its rail row; a click types [`marley_agent::RESUME_PROMPT`] while the
//! agent is idle at its prompt. Marley never continues a turn by itself.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use gpui::{App, AppContext as _, Context, Entity, EntityId, Global, TaskExt as _};
use marley_agent::AgentKind;
use marley_agent::claude_events::HookEvent;
use marley_fleet::State;
use settings::Settings as _;
use terminal::Terminal;
use terminal_view::TerminalView;
use util::ResultExt as _;

use crate::{MarleySettings, ResumeAgents};
use persistence::MarleyAgentSessionsDb;

/// The agent a session is, by the name the table keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Agent {
    ClaudeCode,
    Codex,
}

impl Agent {
    const fn name(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
        }
    }

    /// The agent a row names; an unknown name is Claude Code's, the table's default.
    fn named(name: &str) -> Self {
        if name == Self::Codex.name() {
            Self::Codex
        } else {
            Self::ClaudeCode
        }
    }

    const fn kind(self) -> AgentKind {
        match self {
            Self::ClaudeCode => AgentKind::Claude,
            Self::Codex => AgentKind::Codex,
        }
    }
}

/// A terminal's session: its agent, its id and the folder it started in.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Saved {
    agent: Agent,
    session: String,
    folder: String,
}

/// The sessions by terminal id, as the table holds them; each local terminal view's terminal id,
/// by the view's entity id, the rail's seat; the sessions resumed this launch; whether each
/// terminal's turn was open as last written, and the terminals whose turn was open when this
/// launch read the table; the seats offering Continue, with whether their agent has been idle
/// since; and whether Marley is quitting, when a terminal's agent dying says nothing of the user.
#[derive(Default)]
pub(crate) struct Sessions {
    by_terminal: HashMap<String, Saved>,
    views: HashMap<u64, String>,
    resumed: HashSet<String>,
    open: HashMap<String, bool>,
    cut: HashSet<String>,
    offers: HashMap<u64, bool>,
    quitting: bool,
}

impl Global for Sessions {}

/// Reads the table, drops the rows of terminals no longer saved, resumes each restored terminal's
/// session as its view appears, and keeps each session's turn as its seat moves.
///
/// [`crate::init`] calls it once, after [`crate::terminal_ids::init`] and before any window
/// restores its terminals.
pub fn init(cx: &mut App) {
    let db = MarleyAgentSessionsDb::global(cx);
    let known = crate::terminal_ids::known_ids(cx);
    let mut sessions = Sessions::default();
    let mut gone = Vec::new();
    for (terminal_id, session, folder, agent, turn_open) in db.all().log_err().unwrap_or_default() {
        if !known.contains(&terminal_id) {
            gone.push(terminal_id);
            continue;
        }
        let saved = Saved {
            agent: Agent::named(&agent),
            session,
            folder,
        };
        let open = turn_open != 0;
        if open {
            sessions.cut.insert(terminal_id.clone());
        }
        sessions.open.insert(terminal_id.clone(), open);
        sessions.by_terminal.insert(terminal_id, saved);
    }
    if !gone.is_empty() {
        cx.background_spawn(async move {
            for terminal_id in gone {
                db.remove(terminal_id).await?;
            }
            anyhow::Ok(())
        })
        .detach_and_log_err(cx);
    }
    cx.set_global(sessions);
    cx.on_app_quit(|cx| {
        cx.global_mut::<Sessions>().quitting = true;
        futures::future::ready(())
    })
    .detach();
    cx.observe_global::<crate::agent_events::AgentEvents>(note_turns)
        .detach();
    cx.observe_new(
        |view: &mut TerminalView, _, cx: &mut Context<TerminalView>| {
            let Some(terminal_id) = local_id(view.terminal().read(cx)) else {
                return;
            };
            let seat = cx.entity_id().as_u64();
            cx.default_global::<Sessions>()
                .views
                .insert(seat, terminal_id);
            cx.on_release(move |_, cx| {
                if cx.has_global::<Sessions>() {
                    let sessions = cx.global_mut::<Sessions>();
                    sessions.views.remove(&seat);
                    sessions.offers.remove(&seat);
                }
            })
            .detach();
            resume_restored(view, cx);
        },
    )
    .detach();
}

/// Follows the lead's session events from `terminal` (#519): a session that starts is the
/// terminal's, one the user ends is dropped; a compaction changes nothing, and an end for a clear,
/// a resume or anything else waits for what follows, since a quit reads as `other`.
pub(crate) fn on_event(terminal: &Entity<Terminal>, event: &HookEvent, cx: &mut App) {
    let Some(terminal_id) = local_id(terminal.read(cx)) else {
        return;
    };
    match event.event.as_str() {
        // While an agent's report holds the terminal, its session is the report's (#652).
        "SessionStart" if crate::agent_reports::holds_terminal(terminal.entity_id(), cx) => {}
        "SessionStart" if event.source.as_deref() != Some("compact") => {
            let (Some(session), Some(folder)) = (&event.session_id, &event.cwd) else {
                return;
            };
            let saved = Saved {
                agent: Agent::ClaudeCode,
                session: session.clone(),
                folder: folder.clone(),
            };
            save(terminal_id, saved, cx);
        }
        "SessionEnd"
            if matches!(
                event.reason.as_deref(),
                Some("prompt_input_exit" | "logout")
            ) =>
        {
            remove(&terminal_id, cx);
        }
        _ => {}
    }
}

/// Saves the session an agent's report named for `terminal` (#652), in the agent process's
/// folder: it outranks a hook frame's.
pub(crate) fn on_report(terminal: &Entity<Terminal>, session: &str, folder: &Path, cx: &mut App) {
    let Some(terminal_id) = local_id(terminal.read(cx)) else {
        return;
    };
    let saved = Saved {
        agent: Agent::ClaudeCode,
        session: session.to_string(),
        folder: folder.to_string_lossy().into_owned(),
    };
    save(terminal_id, saved, cx);
}

/// Saves `thread`, the lead the App Server Marley started for `terminal` follows, as the
/// terminal's Codex session, in the folder the server started in (#750). A new lead replaces it.
pub(crate) fn on_codex_lead(
    terminal: &Entity<Terminal>,
    thread: &str,
    folder: &Path,
    cx: &mut App,
) {
    let Some(terminal_id) = local_id(terminal.read(cx)) else {
        return;
    };
    let saved = Saved {
        agent: Agent::Codex,
        session: thread.to_string(),
        folder: folder.to_string_lossy().into_owned(),
    };
    save(terminal_id, saved, cx);
}

/// Drops `terminal`'s session when its agent released it while Marley runs (#652): a report's
/// release, a Codex lead that closed or a Codex that left the foreground (#750). A release at the
/// quit keeps it, as a quit's end does.
pub(crate) fn on_release(terminal: &Entity<Terminal>, cx: &mut App) {
    let quitting = cx
        .try_global::<Sessions>()
        .is_some_and(|sessions| sessions.quitting);
    if quitting {
        return;
    }
    if let Some(terminal_id) = local_id(terminal.read(cx)) {
        remove(&terminal_id, cx);
    }
}

/// Drops the sessions of the terminal views `seats`, whose Claude Code left the foreground
/// without a `SessionEnd` (#547), unless Marley is quitting and that is how its terminals go.
pub(crate) fn ended(seats: &[u64], cx: &mut App) {
    let Some(sessions) = cx
        .try_global::<Sessions>()
        .filter(|sessions| !sessions.quitting)
    else {
        return;
    };
    let ids: Vec<String> = seats
        .iter()
        .filter_map(|seat| sessions.views.get(seat).cloned())
        .collect();
    for terminal_id in ids {
        remove(&terminal_id, cx);
    }
}

/// Whether the seat `seat`'s row offers Interrupted · Continue (#750).
pub(crate) fn offered(seat: u64, cx: &App) -> bool {
    cx.try_global::<Sessions>()
        .is_some_and(|sessions| sessions.offers.contains_key(&seat))
}

/// Types [`marley_agent::RESUME_PROMPT`] and Enter into `view`'s terminal, whose session a restart
/// cut off, while its agent is idle at its prompt, and takes the offer away (#750). Otherwise it
/// types nothing and answers what the agent is doing, for the caller's toast.
pub(crate) fn continue_turn(view: &Entity<TerminalView>, cx: &mut App) -> Result<(), String> {
    let seat = view.entity_id().as_u64();
    let agent = cx
        .try_global::<Sessions>()
        .and_then(|sessions| sessions.views.get(&seat))
        .and_then(|terminal_id| cx.global::<Sessions>().by_terminal.get(terminal_id))
        .map(|saved| saved.agent)
        .ok_or_else(|| "its session is no longer kept".to_string())?;
    let state = seat_state(seat, agent, cx);
    if state != Some(State::Idle) {
        return Err(format!(
            "{} is {}; nothing was typed",
            agent.kind().display_name(),
            state_word(state)
        ));
    }
    let terminal = view.read(cx).terminal().clone();
    crate::terminal_drive::paste_then(
        &terminal,
        marley_agent::RESUME_PROMPT,
        |terminal| terminal.input(b"\r".to_vec()),
        cx,
    )
    .detach();
    cx.global_mut::<Sessions>().offers.remove(&seat);
    Ok(())
}

/// What an agent's seat is doing, in a toast's words, as review notes name it (#522).
const fn state_word(state: Option<State>) -> &'static str {
    match state {
        Some(State::Working) => "working",
        Some(State::Waiting) => "asking for permission",
        Some(State::Error) => "stopped on an error",
        _ => "not at its prompt",
    }
}

/// The state of the seat `seat` when `agent` holds it.
fn seat_state(seat: u64, agent: Agent, cx: &App) -> Option<State> {
    crate::agent_events::seat_of(EntityId::from(seat), agent.kind(), cx).map(|seat| seat.state)
}

/// The seats moved (#750): each saved session's turn is written as it opens (working or waiting)
/// or closes (idle, failed, ended), never while Marley quits; an offer goes once its agent, idle
/// since, starts a turn, or its session ends.
fn note_turns(cx: &mut App) {
    let Some(sessions) = cx
        .try_global::<Sessions>()
        .filter(|sessions| !sessions.quitting)
    else {
        return;
    };
    let mut turns = Vec::new();
    let mut offers = Vec::new();
    for (seat, terminal_id) in &sessions.views {
        let Some(saved) = sessions.by_terminal.get(terminal_id) else {
            continue;
        };
        let Some(state) = seat_state(*seat, saved.agent, cx) else {
            continue;
        };
        let open = matches!(state, State::Working | State::Waiting);
        if sessions.open.get(terminal_id).copied().unwrap_or(false) != open {
            turns.push((terminal_id.clone(), open));
        }
        if let Some(idle_since) = sessions.offers.get(seat) {
            let gone = state == State::Done || (*idle_since && open);
            offers.push((
                *seat,
                (!gone).then_some(*idle_since || state == State::Idle),
            ));
        }
    }
    let sessions = cx.global_mut::<Sessions>();
    for (seat, offer) in offers {
        match offer {
            Some(idle_since) => {
                sessions.offers.insert(seat, idle_since);
            }
            None => {
                sessions.offers.remove(&seat);
            }
        }
    }
    if turns.is_empty() {
        return;
    }
    for (terminal_id, open) in &turns {
        sessions.open.insert(terminal_id.clone(), *open);
    }
    let db = MarleyAgentSessionsDb::global(cx);
    cx.background_spawn(async move {
        for (terminal_id, open) in turns {
            db.set_turn(i64::from(open), terminal_id).await?;
        }
        anyhow::Ok(())
    })
    .detach_and_log_err(cx);
}

/// The id of `terminal` when it is a local one, whose sessions live on this machine.
fn local_id(terminal: &Terminal) -> Option<String> {
    if crate::remote::is_remote(terminal) {
        return None;
    }
    terminal.marley_terminal_id().map(str::to_string)
}

fn save(terminal_id: String, saved: Saved, cx: &mut App) {
    let sessions = cx.default_global::<Sessions>();
    if sessions.by_terminal.get(&terminal_id) == Some(&saved) {
        return;
    }
    sessions
        .by_terminal
        .insert(terminal_id.clone(), saved.clone());
    // A new session starts with its turn closed, as the row does.
    sessions.open.insert(terminal_id.clone(), false);
    let db = MarleyAgentSessionsDb::global(cx);
    cx.background_spawn(async move {
        db.save(
            terminal_id,
            saved.session,
            saved.folder,
            saved.agent.name().to_string(),
        )
        .await
    })
    .detach_and_log_err(cx);
}

fn remove(terminal_id: &str, cx: &mut App) {
    let sessions = cx.default_global::<Sessions>();
    sessions.open.remove(terminal_id);
    if sessions.by_terminal.remove(terminal_id).is_none() {
        return;
    }
    let db = MarleyAgentSessionsDb::global(cx);
    let terminal_id = terminal_id.to_string();
    cx.background_spawn(async move { db.remove(terminal_id).await })
        .detach_and_log_err(cx);
}

/// Resumes the session saved for `view`'s terminal, when the setting is on, the terminal is a
/// local one whose id holds a session no other terminal resumed this launch, and no agent runs in
/// it already: the terminal's shell gets the resume line once it is ready. A Codex joins a new App
/// Server first when the switch and its version allow one (#750). A session whose turn was open
/// at the quit offers Continue.
fn resume_restored(view: &TerminalView, cx: &mut Context<TerminalView>) {
    if MarleySettings::get_global(cx).resume_agents == ResumeAgents::Off {
        return;
    }
    let terminal = view.terminal().clone();
    let Some(terminal_id) = local_id(terminal.read(cx)) else {
        return;
    };
    if crate::agent_bar::agent_in(terminal.read(cx)).is_some() {
        return;
    }
    let Some(saved) = cx
        .try_global::<Sessions>()
        .and_then(|sessions| sessions.by_terminal.get(&terminal_id))
        .filter(|saved| !cx.global::<Sessions>().resumed.contains(&saved.session))
        .cloned()
    else {
        return;
    };
    let workspace = view.marley_workspace().upgrade();
    let mode = workspace
        .as_ref()
        .map(|workspace| crate::agents::launch_mode(workspace.read(cx), saved.agent.kind(), cx))
        .unwrap_or_default();
    let Some(plain) = marley_agent::resume_line(
        saved.agent.kind(),
        mode,
        &saved.session,
        &saved.folder,
        None,
    ) else {
        log::warn!("the session saved for terminal {terminal_id} is no session id; not resumed");
        return;
    };
    // A Codex resumed on a new App Server of its terminal's own, which #650 then follows.
    let project = workspace.map(|workspace| workspace.read(cx).project().clone());
    let joining = (saved.agent == Agent::Codex)
        .then(|| {
            let project = project.as_ref()?;
            crate::codex_server::prepare(project.read(cx), Some(Path::new(&saved.folder)), cx)
        })
        .flatten()
        .and_then(|prepared| {
            let line = marley_agent::resume_line(
                saved.agent.kind(),
                mode,
                &saved.session,
                &saved.folder,
                Some(&prepared.remote()),
            )?;
            Some((prepared, line))
        });
    let seat = cx.entity_id().as_u64();
    let sessions = cx.global_mut::<Sessions>();
    sessions.resumed.insert(saved.session);
    if sessions.cut.remove(&terminal_id) {
        sessions.offers.insert(seat, false);
    }
    cx.spawn(async move |_, cx| {
        let input = match (joining, project) {
            (Some((prepared, line)), Some(project)) => {
                let started = crate::codex_server::start(
                    &prepared,
                    &terminal.downgrade(),
                    &project,
                    Vec::new(),
                    cx,
                )
                .await;
                if started { line } else { plain }
            }
            _ => plain,
        };
        let startup = terminal.update(cx, |terminal, _| {
            terminal.start_init_command_startup_handshake()
        });
        let timeout = cx
            .background_executor()
            .timer(crate::agents::STARTUP_TIMEOUT);
        // A shell that never echoes the handshake's marker still gets the line after the timeout.
        futures::future::select(startup, timeout).await;
        let written = terminal.update(cx, |terminal, cx| {
            terminal.write_init_command_after_startup(input, cx)
        });
        if !written {
            log::info!("terminal {terminal_id} took input first; its session was not resumed");
        }
        anyhow::Ok(())
    })
    .detach_and_log_err(cx);
}

mod persistence {
    use db::query;
    use db::sqlez::domain::Domain;
    use db::sqlez::thread_safe_connection::ThreadSafeConnection;
    use db::sqlez_macros::sql;

    /// The agent session each terminal ran, by the terminal's `MARLEY_TERMINAL_ID`, which a
    /// restored terminal keeps (#575): the agent, Claude Code's or Codex's (#750), and whether its
    /// turn was open.
    pub(super) struct MarleyAgentSessionsDb(ThreadSafeConnection);

    impl Domain for MarleyAgentSessionsDb {
        const NAME: &str = stringify!(MarleyAgentSessionsDb);

        const MIGRATIONS: &[&str] = &[
            sql!(
                CREATE TABLE marley_agent_sessions (
                    terminal_id TEXT PRIMARY KEY,
                    session_id TEXT NOT NULL,
                    folder TEXT NOT NULL
                ) STRICT;
            ),
            // A string default needs SQL's single quotes, which `sql!`'s tokens cannot hold.
            "ALTER TABLE marley_agent_sessions ADD COLUMN agent TEXT NOT NULL DEFAULT 'claude-code';",
            "ALTER TABLE marley_agent_sessions ADD COLUMN turn_open INTEGER NOT NULL DEFAULT 0;",
        ];
    }

    db::static_connection!(MarleyAgentSessionsDb, []);

    impl MarleyAgentSessionsDb {
        query! {
            pub(super) async fn save(
                terminal_id: String,
                session_id: String,
                folder: String,
                agent: String
            ) -> Result<()> {
                INSERT OR REPLACE INTO marley_agent_sessions(terminal_id, session_id, folder, agent)
                VALUES (?, ?, ?, ?)
            }
        }

        query! {
            pub(super) async fn set_turn(turn_open: i64, terminal_id: String) -> Result<()> {
                UPDATE marley_agent_sessions
                SET turn_open = ?
                WHERE terminal_id = ?
            }
        }

        query! {
            pub(super) async fn remove(terminal_id: String) -> Result<()> {
                DELETE FROM marley_agent_sessions
                WHERE terminal_id = ?
            }
        }

        query! {
            pub(super) fn all() -> Result<Vec<(String, String, String, String, i64)>> {
                SELECT terminal_id, session_id, folder, agent, turn_open
                FROM marley_agent_sessions
            }
        }
    }
}
