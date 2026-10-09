//! The Agent Panel's threads for agents over Marley's MCP server (#706).
//!
//! `thread_list`, `thread_read`, `thread_post` and `thread_answer` pass the threads area's mode
//! (`agent_control`) and #703's log and switch.
//!
//! A thread is a conversation an Agent Panel holds, shown or kept in the background, named by its
//! key as the rail's rows are. A post goes through the thread's own message editor and send, so a
//! running thread queues it as the user's would be; an answer is the rail's (#508): Allow Once or
//! Reject Once, never for a sandbox escalation.

use acp_thread::{SelectedPermissionOutcome, ThreadStatus};
use agent_client_protocol::schema::v1 as acp;
use agent_ui::{AgentPanel, ConversationView};
use gpui::{App, Entity};
use marley_mcp::{AppCall, Refusal, ToolAnswer};
use serde_json::{Value, json};
use workspace::{MultiWorkspace, Workspace};

use crate::agent_control::{Area, Level, admit};

/// Answers `thread_list`, `thread_read`, `thread_post` and `thread_answer`.
pub(crate) fn answer(mut call: AppCall, cx: &App) {
    let tool = call.tool.clone();
    match tool.as_str() {
        "thread_list" => after_admit(call, Level::Read, String::new(), list, cx),
        "thread_read" => {
            // A read reveals a conversation, so it is listed in Agent Activity (#703).
            crate::agent_activity::log(&mut call, cx);
            after_admit(call, Level::Read, String::new(), read, cx);
        }
        "thread_post" => {
            let what = post_words(&call.arguments, cx);
            after_admit(call, Level::Act, what, post, cx);
        }
        "thread_answer" => {
            let what = answer_words(&call.arguments, cx);
            after_admit(call, Level::Sensitive, what, give_answer, cx);
        }
        other => call.answer(Err(format!("Marley answers no tool named {other}"))),
    }
}

/// Runs `then` once the threads area admits `call`, else answers the refusal.
fn after_admit(call: AppCall, level: Level, what: String, then: fn(AppCall, &mut App), cx: &App) {
    let admitted = admit(&call, Area::Threads, level, what, cx);
    cx.spawn(async move |cx| match admitted.await {
        Ok(()) => cx.update(|cx| then(call, cx)),
        Err(refusal) => call.answer(Err(refusal)),
    })
    .detach();
}

/// A conversation an Agent Panel holds, with the workspace it belongs to.
struct LiveThread {
    key: String,
    conversation: Entity<ConversationView>,
    workspace: Entity<Workspace>,
}

/// Every conversation the Agent Panels of Marley's windows hold, each once.
fn live(cx: &App) -> Vec<LiveThread> {
    let workspaces: Vec<Entity<Workspace>> = cx
        .windows()
        .into_iter()
        .filter_map(|window| window.downcast::<MultiWorkspace>()?.read(cx).ok())
        .flat_map(|multi_workspace| multi_workspace.workspaces().cloned().collect::<Vec<_>>())
        .collect();
    let mut found: Vec<LiveThread> = Vec::new();
    for workspace in workspaces {
        let Some(panel) = workspace.read(cx).panel::<AgentPanel>(cx) else {
            continue;
        };
        for conversation in panel.read(cx).conversation_views() {
            let key = conversation.read(cx).parent_id().to_key_string();
            if found.iter().any(|live| live.key == key) {
                continue;
            }
            found.push(LiveThread {
                key,
                conversation,
                workspace: workspace.clone(),
            });
        }
    }
    found
}

/// The thread `arguments` name.
fn thread_named(arguments: &Value, cx: &App) -> Result<LiveThread, Refusal> {
    let Some(key) = arguments.get("thread").and_then(Value::as_str) else {
        return Err(Refusal::new("bad_argument", "give the thread's id")
            .next("take an id from thread_list"));
    };
    live(cx)
        .into_iter()
        .find(|live| live.key == key)
        .ok_or_else(|| {
            Refusal::new("no_thread", format!("no Agent Panel holds a thread {key}"))
                .next("list the threads with thread_list")
        })
}

/// A thread's title, or "New Thread" before it has one.
fn title_of(live: &LiveThread, cx: &App) -> String {
    live.conversation
        .read(cx)
        .root_thread_view()
        .and_then(|view| view.read(cx).thread.read(cx).title())
        .map_or_else(|| "New Thread".to_string(), |title| title.to_string())
}

