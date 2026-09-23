//! Terminal routing: in the Marley layout nothing opens the bottom Terminal Panel.
//!
//! Tasks run through a task provider of this crate's own, which sends them to center terminals.
//! `workspace::NewTerminal` and `workspace::OpenTerminal` are caught before the panel's own
//! handlers see them, and so are the panel's toggles and the bottom dock's, which switch between
//! the code and the center terminals instead. A folder project opened fresh starts with a center
//! terminal at its root. In the Zed layout all of it passes straight through, so the fork behaves
//! as upstream.

use std::path::PathBuf;
use std::process::ExitStatus;

use gpui::{
    App, AsyncWindowContext, Context, Entity, EntityId, Focusable as _, InteractiveElement as _,
    Task, WeakEntity, Window,
};
use task::{RevealTarget, SpawnInTerminal};
use terminal::Terminal;
use terminal_view::TerminalView;
use terminal_view::terminal_panel::{TerminalPanel, Toggle, ToggleFocus};
use workspace::item::ItemHandle;
use workspace::notifications::DetachAndPromptErr as _;
use workspace::{NewTerminal, OpenTerminal, Pane, ToggleBottomDock, Workspace};

use crate::marley_layout;

/// Installs the routing on every workspace. [`crate::init`] calls it once, before any window opens.
///
/// The actions are caught at the workspace's root in the capture phase, and the task provider
/// replaces the panel's once the panel is added, after the panel's own load has installed Zed's.
pub fn init(cx: &App) {
    cx.observe_new(
        |workspace: &mut Workspace, window, cx: &mut Context<Workspace>| {
            workspace.register_action_renderer(|div, _, _, cx| {
                div.capture_action(cx.listener(new_terminal))
                    .capture_action(cx.listener(open_terminal))
                    .capture_action(cx.listener(toggle::<Toggle>))
                    .capture_action(cx.listener(toggle::<ToggleFocus>))
                    .capture_action(cx.listener(toggle_bottom_dock))
            });
            cx.subscribe_self(|workspace, event: &workspace::Event, _| {
                if let workspace::Event::PanelAdded(panel) = event
                    && let Ok(panel) = panel.clone().downcast::<TerminalPanel>()
                {
                    let routed = RoutedTerminals {
                        workspace: workspace.weak_handle(),
                        panel,
                    };
                    workspace.set_terminal_provider(routed);
                }
            })
            .detach();
            seed_first_terminal(workspace, window, cx);
        },
    )
    .detach();
}

/// A folder project opened fresh in the Marley layout starts with a terminal at its root, since
/// the terminal is the layout's main surface. A project opened from saved state keeps what it
/// saved, terminals or none, and a workspace made any other way gets nothing.
///
/// New-workspace observers run after the update that made the workspace, so it is in its window
/// by now, and a file opened with the project takes focus after the terminal.
fn seed_first_terminal(
    workspace: &mut Workspace,
    window: Option<&mut Window>,
    cx: &mut Context<Workspace>,
) {
    let root = workspace
        .project()
        .read(cx)
        .visible_worktrees(cx)
        .next()
        .map(|worktree| worktree.read(cx).abs_path().to_path_buf());
    let fresh = workspace.opened_from_saved_state() == Some(false);
    let no_terminal = workspace.items_of_type::<TerminalView>(cx).next().is_none();
    if let Some(window) = window
        && let Some(root) = root
        && fresh
        && no_terminal
        && marley_layout(cx)
    {
        open_center_terminal(workspace, false, Some(root), window, cx);
    }
}

/// The workspace's task provider in both layouts. Every task runs through the Terminal Panel,
/// which opens it in the center when its reveal target says so; in the Marley layout the target
/// always does.
struct RoutedTerminals {
    workspace: WeakEntity<Workspace>,
    panel: Entity<TerminalPanel>,
}

