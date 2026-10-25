//! The panel buttons as one labeled group (#862).
//!
//! Zed draws three groups of unlabeled panel buttons in the status bar, one per dock. In the Marley
//! layout they step aside (`workspace::dock::MarleyPanelButtons`) for this item at the bottom row's
//! right, as the `BottomRow` board draws it: every dock's panels, each with its icon and its name,
//! the one showing filled, and an accent dot on a panel open as a center tab (#747). A click does
//! what Zed's button does. The Zed layout keeps Zed's buttons.

use std::collections::HashSet;

use gpui::{
    App, Context, Entity, EntityId, Focusable as _, Render, Subscription, WeakEntity, Window, px,
};
use settings::SettingsStore;
use ui::{ButtonLike, ContextMenu, Indicator, PopoverMenu, Tooltip, prelude::*};
use workspace::dock::{Dock, DockPosition, MarleyPanelButtons};
use workspace::{HideStatusItem, ItemHandle, StatusItemView, Workspace};

use crate::panel_tabs::PanelTab;

/// The window width from which the buttons show their names (#862, D2).
const NAMED_FROM: f32 = 1200.;

/// Makes Zed's panel buttons step aside in the Marley layout; [`crate::init`] calls it once.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(MarleyPanelButtons(crate::marley_layout));
    // The problems count's card above the row (#863).
    cx.set_global(diagnostics::items::MarleyProblemsCard(crate::marley_layout));
}

/// Where the panel `persistent_name` opens in the Marley layout (#863, D2).
fn where_it_opens(persistent_name: &str, position: DockPosition) -> &'static str {
    if crate::panel_tabs::is_moved(persistent_name) {
        "tab"
    } else if persistent_name == "Project Panel" {
        "beside the rail"
    } else {
        match position {
            DockPosition::Left => "left dock",
            DockPosition::Right => "right dock",
            DockPosition::Bottom => "bottom drawer",
        }
    }
}

