//! Terminal routing: in the Marley layout nothing opens the bottom Terminal Panel.
//!
//! Tasks run through a task provider of this crate's own, which sends them to center terminals,
//! and `workspace::NewTerminal` and `workspace::OpenTerminal` are caught before the panel's own
//! handlers see them. In the Zed layout all of it passes straight through, so the fork behaves
//! as upstream.

use std::path::PathBuf;
use std::process::ExitStatus;

use gpui::{
    App, AsyncWindowContext, Context, Entity, EntityId, InteractiveElement as _, Task, WeakEntity,
    Window,
};
use task::{RevealTarget, SpawnInTerminal};
use terminal::Terminal;
use terminal_view::TerminalView;
use terminal_view::terminal_panel::TerminalPanel;
use workspace::notifications::DetachAndPromptErr as _;
use workspace::{NewTerminal, OpenTerminal, Pane, Workspace};

use crate::marley_layout;

/// Installs the routing on every workspace. [`crate::init`] calls it once, before any window opens.
///
/// The two actions are caught at the workspace's root in the capture phase, and the task
/// provider replaces the panel's once the panel is added, after the panel's own load has
/// installed Zed's.
pub fn init(cx: &App) {
    cx.observe_new(
        |workspace: &mut Workspace, _, cx: &mut Context<Workspace>| {
            workspace.register_action_renderer(|div, _, _, cx| {
                div.capture_action(cx.listener(new_terminal))
                    .capture_action(cx.listener(open_terminal))
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
        },
    )
    .detach();
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
