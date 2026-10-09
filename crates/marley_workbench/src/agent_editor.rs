//! Agent prompts in a tab (#649): Marley's own editor for the terminals it opens for agents.
//!
//! Each of the four agents Marley knows opens its prompt in `$VISUAL` or `$EDITOR` on a key of
//! its own and reads the file back when the editor exits. While
//! `marley.agent_editor_in_tab` is on, a terminal Marley opens for an agent in a local project
//! gets `marley-edit` as both, a program beside the opener in Marley's data directory. It asks
//! Marley, through its MCP endpoint, to open the file in a tab of that terminal's workspace
//! (`prompt_open`), and waits for the tab to close (`prompt_wait`), as `zed --wait` does: a save
//! alone does not end the edit. When it ends the terminal comes back to the front with the focus.
//! Ctrl-G and the agent bar's Rich Input button send such a terminal's agent its own key, so the
//! prompt comes back through the agent's path, with nothing pasted and nothing sent.

use std::collections::{HashMap, HashSet};
use std::mem;
use std::path::{Path, PathBuf};
use std::time::Duration;

use futures::channel::oneshot;
use gpui::{
    AnyWindowHandle, App, Entity, EntityId, Focusable as _, Global, Subscription, WeakEntity,
    Window,
};
use marley_agent::AgentKind;
use marley_mcp::{AppCall, ToolAnswer};
use serde_json::{Value, json};
use settings::Settings as _;
use terminal_view::TerminalView;
use workspace::{OpenOptions, OpenVisible, Workspace};

use crate::MarleySettings;

/// Where an agent's prompt opens on Ctrl-G in the terminals Marley opens for agents:
/// `marley.agent_editor_in_tab`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AgentPrompts {
    /// Marley's Rich Input overlay (#481), the default.
    #[default]
    Overlay,
    /// A tab, through the agent's own editor key and `marley-edit`.
    InTab,
}

impl AgentPrompts {
    /// The switch as the settings hold it: off unless set.
    pub(crate) fn from_content(marley: Option<&settings::MarleySettingsContent>) -> Self {
        if marley
            .and_then(|marley| marley.agent_editor_in_tab)
            .unwrap_or(false)
        {
            Self::InTab
        } else {
            Self::Overlay
        }
    }
}

/// The helper, as Marley ships it.
pub(crate) const HELPER: &str = include_str!("../bin/marley-edit");

/// Its file in Marley's `mcp` folder, beside the opener's.
pub(crate) const HELPER_FILE: &str = "marley-edit";

/// How long an ended edit nobody waited on is kept, for a helper that asks late.
const KEPT_AFTER_END: Duration = Duration::from_secs(60);

/// The longest `prompt_wait`, under the server's 30 s for an app's answer.
const MOST_WAIT_SECONDS: u64 = 20;

/// One file open for an agent's editor key.
struct Edit {
    /// The terminal the call came from, and its workspace, brought back when the tab closes.
    view: WeakEntity<TerminalView>,
    workspace: WeakEntity<Workspace>,
    closed: bool,
    waiters: Vec<oneshot::Sender<()>>,
    release: Option<Subscription>,
}

/// The helper's path, the terminals given it, and the edits under way.
#[derive(Default)]
struct AgentEditor {
    helper: Option<PathBuf>,
    given: HashSet<EntityId>,
    edits: HashMap<u64, Edit>,
    next: u64,
}

impl Global for AgentEditor {}

/// Publishes the helper once it is written: only a path with no whitespace, since three of the
/// four agents split `$VISUAL` on spaces; otherwise agent terminals keep the overlay.
pub(crate) fn set_helper(path: &Path, cx: &mut App) {
    let usable = !path.to_string_lossy().chars().any(char::is_whitespace);
    if !usable {
        log::warn!(
            "agent editor: Marley's data directory has a space in its path ({}), so agents keep the \
             Rich Input overlay",
            path.display()
        );
    }
    cx.default_global::<AgentEditor>().helper = usable.then(|| path.to_path_buf());
}

