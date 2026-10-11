//! Past Claude Code and Codex sessions, read from each CLI's own files (#754).
//!
//! The workbench reads a session file's head and, for a long file, its tail, and hands both here;
//! nothing here touches the disk. A session is kept only when its CLI's own terminal UI started
//! it: Claude Code's records say `entrypoint: "cli"`, and Codex's `session_meta` names the TUI as
//! its originator with a user's thread. `claude -p`, the SDKs (the harness's and Zed's own agent's
//! sessions among them), subagents, reviews and Codex's desktop app are left out. A record that
//! does not parse is skipped, and a cut line at a window's edge is dropped. The formats are the
//! ones Claude Code 2.1.296 and Codex 0.162.0 write; neither CLI documents them.

use std::collections::HashMap;

use serde_json::Value;

use crate::AgentKind;
use crate::claude_events::{PromptOrigin, prompt_origin};

/// The most bytes of the user's prompts a session keeps, for the search.
const MAX_PROMPT_BYTES: usize = 64 * 1024;
/// How many characters of a prompt a search snippet shows around its match.
const SNIPPET_CHARS: usize = 240;
/// Codex's originators for its own terminal UI; `codex_cli_rs` in older releases.
const CODEX_TUI: [&str; 2] = ["codex-tui", "codex_cli_rs"];

/// One past session of an agent CLI, as its files tell it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliSession {
    /// The CLI.
    pub agent: AgentKind,
    /// Its id, which the CLI's resume takes.
    pub id: String,
    /// The folder it ran in.
    pub folder: String,
    /// The CLI's own title for it: Claude Code's custom or generated title, Codex's thread name.
    pub title: Option<String>,
    /// The user's own prompts, in order, up to 64 KiB of them.
    pub prompts: Vec<String>,
}

impl CliSession {
    /// The title a list shows: the CLI's own, else the first prompt on one line, else the id's
    /// start.
    #[must_use]
    pub fn shown_title(&self) -> String {
        self.title
            .clone()
            .or_else(|| self.prompts.first().map(|prompt| one_line(prompt)))
            .unwrap_or_else(|| format!("Session {}", self.id.chars().take(8).collect::<String>()))
    }

    /// Whether `query` (lowercase) is in the title or the folder; then the first prompt that holds
    /// it, with a snippet around the match. `None` when neither does.
    #[must_use]
    pub fn search(&self, query: &str) -> Option<Found> {
        if query.is_empty()
            || self.shown_title().to_lowercase().contains(query)
            || self.folder.to_lowercase().contains(query)
        {
            return Some(Found::Listed);
        }
        self.prompts
            .iter()
            .find_map(|prompt| snippet(prompt, query))
            .map(Found::Prompt)
    }
}

/// What a search found in a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Found {
    /// The title or the folder holds the query, or there is none.
    Listed,
    /// A prompt holds it: the words around the match, on one line.
    Prompt(String),
}

/// A session file's bytes as the workbench read them: the whole file, or its head and its tail.
#[derive(Debug, Clone, Copy)]
pub struct Windows<'a> {
    /// The file's start: the whole file when `tail` is `None`.
    pub head: &'a [u8],
    /// The file's end, for a file longer than both windows.
    pub tail: Option<&'a [u8]>,
}

impl<'a> Windows<'a> {
    /// The records of both windows, in order: each whole line that parses as JSON.
    fn records(self) -> impl Iterator<Item = Value> + 'a {
        let head = match self.tail {
            // The head's last line may be cut where the window ends.
            Some(_) => self
                .head
                .rsplit_once_byte(b'\n')
                .map_or(&[][..], |(whole, _)| whole),
            None => self.head,
        };
        // The tail's first line may be cut where the window starts.
        let tail = self
            .tail
            .and_then(|tail| tail.split_once_byte(b'\n').map(|(_, whole)| whole))
            .unwrap_or_default();
        head.split(|byte| *byte == b'\n')
            .chain(tail.split(|byte| *byte == b'\n'))
            .filter(|line| !line.is_empty())
            .filter_map(|line| serde_json::from_slice(line).ok())
    }
}

/// Splitting a byte slice at its first or last newline.
trait SplitByte {
    fn split_once_byte(&self, byte: u8) -> Option<(&[u8], &[u8])>;
    fn rsplit_once_byte(&self, byte: u8) -> Option<(&[u8], &[u8])>;
}

impl SplitByte for [u8] {
    fn split_once_byte(&self, byte: u8) -> Option<(&[u8], &[u8])> {
        let at = self.iter().position(|found| *found == byte)?;
        Some((self.get(..at)?, self.get(at + 1..)?))
    }

    fn rsplit_once_byte(&self, byte: u8) -> Option<(&[u8], &[u8])> {
        let at = self.iter().rposition(|found| *found == byte)?;
        Some((self.get(..at)?, self.get(at + 1..)?))
    }
}

/// A Claude Code session from its transcript, `file_stem` being the file's name without
/// `.jsonl`; `None` unless the CLI's own terminal UI started it and it holds a prompt of the
/// user's or a title.
#[must_use]
pub fn parse_claude(windows: Windows<'_>, file_stem: &str) -> Option<CliSession> {
    let mut id = None;
    let mut folder = None;
    let mut interactive = None;
    let (mut custom, mut generated) = (None, None);
    let mut prompts = Prompts::default();
    for record in windows.records() {
        let text = |key: &str| record.get(key).and_then(Value::as_str).map(str::to_string);
        if let Some(entrypoint) = record.get("entrypoint").and_then(Value::as_str) {
            interactive = interactive.or_else(|| Some(entrypoint == "cli"));
        }
        id = id.or_else(|| text("sessionId"));
        folder = folder.or_else(|| text("cwd"));
        match record.get("type").and_then(Value::as_str) {
            Some("custom-title") => custom = text("customTitle").or(custom),
            Some("ai-title") => generated = text("aiTitle").or(generated),
            Some("user") if claude_prompt_record(&record) => {
                if let Some(prompt) = record.pointer("/message/content").and_then(content_text) {
                    prompts.push(&prompt);
                }
            }
            _ => {}
        }
    }
    let title = custom
        .or(generated)
        .filter(|title| !title.trim().is_empty());
    if interactive != Some(true) || (prompts.list.is_empty() && title.is_none()) {
        return None;
    }
    Some(CliSession {
        agent: AgentKind::Claude,
        id: id.unwrap_or_else(|| file_stem.to_string()),
        folder: folder?,
        title: title.map(|title| one_line(&title)),
        prompts: prompts.list,
    })
}

