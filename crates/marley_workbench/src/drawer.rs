//! The bottom drawer: Problems and Debug as tabs of a dock (#838).
//!
//! In the Marley layout the dock that shows Problems or Debug draws a strip of tabs over it: each
//! tab is a panel of the dock, so its size and focus stay Zed's. Problems is Marley's list
//! (`problems_panel`, #922); the bottom row's count and `diagnostics: deploy` open it instead of a
//! center tab. Debug is Zed's debugger panel, which #774 keeps in the bottom dock. The strip
//! carries each tab's pill and the drawer's own Move, Maximize and Close (#922); at the bottom the
//! dock draws as a rounded card with a grab handle.

use ::fs::Fs;
use diagnostics::MarleyDiagnosticsDeploy;
use gpui::{AnyElement, App, AppContext as _, Context, Entity, WeakEntity, Window, px};
use project::Project;
use ui::{ButtonStyle, prelude::*};
use workspace::dock::{Dock, DockPosition, DockShape, MarleyDockHeader, MarleyDockShape};
use workspace::{ToggleZoom, Workspace};

use crate::ToggleProblems;
use crate::problems_panel::ProblemsPanel;

/// The drawer's tabs: the panels' persistent names and their labels.
const TABS: [(&str, &str); 2] = [("MarleyProblemsPanel", "Problems"), ("DebugPanel", "Debug")];

/// Gives every workspace its Problems panel, and sets the two hooks in Zed's code: the dock's
/// header and the diagnostics' deploy. [`crate::init`] calls it once.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(MarleyDockHeader(tabs));
    cx.set_global(MarleyDockShape(shape));
    cx.set_global(MarleyDiagnosticsDeploy(deploy));
    cx.observe_new(
        |workspace: &mut Workspace, window: Option<&mut Window>, cx: &mut Context<Workspace>| {
            workspace.register_action(|workspace, _: &ToggleProblems, window, cx| {
                workspace.toggle_panel_focus::<ProblemsPanel>(window, cx);
            });
            let Some(window) = window else {
                return;
            };
            let project = workspace.project().clone();
            let weak_workspace = cx.weak_entity();
            let panel = cx.new(|cx| ProblemsPanel::new(project, weak_workspace, window, cx));
            workspace.add_panel(panel, window, cx);
        },
    )
    .detach();
}

/// `deploy`'s hook: in the Marley layout the diagnostics open in the drawer, not a center tab.
fn deploy(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) -> bool {
    if !crate::marley_layout(cx) || workspace.panel::<ProblemsPanel>(cx).is_none() {
        return false;
    }
    workspace.focus_panel::<ProblemsPanel>(window, cx);
    true
}

/// Whether `dock` shows one of the drawer's tabs in the Marley layout.
fn shows_drawer(dock: &Dock, cx: &App) -> bool {
    crate::marley_layout(cx)
        && dock.active_panel().is_some_and(|panel| {
            TABS.iter()
                .any(|(name, _)| *name == panel.persistent_name())
        })
}

/// The dock's shape hook: the drawer at the bottom as a rounded card (#922, D6).
fn shape(dock: &Dock, cx: &App) -> DockShape {
    if dock.position() == DockPosition::Bottom && shows_drawer(dock, cx) {
        DockShape::Card
    } else {
        DockShape::Flush
    }
}

/// What a debug session is doing, for the Debug tab's pill (#922).
enum DebugState {
    Paused,
    Running,
}

/// The project's debug state: paused while a session's thread is stopped, running while a session
/// is live, none without one.
fn debug_state(project: &Entity<Project>, cx: &App) -> Option<DebugState> {
    let sessions: Vec<_> = project
        .read(cx)
        .dap_store()
        .read(cx)
        .sessions()
        .filter(|session| !session.read(cx).is_terminated())
        .cloned()
        .collect();
    if sessions.is_empty() {
        return None;
    }
    if sessions
        .iter()
        .any(|session| session.read(cx).any_stopped_thread())
    {
        Some(DebugState::Paused)
    } else {
        Some(DebugState::Running)
    }
}

/// A tab's pill: its words in `color` on a tint of it (#922).
fn tab_pill(words: SharedString, color: Color, cx: &App) -> AnyElement {
    let tint = color.color(cx);
    div()
        .px_1p5()
        .rounded_full()
        .bg(tint.opacity(0.15))
        .child(Label::new(words).size(LabelSize::XSmall).color(color))
        .into_any_element()
}

