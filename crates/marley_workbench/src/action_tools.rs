//! Zed's palette actions for agents over Marley's MCP server (#707).
//!
//! `action_list` and `action_run` pass the actions area's mode (`agent_control`) and #703's log
//! and switch. Only Marley's safe actions (`ALLOWED`) and those the user names in
//! `marley.agent_control.actions_allowed` run at all: upstream adds actions at every merge, so a
//! deny list would leak. Over both sits `refused`, which no setting lifts: an action that could
//! quit, run code, type into a terminal, answer a consent, change Marley's own controls or delete.

use gpui::{App, Entity, Focusable as _, Window, WindowHandle};
use marley_mcp::{AppCall, Refusal, ToolAnswer};
use serde_json::{Value, json};
use settings::SettingsStore;
use util::ResultExt as _;
use workspace::{MultiWorkspace, Workspace};

use crate::agent_control::{Area, Level, admit};

/// The actions Marley lets agents run: docks and panels, panes and splits, navigation, search,
/// go-to and formatting. None writes a file, runs code or answers for the user.
const ALLOWED: &[&str] = &[
    "workspace::ToggleLeftDock",
    "workspace::ToggleRightDock",
    "workspace::ToggleBottomDock",
    "workspace::ToggleZoom",
    "workspace::CloseAllDocks",
    "workspace::ActivateNextPane",
    "workspace::ActivatePreviousPane",
    "workspace::NewSearch",
    "project_panel::ToggleFocus",
    "outline_panel::ToggleFocus",
    "git_panel::ToggleFocus",
    "terminal_panel::ToggleFocus",
    "pane::SplitRight",
    "pane::SplitDown",
    "pane::SplitLeft",
    "pane::SplitUp",
    "pane::ActivateNextItem",
    "pane::ActivatePreviousItem",
    "pane::GoBack",
    "pane::GoForward",
    "pane::DeploySearch",
    "buffer_search::Deploy",
    "diagnostics::Deploy",
    "editor::Format",
    "editor::GoToDefinition",
    "editor::GoToTypeDefinition",
    "editor::GoToImplementation",
    "editor::FindAllReferences",
    "editor::GoToDiagnostic",
    "editor::GoToPreviousDiagnostic",
    "editor::FoldAll",
    "editor::UnfoldAll",
    "editor::ToggleFold",
    "file_finder::Toggle",
    "outline::Toggle",
    "go_to_line::Toggle",
    "project_symbols::Toggle",
];

/// Namespaces no agent runs an action of: the agents' own consents and threads, tasks and runs,
/// git, installs, accounts and sharing, and Marley's and Rusty's own controls.
const REFUSED_NAMESPACES: &[&str] = &[
    "agent",
    "acp",
    "assistant",
    "task",
    "debugger",
    "repl",
    "notebook",
    "git",
    "extensions",
    "cli",
    "auto_update",
    "client",
    "collab",
    "marley",
    "rusty",
];

/// Words that refuse an action wherever they appear in its name.
const REFUSED_WORDS: &[&str] = &[
    "Quit",
    "Restart",
    "Reload",
    "CloseWindow",
    "OpenBrowser",
    "OpenZedUrl",
    "SendText",
    "SendKeystroke",
    "Allow",
    "Reject",
    "Authorize",
    "Approve",
    "Deny",
    "Delete",
    "Trash",
    "Install",
    "SignIn",
    "SignOut",
    "Share",
    "Run",
    "Rerun",
    "Spawn",
    "Push",
    "Discard",
];

/// Answers `action_list` and `action_run`.
pub(crate) fn answer(call: AppCall, cx: &App) {
    let tool = call.tool.clone();
    match tool.as_str() {
        "action_list" => after_admit(call, Level::Read, String::new(), list, cx),
        "action_run" => {
            let name = name_of(&call.arguments).unwrap_or_default();
            let what = format!("run {name}");
            after_admit(call, Level::Act, what, run, cx);
        }
        other => call.answer(Err(format!("Marley answers no tool named {other}"))),
    }
}

