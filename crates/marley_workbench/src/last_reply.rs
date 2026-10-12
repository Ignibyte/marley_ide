//! An agent's last reply, for the review tab (#729): a Zed agent thread's last turn, or Claude
//! Code's, read from the transcript its hook events name.
//!
//! Claude Code's transcript is one JSON entry a line. The reader takes only an entry's `type`,
//! `isSidechain`, `isMeta` and `message.content`: the reply is the `text` blocks of the assistant
//! entries after the last prompt (a user entry with text and no tool result) up to the last entry
//! with text. The format is Claude Code's, not a published contract, so anything else in it is left
//! alone and a transcript the reader cannot follow holds "no reply".

use std::io::{Read as _, Seek as _, SeekFrom};
use std::path::{Path, PathBuf};

use acp_thread::{AgentThreadEntry, AssistantMessageChunk};
use agent_ui::ConversationView;
use gpui::{App, Entity, EntityId};
use serde_json::Value;

use crate::agent_events::AgentEvents;

/// How much of a transcript's end is read: the reply is at the end, and a long session's file
/// runs to many megabytes.
const TAIL_BYTES: u64 = 16 * 1024 * 1024;

/// The seat's label that holds the transcript's path, from the lead agent's hook events.
const TRANSCRIPT_LABEL: &str = "transcript_path";

/// The thread's last reply as Markdown: the assistant's messages of the last turn that has one,
/// its thoughts and subagents' output left out.
pub(crate) fn of_thread(conversation: &Entity<ConversationView>, cx: &App) -> Option<String> {
    let view = conversation.read(cx).root_thread_view()?;
    let thread = view.read(cx).thread.clone();
    let entries = thread.read(cx).entries();
    let is_reply = |entry: &AgentThreadEntry| matches!(entry, AgentThreadEntry::AssistantMessage(message) if !message.is_subagent_output);
    let last = entries.iter().rposition(is_reply)?;
    let start = entries
        .get(..last)?
        .iter()
        .rposition(|entry| matches!(entry, AgentThreadEntry::UserMessage(_)))
        .map_or(0, |at| at + 1);
    let texts: Vec<String> = entries
        .get(start..=last)?
        .iter()
        .filter_map(|entry| match entry {
            AgentThreadEntry::AssistantMessage(message) if !message.is_subagent_output => {
                Some(message)
            }
            _ => None,
        })
        .flat_map(|message| message.chunks.iter())
        .filter_map(|chunk| match chunk {
            AssistantMessageChunk::Message { block, .. } => Some(block.to_markdown(cx)),
            AssistantMessageChunk::Thought { .. } => None,
        })
        .filter(|text| !text.trim().is_empty())
        .collect();
    (!texts.is_empty()).then(|| texts.join("\n\n"))
}

/// The transcript the agent in the terminal `view` writes, as its hook events named it.
pub(crate) fn transcript_of(view: EntityId, cx: &App) -> Option<PathBuf> {
    cx.try_global::<AgentEvents>()?
        .seat(view)?
        .labels
        .get(TRANSCRIPT_LABEL)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
}

/// The last reply in the transcript at `path`, read from its last [`TAIL_BYTES`]. Blocking: the
/// background executor runs it.
pub(crate) fn read_reply(path: &Path) -> Result<String, &'static str> {
    const UNREADABLE: &str = "Claude Code's transcript could not be read.";
    let mut file = std::fs::File::open(path).map_err(|error| {
        log::warn!("review last reply: {}: {error}", path.display());
        UNREADABLE
    })?;
    let length = file.metadata().map_or(0, |metadata| metadata.len());
    let start = length.saturating_sub(TAIL_BYTES);
    let mut bytes = Vec::new();
    file.seek(SeekFrom::Start(start))
        .and_then(|_| file.take(TAIL_BYTES).read_to_end(&mut bytes))
        .map_err(|error| {
            log::warn!("review last reply: {}: {error}", path.display());
            UNREADABLE
        })?;
    let text = String::from_utf8_lossy(&bytes);
    // A read from inside the file starts inside a line.
    let text = if start > 0 {
        text.split_once('\n').map_or("", |(_, rest)| rest)
    } else {
        &text
    };
    reply_in(text).ok_or("Claude Code's transcript holds no reply yet.")
}

/// The last reply in a transcript's lines.
fn reply_in(jsonl: &str) -> Option<String> {
    let entries: Vec<Value> = jsonl
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .filter(|entry: &Value| !flag(entry, "isSidechain") && !flag(entry, "isMeta"))
        .collect();
    let last = entries
        .iter()
        .rposition(|entry| !assistant_texts(entry).is_empty())?;
    let start = entries
        .get(..last)?
        .iter()
        .rposition(is_prompt)
        .map_or(0, |at| at + 1);
    let texts: Vec<String> = entries
        .get(start..=last)?
        .iter()
        .flat_map(assistant_texts)
        .collect();
    Some(texts.join("\n\n"))
}

fn flag(entry: &Value, name: &str) -> bool {
    entry.get(name).and_then(Value::as_bool) == Some(true)
}

fn kind(value: &Value) -> Option<&str> {
    value.get("type").and_then(Value::as_str)
}

/// An assistant entry's text blocks.
fn assistant_texts(entry: &Value) -> Vec<String> {
    if kind(entry) != Some("assistant") {
        return Vec::new();
    }
    entry
        .pointer("/message/content")
        .and_then(Value::as_array)
        .map(|blocks| {
            blocks
                .iter()
                .filter(|block| kind(block) == Some("text"))
                .filter_map(|block| block.get("text").and_then(Value::as_str))
                .filter(|text| !text.trim().is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Whether a user entry is a prompt the person typed, not a tool's result.
fn is_prompt(entry: &Value) -> bool {
    if kind(entry) != Some("user") {
        return false;
    }
    match entry.pointer("/message/content") {
        Some(Value::String(text)) => !text.trim().is_empty(),
        Some(Value::Array(blocks)) => {
            blocks.iter().any(|block| kind(block) == Some("text"))
                && !blocks
                    .iter()
                    .any(|block| kind(block) == Some("tool_result"))
        }
        _ => false,
    }
}