/// The dock's header hook: the drawer's tabs, each with its pill, and the drawer's own Move,
/// Maximize and Close, over the dock while it shows one of them; with a grab handle at the
/// bottom (#838, #922).
fn tabs(
    dock: &Dock,
    handle: &WeakEntity<Dock>,
    window: &mut Window,
    cx: &mut App,
) -> Option<AnyElement> {
    if !shows_drawer(dock, cx) {
        return None;
    }
    let active = dock.active_panel()?;
    let shown = active.persistent_name();
    let zoomed = active.is_zoomed(window, cx);
    let at_bottom = dock.position() == DockPosition::Bottom;
    let problems_panel = dock.panel::<ProblemsPanel>();
    let (problems, errors) = problems_panel
        .as_ref()
        .map_or((0, false), |panel| panel.read(cx).count(cx));
    let debug = problems_panel
        .as_ref()
        .and_then(|panel| debug_state(panel.read(cx).project(), cx));
    let colors = cx.theme().colors();
    let tab_buttons = TABS
        .iter()
        .enumerate()
        .filter(|(_, (name, _))| dock.panel_index_for_persistent_name(name, cx).is_some())
        .map(|(index, (name, label))| {
            let pill = match *name {
                "MarleyProblemsPanel" if problems > 0 => Some(tab_pill(
                    problems.to_string().into(),
                    if errors { Color::Error } else { Color::Muted },
                    cx,
                )),
                "DebugPanel" => debug.as_ref().map(|state| match state {
                    DebugState::Paused => {
                        tab_pill(SharedString::new_static("paused"), Color::Warning, cx)
                    }
                    DebugState::Running => {
                        tab_pill(SharedString::new_static("running"), Color::Success, cx)
                    }
                }),
                _ => None,
            };
            let handle = handle.clone();
            let name: &'static str = name;
            h_flex()
                .gap_1()
                .child(
                    Button::new(("marley-drawer-tab", index), *label)
                        .style(ButtonStyle::Subtle)
                        .toggle_state(name == shown)
                        .on_click(move |_, window, cx| show(&handle, name, window, cx)),
                )
                .children(pill)
        });
    let strip = h_flex()
        .debug_selector(|| "marley-drawer-tabs".into())
        .flex_none()
        .w_full()
        // In a side dock the buttons wrap under the tabs (#922).
        .flex_wrap()
        .px_2()
        .py_1()
        .gap_1()
        .border_b_1()
        .border_color(colors.border)
        .children(tab_buttons)
        .child(div().flex_1())
        .child(drawer_buttons(handle, at_bottom, zoomed));
    // At the bottom a short bar on the top edge says the drawer can be pulled (#922).
    let grab = at_bottom.then(|| {
        h_flex()
            .w_full()
            .pt_1()
            .justify_center()
            .child(div().w(px(36.)).h(px(4.)).rounded_full().bg(colors.border))
    });
    Some(
        v_flex()
            .flex_none()
            .w_full()
            .children(grab)
            .child(strip)
            .into_any_element(),
    )
}

/// The drawer's own buttons at the strip's right: Move, Maximize or Restore, and Close (#922).
fn drawer_buttons(handle: &WeakEntity<Dock>, at_bottom: bool, zoomed: bool) -> impl IntoElement {
    let (move_handle, zoom_handle, close_handle) = (handle.clone(), handle.clone(), handle.clone());
    h_flex()
        .gap_1()
        .child(
            Button::new(
                "marley-drawer-move",
                if at_bottom {
                    "Move to the Side"
                } else {
                    "Move to the Bottom"
                },
            )
            .style(ButtonStyle::Subtle)
            .label_size(LabelSize::Small)
            .on_click(move |_, _, cx| move_drawer(&move_handle, at_bottom, cx)),
        )
        .child(
            Button::new(
                "marley-drawer-zoom",
                if zoomed { "Restore" } else { "Maximize" },
            )
            .style(ButtonStyle::Subtle)
            .label_size(LabelSize::Small)
            .on_click(move |_, window, cx| zoom(&zoom_handle, window, cx)),
        )
        .child(
            IconButton::new("marley-drawer-close", IconName::Close)
                .icon_size(IconSize::Small)
                .tooltip(ui::Tooltip::text("Close the Drawer"))
                .on_click(move |_, window, cx| {
                    close_handle
                        .update(cx, |dock, cx| dock.set_open(false, window, cx))
                        .ok();
                }),
        )
}

/// Moves the drawer to the right dock, or back to the bottom, through `debugger.dock`, which
/// Problems follows (#922, D3).
fn move_drawer(dock: &WeakEntity<Dock>, to_side: bool, cx: &App) {
    let position = if to_side {
        settings::DockPosition::Right
    } else {
        settings::DockPosition::Bottom
    };
    if dock.upgrade().is_none() {
        return;
    }
    settings::update_settings_file(<dyn Fs>::global(cx), cx, move |content, _| {
        content.debugger.get_or_insert_default().dock = Some(position);
    });
}

/// Maximizes the shown panel, or restores it, through Zed's zoom (#922, D4).
fn zoom(dock: &WeakEntity<Dock>, window: &mut Window, cx: &mut App) {
    let Some(focus) = dock
        .upgrade()
        .and_then(|dock| dock.read(cx).active_panel().cloned())
        .map(|panel| panel.panel_focus_handle(cx))
    else {
        return;
    };
    window.focus(&focus, cx);
    window.dispatch_action(Box::new(ToggleZoom), cx);
}

/// Shows the dock's panel `name` and gives it the focus.
fn show(dock: &WeakEntity<Dock>, name: &str, window: &mut Window, cx: &mut App) {
    dock.update(cx, |dock, cx| {
        let Some(index) = dock.panel_index_for_persistent_name(name, cx) else {
            return;
        };
        dock.activate_panel(index, window, cx);
        if let Some(panel) = dock.active_panel() {
            window.focus(&panel.activation_focus_handle(cx), cx);
        }
    })
    .ok();
}
