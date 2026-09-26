//! Desktop notifications (T7b), from a terminal you are not looking at.
//!
//! Zed's `Terminal` emits `Event::MarleyNotification` for each OSC 9 or OSC 777 notify a program
//! prints, and its view marks itself as a bell does. [`init`] subscribes every terminal view to
//! its terminal and shows a desktop notification unless the view is the focused one of the
//! active window. Clicking the notification shows that terminal. A notify titled
//! `marley-event` carries a Claude Code hook event instead, which goes to the rail
//! ([`crate::agent_events`]).

use std::collections::HashMap;

use gpui::{
    AnyWindowHandle, App, Context, Entity, Focusable as _, Global, SharedString,
    SystemNotification, SystemNotificationResponse, WeakEntity, Window, WindowHandle,
};
use terminal::Event;
use terminal_view::TerminalView;
use util::ResultExt as _;
use workspace::item::Item as _;
use workspace::{MultiWorkspace, Workspace};

/// The terminal each notification came from, by its tag, for the click.
#[derive(Default)]
struct Senders(HashMap<SharedString, (AnyWindowHandle, WeakEntity<TerminalView>)>);

impl Global for Senders {}

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
            cx.subscribe_in(&terminal, window, |view, _, event, window, cx| {
                if let Event::MarleyNotification { title, body } = event {
                    // Claude Code's hook events are for the rail, not the desktop (#519), and
                    // for the phone (#535).
                    if title.as_deref() == Some(marley_terminal::AGENT_EVENT_TITLE) {
                        if let Some((before, seat)) = crate::agent_events::on_frame(view, body, cx)
                        {
                            crate::push::on_change(view, before, &seat, window, cx);
                        }
                    } else {
                        notify(view, title.as_deref(), body, window, cx);
                    }
                }
            })
            .detach();
            let view = cx.entity_id();
            cx.on_release(move |_, cx| crate::agent_events::forget(view, cx))
                .detach();
        },
    )
    .detach();
}

/// Whether the user is looking at `view`: the focused terminal of the active window, for which
/// nothing is shown on the desktop (#478) or pushed to the phone (#535).
pub(crate) fn looking_at(view: &TerminalView, window: &Window, cx: &App) -> bool {
    window.is_window_active() && view.focus_handle(cx).contains_focused(window, cx)
}

/// Shows a desktop notification from `view`, unless the user is looking at it. An OSC 9 gives no
/// title, so the terminal's tab names it.
fn notify(
    view: &TerminalView,
    title: Option<&str>,
    body: &str,
    window: &Window,
    cx: &mut Context<TerminalView>,
) {
    if looking_at(view, window, cx) {
        return;
    }
    let tag = SharedString::from(format!("marley-terminal-{}", cx.entity_id()));
    let title = title.map_or_else(
        || view.tab_content_text(0, cx),
        |title| SharedString::from(title.to_owned()),
    );
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
