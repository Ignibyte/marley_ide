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
//!
//! A tab attached to a tmux session Marley opened asks first, when it alone closes from its tab
//! (#821): Close Session ends the session on its host, Hide closes only the tab and leaves the
//! session running, Cancel keeps the tab. A window's close or a quit detaches every attach and
//! asks nothing, since the sessions run on.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::{
    AnyWindowHandle, App, Context, Entity, EntityId, Global, PromptLevel, SharedString,
    Subscription, Task, TaskExt as _, WeakEntity, Window,
};
use marley_agent::{AgentKind, AgentStatus, claude_events};
use marley_fleet::State;
use settings::Settings as _;
use terminal::{Event, TaskStatus};
use terminal_view::TerminalView;
use ui::ActiveTheme as _;
use util::ResultExt as _;
use workspace::item::ItemHandle;
use workspace::notifications::{DetachAndPromptErr as _, MarleyToastKind, NotificationId};
use workspace::{
    CloseIntent, MarleyClose, MarleyCloseGuard, MultiWorkspace, Pane, SaveIntent, Toast, Workspace,
};

use crate::agent_events::AgentEvents;
use crate::tmux_sessions::TmuxHosts;
use crate::{MarleySettings, UndoCloseTerminal};

/// A working agent in a terminal about to close.
struct Working {
    view: Entity<TerminalView>,
    kind: AgentKind,
    project: String,
    status: &'static str,
    /// The turn it is in, by its prompt (#913).
    turn: Option<String>,
    /// Whether a seat reports its turns, so a wait can tell when the turn ends (#913, D1).
    reports_turns: bool,
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
    /// The tabs that close when their agent's turn ends (#913).
    waits: Vec<Wait>,
    next_id: u64,
    asking: bool,
    last_output: HashMap<EntityId, Instant>,
}

impl Global for Guard {}

/// A tab that closes when its agent's turn ends (#913): its view, and the watch on the seats.
struct Wait {
    id: u64,
    _watch: Subscription,
}

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
    if close.intent.is_none()
        && let Some((host, session)) = tmux_attach(&close.items, cx)
    {
        return ask_tmux(host, session, window, cx);
    }
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
    let (message, detail, confirm) = question(&working, close.intent, hold_for);
    // Wait where it can work: one agent closed from its tab, whose seat reports turns (D1).
    let waits =
        close.intent.is_none() && matches!(working.as_slice(), [agent] if agent.reports_turns);
    let second = if waits { "Wait" } else { "Show" };
    let answer = window.prompt(
        PromptLevel::Warning,
        &message,
        Some(&detail),
        &[confirm, second, "Cancel"],
        cx,
    );
    let window_handle = window.window_handle();
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
                Ok(1) if waits => {
                    if let Some(agent) = working.into_iter().next() {
                        wait(&agent, pane, window_handle, cx);
                    }
                    false
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

/// The host's and the session's names when `items` is one terminal running a tmux attach Marley
/// opened (#821).
fn tmux_attach(items: &[Box<dyn ItemHandle>], cx: &App) -> Option<(SharedString, String)> {
    let [item] = items else {
        return None;
    };
    let view = item.act_as::<TerminalView>(cx)?;
    let terminal = view.read(cx).terminal().read(cx);
    let task = terminal.task()?;
    if task.status != TaskStatus::Running {
        return None;
    }
    let tmux = cx.try_global::<TmuxHosts>()?;
    let attach = tmux.attach_of(&task.spawned_task.full_label)?;
    Some((attach.host.clone(), attach.session.clone()))
}

/// Asks whether closing the tab of the tmux session `session` on the host named `host` ends the
/// session or only hides it; a failed end shows in a prompt of its own.
fn ask_tmux(host: SharedString, session: String, window: &mut Window, cx: &mut App) -> Task<bool> {
    let guard = cx.default_global::<Guard>();
    if guard.asking {
        // gpui cannot show a second prompt over the first.
        return Task::ready(false);
    }
    guard.asking = true;
    let message = format!("Close the tmux session \u{201c}{session}\u{201d}?");
    let detail = "Close Session ends what runs in it. Hide detaches the tab; the session keeps \
                  running and its row stays in the rail.";
    // The TmuxSessions board's head (#913): a `tx` tile, the host, the windows, the program.
    crate::prompts::set_head(
        crate::prompts::PromptHead {
            message: message.clone(),
            glyph: SharedString::new_static("tx"),
            ink: cx.theme().colors().terminal_ansi_cyan,
            line: tmux_line(&host, &session, cx),
        },
        cx,
    );
    let answer = window.prompt(
        PromptLevel::Warning,
        &message,
        Some(detail),
        &["Close Session", "Hide", "Cancel"],
        cx,
    );
    let window_handle = window.window_handle();
    cx.spawn(async move |cx| {
        let answer = answer.await;
        cx.update(|cx| {
            cx.default_global::<Guard>().asking = false;
            crate::prompts::clear_head(&message, cx);
        });
        match answer {
            Ok(0) => {
                let killed = cx.update(|cx| crate::tmux_sessions::kill(&host, &session, cx));
                cx.spawn(async move |cx| {
                    if let Err(error) = killed.await {
                        window_handle
                            .update(cx, |_, window, cx| {
                                Task::ready(Err::<(), _>(error)).detach_and_prompt_err(
                                    "Could not end the tmux session",
                                    window,
                                    cx,
                                    |_, _, _| None,
                                );
                            })
                            .log_err();
                    }
                })
                .detach();
                true
            }
            Ok(1) => true,
            _ => false,
        }
    })
}

/// The line under the tmux prompt's title: where the session runs, its windows, and the program it
/// runs when that is not a shell (#913).
fn tmux_line(host: &str, session: &str, cx: &App) -> String {
    let place = if crate::tmux_sessions::is_local(host) {
        "On this machine".to_string()
    } else {
        format!("On {host}")
    };
    let found = cx
        .try_global::<TmuxHosts>()
        .and_then(|tmux| tmux.session_of(host, session));
    let Some(found) = found else {
        return place;
    };
    let windows = if found.windows == 1 {
        "1 window".to_string()
    } else {
        format!("{} windows", found.windows)
    };
    found.command.as_ref().map_or_else(
        || format!("{place} \u{b7} {windows}"),
        |command| format!("{place} \u{b7} {windows} \u{b7} {command} is running"),
    )
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
            let seat = crate::agent_events::seat_of(view.entity_id(), kind, cx);
            // The turn's prompt, as the rail's line names it (#913).
            let turn = seat
                .and_then(|seat| seat.labels.get(claude_events::PROMPT_LABEL))
                .map(|prompt| prompt.split_whitespace().collect::<Vec<_>>().join(" "))
                .filter(|prompt| !prompt.is_empty());
            let reports_turns = seat.is_some();
            Some(Working {
                view,
                kind,
                project,
                status,
                turn,
                reports_turns,
            })
        })
        .collect()
}

