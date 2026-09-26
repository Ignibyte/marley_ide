//! Asking before a close or a quit ends a working agent, and holding a closed working terminal
//! for undo (#550).
//!
//! Zed's close paths ask the [`MarleyCloseGuard`] that [`init`] sets: a tab's close
//! (`Pane::close_items`, which the rail's Close and `ctrl-shift-w` reach), a window's close, the
//! quit and a replace. When an agent in a terminal about to close is working (its seat's state
//! for Claude Code, else output within the quiet timer's two seconds), the guard asks and names
//! each one; an idle agent closes as before. A working terminal closed from its tab is held, PTY
//! and all, for `marley.undo_close_seconds`, with a toast whose Undo, or `ctrl-shift-t`, puts it
//! back in its pane. A signal, a logout or a shutdown meets no question: nothing here waits
//! outside Zed's close paths.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::{App, Context, Entity, EntityId, Global, PromptLevel, Task, WeakEntity, Window};
use marley_agent::{AgentKind, AgentStatus, claude_events};
use marley_fleet::State;
use settings::Settings as _;
use terminal::Event;
use terminal_view::TerminalView;
use util::ResultExt as _;
use workspace::item::ItemHandle;
use workspace::notifications::NotificationId;
use workspace::{
    CloseIntent, MarleyClose, MarleyCloseGuard, MultiWorkspace, Pane, Toast, Workspace,
};

use crate::agent_events::AgentEvents;
use crate::{MarleySettings, UndoCloseTerminal};

/// A working agent in a terminal about to close.
struct Working {
    view: Entity<TerminalView>,
    kind: AgentKind,
    project: String,
    status: &'static str,
}

/// A working terminal closed from its tab, kept until its deadline.
struct Held {
    view: Entity<TerminalView>,
    pane: Option<WeakEntity<Pane>>,
    workspace: WeakEntity<Workspace>,
    id: u64,
    _deadline: Task<()>,
}

/// The held terminals, newest last; whether a question is open; when each terminal last printed.
#[derive(Default)]
struct Guard {
    held: Vec<Held>,
    next_id: u64,
    asking: bool,
    last_output: HashMap<EntityId, Instant>,
}

impl Global for Guard {}

/// Sets the guard Zed's close paths ask, follows every terminal's output for the quiet timer, and
/// registers the undo. [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(MarleyCloseGuard(Arc::new(guard)));
    cx.observe_new(
        |view: &mut TerminalView, _, cx: &mut Context<TerminalView>| {
            let terminal = view.terminal().clone();
            cx.subscribe(&terminal, |_, _, event, cx| {
                if matches!(event, Event::Wakeup) {
                    let (view, now) = (cx.entity_id(), cx.background_executor().now());
                    let _previous = cx.default_global::<Guard>().last_output.insert(view, now);
                }
            })
            .detach();
            let view = cx.entity_id();
            cx.on_release(move |_, cx| {
                if cx.has_global::<Guard>() {
                    let _last = cx.default_global::<Guard>().last_output.remove(&view);
                }
            })
            .detach();
        },
    )
    .detach();
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &UndoCloseTerminal, window, cx| {
            // With nothing held the key goes on to Zed's Reopen Closed Item.
            if !undo(workspace, window, cx) {
                cx.propagate();
            }
        });
    })
    .detach();
}