/// Whether a Claude Code `user` record is a prompt the user typed: not meta, a sidechain's or a
/// tool's result.
fn claude_prompt_record(record: &Value) -> bool {
    let flag = |key: &str| record.get(key).and_then(Value::as_bool) == Some(true);
    !flag("isMeta") && !flag("isSidechain") && record.get("toolUseResult").is_none()
}

/// A Codex session from its rollout; `None` unless Codex's own terminal UI started it as a user's
/// thread.
///
/// It is untitled: Codex names its threads in its session index ([`codex_names`]), which changes
/// without the rollout.
#[must_use]
pub fn parse_codex(windows: Windows<'_>) -> Option<CliSession> {
    let mut records = windows.records();
    let meta = records.next()?;
    if meta.get("type").and_then(Value::as_str) != Some("session_meta") {
        return None;
    }
    let meta = meta.get("payload")?;
    let text = |key: &str| meta.get(key).and_then(Value::as_str);
    let from_tui = text("originator").is_some_and(|originator| CODEX_TUI.contains(&originator));
    let users_thread = text("thread_source").is_none_or(|source| source == "user")
        && meta.get("parent_thread_id").is_none_or(Value::is_null);
    if !from_tui || !users_thread {
        return None;
    }
    let id = text("id")?.to_string();
    let folder = text("cwd")?.to_string();
    let mut prompts = Prompts::default();
    for record in records {
        if record.get("type").and_then(Value::as_str) != Some("event_msg") {
            continue;
        }
        let Some(payload) = record.get("payload") else {
            continue;
        };
        let prompt = match payload.get("type").and_then(Value::as_str) {
            Some("item_completed") => payload
                .get("item")
                .filter(|item| item.get("type").and_then(Value::as_str) == Some("UserMessage"))
                .and_then(|item| item.get("content"))
                .and_then(content_text),
            // Releases before 0.162 wrote the user's prompt as its own event.
            Some("user_message") => payload
                .get("message")
                .and_then(Value::as_str)
                .map(str::to_string),
            _ => None,
        };
        if let Some(prompt) = prompt {
            prompts.push(&prompt);
        }
    }
    if prompts.list.is_empty() {
        return None;
    }
    Some(CliSession {
        agent: AgentKind::Codex,
        title: None,
        id,
        folder,
        prompts: prompts.list,
    })
}

/// The thread names in Codex's `session_index.jsonl`, by session id; a later line names a session
/// again over an earlier one.
#[must_use]
pub fn codex_names(index: &[u8]) -> HashMap<String, String> {
    Windows {
        head: index,
        tail: None,
    }
    .records()
    .filter_map(|record| {
        let id = record.get("id")?.as_str()?.to_string();
        let name = record.get("thread_name")?.as_str()?.to_string();
        Some((id, name))
    })
    .collect()
}

/// The text of a message's content: a string, or the `text` of each text part.
fn content_text(content: &Value) -> Option<String> {
    let text = match content {
        Value::String(text) => text.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter(|part| {
                part.get("type")
                    .and_then(Value::as_str)
                    .is_none_or(|kind| kind == "text" || kind == "input_text")
            })
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => return None,
    };
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// The user's prompts as they are found, up to [`MAX_PROMPT_BYTES`]: a slash command's envelope,
/// a harness's injection, a command's output and the compaction's continuation are not the user's
/// words.
#[derive(Default)]
struct Prompts {
    list: Vec<String>,
    bytes: usize,
}

impl Prompts {
    fn push(&mut self, prompt: &str) {
        if self.bytes >= MAX_PROMPT_BYTES
            || prompt.starts_with('<')
            || prompt_origin(prompt) != PromptOrigin::User
        {
            return;
        }
        self.bytes += prompt.len();
        self.list.push(prompt.to_string());
    }
}

/// Up to [`SNIPPET_CHARS`] characters of `prompt` around the first match of `query` (lowercase),
/// on one line, with `…` where it is cut; `None` when the prompt does not hold it.
fn snippet(prompt: &str, query: &str) -> Option<String> {
    let characters: Vec<char> = prompt.chars().collect();
    let wanted: Vec<char> = query.chars().collect();
    let at = (0..characters.len()).find(|start| {
        characters
            .get(*start..start + wanted.len())
            .is_some_and(|window| {
                window
                    .iter()
                    .zip(&wanted)
                    .all(|(character, wanted)| character.to_lowercase().eq(wanted.to_lowercase()))
            })
    })?;
    let start = at.saturating_sub((SNIPPET_CHARS - wanted.len().min(SNIPPET_CHARS)) / 2);
    let end = (start + SNIPPET_CHARS).min(characters.len());
    let mut words: String = characters.get(start..end)?.iter().collect();
    words = one_line(&words);
    if start > 0 {
        words.insert(0, '…');
    }
    if end < characters.len() {
        words.push('…');
    }
    Some(words)
}

/// `text` on one line, its runs of whitespace one space.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
