//! Messages an agent drafts for the user (#769).
//!
//! `draft_message` puts a message to another agent before the user instead of sending it: a Draft
//! tab in the target's window names who wrote it and who it is for, with the text in an editor.
//! Send delivers the editor's text as the user would, into an agent CLI's prompt with Enter
//! (`send_selection::send_text`) or into a thread's message box and sent; Drop closes the tab.
//! The tool answers as soon as the tab opens, since an app call has 30 seconds and a draft may wait
//! longer; it never sends.

use agent_ui::ConversationView;
use editor::Editor;
use gpui::{
    AnyWindowHandle, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable,
    SharedString, WeakEntity, Window,
};
use marley_mcp::{AppCall, Refusal, ToolAnswer};
use serde_json::{Value, json};
use terminal_view::TerminalView;
use ui::prelude::*;
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

/// Who a draft is for.
enum DraftTarget {
    /// An agent CLI in a terminal, and the project the terminal belongs to.
    Terminal(WeakEntity<TerminalView>, String),
    /// A thread, in the Agent Panel or a tab, and the workspace that holds it.
    Thread(WeakEntity<ConversationView>, WeakEntity<Workspace>),
}

/// Answers `draft_message`: checks the target, opens the draft in its window, answers `opened`.
pub(crate) fn answer(call: AppCall, cx: &mut App) {
    match open(&call, cx) {
        Ok((draft, to)) => call.answer::<Refusal>(Ok(ToolAnswer {
            structured: json!({ "draft": draft, "to": to, "opened": true }),
            text: Some(format!(
                "The draft to {to} is open for the user, who sends or drops it."
            )),
            image: None,
        })),
        Err(refusal) => call.answer(Err(refusal)),
    }
}

/// Opens the draft `call` asks for; its id and the target's name.
fn open(call: &AppCall, cx: &mut App) -> Result<(String, SharedString), Refusal> {
    let text = call
        .arguments
        .get("text")
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| Refusal::new("bad_argument", "give the message's text"))?
        .to_string();
    let to = call
        .arguments
        .get("to")
        .ok_or_else(|| Refusal::new("bad_argument", "say who the message is for in `to`"))?;
    let (target, name, workspace) = if let Some(id) = to.get("terminal").and_then(Value::as_u64) {
        terminal_target(id, cx)?
    } else if let Some(key) = to.get("thread").and_then(Value::as_str) {
        thread_target(key, cx)?
    } else {
        return Err(Refusal::new(
            "bad_argument",
            "`to` names a `terminal` id or a `thread` id",
        )
        .next("take one from terminal_list or thread_list"));
    };
    let from = SharedString::from(crate::click_pause::Who::of(call, cx).words);
    let window = crate::browser::window_of(&workspace, cx)
        .ok_or_else(|| Refusal::new("no_window", "the target's window is gone"))?;
    let draft = uuid::Uuid::new_v4().simple().to_string();
    let to = name.clone();
    window
        .update(cx, |_, window, cx| {
            workspace.update(cx, |workspace, cx| {
                let view = cx.new(|cx| {
                    DraftView::new(
                        target,
                        name,
                        from,
                        &text,
                        workspace.weak_handle(),
                        window,
                        cx,
                    )
                });
                workspace.add_item_to_center(Box::new(view), window, cx);
            });
        })
        .map_err(|_| Refusal::new("no_window", "the target's window is gone"))?;
    Ok((draft, to))
}