/// The permission `live` waits on: its tool's words, and whether an agent may answer it, which a
/// sandbox escalation never is.
fn pending_of(live: &LiveThread, cx: &App) -> Option<(String, bool)> {
    let conversation = live.conversation.read(cx);
    let (session, tool_call, _) = conversation.pending_tool_call(cx)?;
    let view = conversation.thread_view(&session)?;
    let view = view.read(cx);
    let (_, call) = view.thread.read(cx).tool_call(&tool_call)?;
    let words = call
        .label
        .read(cx)
        .source()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    Some((words, call.sandbox_authorization_details.is_none()))
}

/// `thread_list`.
fn list(call: AppCall, cx: &mut App) {
    let threads: Vec<Value> = live(cx)
        .iter()
        .map(|live| {
            let conversation = live.conversation.read(cx);
            let view = conversation.root_thread_view();
            let project = live.workspace.read(cx).project().clone();
            let agent = view
                .as_ref()
                .map(|view| {
                    crate::agents::thread_agent_name(&view.read(cx).agent_id, &project, cx)
                        .to_string()
                })
                .unwrap_or_default();
            let running = view.as_ref().is_some_and(|view| {
                matches!(
                    view.read(cx).thread.read(cx).status(),
                    ThreadStatus::Generating
                )
            });
            let folder = project
                .read(cx)
                .visible_worktrees(cx)
                .next()
                .map(|worktree| worktree.read(cx).root_name().as_unix_str().to_string())
                .unwrap_or_default();
            let pending = pending_of(live, cx)
                .map(|(tool, answerable)| json!({ "tool": tool, "answerable": answerable }));
            json!({
                "id": live.key,
                "title": title_of(live, cx),
                "agent": agent,
                "project": folder,
                "running": running,
                "pending": pending,
            })
        })
        .collect();
    let text = format!("{} threads", threads.len());
    call.answer::<Refusal>(Ok(ToolAnswer {
        structured: json!({ "threads": threads }),
        text: Some(text),
        image: None,
    }));
}

/// `thread_read`.
fn read(call: AppCall, cx: &mut App) {
    let live = match thread_named(&call.arguments, cx) {
        Ok(live) => live,
        Err(refusal) => {
            call.answer(Err(refusal));
            return;
        }
    };
    let Some(view) = live.conversation.read(cx).root_thread_view() else {
        call.answer(Err("the thread has nothing to read yet".to_string()));
        return;
    };
    let markdown = view.read(cx).thread.read(cx).to_markdown(cx);
    let redacted = crate::mcp::for_agents(&markdown, crate::mcp::agent_redactor(cx).as_deref());
    let after = call
        .arguments
        .get("start_line")
        .and_then(Value::as_u64)
        .and_then(|line| usize::try_from(line).ok())
        .filter(|line| *line > 1)
        .map(|line| line - 1);
    let page = match crate::mcp::page_from(&redacted.text, after) {
        Ok(page) => page,
        Err(refusal) => {
            call.answer(Err(refusal));
            return;
        }
    };
    let mut structured = json!({ "thread": live.key, "redacted": redacted.count });
    page.fill_forward(&mut structured);
    let next = structured.get("next").and_then(Value::as_u64);
    if let Some(fields) = structured.as_object_mut() {
        fields.insert(
            "next_line".into(),
            next.map_or(Value::Null, |next| Value::from(next + 1)),
        );
    }
    let text = structured
        .get("text")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let text = match next {
        Some(next) => format!("[read on with thread_read start_line={}]\n{text}", next + 1),
        None => text,
    };
    call.answer::<Refusal>(Ok(ToolAnswer {
        structured,
        text: Some(text),
        image: None,
    }));
}

/// The thread `arguments` name, in words, for a question.
fn thread_words(arguments: &Value, cx: &App) -> String {
    thread_named(arguments, cx).map_or_else(
        |_| "a thread".to_string(),
        |live| format!("the thread \"{}\"", title_of(&live, cx)),
    )
}

fn post_words(arguments: &Value, cx: &App) -> String {
    let message = arguments
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let first = message.lines().next().unwrap_or_default();
    format!("send \"{first}\" into {}", thread_words(arguments, cx))
}

fn answer_words(arguments: &Value, cx: &App) -> String {
    let allow = arguments
        .get("allow")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let tool = thread_named(arguments, cx)
        .ok()
        .and_then(|live| pending_of(&live, cx))
        .map_or_else(|| "its pending call".to_string(), |(tool, _)| tool);
    format!(
        "{} {tool} in {}",
        if allow { "allow" } else { "reject" },
        thread_words(arguments, cx)
    )
}

