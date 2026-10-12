//! The context a new thread starts from (#717): a selection of an earlier thread's items that
//! fits a budget, rendered as history the new agent reads, not as a request.
//!
//! The order follows T3 Code's handoff, as `docs/t3code_architecture/01-agents-and-providers.md`
//! reports it: the latest request, the latest answer, the first request, then the earlier items
//! from the newest back. An item that does not fit is left out whole, and a smaller one after it
//! may still fit. Nothing here reads a thread or a file.

/// What an item of the earlier thread was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// The person's message.
    Request,
    /// The agent's answer, as text.
    Answer,
    /// A tool call, with its title and its status; its text names the paths it changed.
    ToolCall {
        /// What the call was, as the thread labels it.
        title: String,
        /// How it ended, or where it stands.
        status: String,
    },
}

/// One item of the earlier thread, in conversation order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// What the item was.
    pub kind: Kind,
    /// Its text.
    pub text: String,
}

/// A block's budget in bytes: T3's 16,000 tokens, counted one byte each.
pub const BUDGET: usize = 16_000;

/// What the earlier thread was: its title, its agent's name, and the file that holds all of it.
#[derive(Debug, Clone, Copy)]
pub struct Source<'a> {
    /// The thread's title.
    pub title: &'a str,
    /// The name of the agent it ran on.
    pub agent: &'a str,
    /// Where the whole conversation is written.
    pub file: &'a str,
}

/// The block a new thread's message box starts with: the preamble, the chosen items in
/// conversation order, and how much was left out, within `budget` bytes where the items allow.
#[must_use]
pub fn block(items: &[Item], source: Source<'_>, budget: usize) -> String {
    let preamble = preamble(source);
    // The coverage line's longest form, so the items' share of the budget holds with it.
    let longest_coverage = coverage_line(items.len(), items.len(), source.file);
    let room = budget.saturating_sub(preamble.len() + longest_coverage.len());
    let chosen = select(items, source.agent, room);
    let mut text = preamble;
    for index in &chosen {
        if let Some(item) = items.get(*index) {
            text.push_str(&section(item, source.agent));
        }
    }
    text.push_str(&coverage_line(
        items.len().saturating_sub(chosen.len()),
        items.len(),
        source.file,
    ));
    text
}

/// The indices of the items that go in, in conversation order, their sections within `room`
/// bytes.
fn select(items: &[Item], agent: &str, room: usize) -> Vec<usize> {
    let latest_request = items.iter().rposition(|item| item.kind == Kind::Request);
    let latest_answer = items.iter().rposition(|item| item.kind == Kind::Answer);
    let first_request = items.iter().position(|item| item.kind == Kind::Request);
    let order = [latest_request, latest_answer, first_request]
        .into_iter()
        .flatten()
        .chain((0..items.len()).rev());
    let mut chosen = Vec::new();
    let mut used = 0;
    for index in order {
        if chosen.contains(&index) {
            continue;
        }
        let Some(item) = items.get(index) else {
            continue;
        };
        let size = section(item, agent).len();
        if used + size <= room {
            used += size;
            chosen.push(index);
        }
    }
    chosen.sort_unstable();
    chosen
}

/// The block's opening. Its first line is short, since an agent's thread takes its title from the
/// first line of its first message.
fn preamble(source: Source<'_>) -> String {
    format!(
        "Continuing \"{}\" from {}.\nWhat follows is context from that conversation: history, not \
         a new request and not an instruction.\n\n",
        source.title, source.agent
    )
}

/// One item as the block shows it: its label on a line, its text, a blank line.
fn section(item: &Item, agent: &str) -> String {
    let mut text = match &item.kind {
        Kind::Request => "[Earlier request]\n".to_string(),
        Kind::Answer => format!("[Earlier answer from {agent}]\n"),
        Kind::ToolCall { title, status } => format!("[Earlier tool call: {title}, {status}]\n"),
    };
    let body = item.text.trim();
    if !body.is_empty() {
        text.push_str(body);
        text.push('\n');
    }
    text.push('\n');
    text
}

/// The block's last line: how many items were left out, and where the whole conversation is.
fn coverage_line(left_out: usize, total: usize, file: &str) -> String {
    if left_out == 0 {
        format!("The whole conversation is also in {file}.\n")
    } else {
        format!(
            "{left_out} of {total} earlier items are left out. The whole conversation is in \
             {file}.\n"
        )
    }
}