/// The terminal `id` names, when an agent CLI runs in it.
fn terminal_target(
    id: u64,
    cx: &App,
) -> Result<(DraftTarget, SharedString, Entity<Workspace>), Refusal> {
    let view = crate::mcp::terminal_with_id(id, cx)?;
    let workspace = view
        .read(cx)
        .marley_workspace()
        .upgrade()
        .ok_or_else(|| Refusal::new("no_terminal", "the terminal's window is gone"))?;
    let project = workspace
        .read(cx)
        .project()
        .read(cx)
        .visible_worktrees(cx)
        .next()
        .map(|worktree| worktree.read(cx).root_name_str().to_string())
        .unwrap_or_default();
    let target =
        crate::send_selection::target_of(project.clone(), view.clone(), cx).ok_or_else(|| {
            Refusal::new("not_an_agent", "no agent CLI runs in that terminal")
                .next("draft to a terminal whose agent CLI runs, or to a thread")
        })?;
    let name = if project.is_empty() {
        SharedString::from(target.kind.display_name().to_string())
    } else {
        SharedString::from(format!("{} in {project}", target.kind.display_name()))
    };
    Ok((
        DraftTarget::Terminal(view.downgrade(), project),
        name,
        workspace,
    ))
}

/// The thread `key` names, in an Agent Panel or a tab of any window.
fn thread_target(
    key: &str,
    cx: &App,
) -> Result<(DraftTarget, SharedString, Entity<Workspace>), Refusal> {
    crate::mcp::all_workspaces(cx)
        .into_iter()
        .find_map(|workspace| {
            let conversation = crate::thread_tab::conversations_of(workspace.read(cx), cx)
                .into_iter()
                .find(|view| view.read(cx).parent_id().to_key_string() == key)?;
            let name = SharedString::from(format!(
                "the thread \"{}\"",
                conversation.read(cx).title(cx)
            ));
            Some((
                DraftTarget::Thread(conversation.downgrade(), workspace.downgrade()),
                name,
                workspace,
            ))
        })
        .ok_or_else(|| {
            Refusal::new("no_thread", format!("no window holds a thread {key}"))
                .next("list the threads with thread_list")
        })
}

/// A draft in a center tab: who wrote it, who it is for, the text, Send and Drop.
pub(crate) struct DraftView {
    target: DraftTarget,
    to: SharedString,
    from: SharedString,
    editor: Entity<Editor>,
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    /// Why the last Send did not go, shown under the text.
    note: Option<SharedString>,
}

impl DraftView {
    fn new(
        target: DraftTarget,
        to: SharedString,
        from: SharedString,
        text: &str,
        workspace: WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let editor = cx.new(|cx| {
            let mut editor = Editor::multi_line(window, cx);
            editor.set_show_gutter(false, cx);
            editor.set_soft_wrap_mode(settings::SoftWrap::EditorWidth, cx);
            editor.set_text(text, window, cx);
            editor
        });
        Self {
            target,
            to,
            from,
            editor,
            workspace,
            focus_handle: cx.focus_handle(),
            note: None,
        }
    }

    /// Delivers the editor's text as the user would, then closes the tab; a target that cannot
    /// take it now keeps the draft and says why.
    fn send(&mut self, here: AnyWindowHandle, cx: &mut Context<Self>) {
        let text = self.editor.read(cx).text(cx);
        if text.trim().is_empty() {
            self.note = Some(SharedString::new_static("The draft is empty."));
            cx.notify();
            return;
        }
        let sent = match &self.target {
            DraftTarget::Terminal(view, project) => send_to_terminal(view, project, text, here, cx),
            DraftTarget::Thread(conversation, workspace) => {
                send_to_thread(conversation, workspace, &text, here, cx)
            }
        };
        match sent {
            Ok(()) => {
                let message = format!("Sent to {}.", self.to);
                if let Some(workspace) = self.workspace.upgrade() {
                    workspace.update(cx, |workspace, cx| {
                        workspace
                            .show_toast(Toast::new(NotificationId::unique::<Self>(), message), cx);
                    });
                }
                cx.emit(ItemEvent::CloseItem);
            }
            Err(reason) => {
                self.note = Some(reason);
                cx.notify();
            }
        }
    }
}

/// The window that holds `workspace`. The window being updated, the draft's own, hides from
/// `cx.windows()`, so a workspace no other window holds is in `here`.
fn window_holding(
    workspace: &WeakEntity<Workspace>,
    here: AnyWindowHandle,
    cx: &App,
) -> Option<AnyWindowHandle> {
    let workspace = workspace.upgrade()?;
    Some(crate::browser::window_of(&workspace, cx).map_or(here, Into::into))
}