/// Zed's close paths call this before a close; `false` cancels it.
fn guard(close: MarleyClose, window: &mut Window, cx: &mut App) -> Task<bool> {
    let working = working_agents(&close.items, cx);
    if working.is_empty() {
        return Task::ready(true);
    }
    let settings = MarleySettings::get_global(cx);
    let ask = settings.ask_before_ending_a_working_agent;
    let hold_for = Duration::from_secs(settings.undo_close_seconds);
    // Only a tab's close holds: a window's close, a quit and a replace end what they close.
    let hold_them = close.intent.is_none() && !hold_for.is_zero();
    let pane = close.pane;
    if !ask {
        // The hold waits for this update to end: the pane closing the tab is being updated.
        return cx.spawn(async move |cx| {
            if hold_them {
                cx.update(|cx| hold(working, pane.as_ref(), hold_for, cx));
            }
            true
        });
    }
    let guard = cx.default_global::<Guard>();
    if guard.asking {
        // gpui cannot show a second prompt over the first.
        return Task::ready(false);
    }
    guard.asking = true;
    let (message, detail, confirm) = question(&working, close.intent);
    let answer = window.prompt(
        PromptLevel::Warning,
        &message,
        Some(&detail),
        &[confirm, "Show", "Cancel"],
        cx,
    );
    cx.spawn(async move |cx| {
        let answer = answer.await;
        cx.update(|cx| {
            cx.default_global::<Guard>().asking = false;
            match answer {
                Ok(0) => {
                    if hold_them {
                        hold(working, pane.as_ref(), hold_for, cx);
                    }
                    true
                }
                Ok(1) => {
                    if let Some(first) = working.first() {
                        show(&first.view, cx);
                    }
                    false
                }
                _ => false,
            }
        })
    })
}

/// The agents at work in `items`: each terminal whose foreground program is a known agent that
/// is working.
fn working_agents(items: &[Box<dyn ItemHandle>], cx: &App) -> Vec<Working> {
    items
        .iter()
        .filter_map(|item| item.act_as::<TerminalView>(cx))
        .filter_map(|view| {
            let terminal = view.read(cx).terminal().read(cx);
            let kind = crate::agent_bar::agent_in(terminal)?;
            let project = terminal
                .working_directory()
                .as_deref()
                .and_then(|directory| directory.file_name())
                .map_or_else(
                    || "a terminal".to_string(),
                    |name| name.to_string_lossy().into_owned(),
                );
            let status = working_status(&view, kind, cx)?;
            Some(Working {
                view,
                kind,
                project,
                status,
            })
        })
        .collect()
}

/// The word for an agent that is working, or `None` when it may close unasked. For Claude Code
/// with a seat: a turn, a permission or a question in flight. Otherwise the quiet timer's:
/// output within two seconds, and no bell.
fn working_status(view: &Entity<TerminalView>, kind: AgentKind, cx: &App) -> Option<&'static str> {
    let seat = (kind == AgentKind::Claude)
        .then(|| cx.try_global::<AgentEvents>()?.seat(view.entity_id()))
        .flatten();
    if let Some(seat) = seat {
        return matches!(
            seat.state,
            State::Starting | State::Working | State::Waiting
        )
        .then(|| claude_events::seat_status(seat.state).label());
    }
    let quiet_for = cx
        .try_global::<Guard>()
        .and_then(|guard| guard.last_output.get(&view.entity_id()))
        .map_or(Duration::MAX, |at| {
            cx.background_executor()
                .now()
                .saturating_duration_since(*at)
        });
    (marley_agent::agent_status(quiet_for, view.read(cx).has_bell()) == AgentStatus::Working)
        .then_some(AgentStatus::Working.label())
}