/// The editor a terminal opened for an agent in a local project gets: the helper, while the switch
/// is on and the helper is written.
pub(crate) fn editor_path(cx: &App) -> Option<String> {
    if MarleySettings::get_global(cx).agent_prompts != AgentPrompts::InTab {
        return None;
    }
    let helper = cx.try_global::<AgentEditor>()?.helper.as_ref()?;
    Some(helper.to_string_lossy().into_owned())
}

/// The variables that give an agent's terminal `editor`: `VISUAL` and `EDITOR`, and the one the
/// shell integration exports them from again once the user's files have run.
pub(crate) fn add_env(env: &mut impl Extend<(String, String)>, editor: &str) {
    env.extend(
        [
            "VISUAL",
            "EDITOR",
            marley_terminal::shell_integration::AGENT_EDITOR_VARIABLE,
        ]
        .map(|name| (name.to_string(), editor.to_string())),
    );
}

/// Records the terminal `id` as one Marley gave its editor, until it is released.
pub(crate) fn give(id: EntityId, cx: &mut App) {
    let _added = cx.default_global::<AgentEditor>().given.insert(id);
}

/// Forgets a released terminal.
pub(crate) fn forget(id: EntityId, cx: &mut App) {
    let _removed = cx.default_global::<AgentEditor>().given.remove(&id);
}

/// Whether Ctrl-G and the Rich Input button send `kind` its own editor key in `view`'s terminal:
/// the switch on, and the terminal one Marley gave its editor.
pub(crate) fn takes_key(view: &Entity<TerminalView>, cx: &App) -> bool {
    MarleySettings::get_global(cx).agent_prompts == AgentPrompts::InTab
        && cx
            .try_global::<AgentEditor>()
            .is_some_and(|editor| editor.given.contains(&view.read(cx).terminal().entity_id()))
}

/// Writes `kind`'s editor key into `view`'s terminal and gives the terminal the focus.
pub(crate) fn press_key(
    view: &Entity<TerminalView>,
    kind: AgentKind,
    window: &mut Window,
    cx: &mut App,
) {
    let terminal = view.read(cx).terminal().clone();
    terminal.update(cx, |terminal, _| terminal.input(kind.editor_key().to_vec()));
    window.focus(&view.focus_handle(cx), cx);
}

/// Answers `prompt_open` and `prompt_wait`.
pub(crate) fn answer(call: AppCall, cx: &mut App) {
    let tool = call.tool.clone();
    match tool.as_str() {
        "prompt_open" => open(call, cx),
        "prompt_wait" => wait(call, cx),
        other => call.answer(Err(format!("Marley answers no tool named {other}"))),
    }
}

/// `prompt_open`: the file in a tab of the calling terminal's workspace, with the focus; the
/// answer once it is open, and the edit's end on the tab's release.
fn open(call: AppCall, cx: &mut App) {
    let Some((workspace, view)) = crate::mcp::caller_terminal(call.caller(), cx) else {
        call.answer(Err(
            "it serves Marley's own terminals, and this call comes from none".to_string(),
        ));
        return;
    };
    let path = call
        .arguments
        .get("path")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute());
    let Some(path) = path else {
        call.answer(Err("give the file's absolute path".to_string()));
        return;
    };
    let Some(window) = crate::browser::window_of(&workspace, cx) else {
        call.answer(Err("the terminal's window is gone".to_string()));
        return;
    };
    let options = OpenOptions {
        visible: Some(OpenVisible::None),
        focus: Some(true),
        ..OpenOptions::default()
    };
    let opening = AnyWindowHandle::from(window).update(cx, |_, window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.open_abs_path(path.clone(), options, window, cx)
        })
    });
    let Ok(opening) = opening else {
        call.answer(Err("the terminal's window is gone".to_string()));
        return;
    };
    let editor = cx.default_global::<AgentEditor>();
    let id = editor.next;
    editor.next += 1;
    let _previous = editor.edits.insert(
        id,
        Edit {
            view: view.downgrade(),
            workspace: workspace.downgrade(),
            closed: false,
            waiters: Vec::new(),
            release: None,
        },
    );
    let file = path
        .file_name()
        .map_or_else(|| path.to_string_lossy(), |name| name.to_string_lossy())
        .into_owned();
    cx.spawn(async move |cx| match opening.await {
        Ok(item) => {
            cx.update(|cx| {
                let release =
                    item.on_release(cx, Box::new(move |cx| cx.defer(move |cx| ended(id, cx))));
                if let Some(edit) = cx.default_global::<AgentEditor>().edits.get_mut(&id) {
                    edit.release = Some(release);
                }
            });
            // The tab holds the item now; a handle kept here would keep the edit from ending.
            drop(item);
            call.answer::<marley_mcp::Refusal>(Ok(ToolAnswer {
                structured: json!({ "edit": id, "file": file }),
                text: Some(format!(
                    "{file} is open in a Marley tab; closing the tab ends the edit"
                )),
                image: None,
            }));
        }
        Err(error) => {
            cx.update(|cx| {
                let _gone = cx.default_global::<AgentEditor>().edits.remove(&id);
            });
            call.answer(Err(format!("Marley could not open {file}: {error:#}")));
        }
    })
    .detach();
}

