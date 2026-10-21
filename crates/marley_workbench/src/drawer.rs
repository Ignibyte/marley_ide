//! The bottom drawer: Problems and Debug as tabs of the bottom dock (#838).
//!
//! In the Marley layout the bottom dock draws a strip of tabs over the panel it shows, when that is
//! Problems or Debug: each tab is a panel of the dock, so its size, focus and buttons stay Zed's.
//! Problems is a panel of Marley's drawing Zed's own diagnostics editor; the bottom row's count and
//! `diagnostics: deploy` open it instead of a center tab. Debug is Zed's debugger panel, which #774
//! keeps in the bottom dock.

use diagnostics::{MarleyDiagnosticsDeploy, ProjectDiagnosticsEditor};
use gpui::{
    Action, AnyElement, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle,
    Focusable, Pixels, Render, WeakEntity, Window, px,
};
use project::project_settings::ProjectSettings;
use settings::Settings as _;
use ui::{ButtonStyle, prelude::*};
use workspace::Workspace;
use workspace::dock::{Dock, DockPosition, MarleyDockHeader, Panel, PanelEvent};

use crate::ToggleProblems;

const ACTIVATION_PRIORITY: u32 = 23;

/// The drawer's tabs: the panels' persistent names and their labels.
const TABS: [(&str, &str); 2] = [("MarleyProblemsPanel", "Problems"), ("DebugPanel", "Debug")];

/// Gives every workspace its Problems panel, and sets the two hooks in Zed's code: the dock's
/// header and the diagnostics' deploy. [`crate::init`] calls it once.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(MarleyDockHeader(tabs));
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

/// The dock's header hook: the drawer's tabs over the bottom dock while it shows one of them.
fn tabs(
    dock: &Dock,
    handle: &WeakEntity<Dock>,
    _window: &mut Window,
    cx: &mut App,
) -> Option<AnyElement> {
    if dock.position() != DockPosition::Bottom || !crate::marley_layout(cx) {
        return None;
    }
    let shown = dock.active_panel()?.persistent_name();
    if !TABS.iter().any(|(name, _)| *name == shown) {
        return None;
    }
    let problems = dock
        .panel::<ProblemsPanel>()
        .map_or(0, |panel| panel.read(cx).count(cx));
    let colors = cx.theme().colors();
    let strip = h_flex()
        .debug_selector(|| "marley-drawer-tabs".into())
        .flex_none()
        .w_full()
        .px_2()
        .py_1()
        .gap_1()
        .border_b_1()
        .border_color(colors.border)
        .children(
            TABS.iter()
                .enumerate()
                .filter(|(_, (name, _))| dock.panel_index_for_persistent_name(name, cx).is_some())
                .map(|(index, (name, label))| {
                    let label: SharedString = if *name == "MarleyProblemsPanel" && problems > 0 {
                        format!("{label} {problems}").into()
                    } else {
                        SharedString::new_static(label)
                    };
                    let handle = handle.clone();
                    let name: &'static str = name;
                    Button::new(("marley-drawer-tab", index), label)
                        .style(ButtonStyle::Subtle)
                        .toggle_state(name == shown)
                        .on_click(move |_, window, cx| show(&handle, name, window, cx))
                }),
        );
    Some(strip.into_any_element())
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

/// Zed's diagnostics editor for the workspace's project, in the bottom dock.
pub struct ProblemsPanel {
    diagnostics: Entity<ProjectDiagnosticsEditor>,
    project: Entity<project::Project>,
    focus: FocusHandle,
}

impl std::fmt::Debug for ProblemsPanel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProblemsPanel")
            .finish_non_exhaustive()
    }
}

impl ProblemsPanel {
    fn new(
        project: Entity<project::Project>,
        workspace: WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let include_warnings = ProjectSettings::get_global(cx).diagnostics.include_warnings;
        let diagnostics = {
            let project = project.clone();
            cx.new(|cx| {
                ProjectDiagnosticsEditor::new(include_warnings, project, workspace, window, cx)
            })
        };
        cx.observe(&diagnostics, |_, _, cx| cx.notify()).detach();
        Self {
            diagnostics,
            project,
            focus: cx.focus_handle(),
        }
    }

    /// The project's errors and warnings, the Problems tab's count.
    fn count(&self, cx: &App) -> usize {
        let summary = self.project.read(cx).diagnostic_summary(false, cx);
        summary.error_count + summary.warning_count
    }
}

impl EventEmitter<PanelEvent> for ProblemsPanel {}

impl Focusable for ProblemsPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for ProblemsPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .debug_selector(|| "marley-problems".into())
            .track_focus(&self.focus)
            .size_full()
            .child(self.diagnostics.clone())
    }
}

impl Panel for ProblemsPanel {
    fn persistent_name() -> &'static str {
        "MarleyProblemsPanel"
    }

    fn panel_key() -> &'static str {
        "MarleyProblemsPanel"
    }

    fn activation_focus_handle(&self, cx: &App) -> FocusHandle {
        self.diagnostics.focus_handle(cx)
    }

    fn position(&self, _window: &Window, _cx: &App) -> DockPosition {
        DockPosition::Bottom
    }

    fn position_is_valid(&self, position: DockPosition) -> bool {
        matches!(position, DockPosition::Bottom)
    }

    fn set_position(
        &mut self,
        _position: DockPosition,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn default_size(&self, _window: &Window, _cx: &App) -> Pixels {
        px(300.)
    }

    // No button of its own: the bottom row's diagnostics count is the way in.
    fn icon(&self, _window: &Window, _cx: &App) -> Option<IconName> {
        None
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("Problems")
    }

    fn toggle_action(&self) -> Box<dyn Action> {
        Box::new(ToggleProblems)
    }

    fn activation_priority(&self) -> u32 {
        ACTIVATION_PRIORITY
    }

    fn enabled(&self, cx: &App) -> bool {
        crate::marley_layout(cx)
    }
}