/// The Panels card (#863): each panel, where it opens and its key, in the group's order; a row
/// runs the panel's toggle.
fn panels_card(
    workspace: &WeakEntity<Workspace>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<ContextMenu> {
    let mut panels: Vec<(usize, String, Box<dyn gpui::Action>)> = workspace
        .upgrade()
        .map(|workspace| {
            workspace
                .read(cx)
                .all_docks()
                .into_iter()
                .flat_map(|dock| {
                    dock.read(cx)
                        .panel_handles()
                        .filter(|panel| panel.persistent_name() != "DebugPanel")
                        .filter_map(|panel| {
                            let tooltip = panel.icon_tooltip(window, cx)?;
                            let (place, name) = board_place(panel.persistent_name())
                                .unwrap_or((BOARD.len(), tooltip));
                            let place_words =
                                where_it_opens(panel.persistent_name(), panel.position(window, cx));
                            Some((
                                place,
                                format!("{name} · {place_words}"),
                                panel.toggle_action(window, cx),
                            ))
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        })
        .unwrap_or_default();
    panels.sort_by_key(|(place, _, _)| *place);
    ContextMenu::build(window, cx, move |menu, _, _| {
        panels
            .into_iter()
            .fold(menu.header("Panels"), |menu, (_, label, action)| {
                menu.action(label, action)
            })
    })
}

/// The panels the board names, by their persistent names, in the board's order (#862, D1).
const BOARD: [(&str, &str); 10] = [
    ("Project Panel", "Files"),
    ("GitPanel", "Changes"),
    ("Outline Panel", "Outline"),
    ("MarleyKnowledgePanel", "Knowledge"),
    ("MarleyFleetPanel", "Fleet"),
    ("MarleyContainersPanel", "Containers"),
    ("MarleyProblemsPanel", "Problems"),
    ("CollabPanel", "Collab"),
    ("AgentPanel", "Agent"),
    ("TerminalPanel", "Terminal"),
];

/// The board's name for the panel `persistent_name`, which its tab takes too (#906, D2).
pub(crate) fn board_label(persistent_name: &str) -> Option<&'static str> {
    board_place(persistent_name).map(|(_, label)| label)
}

/// The board's place and name for the panel `persistent_name`; a panel the board does not name
/// comes after, under its tooltip.
fn board_place(persistent_name: &str) -> Option<(usize, &'static str)> {
    BOARD
        .iter()
        .position(|(name, _)| *name == persistent_name)
        .and_then(|place| BOARD.get(place).map(|(_, label)| (place, *label)))
}

/// The group, while the workspace's layout is Marley's.
pub(crate) struct PanelGroup {
    workspace: WeakEntity<Workspace>,
    _observed: Vec<Subscription>,
}

impl PanelGroup {
    pub(crate) fn new(workspace: &Workspace, cx: &mut Context<Self>) -> Self {
        let mut observed: Vec<Subscription> = workspace
            .all_docks()
            .into_iter()
            .map(|dock| cx.observe(dock, |_, _, cx| cx.notify()))
            .collect();
        observed.push(cx.observe_global::<SettingsStore>(|_, cx| cx.notify()));
        Self {
            workspace: workspace.weak_handle(),
            _observed: observed,
        }
    }

    /// One dock's buttons, from the `first` index of the group's ids.
    fn dock_buttons(
        dock: &Entity<Dock>,
        first: usize,
        tabbed: &HashSet<EntityId>,
        named: bool,
        window: &Window,
        cx: &App,
    ) -> Vec<(usize, AnyElement)> {
        let dock_entity = dock;
        let dock = dock.read(cx);
        dock.panel_handles()
            .enumerate()
            .filter_map(|(index, panel)| {
                // Debug keeps its own button at the row's left (#774).
                if panel.persistent_name() == "DebugPanel" {
                    return None;
                }
                let icon = panel.icon(window, cx)?;
                let tooltip = panel.icon_tooltip(window, cx)?;
                let showing = dock.is_open() && dock.active_panel_index() == Some(index);
                let action = if showing {
                    dock.toggle_action()
                } else {
                    panel.toggle_action(window, cx)
                };
                let (place, name) =
                    board_place(panel.persistent_name()).unwrap_or((BOARD.len(), tooltip));
                let dot = tabbed.contains(&panel.panel_id());
                let focus = dock_entity.focus_handle(cx);
                Some((
                    place,
                    ButtonLike::new(("marley-panel-group", first + index))
                        .style(ButtonStyle::Subtle)
                        .toggle_state(showing)
                        .child(
                            h_flex()
                                .gap_1()
                                .child(Icon::new(icon).size(IconSize::Small).color(if showing {
                                    Color::Default
                                } else {
                                    Color::Muted
                                }))
                                .when(named, |button| {
                                    button.child(Label::new(name).size(LabelSize::Small).color(
                                        if showing {
                                            Color::Default
                                        } else {
                                            Color::Muted
                                        },
                                    ))
                                })
                                .when(dot, |button| {
                                    button.child(Indicator::dot().color(Color::Accent))
                                }),
                        )
                        .tooltip({
                            let action = action.boxed_clone();
                            move |_, cx| Tooltip::for_action(tooltip, &*action, cx)
                        })
                        .on_click(move |_, window, cx| {
                            window.focus(&focus, cx);
                            window.dispatch_action(action.boxed_clone(), cx);
                        })
                        .into_any_element(),
                ))
            })
            .collect()
    }
}

impl Render for PanelGroup {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(workspace) = self
            .workspace
            .upgrade()
            .filter(|_| crate::marley_layout(cx))
        else {
            return h_flex();
        };
        let workspace = workspace.read(cx);
        let tabbed: HashSet<EntityId> = workspace
            .items_of_type::<PanelTab>(cx)
            .map(|tab| tab.read(cx).panel_id())
            .collect();
        let named = window.viewport_size().width >= px(NAMED_FROM);
        let mut buttons = Vec::new();
        for (dock_index, dock) in workspace.all_docks().into_iter().enumerate() {
            buttons.extend(Self::dock_buttons(
                dock,
                dock_index * 100,
                &tabbed,
                named,
                window,
                cx,
            ));
        }
        buttons.sort_by_key(|(place, _)| *place);
        let card_workspace = self.workspace.clone();
        h_flex()
            .gap_0p5()
            .children(buttons.into_iter().map(|(_, button)| button))
            .child(
                PopoverMenu::new("marley-panels-card")
                    .trigger_with_tooltip(
                        IconButton::new("marley-panels-card-button", IconName::ChevronUp)
                            .icon_size(IconSize::XSmall),
                        Tooltip::text("Where each panel opens, and its key"),
                    )
                    .anchor(gpui::Anchor::BottomRight)
                    .menu(move |window, cx| Some(panels_card(&card_workspace, window, cx))),
            )
    }
}

impl StatusItemView for PanelGroup {
    fn set_active_pane_item(
        &mut self,
        _: Option<&dyn ItemHandle>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // A panel's tab opening or closing moves the active item: its dot follows.
        cx.notify();
    }

    /// Each panel hides through its own setting, as Zed's buttons do.
    fn hide_setting(&self, _: &App) -> Option<HideStatusItem> {
        None
    }
}