/// `thread_post`: the message typed into the thread's editor and sent, as the user would.
fn post(call: AppCall, cx: &mut App) {
    let live = match thread_named(&call.arguments, cx) {
        Ok(live) => live,
        Err(refusal) => {
            call.answer(Err(refusal));
            return;
        }
    };
    let Some(message) = call
        .arguments
        .get("message")
        .and_then(Value::as_str)
        .filter(|message| !message.trim().is_empty())
        .map(str::to_string)
    else {
        call.answer(Err(Refusal::new(
            "bad_argument",
            "give the message to send",
        )));
        return;
    };
    let Some(view) = live.conversation.read(cx).root_thread_view() else {
        call.answer(Err("the thread can't take a message yet".to_string()));
        return;
    };
    // A draft the user is typing there is theirs: a post would send it along with the message.
    if !view.read(cx).message_editor.read(cx).is_empty(cx) {
        call.answer(Err(Refusal::new(
            "draft_in_progress",
            "the user has a message typed but not sent in that thread",
        )
        .next("wait until they send it, or ask them to")));
        return;
    }
    let Some(window) = crate::browser::window_of(&live.workspace, cx) else {
        call.answer(Err("the thread's window is gone".to_string()));
        return;
    };
    let sent = window.update(cx, |_, window, cx| {
        view.update(cx, |view, cx| {
            view.message_editor.update(cx, |editor, cx| {
                editor.insert_text(&message, window, cx);
            });
            view.send(window, cx);
        });
    });
    match sent {
        Ok(()) => call.answer::<Refusal>(Ok(ToolAnswer {
            structured: json!({ "thread": live.key, "sent": true }),
            text: Some("sent".to_string()),
            image: None,
        })),
        Err(_) => call.answer(Err("the thread's window is gone".to_string())),
    }
}

/// `thread_answer`: the thread's pending permission allowed once or rejected, as the rail answers
/// it (#508).
fn give_answer(call: AppCall, cx: &mut App) {
    let live = match thread_named(&call.arguments, cx) {
        Ok(live) => live,
        Err(refusal) => {
            call.answer(Err(refusal));
            return;
        }
    };
    let allow = call
        .arguments
        .get("allow")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let conversation = live.conversation.read(cx);
    let Some((session, tool_call, options)) = conversation.pending_tool_call(cx) else {
        call.answer(Err(Refusal::new(
            "no_pending",
            "the thread waits on no permission",
        )));
        return;
    };
    let Some(view) = conversation.thread_view(&session) else {
        call.answer(Err(Refusal::new("no_pending", "the thread's view is gone")));
        return;
    };
    let escalation = view
        .read(cx)
        .thread
        .read(cx)
        .tool_call(&tool_call)
        .is_some_and(|(_, call)| call.sandbox_authorization_details.is_some());
    if escalation {
        call.answer(Err(Refusal::new(
            "sandbox_escalation",
            "the thread asks to leave its sandbox; only the user answers that, in the Agent Panel",
        )));
        return;
    }
    let kind = if allow {
        acp::PermissionOptionKind::AllowOnce
    } else {
        acp::PermissionOptionKind::RejectOnce
    };
    let Some(option) = options
        .first_option_of_kind(kind)
        .map(|option| (option.option_id.clone(), option.kind))
    else {
        call.answer(Err(Refusal::new(
            "no_option",
            "the permission offers no such answer",
        )));
        return;
    };
    let tool = pending_of(&live, cx)
        .map(|(tool, _)| tool)
        .unwrap_or_default();
    let Some(window) = crate::browser::window_of(&live.workspace, cx) else {
        call.answer(Err("the thread's window is gone".to_string()));
        return;
    };
    let answered = window.update(cx, |_, window, cx| {
        view.update(cx, |view, cx| {
            let Some(request) = view
                .thread
                .read(cx)
                .permission_request_for_tool(&tool_call)
                .map(|request| request.id)
            else {
                return false;
            };
            view.authorize_permission_request(
                session,
                request,
                SelectedPermissionOutcome::new(option.0, option.1),
                window,
                cx,
            );
            true
        })
    });
    if !matches!(answered, Ok(true)) {
        call.answer(Err(Refusal::new(
            "no_pending",
            "the permission was answered, or its window closed, before this answer",
        )));
        return;
    }
    call.answer::<Refusal>(Ok(ToolAnswer {
        structured: json!({ "thread": live.key, "tool": tool, "allowed": allow }),
        text: Some(format!(
            "{} {tool}",
            if allow { "allowed" } else { "rejected" }
        )),
        image: None,
    }));
}
