//! Desktop notifications (T7b), from a terminal you are not looking at.
//!
//! Zed's `Terminal` emits `Event::MarleyNotification` for each OSC 9 or OSC 777 notify a program
//! prints, and its view marks itself as a bell does. [`init`] subscribes every terminal view to
//! its terminal and shows a desktop notification unless the view is the focused one of the
//! active window. Clicking the notification shows that terminal. A notify titled
//! `marley-event` carries a Claude Code hook event instead, which goes to the rail
//! ([`crate::agent_events`]).
//!
//! A Claude Code event that needs input, finishes or fails a turn shows a banner saying what
//! happened, `<project>: Claude finished` over the turn's last message (#538), and marks its
//! terminal unread, the rail's dot, until the user looks at it. A burst from one project within
//! five seconds shows one banner, of any kind; the held-back events still mark.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use gpui::{
    AnyWindowHandle, App, Context, Entity, EntityId, Focusable as _, Global, SharedString,
    SystemNotification, SystemNotificationResponse, WeakEntity, Window, WindowHandle,
};
use marley_agent::claude_events::banner_body;
use marley_agent::{AgentKind, TurnEvent, event_line};
use marley_fleet::{Session, State};
use terminal::{Event, Terminal};
use terminal_view::TerminalView;
use util::ResultExt as _;
use workspace::item::Item as _;
use workspace::{MultiWorkspace, Workspace};

/// The terminal each notification came from, by its tag, for the click.
#[derive(Default)]
struct Senders(HashMap<SharedString, (AnyWindowHandle, WeakEntity<TerminalView>)>);

impl Global for Senders {}

/// A burst of banners from one project within this long shows one, as Orca's cooldown does.
const COOLDOWN: Duration = Duration::from_secs(5);

/// The terminal views with an agent event the user has not seen, and when each project, by its
/// workspace, last showed a banner (#538).
#[derive(Default)]
pub(crate) struct Attention {
    unread: HashSet<EntityId>,
    last_banner: HashMap<EntityId, Instant>,
}

impl Global for Attention {}

/// Whether the terminal view `view` has an agent event its user has not seen, for the rail's dot.
pub(crate) fn unread(view: EntityId, cx: &App) -> bool {
    cx.try_global::<Attention>()
        .is_some_and(|attention| attention.unread.contains(&view))
}

/// Names Marley on its notifications and watches every terminal view for them. [`crate::init`]
/// calls it once.
pub fn init(cx: &App) {
    cx.set_app_identity("marley", "Marley");
    cx.on_system_notification_response(|response, cx| show_sender(&response, cx));
    cx.observe_new(
        |view: &mut TerminalView, window, cx: &mut Context<TerminalView>| {
            // A terminal view is always made in a window: `TerminalView::new` takes one.
            let Some(window) = window else { return };
            let terminal = view.terminal().clone();
            watch(&terminal, window, cx);
            // A task's Rerun gives the view a new terminal (`TerminalView::set_terminal`), which
            // the subscription above never hears; the view notifies once the new one draws, so a
            // remote terminal attached again reports its host's events (#543).
            let mut watched = terminal.entity_id();
            cx.observe_in(&cx.entity(), window, move |view, _, window, cx| {
                let terminal = view.terminal().clone();
                if terminal.entity_id() != watched {
                    watched = terminal.entity_id();
                    watch(&terminal, window, cx);
                }
            })
            .detach();
            // Looking at the terminal, by focusing it or by coming back to its window, sees its
            // events (#538).
            let focus_handle = view.focus_handle(cx);
            cx.on_focus_in(&focus_handle, window, |view, window, cx| {
                seen(view, window, cx);
            })
            .detach();
            cx.observe_window_activation(window, |view, window, cx| {
                seen(view, window, cx);
            })
            .detach();
            let view = cx.entity_id();
            cx.on_release(move |_, cx| {
                crate::agent_events::forget(view, cx);
                if cx.has_global::<Attention>() {
                    let _was_unread = cx.global_mut::<Attention>().unread.remove(&view);
                }
            })
            .detach();
        },
    )
    .detach();
}

/// Hands `terminal`'s notifications to the desktop and its agent events to the rail, for the view
/// that shows it.
fn watch(terminal: &Entity<Terminal>, window: &Window, cx: &mut Context<TerminalView>) {
    cx.subscribe_in(terminal, window, |view, _, event, window, cx| {
        if let Event::MarleyNotification { title, body } = event {
            // Claude Code's hook events are for the rail, not the desktop (#519), and for the
            // phone (#535).
            if title.as_deref() == Some(marley_terminal::AGENT_EVENT_TITLE) {
                if let Some((before, seat, session_start)) =
                    crate::agent_events::on_frame(view, body, cx)
                    && !session_start
                {
                    on_seat_change(view, before, &seat, window, cx);
                    crate::push::on_change(view, before, &seat, window, cx);
                }
            } else {
                notify(view, title.as_deref(), body, window, cx);
            }
        }
    })
    .detach();
}

/// Whether the user is looking at `view`: the focused terminal of the active window, for which
/// nothing is shown on the desktop (#478) or pushed to the phone (#535).
pub(crate) fn looking_at(view: &TerminalView, window: &Window, cx: &App) -> bool {
    window.is_window_active() && crate::rich_input::holds_focus(view, window, cx)
}

