//! An agent thread's width as a step: a third, half, two thirds or the whole window (#798, #927).
//!
//! The width button in a thread's message editor (`agent_ui`'s `thread_view.rs`) shows the step
//! nearest the width of the thread's place, labelled by `width_label` through
//! `MarleyThreadHost`, and runs [`CycleAgentPanelWidth`] for the thread; the palette runs it for
//! the thread in front. The place is the Agent Panel's dock, or the pane that holds the thread's
//! tab, which is where the Marley layout shows threads (#734). Each run steps the place to the next
//! of a third, half and two thirds of the window; from the widest the place zooms over the whole
//! window, and from the zoom it comes back at a third.
//!
//! The dock's width is its share of the window, set through the workspace's dock sizing, the path a
//! drag of its edge takes: with the panel's flexible size on (`agent.flexible`, Zed's default) it
//! keeps the share as the window resizes, and the workspace keeps it across a restart. A pane's
//! width is its share of the nearest row of the center that holds it, the flex a drag between two
//! panes changes; the other panes of the row keep their proportions in what is left. A pane alone
//! in its row has nothing to share its width with and gets no button.

use agent_ui::AgentPanel;
use gpui::{Action, App, Context, Entity, EntityId, SharedString, Window};
use schemars::JsonSchema;
use serde::Deserialize;
use workspace::{Pane, Panel as _, ToggleZoom, Workspace, dock::DockPosition};

use crate::thread_tab::ThreadTab;

/// Steps an agent thread's width: a third of the window, half, two thirds, the whole window.
///
/// After the whole window it comes back to a third. With no thread named, it steps the thread in
/// front: the active pane's thread tab, else the Agent Panel's thread.
#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Action)]
#[action(namespace = marley)]
pub struct CycleAgentPanelWidth {
    /// The entity id of the thread's view, as the width button sends it.
    pub thread: Option<u64>,
}

/// `CycleAgentPanelWidth`'s fields as the button and a keymap write them, read through this
/// struct for the reason `rusty::OpenPage` gives (clippy's `unsafe_derive_deserialize` on a
/// derived `Deserialize`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CycleAgentPanelWidthFields {
    #[serde(default)]
    thread: Option<u64>,
}

impl<'de> Deserialize<'de> for CycleAgentPanelWidth {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let CycleAgentPanelWidthFields { thread } =
            CycleAgentPanelWidthFields::deserialize(deserializer)?;
        Ok(Self { thread })
    }
}

/// The widths, as shares of the window, narrowest first.
const SHARES: [f32; 3] = [1.0 / 3.0, 0.5, 2.0 / 3.0];

/// How much wider than the place a share must be to be the next: a width dragged to 49% goes to
/// 50%, and 50% goes to 66%.
const STEP_MARGIN: f32 = 0.01;

/// Where a thread is shown.
enum Place {
    /// The Agent Panel, in the dock at this position.
    Panel(Entity<AgentPanel>, DockPosition),
    /// The pane whose active tab is the thread's.
    Pane(Entity<Pane>),
}

/// Registers the action on every workspace.
pub(crate) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(cycle);
    })
    .detach();
}

/// A width step's share of the window and the button's word for it (#927).
struct WidthStep {
    share: f32,
    label: &'static str,
}

/// The steps the button names, narrowest first; the whole window is the zoom.
const STEPS: [WidthStep; 4] = [
    WidthStep {
        share: 1.0 / 3.0,
        label: "\u{2153}",
    },
    WidthStep {
        share: 0.5,
        label: "\u{bd}",
    },
    WidthStep {
        share: 2.0 / 3.0,
        label: "\u{2154}",
    },
    WidthStep {
        share: 1.0,
        label: WHOLE_WINDOW,
    },
];

/// The widest step's word, and a zoomed place's.
const WHOLE_WINDOW: &str = "Whole window";

/// The word of the step nearest `share` (#927, D1): a width dragged to 45% reads "½".
fn step_label(share: f32) -> &'static str {
    STEPS
        .iter()
        .min_by(|first, second| {
            (first.share - share)
                .abs()
                .total_cmp(&(second.share - share).abs())
        })
        .map_or(WHOLE_WINDOW, |step| step.label)
}