/// Runs `then` once the actions area admits `call`, else answers the refusal.
fn after_admit(call: AppCall, level: Level, what: String, then: fn(AppCall, &mut App), cx: &App) {
    let admitted = admit(&call, Area::Actions, level, what, cx);
    cx.spawn(async move |cx| match admitted.await {
        Ok(()) => cx.update(|cx| then(call, cx)),
        Err(refusal) => call.answer(Err(refusal)),
    })
    .detach();
}

fn name_of(arguments: &Value) -> Option<String> {
    arguments
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
}

/// Whether no setting lets an agent run `name`.
fn refused(name: &str) -> bool {
    let (namespace, local) = name.split_once("::").unwrap_or(("", name));
    REFUSED_NAMESPACES.contains(&namespace) || REFUSED_WORDS.iter().any(|word| local.contains(word))
}

/// The actions the user named in `marley.agent_control.actions_allowed`.
fn user_allowed(cx: &App) -> Vec<String> {
    cx.global::<SettingsStore>()
        .merged_settings()
        .marley
        .as_ref()
        .and_then(|marley| marley.agent_control.as_ref())
        .and_then(|agent_control| agent_control.actions_allowed.clone())
        .unwrap_or_default()
}

/// `action_list`: the actions an agent may run, and the allowlisted names no action is
/// registered under, which an upstream rename leaves behind.
fn list(call: AppCall, cx: &mut App) {
    let registered = cx.all_action_names();
    let documentation = cx.action_documentation();
    let named = user_allowed(cx);
    let mut actions = Vec::new();
    let mut unknown = Vec::new();
    for name in ALLOWED {
        if registered.contains(name) {
            actions.push(json!({
                "name": name,
                "documentation": documentation.get(name).copied().unwrap_or_default(),
                "named_by_user": false,
            }));
        } else {
            unknown.push(*name);
        }
    }
    for name in &named {
        if ALLOWED.contains(&name.as_str()) || refused(name) {
            continue;
        }
        if let Some(registered) = registered
            .iter()
            .find(|registered| **registered == name.as_str())
        {
            actions.push(json!({
                "name": registered,
                "documentation": documentation.get(registered).copied().unwrap_or_default(),
                "named_by_user": true,
            }));
        }
    }
    let text = format!(
        "{} actions an agent may run{}",
        actions.len(),
        if unknown.is_empty() {
            String::new()
        } else {
            format!("; not registered: {}", unknown.join(", "))
        }
    );
    call.answer::<Refusal>(Ok(ToolAnswer {
        structured: json!({ "actions": actions, "unknown": unknown }),
        text: Some(text),
        image: None,
    }));
}

/// The window and workspace an action runs in: the project `arguments` names, by folder name or
/// path, else the active window's shown workspace, else the first window's.
fn target(
    arguments: &Value,
    cx: &App,
) -> Result<(WindowHandle<MultiWorkspace>, Entity<Workspace>), Refusal> {
    let windows: Vec<WindowHandle<MultiWorkspace>> = cx
        .windows()
        .into_iter()
        .filter_map(|window| window.downcast::<MultiWorkspace>())
        .collect();
    if let Some(project) = arguments.get("project").and_then(Value::as_str) {
        return windows
            .iter()
            .find_map(|window| {
                let workspace = window
                    .read(cx)
                    .ok()?
                    .workspaces()
                    .find(|workspace| {
                        workspace
                            .read(cx)
                            .project()
                            .read(cx)
                            .visible_worktrees(cx)
                            .any(|worktree| {
                                let worktree = worktree.read(cx);
                                worktree.root_name().as_unix_str() == project
                                    || worktree.abs_path().to_string_lossy() == project
                            })
                    })?
                    .clone();
                Some((*window, workspace))
            })
            .ok_or_else(|| {
                Refusal::new("no_project", format!("no open project is named {project}"))
                    .next("leave project out to use the active window")
            });
    }
    let active = cx
        .active_window()
        .and_then(|window| window.downcast::<MultiWorkspace>());
    active
        .into_iter()
        .chain(windows)
        .find_map(|window| {
            let workspace = window.read(cx).ok()?.workspace().clone();
            Some((window, workspace))
        })
        .ok_or_else(|| Refusal::new("no_window", "Marley has no window open"))
}