/// Marks `view` unread and shows a banner for the event its Claude Code seat made by moving from
/// `before` to its state in `seat`, unless the user is looking at the terminal (#538). A state the
/// seat was already in makes no event, so a repeated ping marks nothing.
pub(crate) fn on_seat_change(
    view: &TerminalView,
    before: State,
    seat: &Session,
    window: &Window,
    cx: &mut Context<TerminalView>,
) {
    let Some(event) = TurnEvent::of_change(before, seat.state) else {
        return;
    };
    if looking_at(view, window, cx) {
        return;
    }
    let id = cx.entity_id();
    let _newly = cx.default_global::<Attention>().unread.insert(id);
    cx.notify();
    if !banner_allowed(view, cx) {
        return;
    }
    let tag = SharedString::from(format!("marley-terminal-{}", cx.entity_id()));
    let title = event_line(&crate::push::project_name(seat), AgentKind::Claude, event);
    post(
        tag,
        SharedString::from(title),
        &banner_body(event, seat),
        window,
        cx,
    );
}

/// Marks the terminal view of `cx` unread, as an agent's event does (#538), for a long command's
/// end or a password prompt the user was not looking at (#551).
pub(crate) fn mark_unread(cx: &mut Context<TerminalView>) {
    let id = cx.entity_id();
    let _newly = cx.default_global::<Attention>().unread.insert(id);
    cx.notify();
}

/// Clears `view`'s mark once the user looks at it (#538).
fn seen(view: &TerminalView, window: &Window, cx: &mut Context<TerminalView>) {
    let id = cx.entity_id();
    if unread(id, cx) && looking_at(view, window, cx) {
        let _was_unread = cx.global_mut::<Attention>().unread.remove(&id);
        cx.notify();
    }
}

/// Whether `view`'s project may show a banner now: none in the last five seconds, of any kind.
/// A yes starts the project's cooldown.
fn banner_allowed(view: &TerminalView, cx: &mut Context<TerminalView>) -> bool {
    let project = view.marley_workspace().entity_id();
    let now = cx.background_executor().now();
    let last_banner = &mut cx.default_global::<Attention>().last_banner;
    if last_banner
        .get(&project)
        .is_some_and(|at| now.saturating_duration_since(*at) < COOLDOWN)
    {
        return false;
    }
    let _previous = last_banner.insert(project, now);
    true
}

/// Shows a desktop notification from `view`, unless the user is looking at it or its project
/// showed a banner in the last five seconds (#538). An OSC 9 gives no title, so the terminal's tab
/// names it. A long command's end and a password prompt come here too (#551).
pub(crate) fn notify(
    view: &TerminalView,
    title: Option<&str>,
    body: &str,
    window: &Window,
    cx: &mut Context<TerminalView>,
) {
    if looking_at(view, window, cx) || !banner_allowed(view, cx) {
        return;
    }
    let tag = SharedString::from(format!("marley-terminal-{}", cx.entity_id()));
    let title = title.map_or_else(
        || view.tab_content_text(0, cx),
        |title| SharedString::from(title.to_owned()),
    );
    post(tag, title, body, window, cx);
}

/// Shows a desktop notification that `view`'s agent may be stuck (#569), unless the user is
/// looking at it. Its tag is its own, so it neither replaces the terminal's other banners, such
/// as Claude Code's own, nor is replaced by them; a click shows the terminal as theirs does.
pub(crate) fn notify_stall(
    view: &TerminalView,
    title: &str,
    body: &str,
    window: &Window,
    cx: &mut Context<TerminalView>,
) {
    if looking_at(view, window, cx) {
        return;
    }
    let tag = SharedString::from(format!("marley-stall-{}", cx.entity_id()));
    post(tag, SharedString::from(title.to_owned()), body, window, cx);
}

/// Shows the notification `tag` from the terminal of `cx`, and keeps its sender for the click.
fn post(
    tag: SharedString,
    title: SharedString,
    body: &str,
    window: &Window,
    cx: &mut Context<TerminalView>,
) {
    let sender = (window.window_handle(), cx.entity().downgrade());
    cx.default_global::<Senders>().0.insert(tag.clone(), sender);
    cx.show_system_notification(SystemNotification {
        tag,
        title,
        body: SharedString::from(body.to_owned()),
        actions: Vec::new(),
    });
}

/// Shows the terminal a notification came from: its window, its project and its item. A
/// notification from a terminal closed since, or one Marley did not post, does nothing.
fn show_sender(response: &SystemNotificationResponse, cx: &mut App) {
    let Some((window, view, workspace)) = sender(response, cx) else {
        return;
    };
    window
        .update(cx, |multi_workspace, window, cx| {
            window.activate_window();
            multi_workspace.activate(workspace.clone(), None, window, cx);
            workspace.update(cx, |workspace, cx| {
                workspace.activate_item(&view, true, true, window, cx);
            });
            view.update(cx, TerminalView::clear_bell);
        })
        .log_err();
}

/// The window, the terminal view and the workspace of the notification `response` answers,
/// while the view and its workspace are still open.
fn sender(
    response: &SystemNotificationResponse,
    cx: &App,
) -> Option<(
    WindowHandle<MultiWorkspace>,
    Entity<TerminalView>,
    Entity<Workspace>,
)> {
    let (window, view) = cx.try_global::<Senders>()?.0.get(&response.tag)?.clone();
    let window = window.downcast::<MultiWorkspace>()?;
    let view = view.upgrade()?;
    let workspace = view.read(cx).marley_workspace().upgrade()?;
    Some((window, view, workspace))
}

#[cfg(test)]
#[path = "notifications_tests.rs"]
mod tests;
