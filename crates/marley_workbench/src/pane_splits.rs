//! Splits in the Marley layout (#785).
//!
//! A split of a center terminal, which Zed answers with a new shell in the new pane, opens Home
//! there instead; Home itself splits into another Home ([`crate::home_page`]). A center tab's
//! right-click menu moves the tab into a new pane on any side. The Zed layout keeps Zed's splits,
//! and a dock's panes split their own way.

use std::sync::Arc;

use gpui::{
    App, Context, Entity, EntityId, Focusable as _, InteractiveElement as _, WeakEntity, Window,
};
use terminal_view::TerminalView;
use ui::ContextMenu;
use util::ResultExt as _;
use workspace::pane::{
    MarleyTabMenu, SplitDown, SplitHorizontal, SplitLeft, SplitMode, SplitRight, SplitUp,
    SplitVertical,
};
use workspace::{Pane, SplitDirection, Workspace};

use crate::home_page;

/// Catches the split actions on every workspace and adds the Split entries to the tab menu.
/// [`crate::init`] calls it once.
pub(crate) fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action_renderer(|div, _, _, cx| {
            div.capture_action(cx.listener(|workspace, split: &SplitLeft, window, cx| {
                split_to_home(workspace, SplitDirection::Left, split.mode, window, cx);
            }))
            .capture_action(cx.listener(|workspace, split: &SplitRight, window, cx| {
                split_to_home(workspace, SplitDirection::Right, split.mode, window, cx);
            }))
            .capture_action(cx.listener(|workspace, split: &SplitUp, window, cx| {
                split_to_home(workspace, SplitDirection::Up, split.mode, window, cx);
            }))
            .capture_action(cx.listener(|workspace, split: &SplitDown, window, cx| {
                split_to_home(workspace, SplitDirection::Down, split.mode, window, cx);
            }))
            .capture_action(
                cx.listener(|workspace, split: &SplitHorizontal, window, cx| {
                    let direction = SplitDirection::horizontal(cx);
                    split_to_home(workspace, direction, split.mode, window, cx);
                }),
            )
            .capture_action(cx.listener(
                |workspace, split: &SplitVertical, window, cx| {
                    let direction = SplitDirection::vertical(cx);
                    split_to_home(workspace, direction, split.mode, window, cx);
                },
            ))
        });
    })
    .detach();
    cx.set_global(MarleyTabMenu(Arc::new(tab_menu)));
}

/// A split of the center's active terminal, from a key, the palette or the split button: Home goes
/// in the new pane in place of the shell Zed's clone of a terminal starts. Any other split goes on
/// to Zed.
fn split_to_home(
    workspace: &mut Workspace,
    direction: SplitDirection,
    mode: SplitMode,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if mode != SplitMode::ClonePane || !crate::marley_layout(cx) {
        return;
    }
    // The pane the split is for holds the focus: the split button's menu focuses its own pane
    // first, which need not be the active one yet. A dock's pane, the terminal dock's among them,
    // is none of these and splits its own way.
    let Some(pane) = workspace
        .panes()
        .iter()
        .find(|pane| pane.focus_handle(cx).contains_focused(window, cx))
        .cloned()
    else {
        return;
    };
    let terminal = pane
        .read(cx)
        .active_item()
        .is_some_and(|item| item.downcast::<TerminalView>().is_some());
    if !terminal {
        return;
    }
    cx.stop_propagation();
    let new_pane = workspace.split_pane(pane, direction, window, cx);
    add_home(workspace, &new_pane, true, window, cx);
}

/// Puts a new Home page in `pane`, in front, with the focus when `focus` says so.
fn add_home(
    workspace: &Workspace,
    pane: &Entity<Pane>,
    focus: bool,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let home = home_page::new_page(workspace, window, cx);
    pane.update(cx, |pane, cx| {
        pane.add_item(Box::new(home), true, focus, None, window, cx);
    });
}

/// Split Right, Left, Up and Down at the end of a center tab's right-click menu, in the Marley
/// layout: each moves the tab into a new pane on that side.
fn tab_menu(
    menu: ContextMenu,
    pane: &Entity<Pane>,
    workspace: &WeakEntity<Workspace>,
    item_id: EntityId,
    _window: &mut Window,
    cx: &mut App,
) -> ContextMenu {
    let in_center = workspace
        .upgrade()
        .is_some_and(|workspace| workspace.read(cx).panes().contains(pane));
    if !in_center || !crate::marley_layout(cx) {
        return menu;
    }
    [
        ("Split Right", SplitDirection::Right),
        ("Split Left", SplitDirection::Left),
        ("Split Up", SplitDirection::Up),
        ("Split Down", SplitDirection::Down),
    ]
    .into_iter()
    .fold(menu.separator(), |menu, (label, direction)| {
        let workspace = workspace.clone();
        let pane = pane.downgrade();
        menu.entry(label, None, move |window, cx| {
            split_out(&workspace, &pane, item_id, direction, window, cx);
        })
    })
}

/// Moves the tab `item_id` of `pane` into a new pane on `direction`'s side. A pane's only tab stays
/// where it is and the new pane opens on the other side, as Zed's move does with one tab, so the
/// tab still ends up on the side picked; the new pane shows Home rather than nothing, and the tab
/// keeps the focus.
fn split_out(
    workspace: &WeakEntity<Workspace>,
    pane: &WeakEntity<Pane>,
    item_id: EntityId,
    direction: SplitDirection,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(pane) = pane.upgrade() else {
        return;
    };
    workspace
        .update(cx, |workspace, cx| {
            let (index, count) = {
                let pane = pane.read(cx);
                let index = pane.items().position(|item| item.item_id() == item_id);
                (index, pane.items_len())
            };
            let Some(index) = index else {
                return;
            };
            if count > 1 {
                pane.update(cx, |pane, cx| {
                    pane.activate_item(index, true, true, window, cx);
                });
                workspace.split_and_move(pane, direction, window, cx);
            } else {
                let new_pane = workspace.split_pane(pane.clone(), direction.opposite(), window, cx);
                add_home(workspace, &new_pane, false, window, cx);
                pane.update(cx, |pane, cx| {
                    pane.activate_item(index, true, true, window, cx);
                });
            }
        })
        .log_err();
}