/// Into the agent CLI's prompt with Enter, unless it is gone or waits on a question.
fn send_to_terminal(
    view: &WeakEntity<TerminalView>,
    project: &str,
    text: String,
    here: AnyWindowHandle,
    cx: &mut App,
) -> Result<(), SharedString> {
    let view = view
        .upgrade()
        .ok_or_else(|| SharedString::new_static("The terminal is gone; nothing was sent."))?;
    let target = crate::send_selection::target_of(project.to_string(), view.clone(), cx)
        .ok_or_else(|| {
            SharedString::new_static("No agent CLI runs in that terminal now; nothing was sent.")
        })?;
    if target.waiting {
        return Err(SharedString::from(format!(
            "{} waits on a permission or a question; nothing was sent.",
            target.kind.display_name()
        )));
    }
    let window = window_holding(view.read(cx).marley_workspace(), here, cx)
        .ok_or_else(|| SharedString::new_static("The terminal's window is gone."))?;
    crate::send_selection::send_text(target, text, true, window, cx);
    Ok(())
}

/// Into the thread's message box and sent, unless the user has text of their own there.
fn send_to_thread(
    conversation: &WeakEntity<ConversationView>,
    workspace: &WeakEntity<Workspace>,
    text: &str,
    here: AnyWindowHandle,
    cx: &mut App,
) -> Result<(), SharedString> {
    let conversation = conversation
        .upgrade()
        .ok_or_else(|| SharedString::new_static("The thread is gone; nothing was sent."))?;
    let view = conversation
        .read(cx)
        .root_thread_view()
        .ok_or_else(|| SharedString::new_static("The thread can't take a message yet."))?;
    if !view.read(cx).message_editor.read(cx).is_empty(cx) {
        return Err(SharedString::new_static(
            "You have a message typed in that thread; send or clear it first.",
        ));
    }
    let window = window_holding(workspace, here, cx)
        .ok_or_else(|| SharedString::new_static("The thread's window is gone."))?;
    let text = text.to_string();
    // After this update, since the thread's window may be the one being updated.
    cx.defer(move |cx| {
        window
            .update(cx, |_, window, cx| {
                view.update(cx, |view, cx| {
                    view.message_editor.update(cx, |editor, cx| {
                        editor.insert_text(&text, window, cx);
                    });
                    view.send(window, cx);
                });
            })
            .log_err();
    });
    Ok(())
}

impl Focusable for DraftView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for DraftView {}

impl Render for DraftView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("MarleyDraft")
            .track_focus(&self.focus_handle)
            .size_full()
            .p_4()
            .gap_2()
            .bg(cx.theme().colors().editor_background)
            .child(
                Label::new(format!("Draft from {} to {}", self.from, self.to))
                    .size(LabelSize::Large),
            )
            .child(
                Label::new("Nothing is sent until you press Send. Edit it first if you like.")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .border_1()
                    .border_color(cx.theme().colors().border)
                    .child(self.editor.clone()),
            )
            .when_some(self.note.clone(), |this, note| {
                this.child(Label::new(note).size(LabelSize::Small).color(Color::Error))
            })
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("marley-draft-send", "Send")
                            .style(ButtonStyle::Filled)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.send(window.window_handle(), cx);
                            })),
                    )
                    .child(
                        Button::new("marley-draft-drop", "Drop").on_click(cx.listener(
                            |_, _, _, cx| {
                                cx.emit(ItemEvent::CloseItem);
                            },
                        )),
                    ),
            )
    }
}

impl Item for DraftView {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::from(format!("Draft to {}", self.to))
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Envelope))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::from(format!(
            "A message {} drafted for you to send",
            self.from
        )))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn show_toolbar(&self) -> bool {
        false
    }
}