/// The width button's label for the thread view `view`: the step nearest the dock's share of the
/// window, or the pane's share of its row, "Whole window" while the place is zoomed, `None` when
/// the button has nothing to step (the thread is not shown, or its pane is alone in its row).
pub(crate) fn width_label(
    workspace: &Workspace,
    view: EntityId,
    window: &Window,
    cx: &App,
) -> Option<SharedString> {
    let share = match place(workspace, Some(view), window, cx)? {
        Place::Panel(panel, position) => {
            if panel.read(cx).is_zoomed(window, cx) {
                return Some(SharedString::new_static(WHOLE_WINDOW));
            }
            workspace.marley_dock_width_share(position, window, cx)?
        }
        Place::Pane(pane) => {
            if pane.read(cx).is_zoomed() {
                return Some(SharedString::new_static(WHOLE_WINDOW));
            }
            workspace.marley_pane_row_share(&pane)?
        }
    };
    Some(SharedString::new_static(step_label(share.clamp(0.0, 1.0))))
}

/// Where the workspace shows the thread view `view`, or with `None` the thread in front.
fn place(
    workspace: &Workspace,
    view: Option<EntityId>,
    window: &Window,
    cx: &App,
) -> Option<Place> {
    let shows_thread = |pane: &Entity<Pane>| {
        pane.read(cx)
            .active_item()
            .and_then(|item| item.downcast::<ThreadTab>())
            .is_some_and(|tab| view.is_none_or(|view| tab.read(cx).conversation_view_id() == view))
    };
    let active = workspace.active_pane();
    if view.is_none() && shows_thread(active) {
        return Some(Place::Pane(active.clone()));
    }
    if view.is_some()
        && let Some(pane) = workspace.panes().iter().find(|pane| shows_thread(pane))
    {
        return Some(Place::Pane(pane.clone()));
    }
    let panel = workspace.panel::<AgentPanel>(cx)?;
    let shown = panel.read(cx).active_conversation_view()?.entity_id();
    if view.is_some_and(|view| view != shown) {
        return None;
    }
    let position = panel.read(cx).position(window, cx);
    Some(Place::Panel(panel, position))
}

/// The share after `current`, or `None` when the next step is the whole window.
fn next_share(current: f32) -> Option<f32> {
    SHARES
        .into_iter()
        .find(|share| *share > current + STEP_MARGIN)
}

fn cycle(
    workspace: &mut Workspace,
    action: &CycleAgentPanelWidth,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let view = action.thread.map(EntityId::from);
    match place(workspace, view, window, cx) {
        Some(Place::Panel(panel, position)) => {
            if panel.read(cx).is_zoomed(window, cx) {
                panel.update(cx, |panel, cx| panel.toggle_zoom(&ToggleZoom, window, cx));
                workspace.marley_set_dock_width_share(position, SHARES[0], window, cx);
                return;
            }
            // A closed dock has no width to step.
            let Some(current) = workspace.marley_dock_width_share(position, window, cx) else {
                return;
            };
            match next_share(current) {
                Some(share) => workspace.marley_set_dock_width_share(position, share, window, cx),
                None => panel.update(cx, |panel, cx| panel.toggle_zoom(&ToggleZoom, window, cx)),
            }
        }
        Some(Place::Pane(pane)) => {
            if pane.read(cx).is_zoomed() {
                pane.update(cx, |pane, cx| pane.toggle_zoom(&ToggleZoom, window, cx));
                workspace.marley_set_pane_row_share(&pane, SHARES[0], cx);
                return;
            }
            let Some(current) = workspace.marley_pane_row_share(&pane) else {
                return;
            };
            match next_share(current) {
                Some(share) => workspace.marley_set_pane_row_share(&pane, share, cx),
                None => pane.update(cx, |pane, cx| pane.toggle_zoom(&ToggleZoom, window, cx)),
            }
        }
        None => {}
    }
}
