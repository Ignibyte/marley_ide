//! Claude Code sessions resumed after a restart (#540).
//!
//! A Marley quit ends every Claude Code running in its terminals, and Zed brings each terminal back
//! as a new shell. A terminal keeps its `MARLEY_TERMINAL_ID` across the restore (#575), so the
//! session each terminal runs is kept in a table of Marley's own under that id, from the session
//! events #519 receives: a `SessionStart` names the session and the folder it started in, and an
//! exit the user makes, or Claude Code leaving the foreground while Marley runs, drops it. When a
//! restored terminal's view appears and its id holds a session, its shell gets
//! `cd <folder> && claude --resume <id>` once it is ready, so the conversation comes back where it
//! stopped. One session resumes in one terminal per launch; `marley.resume_agents` turns it off.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use gpui::{App, AppContext as _, Context, Entity, Global, TaskExt as _};
use marley_agent::AgentKind;
use marley_agent::claude_events::HookEvent;
use settings::Settings as _;
use terminal::Terminal;
use terminal_view::TerminalView;
use util::ResultExt as _;

use crate::{MarleySettings, ResumeAgents};
use persistence::MarleyAgentSessionsDb;

/// A terminal's session: its id and the folder it started in.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Saved {
    session: String,
    folder: String,
}

/// The sessions by terminal id, as the table holds them; each local terminal view's terminal id,
/// by the view's entity id, the rail's seat; the sessions resumed this launch; and whether Marley
/// is quitting, when a terminal's Claude Code dying says nothing of the user.
#[derive(Default)]
struct Sessions {
    by_terminal: HashMap<String, Saved>,
    views: HashMap<u64, String>,
    resumed: HashSet<String>,
    quitting: bool,
}

impl Global for Sessions {}

/// Reads the table, drops the rows of terminals no longer saved, and resumes each restored
/// terminal's session as its view appears.
///
/// [`crate::init`] calls it once, after [`crate::terminal_ids::init`] and before any window
/// restores its terminals.
pub fn init(cx: &mut App) {
    let db = MarleyAgentSessionsDb::global(cx);
    let known = crate::terminal_ids::known_ids(cx);
    let mut by_terminal = HashMap::new();
    let mut gone = Vec::new();
    for (terminal_id, session, folder) in db.all().log_err().unwrap_or_default() {
        if known.contains(&terminal_id) {
            by_terminal.insert(terminal_id, Saved { session, folder });
        } else {
            gone.push(terminal_id);
        }
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
    cx.set_global(Sessions {
        by_terminal,
        ..Sessions::default()
    });
    cx.on_app_quit(|cx| {
        cx.global_mut::<Sessions>().quitting = true;
        futures::future::ready(())
    })
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
                    cx.global_mut::<Sessions>().views.remove(&seat);
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
        session: session.to_string(),
        folder: folder.to_string_lossy().into_owned(),
    };
    save(terminal_id, saved, cx);
}

/// Drops `terminal`'s session when its agent released it while Marley runs (#652); a release at
/// the quit keeps it, as a quit's end does.
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
    let db = MarleyAgentSessionsDb::global(cx);
    cx.background_spawn(async move { db.save(terminal_id, saved.session, saved.folder).await })
        .detach_and_log_err(cx);
}

fn remove(terminal_id: &str, cx: &mut App) {
    if cx
        .default_global::<Sessions>()
        .by_terminal
        .remove(terminal_id)
        .is_none()
    {
        return;
    }
    let db = MarleyAgentSessionsDb::global(cx);
    let terminal_id = terminal_id.to_string();
    cx.background_spawn(async move { db.remove(terminal_id).await })
        .detach_and_log_err(cx);
}

/// Resumes the session saved for `view`'s terminal, when the setting is on, the terminal is a
/// local one whose id holds a session no other terminal resumed this launch, and Claude Code does
/// not run in it already: the terminal's shell gets the resume line once it is ready.
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
    let mode = view
        .marley_workspace()
        .upgrade()
        .map(|workspace| crate::agents::launch_mode(workspace.read(cx), AgentKind::Claude, cx))
        .unwrap_or_default();
    let Some(input) = marley_agent::resume_line(mode, &saved.session, &saved.folder) else {
        log::warn!("the session saved for terminal {terminal_id} is no session id; not resumed");
        return;
    };
    cx.global_mut::<Sessions>().resumed.insert(saved.session);
    cx.spawn(async move |_, cx| {
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

    /// The Claude Code session each terminal ran, by the terminal's `MARLEY_TERMINAL_ID`, which a
    /// restored terminal keeps (#575).
    pub(super) struct MarleyAgentSessionsDb(ThreadSafeConnection);

    impl Domain for MarleyAgentSessionsDb {
        const NAME: &str = stringify!(MarleyAgentSessionsDb);

        const MIGRATIONS: &[&str] = &[sql!(
            CREATE TABLE marley_agent_sessions (
                terminal_id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                folder TEXT NOT NULL
            ) STRICT;
        )];
    }

    db::static_connection!(MarleyAgentSessionsDb, []);

    impl MarleyAgentSessionsDb {
        query! {
            pub(super) async fn save(
                terminal_id: String,
                session_id: String,
                folder: String
            ) -> Result<()> {
                INSERT OR REPLACE INTO marley_agent_sessions(terminal_id, session_id, folder)
                VALUES (?, ?, ?)
            }
        }

        query! {
            pub(super) async fn remove(terminal_id: String) -> Result<()> {
                DELETE FROM marley_agent_sessions
                WHERE terminal_id = ?
            }
        }

        query! {
            pub(super) fn all() -> Result<Vec<(String, String, String)>> {
                SELECT terminal_id, session_id, folder
                FROM marley_agent_sessions
            }
        }
    }
}