impl workspace::TerminalProvider for RoutedTerminals {
    fn spawn(
        &self,
        task: SpawnInTerminal,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<Option<anyhow::Result<ExitStatus>>> {
        let marley = marley_layout(cx);
        let task = if marley {
            SpawnInTerminal {
                reveal_target: RevealTarget::Center,
                ..task
            }
        } else {
            task
        };
        // The workspace calls this while it is being updated, and the panel reads the workspace
        // when it spawns, so the spawn waits for the window's next turn.
        let workspace = self.workspace.clone();
        let panel = self.panel.clone();
        window.spawn(cx, async move |cx| {
            if marley {
                // A task reruns in its last terminal, which is still in the panel when the task
                // last ran in the Zed layout.
                let center =
                    workspace.read_with(cx, |workspace, _| workspace.active_pane().clone());
                let center = center.ok()?;
                let moved = cx.update(|window, cx| {
                    move_to_center(&panel, &task.full_label, &center, window, cx);
                });
                moved.ok()?;
            }
            run_task(&panel, &task, cx).await
        })
    }
}

/// Moves the Terminal Panel's terminals for the task labelled `label` to the end of `center`,
/// as dragging their tabs out of the panel would.
fn move_to_center(
    panel: &Entity<TerminalPanel>,
    label: &str,
    center: &Entity<Pane>,
    window: &mut Window,
    cx: &mut App,
) {
    let moves: Vec<(Entity<Pane>, EntityId)> = panel
        .read(cx)
        .panes()
        .into_iter()
        .flat_map(|pane| {
            pane.read(cx)
                .items_of_type::<TerminalView>()
                .filter(|view| {
                    view.read(cx)
                        .terminal()
                        .read(cx)
                        .task()
                        .is_some_and(|task| task.spawned_task.full_label == label)
                })
                .map(|view| (pane.clone(), view.entity_id()))
                .collect::<Vec<_>>()
        })
        .collect();
    for (pane, item) in moves {
        let end = center.read(cx).items_len();
        workspace::move_item(&pane, center, item, end, false, window, cx);
    }
}

/// Spawns a task in `panel` and waits for it: a spawn that fails is an error, and a window or
/// terminal closed before the task finished has no exit status to report.
async fn run_task(
    panel: &Entity<TerminalPanel>,
    task: &SpawnInTerminal,
    cx: &mut AsyncWindowContext,
) -> Option<anyhow::Result<ExitStatus>> {
    let spawned = panel.update_in(cx, |panel, window, cx| panel.spawn_task(task, window, cx));
    let terminal = match spawned.ok()?.await {
        Ok(terminal) => terminal,
        Err(error) => return Some(Err(error)),
    };
    let completed = terminal.read_with(cx, Terminal::wait_for_completed_task);
    completed.ok()?.await.map(Ok)
}

/// `workspace::NewTerminal`: in the Marley layout a center terminal where the panel would have
/// started one; in the Zed layout the action goes on to the panel.
fn new_terminal(
    workspace: &mut Workspace,
    action: &NewTerminal,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if marley_layout(cx) {
        cx.stop_propagation();
        let directory = terminal_view::default_working_directory(workspace, cx);
        open_center_terminal(workspace, action.local, directory, window, cx);
    }
}

/// `workspace::OpenTerminal`: in the Marley layout a center terminal in the directory it names;
/// in the Zed layout the action goes on to the panel.
fn open_terminal(
    workspace: &mut Workspace,
    action: &OpenTerminal,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if marley_layout(cx) {
        cx.stop_propagation();
        let directory = Some(action.working_directory.clone());
        open_center_terminal(workspace, action.local, directory, window, cx);
    }
}

/// `terminal_panel::Toggle` (`` ctrl-` ``) and `terminal_panel::ToggleFocus`: in the Marley
/// layout the center terminals' toggle; in the Zed layout the action goes on to the panel.
fn toggle<A>(workspace: &mut Workspace, _: &A, window: &mut Window, cx: &mut Context<Workspace>) {
    if marley_layout(cx) {
        cx.stop_propagation();
        toggle_terminal(workspace, window, cx);
    }
}

/// `workspace::ToggleBottomDock` (`ctrl-j`): in the Marley layout, while the closed dock would
/// show the Terminal Panel, the center terminals' toggle; otherwise the action goes on to Zed.
fn toggle_bottom_dock(
    workspace: &mut Workspace,
    _: &ToggleBottomDock,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if marley_layout(cx) && bottom_dock_would_show_terminal_panel(workspace, cx) {
        cx.stop_propagation();
        toggle_terminal(workspace, window, cx);
    }
}

/// Whether toggling the bottom dock would show the Terminal Panel. A closed dock shows its active
/// panel, or with none its first enabled one. The Terminal Panel sorts first among Zed's bottom
/// panels, so the check takes the first as it is: asking the panels whether they are enabled
/// could read the workspace this listener is updating.
fn bottom_dock_would_show_terminal_panel(workspace: &Workspace, cx: &App) -> bool {
    let dock = workspace.bottom_dock().read(cx);
    let shown = dock.active_panel_index().unwrap_or(0);
    !dock.is_open() && dock.panel_index_for_type::<TerminalPanel>() == Some(shown)
}

/// Switches between the code and the center terminals. From a focused center terminal it goes
/// back to the center item used last that is not a terminal, if there is one; from anywhere
/// else it focuses the center terminal used last, or opens one where New Terminal would.
fn toggle_terminal(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let in_terminal = workspace
        .active_item_as::<TerminalView>(cx)
        .is_some_and(|view| view.focus_handle(cx).contains_focused(window, cx));
    if in_terminal {
        if let Some(item) = last_item_besides_terminals(workspace, cx) {
            workspace.activate_item(item.as_ref(), true, true, window, cx);
        }
    } else if let Some(terminal) = workspace.recent_active_item_by_type::<TerminalView>(cx) {
        workspace.activate_item(&terminal, true, true, window, cx);
    } else {
        let directory = terminal_view::default_working_directory(workspace, cx);
        open_center_terminal(workspace, false, directory, window, cx);
    }
}

/// The center item used last that is not a terminal, by the panes' activation history.
fn last_item_besides_terminals(workspace: &Workspace, cx: &App) -> Option<Box<dyn ItemHandle>> {
    workspace
        .panes()
        .iter()
        .flat_map(|pane| {
            let pane = pane.read(cx);
            pane.activation_history().iter().filter_map(move |entry| {
                pane.items()
                    .find(|item| item.item_id() == entry.entity_id)
                    .filter(|item| item.downcast::<TerminalView>().is_none())
                    .map(|item| (entry.timestamp, item.boxed_clone()))
            })
        })
        .max_by_key(|(timestamp, _)| *timestamp)
        .map(|(_, item)| item)
}

/// A local terminal ignores the directory, as the panel's own does.
fn open_center_terminal(
    workspace: &mut Workspace,
    local: bool,
    directory: Option<PathBuf>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    TerminalPanel::add_center_terminal(workspace, window, cx, move |project, cx| {
        if local {
            project.create_local_terminal(cx)
        } else {
            project.create_terminal_shell(directory, cx)
        }
    })
    .detach_and_prompt_err("Could not open a terminal", window, cx, |_, _, _| None);
}

#[cfg(test)]
#[path = "routing_tests.rs"]
mod tests;