/// The edit `id`'s tab closed: its waiters wake, and its terminal comes back to the front with
/// the focus. Called deferred, out of the pane's close.
fn ended(id: u64, cx: &mut App) {
    let Some(edit) = cx.default_global::<AgentEditor>().edits.get_mut(&id) else {
        return;
    };
    edit.closed = true;
    edit.release = None;
    let waiters = mem::take(&mut edit.waiters);
    let (view, workspace) = (edit.view.clone(), edit.workspace.clone());
    // A wait that timed out left its sender behind; only a live one is a helper still asking.
    let waited = waiters.iter().any(|waiter| !waiter.is_canceled());
    for waiter in waiters {
        waiter.send(()).ok();
    }
    if !waited {
        // A helper that asks after the close still hears of it; one that never asks is dropped.
        let timer = cx.background_executor().timer(KEPT_AFTER_END);
        cx.spawn(async move |cx| {
            timer.await;
            cx.update(|cx| {
                let _gone = cx.default_global::<AgentEditor>().edits.remove(&id);
            });
        })
        .detach();
    }
    let (Some(view), Some(workspace)) = (view.upgrade(), workspace.upgrade()) else {
        return;
    };
    let Some(window) = crate::browser::window_of(&workspace, cx) else {
        return;
    };
    let shown = AnyWindowHandle::from(window).update(cx, |_, window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.activate_item(&view, true, true, window, cx);
        });
    });
    if shown.is_err() {
        log::debug!("agent editor: the terminal's window closed before its edit ended");
    }
}

/// `prompt_wait`: `closed: true` once the edit ended, else `closed: false` after `wait_seconds`.
fn wait(call: AppCall, cx: &mut App) {
    let Some(id) = call.arguments.get("edit").and_then(Value::as_u64) else {
        call.answer(Err("give the edit's id from prompt_open".to_string()));
        return;
    };
    let seconds = call
        .arguments
        .get("wait_seconds")
        .and_then(Value::as_u64)
        .unwrap_or(MOST_WAIT_SECONDS)
        .clamp(1, MOST_WAIT_SECONDS);
    let editor = cx.default_global::<AgentEditor>();
    let Some(edit) = editor.edits.get_mut(&id) else {
        call.answer(Err(format!("no edit {id} is open")));
        return;
    };
    if edit.closed {
        let _gone = editor.edits.remove(&id);
        call.answer::<marley_mcp::Refusal>(Ok(closed_answer(true)));
        return;
    }
    let (woken, waiting) = oneshot::channel();
    edit.waiters.push(woken);
    let timer = cx.background_executor().timer(Duration::from_secs(seconds));
    cx.spawn(async move |cx| {
        let closed = matches!(
            futures::future::select(waiting, timer).await,
            futures::future::Either::Left((Ok(()), _))
        );
        if closed {
            cx.update(|cx| {
                let _gone = cx.default_global::<AgentEditor>().edits.remove(&id);
            });
        }
        call.answer::<marley_mcp::Refusal>(Ok(closed_answer(closed)));
    })
    .detach();
}

fn closed_answer(closed: bool) -> ToolAnswer {
    ToolAnswer {
        structured: json!({ "closed": closed }),
        text: None,
        image: None,
    }
}
