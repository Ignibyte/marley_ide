//! New Thread with Context (#717): from an open thread, a new thread on an agent the person
//! picks, in a split beside it, with the old thread's history in its message box, unsent.
//!
//! The history is a selection of the old thread's items (`marley_agent::handoff`), each request,
//! answer and tool call whole, under a preamble that calls it history. The whole conversation is
//! written to `<data dir>/handoffs/`, mode 0600, and the block names the file. Both pass through
//! the agents' redaction (#516) while it is on. The old thread stays as it was.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use acp_thread::{AcpThread, AgentThreadEntry, AssistantMessageChunk};
use agent_client_protocol::schema::v2 as acp_v2;
use agent_ui::{Agent, AgentInitialContent, AgentPanel, ConversationView};
use anyhow::Context as _;
use gpui::{
    App, AppContext as _, Context, DismissEvent, Entity, SharedString, Task, WeakEntity, Window,
    actions,
};
use marley_agent::handoff::{self, Item, Kind, Source};
use picker::{Picker, PickerDelegate};
use project::AgentId;
use ui::{ListItem, ListItemSpacing, prelude::*};
use util::ResultExt as _;
use util::path_list::PathList;
use workspace::Workspace;

actions!(
    marley,
    [
        /// Starts a thread on an agent you pick, in a split beside the focused thread, with that
        /// thread's history in its message box.
        #[derive(Eq)]
        NewThreadWithContext
    ]
);

/// Registers the action in every workspace; `thread_tab::init` calls it once.
pub fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &NewThreadWithContext, window, cx| {
            open_picker(workspace, window, cx);
        });
    })
    .detach();
}

/// The thread the action acts on: the active thread tab, else the Agent Panel's thread.
fn source_view(workspace: &Workspace, cx: &App) -> Option<Entity<ConversationView>> {
    crate::thread_tab::active_conversation(workspace, cx).or_else(|| {
        workspace
            .panel::<AgentPanel>(cx)?
            .read(cx)
            .active_conversation_view()
            .cloned()
    })
}

/// The live thread of `view`, once it has started.
fn thread_of(view: &Entity<ConversationView>, cx: &App) -> Option<Entity<AcpThread>> {
    Some(view.read(cx).root_thread_view()?.read(cx).thread.clone())
}

/// Opens the agent picker for the focused thread, which needs one of the person's messages.
fn open_picker(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let Some(view) = source_view(workspace, cx) else {
        refuse(
            workspace,
            "Focus a thread first: a tab or the Agent Panel.",
            cx,
        );
        return;
    };
    let Some(thread) = thread_of(&view, cx) else {
        refuse(workspace, "This thread has not started yet.", cx);
        return;
    };
    let items = items_of(thread.read(cx), cx);
    if !items.iter().any(|item| item.kind == Kind::Request) {
        refuse(
            workspace,
            "This thread has no message of yours to carry over yet.",
            cx,
        );
        return;
    }
    let own = view.read(cx).agent_key().id();
    let agents = picker_agents(workspace, &own, cx);
    let delegate = AgentPickerDelegate {
        workspace: workspace.weak_handle(),
        source: view.downgrade(),
        agents,
        matches: Vec::new(),
        selected_index: 0,
    };
    workspace.toggle_modal(window, cx, |window, cx| {
        Picker::uniform_list(delegate, window, cx)
    });
}

/// The agents a new thread can run, the old thread's own first; seat and Manager entries are
/// windows onto agents that already exist, so they are left out.
fn picker_agents(workspace: &Workspace, own: &AgentId, cx: &App) -> Vec<(AgentId, SharedString)> {
    let mut agents: Vec<(AgentId, SharedString)> =
        crate::agents::thread_agents(workspace.project(), cx)
            .into_iter()
            .filter(|(agent, _, _)| crate::harness::manager_of_entry(agent, cx).is_none())
            .map(|(agent, name, _)| (agent, name))
            .collect();
    if let Some(at) = agents.iter().position(|(agent, _)| agent == own) {
        let first = agents.remove(at);
        agents.insert(0, first);
    }
    agents
}

/// The old thread's items in conversation order: the person's messages, the agent's answers as
/// text without its thinking, and each tool call as its title and status with the paths it
/// changed. Tool output stays out.
fn items_of(thread: &AcpThread, cx: &App) -> Vec<Item> {
    thread
        .entries()
        .iter()
        .filter_map(|entry| match entry {
            AgentThreadEntry::UserMessage(message) => Some(Item {
                kind: Kind::Request,
                text: message.content.to_markdown(cx),
            }),
            AgentThreadEntry::AssistantMessage(message) => {
                let text: String = message
                    .chunks
                    .iter()
                    .filter_map(|chunk| match chunk {
                        AssistantMessageChunk::Message { block, .. } => Some(block.to_markdown(cx)),
                        AssistantMessageChunk::Thought { .. } => None,
                    })
                    .collect();
                (!text.trim().is_empty()).then_some(Item {
                    kind: Kind::Answer,
                    text,
                })
            }
            AgentThreadEntry::ToolCall(call) => {
                let mut paths: Vec<String> = call
                    .diffs()
                    .filter_map(|diff| diff.read(cx).file_path(cx))
                    .chain(
                        call.locations
                            .iter()
                            .map(|location| location.path.display().to_string()),
                    )
                    .collect();
                paths.dedup();
                Some(Item {
                    kind: Kind::ToolCall {
                        title: call.label.read(cx).source().to_string(),
                        status: call.status().to_string(),
                    },
                    text: paths.join("\n"),
                })
            }
            _ => None,
        })
        .collect()
}

/// `text` with the agents' redaction (#516) while it is on.
fn redacted(text: &str, cx: &App) -> String {
    crate::mcp::agent_redactor(cx)
        .map_or_else(|| text.to_string(), |redactor| redactor.redact(text).text)
}