/// The question's message, its detail (a line per agent) and the button that goes on.
fn question(working: &[Working], intent: Option<CloseIntent>) -> (String, String, &'static str) {
    let detail = working
        .iter()
        .map(|agent| {
            format!(
                "{} · {} · {}",
                agent.project,
                agent.kind.display_name(),
                agent.status
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let count = match working.len() {
        1 => "1 agent is working".to_string(),
        count => format!("{count} agents are working"),
    };
    match (intent, working) {
        (None, [agent]) => (
            format!(
                "Close {} in {}? It is {}.",
                agent.kind.display_name(),
                agent.project,
                agent.status
            ),
            detail,
            "Close",
        ),
        (None, _) => (format!("Close these terminals? {count}:"), detail, "Close"),
        (Some(CloseIntent::Quit), _) => (format!("Quit Marley? {count}:"), detail, "Quit"),
        (Some(CloseIntent::CloseWindow), _) => (
            format!("Close this window? {count}:"),
            detail,
            "Close Window",
        ),
        (Some(CloseIntent::ReplaceWindow), _) => {
            (format!("Close this project? {count}:"), detail, "Close")
        }
    }
}

/// Keeps each working terminal closed from its tab for `hold_for`, with a toast to undo it; at
/// the deadline the view drops and its PTY ends as a close ends it.
fn hold(working: Vec<Working>, pane: Option<&WeakEntity<Pane>>, hold_for: Duration, cx: &mut App) {
    for agent in working {
        let workspace = agent.view.read(cx).marley_workspace().clone();
        let guard = cx.default_global::<Guard>();
        let id = guard.next_id;
        guard.next_id = guard.next_id.wrapping_add(1);
        let deadline = cx.spawn({
            let workspace = workspace.clone();
            async move |cx| {
                cx.background_executor().timer(hold_for).await;
                cx.update(|cx| {
                    let guard = cx.default_global::<Guard>();
                    guard.held.retain(|held| held.id != id);
                    if guard.held.is_empty() {
                        dismiss_toast(&workspace, cx);
                    }
                });
            }
        });
        cx.default_global::<Guard>().held.push(Held {
            view: agent.view,
            pane: pane.cloned(),
            workspace: workspace.clone(),
            id,
            _deadline: deadline,
        });
        let message = format!("Closed {} in {}", agent.kind.display_name(), agent.project);
        workspace
            .update(cx, |workspace, cx| {
                workspace.show_toast(
                    Toast::new(NotificationId::unique::<Guard>(), message).on_click(
                        "Undo",
                        |window, cx| {
                            window.dispatch_action(Box::new(UndoCloseTerminal), cx);
                        },
                    ),
                    cx,
                );
            })
            .log_err();
    }
}

/// Puts the newest held terminal back in its pane, or in `workspace`'s active pane when it came
/// from another workspace or its pane is gone; false when nothing is held.
fn undo(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) -> bool {
    let Some(held) = cx
        .try_global::<Guard>()
        .is_some()
        .then(|| cx.default_global::<Guard>().held.pop())
        .flatten()
    else {
        return false;
    };
    let pane = (held.workspace.entity_id() == cx.entity_id())
        .then(|| held.pane.as_ref().and_then(WeakEntity::upgrade))
        .flatten()
        .unwrap_or_else(|| workspace.active_pane().clone());
    let view = held.view;
    pane.update(cx, |pane, cx| {
        pane.add_item(Box::new(view), true, true, None, window, cx);
    });
    if cx
        .try_global::<Guard>()
        .is_none_or(|guard| guard.held.is_empty())
    {
        workspace.dismiss_toast(&NotificationId::unique::<Guard>(), cx);
    }
    true
}

fn dismiss_toast(workspace: &WeakEntity<Workspace>, cx: &mut App) {
    workspace
        .update(cx, |workspace, cx| {
            workspace.dismiss_toast(&NotificationId::unique::<Guard>(), cx);
        })
        .log_err();
}

/// Brings `view` to the front, as a click on its notification does: its window, its project and
/// its tab.
fn show(view: &Entity<TerminalView>, cx: &mut App) {
    let Some(workspace) = view.read(cx).marley_workspace().upgrade() else {
        return;
    };
    for window in cx
        .windows()
        .into_iter()
        .filter_map(|window| window.downcast::<MultiWorkspace>())
    {
        let shown = window
            .update(cx, |multi_workspace, window, cx| {
                if !multi_workspace
                    .workspaces()
                    .any(|candidate| *candidate == workspace)
                {
                    return false;
                }
                window.activate_window();
                multi_workspace.activate(workspace.clone(), None, window, cx);
                workspace.update(cx, |workspace, cx| {
                    workspace.activate_item(view, true, true, window, cx);
                });
                true
            })
            .unwrap_or(false);
        if shown {
            return;
        }
    }
}