/// The word for an agent that is working, or `None` when it may close unasked. For Claude Code
/// with a seat, or Codex with its App Server's (#650): a turn, a permission or a question in
/// flight. Otherwise the quiet timer's: output within two seconds, and no bell.
pub(crate) fn working_status(
    view: &Entity<TerminalView>,
    kind: AgentKind,
    cx: &App,
) -> Option<&'static str> {
    if let Some(seat) = crate::agent_events::seat_of(view.entity_id(), kind, cx) {
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
fn question(
    working: &[Working],
    intent: Option<CloseIntent>,
    hold_for: Duration,
) -> (String, String, &'static str) {
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
        // The board's words (#913): the agent, its turn, and the undo hold.
        (None, [agent]) => {
            let turn = agent.turn.as_ref().map_or_else(
                || "It is in a turn.".to_string(),
                |turn| format!("\u{201c}{turn}\u{201d} is in a turn."),
            );
            let hold = if hold_for.is_zero() {
                String::new()
            } else {
                format!(
                    " Close keeps it running for {} s so Undo can bring it back.",
                    hold_for.as_secs()
                )
            };
            (
                format!("{} is still working", agent.kind.display_name()),
                format!("{turn}{hold}"),
                "Close",
            )
        }
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
                    Toast::new(NotificationId::unique::<Guard>(), message)
                        .marley_kind(MarleyToastKind::Warning)
                        .on_click("Undo", |window, cx| {
                            window.dispatch_action(Box::new(UndoCloseTerminal), cx);
                        }),
                    cx,
                );
            })
            .log_err();
    }
}

/// Wait (#913): the tab stays, with a toast, until the agent's seat leaves its turn; then it closes
/// through the pane's own close, so the guard runs again and finds the agent idle (D2). A seat
/// waiting on the user keeps the wait (D3); Keep It Open, or the tab closed by hand, ends it.
fn wait(agent: &Working, pane: Option<WeakEntity<Pane>>, window: AnyWindowHandle, cx: &mut App) {
    let guard = cx.default_global::<Guard>();
    let id = guard.next_id;
    guard.next_id = guard.next_id.wrapping_add(1);
    let view = agent.view.downgrade();
    let kind = agent.kind;
    let workspace = agent.view.read(cx).marley_workspace().clone();
    let watch = cx.observe_global::<AgentEvents>({
        let workspace = workspace.clone();
        move |cx| {
            let Some(entity) = view.upgrade() else {
                end_wait(id, &workspace, cx);
                return;
            };
            let in_turn =
                crate::agent_events::seat_of(entity.entity_id(), kind, cx).is_some_and(|seat| {
                    matches!(
                        seat.state,
                        State::Starting | State::Working | State::Waiting
                    )
                });
            if in_turn {
                return;
            }
            end_wait(id, &workspace, cx);
            let Some(pane) = pane.as_ref().and_then(WeakEntity::upgrade) else {
                return;
            };
            let item = entity.entity_id();
            window
                .update(cx, |_, window, cx| {
                    pane.update(cx, |pane, cx| {
                        pane.close_item_by_id(item, SaveIntent::Skip, window, cx)
                            .detach_and_log_err(cx);
                    });
                })
                .log_err();
        }
    });
    cx.default_global::<Guard>()
        .waits
        .push(Wait { id, _watch: watch });
    let message = format!(
        "{} in {} closes when its turn ends",
        agent.kind.display_name(),
        agent.project
    );
    let toast_workspace = workspace.clone();
    workspace
        .update(cx, |workspace, cx| {
            workspace.show_toast(
                Toast::new(NotificationId::unique::<Wait>(), message)
                    .marley_kind(MarleyToastKind::Warning)
                    .on_click("Keep It Open", move |_, cx| {
                        end_wait(id, &toast_workspace, cx);
                    }),
                cx,
            );
        })
        .log_err();
}

/// Ends the wait `id` and takes its toast away, after the observer that may be running it.
fn end_wait(id: u64, workspace: &WeakEntity<Workspace>, cx: &mut App) {
    let workspace = workspace.clone();
    cx.defer(move |cx| {
        if cx.has_global::<Guard>() {
            cx.default_global::<Guard>()
                .waits
                .retain(|wait| wait.id != id);
        }
        workspace
            .update(cx, |workspace, cx| {
                workspace.dismiss_toast(&NotificationId::unique::<Wait>(), cx);
            })
            .log_err();
    });
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