/// The folder handoff files go in.
fn handoffs_dir() -> PathBuf {
    paths::data_dir().join("handoffs")
}

/// Writes `text` as `name` in `dir`, mode 0600, the folder made first.
fn write_handoff_in(dir: &Path, name: &str, text: &str) -> anyhow::Result<PathBuf> {
    use std::io::Write as _;
    std::fs::create_dir_all(dir).with_context(|| format!("making {}", dir.display()))?;
    let path = dir.join(name);
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options
        .open(&path)
        .with_context(|| format!("opening {}", path.display()))?;
    file.write_all(text.as_bytes())
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

/// Starts the new thread on `agent`: the whole conversation to a file off the main thread, then
/// a tab in a split beside the old thread with the block in its message box.
fn start(
    workspace: &mut Workspace,
    source: &Entity<ConversationView>,
    agent: AgentId,
    window: &Window,
    cx: &mut Context<Workspace>,
) {
    let Some(thread) = thread_of(source, cx) else {
        refuse(workspace, "The thread is gone.", cx);
        return;
    };
    let (items, markdown, title, folders) = {
        let thread = thread.read(cx);
        let folders = thread.work_dirs().cloned();
        (
            items_of(thread, cx),
            thread.to_markdown(cx),
            source.read(cx).title(cx),
            folders,
        )
    };
    let items: Vec<Item> = items
        .into_iter()
        .map(|item| Item {
            text: redacted(&item.text, cx),
            ..item
        })
        .collect();
    let markdown = redacted(&markdown, cx);
    let old_agent = crate::agents::thread_agent_name(
        &source.read(cx).agent_key().id(),
        workspace.project(),
        cx,
    );
    let folders = folders
        .unwrap_or_else(|| PathList::new(&[crate::thread_tab::default_folder(workspace, cx)]));
    let name = format!(
        "{}-{}.md",
        source.read(cx).parent_id().to_key_string(),
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );
    let dir = handoffs_dir();
    let written = cx.background_spawn(futures::future::lazy(move |_| {
        write_handoff_in(&dir, &name, &markdown)
    }));
    cx.spawn_in(window, async move |workspace, cx| {
        let file = written.await.map_or_else(
            |error| {
                log::warn!("handoff: the conversation's file: {error:#}");
                "no file (it could not be written)".to_string()
            },
            |path| path.display().to_string(),
        );
        let text = handoff::block(
            &items,
            Source {
                title: &title,
                agent: &old_agent,
                file: &file,
            },
            handoff::BUDGET,
        );
        let content = AgentInitialContent::ContentBlock {
            blocks: vec![acp_v2::ContentBlock::Text(acp_v2::TextContent::new(text))],
            auto_submit: false,
        };
        workspace
            .update_in(cx, |workspace, window, cx| {
                crate::thread_tab::open_beside(
                    workspace,
                    Agent::from(agent),
                    folders,
                    content,
                    window,
                    cx,
                )
            })
            .log_err();
    })
    .detach();
}

fn refuse(workspace: &mut Workspace, message: &str, cx: &mut Context<Workspace>) {
    crate::thread_tab::refuse(workspace, message.to_string(), cx);
}

/// The agents the picker offers, and the thread it was opened on.
struct AgentPickerDelegate {
    workspace: WeakEntity<Workspace>,
    source: WeakEntity<ConversationView>,
    agents: Vec<(AgentId, SharedString)>,
    /// The indices of the agents whose names hold the query.
    matches: Vec<usize>,
    selected_index: usize,
}

impl PickerDelegate for AgentPickerDelegate {
    type ListItem = ListItem;

    fn name() -> &'static str {
        "marley new thread with context"
    }

    fn match_count(&self) -> usize {
        self.matches.len()
    }

    fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn set_selected_index(&mut self, index: usize, _: &mut Window, _: &mut Context<Picker<Self>>) {
        self.selected_index = index;
    }

    fn placeholder_text(&self, _: &mut Window, _: &mut App) -> Arc<str> {
        "The agent the new thread runs on…".into()
    }

    fn update_matches(
        &mut self,
        query: String,
        _: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Task<()> {
        let query = query.trim().to_lowercase();
        self.matches = self
            .agents
            .iter()
            .enumerate()
            .filter(|(_, (_, name))| name.to_lowercase().contains(&query))
            .map(|(index, _)| index)
            .collect();
        self.selected_index = 0;
        cx.notify();
        Task::ready(())
    }

    fn confirm(&mut self, _: bool, window: &mut Window, cx: &mut Context<Picker<Self>>) {
        let Some((agent, _)) = self
            .matches
            .get(self.selected_index)
            .and_then(|index| self.agents.get(*index))
            .cloned()
        else {
            return;
        };
        let (Some(workspace), Some(source)) = (self.workspace.upgrade(), self.source.upgrade())
        else {
            return;
        };
        self.dismissed(window, cx);
        // After the picker closes, so the old thread's pane is the active one again.
        window.defer(cx, move |window, cx| {
            workspace.update(cx, |workspace, cx| {
                start(workspace, &source, agent, window, cx);
            });
        });
    }

    fn dismissed(&mut self, _: &mut Window, cx: &mut Context<Picker<Self>>) {
        cx.emit(DismissEvent);
    }

    fn render_match(
        &self,
        index: usize,
        selected: bool,
        _: &mut Window,
        _: &mut Context<Picker<Self>>,
    ) -> Option<Self::ListItem> {
        let (_, name) = self.agents.get(*self.matches.get(index)?)?;
        Some(
            ListItem::new(index)
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .toggle_state(selected)
                .child(Label::new(name.clone())),
        )
    }
}