/// `action_run`: the action built from its name and arguments and dispatched from the target
/// window's focus, as the command palette does.
fn run(call: AppCall, cx: &mut App) {
    let Some(name) = name_of(&call.arguments) else {
        call.answer(Err(Refusal::new("bad_argument", "give the action's name")
            .next("take a name from action_list")));
        return;
    };
    if refused(&name) {
        call.answer(Err(Refusal::new(
            "action_refused",
            format!(
                "{name} could quit, run code, answer for the user, change Marley's own controls \
                 or delete, so no agent runs it, even when named in actions_allowed"
            ),
        )
        .next("ask the user to run it themselves")));
        return;
    }
    if !ALLOWED.contains(&name.as_str()) && !user_allowed(cx).contains(&name) {
        call.answer(Err(Refusal::new(
            "action_not_allowed",
            format!("{name} is not one of the actions an agent may run"),
        )
        .next(format!(
            "ask the user to add \"{name}\" to marley.agent_control.actions_allowed, or pick one \
             from action_list"
        ))));
        return;
    }
    let arguments = call
        .arguments
        .get("arguments")
        .filter(|arguments| !arguments.is_null())
        .cloned();
    let action = match cx.build_action(&name, arguments) {
        Ok(action) => action,
        Err(error) => {
            call.answer(Err(Refusal::new("bad_action", format!("{error}"))
                .next("check the name and the arguments in action_list")));
            return;
        }
    };
    // An old name builds the action under its current one, which is checked too.
    if refused(action.name()) {
        call.answer(Err(Refusal::new(
            "action_refused",
            format!("{name} is {}, which no agent runs", action.name()),
        )));
        return;
    }
    let (window, workspace) = match target(&call.arguments, cx) {
        Ok(target) => target,
        Err(refusal) => {
            call.answer(Err(refusal));
            return;
        }
    };
    let switched = window.update(cx, |multi_workspace, window, cx| {
        let switch = multi_workspace.workspace() != &workspace;
        if switch {
            multi_workspace.activate(workspace.clone(), None, window, cx);
        }
        switch
    });
    match switched {
        Ok(false) => {
            window
                .update(cx, |_, window, cx| {
                    dispatch_in(call, &workspace, action, window, cx);
                })
                .log_err();
        }
        // The shown workspace changed: its focus and handlers are in the next frame.
        Ok(true) => {
            window
                .update(cx, |_, window, _| {
                    window.on_next_frame(move |window, cx| {
                        dispatch_in(call, &workspace, action, window, cx);
                    });
                })
                .log_err();
        }
        Err(_) => call.answer(Err("the window is gone".to_string())),
    }
}

/// Dispatches `action` from `window`'s focus, as the palette does, unless nothing there takes it.
fn dispatch_in(
    call: AppCall,
    workspace: &Entity<Workspace>,
    action: Box<dyn gpui::Action>,
    window: &mut Window,
    cx: &mut App,
) {
    // Nothing focused dispatches from the window's root, past the workspace's handlers.
    if window.focused(cx).is_none() {
        let pane = workspace.read(cx).active_pane().clone();
        window.focus(&pane.focus_handle(cx), cx);
    }
    let name = action.name();
    if !window.is_action_available(action.as_ref(), cx) {
        call.answer(Err(Refusal::new(
            "not_available",
            format!("{name} does nothing where the window's focus is now"),
        )
        .next(
            "an editor action needs an editor focused: open one with editor_open first",
        )));
        return;
    }
    window.dispatch_action(action, cx);
    call.answer::<Refusal>(Ok(ToolAnswer {
        structured: json!({ "name": name, "dispatched": true }),
        text: Some(format!("ran {name}")),
        image: None,
    }));
}
