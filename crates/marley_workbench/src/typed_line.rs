//! The typed line (#573): the System One layer's reading of a line at a shell's prompt that #557's
//! rules leave open, a command's name followed by plain words with an English marker among them.
//!
//! The line in front is the shell's prompt editor's text while it is open (#627), else what was
//! typed at the grid's prompt, read where the grid's suggestion is drawn, from the same frame.
//! Each change re-arms a 250 ms timer; when the line has stayed the
//! same that long, is open, holds nothing the redactor finds and its project may send, the layer
//! is asked once about that text. A reading is kept only while the line still reads the same, and
//! shows after it by [`crate::english`]'s hint. Enter never waits: the timer and the call live
//! beside the line, and an answer for a line that changed is dropped. Each call's outcome follows
//! it in the day's file: entered with its exit code, asked of an agent, edited, or cleared.

use std::collections::HashMap;
use std::time::Duration;

use gpui::{App, Context, Entity, EntityId, Global, Task, WeakEntity};
use marley_system_one::TYPED_LINE;
use marley_system_one::reading::{ABSTAINING, Reading, Signal};
use marley_terminal::BlockState;
use settings::{SettingsStore, SystemOneMode};
use terminal::Terminal;
use terminal_view::TerminalView;
use workspace::Workspace;

use crate::system_one::{self, Asking};

/// How long a line stays the same before it is asked about.
const QUIET: Duration = Duration::from_millis(250);

/// What a reading says the line is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Command,
    Request,
    Comment,
    CommandThenEnglish,
}

/// Each terminal's line, by the terminal's entity id.
#[derive(Default)]
struct Lines(HashMap<EntityId, Line>);

impl Global for Lines {}

#[derive(Default)]
struct Line {
    /// The workspace of the terminal's view, whose project the line is asked under.
    workspace: Option<WeakEntity<Workspace>>,
    /// The line in front.
    text: String,
    /// The quiet time's timer, dropped by any change.
    settle: Option<Task<()>>,
    /// The call about `text`, once its answer came.
    call: Option<Call>,
    /// The last line asked about and what it read as, so the same line is not asked again.
    last: Option<(String, Option<Kind>)>,
    /// A call whose line went to the shell, until its block opens.
    entering: Option<Entering>,
    /// A call whose line opened a block, by the block's index, until the block ends.
    running: Option<(String, usize)>,
}

struct Call {
    /// The call's row, for its outcome.
    row: Option<String>,
    text: String,
    kind: Option<Kind>,
    /// The index of the terminal's next block when the line was asked about.
    blocks: usize,
}

struct Entering {
    row: String,
    text: String,
    blocks: usize,
}

/// Follows each terminal view's blocks for the calls' outcomes; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(Lines::default());
    cx.observe_new(
        |view: &mut TerminalView, _, cx: &mut Context<TerminalView>| {
            let terminal = view.terminal().clone();
            let terminal_id = terminal.entity_id();
            line_mut(terminal_id, cx).workspace = Some(view.marley_workspace().clone());
            // A block's start and end notify the terminal.
            cx.observe(&terminal, |_, terminal, cx| follow_blocks(&terminal, cx))
                .detach();
            cx.on_release(move |_, cx| {
                if cx.has_global::<Lines>() {
                    let _forgotten = cx.global_mut::<Lines>().0.remove(&terminal_id);
                }
            })
            .detach();
        },
    )
    .detach();
    // A reading made under other settings, another mode or another allow list, goes.
    cx.observe_global::<SettingsStore>(|cx| {
        for line in cx.global_mut::<Lines>().0.values_mut() {
            line.settle = None;
            line.last = None;
            if let Some(call) = &mut line.call {
                call.kind = None;
            }
        }
    })
    .detach();
}

/// The grid's line, while the shell's editor is closed: what was typed at the prompt, and the
/// prompt left when a command starts. The suggestion hook calls it at each frame of `terminal`,
/// since typing reaches the terminal as wakeups, which notify no observer.
pub(crate) fn follow_grid(terminal: &Entity<Terminal>, cx: &mut App) {
    if crate::rich_input::holds_line_of(terminal.entity_id(), cx) {
        return;
    }
    let typed = crate::autosuggest::typed_text(terminal.read(cx));
    match typed {
        Some(text) => changed(terminal, &text, cx),
        None if !terminal.read(cx).marley_anchored().at_prompt() => entering(terminal, cx),
        // Scrolled back, or the cursor moved left: the line is as it was.
        None => {}
    }
}

/// Writes `entered` when the block a call's line opened shows, and `exit N` when it ends.
fn follow_blocks(terminal: &Entity<Terminal>, cx: &mut App) {
    let id = terminal.entity_id();
    let Some(line) = cx.try_global::<Lines>().and_then(|lines| lines.0.get(&id)) else {
        return;
    };
    let blocks = terminal.read(cx).blocks();
    if let Some(entering) = &line.entering {
        let opened = blocks
            .iter()
            .rev()
            .take_while(|block| block.index >= entering.blocks)
            .find(|block| block.command.trim() == entering.text.trim())
            .map(|block| block.index);
        if let Some(index) = opened {
            let row = entering.row.clone();
            system_one::outcome(&row, "entered".to_string(), cx);
            let line = line_mut(id, cx);
            line.entering = None;
            line.running = Some((row, index));
        }
        return;
    }
    let Some((row, index)) = &line.running else {
        return;
    };
    let Some(block) = blocks.iter().find(|block| block.index == *index) else {
        return;
    };
    if block.state == BlockState::Finished {
        let exit = block
            .exit_code
            .0
            .map_or_else(|| "exit unknown".to_string(), |code| format!("exit {code}"));
        let row = row.clone();
        line_mut(id, cx).running = None;
        system_one::outcome(&row, exit, cx);
    }
}

/// The index the terminal's next block will have.
fn next_block(terminal: &Entity<Terminal>, cx: &App) -> usize {
    terminal
        .read(cx)
        .blocks()
        .last()
        .map_or(0, |block| block.index + 1)
}

fn line_mut(id: EntityId, cx: &mut App) -> &mut Line {
    cx.default_global::<Lines>().0.entry(id).or_default()
}

/// The line in front of `terminal` is now `text`: a call about another line gets its outcome, and
/// an open line is asked about once it has stayed the same for [`QUIET`].
pub(crate) fn changed(terminal: &Entity<Terminal>, text: &str, cx: &mut App) {
    let id = terminal.entity_id();
    let line = line_mut(id, cx);
    if line.text == text {
        return;
    }
    line.text = text.to_string();
    line.settle = None;
    if let Some(row) = line.call.take().and_then(|call| call.row) {
        let outcome = if text.trim().is_empty() {
            "cleared"
        } else {
            "edited"
        };
        system_one::outcome(&row, outcome.to_string(), cx);
    }
    if system_one::use_mode(TYPED_LINE.name, cx) == SystemOneMode::Off
        || !crate::english::open_case(text, terminal.read(cx), cx)
        || crate::mcp::model_redactor(cx).redact(text).count > 0
    {
        return;
    }
    let terminal = terminal.downgrade();
    let text = text.to_string();
    let settle = cx.spawn(async move |cx| {
        cx.background_executor().timer(QUIET).await;
        cx.update(|cx| {
            if let Some(terminal) = terminal.upgrade() {
                settled(&terminal, text, cx);
            }
        });
    });
    line_mut(id, cx).settle = Some(settle);
}

/// Asks the layer about `text`, still the line in front of `terminal` after the quiet time, unless
/// it was asked about last, or its project may send nothing.
fn settled(terminal: &Entity<Terminal>, text: String, cx: &mut App) {
    let id = terminal.entity_id();
    let blocks = next_block(terminal, cx);
    let line = line_mut(id, cx);
    if line.text != text {
        return;
    }
    let cached = line
        .last
        .as_ref()
        .filter(|(last, _)| *last == text)
        .map(|(_, kind)| *kind);
    if let Some(kind) = cached {
        line.call = Some(Call {
            row: None,
            text,
            kind,
            blocks,
        });
        terminal.update(cx, |_, cx| cx.notify());
        crate::rich_input::refresh_hint(id, cx);
        return;
    }
    let Some(workspace) = line.workspace.as_ref().and_then(WeakEntity::upgrade) else {
        return;
    };
    let asking = asking(terminal, &workspace, &text, cx);
    // An unlisted project's lines are not asked about at all, so they leave no refusal rows.
    if system_one::detail(&asking, cx).is_err() {
        return;
    }
    let asked = system_one::ask(TYPED_LINE, &asking, cx);
    let terminal = terminal.downgrade();
    // Never dropped with the line: the call's row is written when its answer comes.
    cx.spawn(async move |cx| {
        let asked = asked.await;
        cx.update(|cx| {
            if let Some(terminal) = terminal.upgrade() {
                answered(&terminal, text, &asked, blocks, cx);
            }
        });
    })
    .detach();
}

/// Keeps the reading of `text` while it is still the line in front, and draws it.
fn answered(
    terminal: &Entity<Terminal>,
    text: String,
    asked: &system_one::Asked,
    blocks: usize,
    cx: &mut App,
) {
    let row = asked
        .row
        .as_ref()
        .filter(|_| !asked.reading.failed())
        .map(|row| row.id.clone());
    let line = line_mut(terminal.entity_id(), cx);
    if line.text != text {
        if let Some(row) = row {
            let outcome = "dropped: the line changed before the reading came".to_string();
            system_one::outcome(&row, outcome, cx);
        }
        return;
    }
    let kind = kind_of(&asked.reading);
    line.last = Some((text.clone(), kind));
    // A line sent earlier whose block never showed is not waited on any more.
    line.entering = None;
    line.call = Some(Call {
        row,
        text,
        kind,
        blocks,
    });
    // The grid's slot is drawn from the terminal's frame, and the editor's hint from its edits.
    terminal.update(cx, |_, cx| cx.notify());
    crate::rich_input::refresh_hint(terminal.entity_id(), cx);
}

/// What a model's reading says the line is, when it says it clearly.
fn kind_of(reading: &Reading) -> Option<Kind> {
    let Reading::Model(reads) = reading else {
        return None;
    };
    reads.iter().find_map(|read| match &read.signal {
        Signal::Choice { option, .. }
            if read.key == "kind" && !ABSTAINING.contains(&option.as_str()) =>
        {
            match option.as_str() {
                "command" => Some(Kind::Command),
                "request" => Some(Kind::Request),
                "comment" => Some(Kind::Comment),
                "command_then_english" => Some(Kind::CommandThenEnglish),
                _ => None,
            }
        }
        _ => None,
    })
}

/// What the layer is told about `text`: the line, masked by the layer, and facts computed here.
fn asking(
    terminal: &Entity<Terminal>,
    workspace: &Entity<Workspace>,
    text: &str,
    cx: &App,
) -> Asking {
    let (folders, local) = system_one::project_of(workspace.read(cx), cx);
    let project = system_one::project_name(&folders);
    let first = text.split_whitespace().next().unwrap_or_default();
    let history = crate::autosuggest::history(terminal, cx);
    let in_history = history
        .iter()
        .any(|command| command.starts_with(text.trim()));
    let last_exit = terminal
        .read(cx)
        .blocks()
        .iter()
        .rev()
        .find(|block| block.state == BlockState::Finished)
        .and_then(|block| block.exit_code.0)
        .map_or_else(|| "none".to_string(), |code| code.to_string());
    let rules = if crate::english::reads_as_english(text, terminal.read(cx), cx) {
        "command_then_english"
    } else {
        "command"
    };
    Asking {
        subject: terminal.entity_id().to_string(),
        project: project.clone(),
        folders,
        local,
        // The first word is a fact only because code knows it as a command (open_case).
        facts: vec![
            ("project", project),
            ("first word", first.to_string()),
            (
                "first word is",
                crate::english::command_source(first, cx).to_string(),
            ),
            ("words", text.split_whitespace().count().to_string()),
            (
                "English markers after the first word",
                marley_terminal::english::marker_count(text).to_string(),
            ),
            (
                "a command in the history starts with the line",
                if in_history { "yes" } else { "no" }.to_string(),
            ),
            ("the last command's exit code", last_exit),
        ],
        texts: vec![("line", text.to_string())],
        verdict: Some(system_one::choice_verdict("kind", rules)),
    }
}

/// The reading to show after `text` at `terminal`, with the use's mode, while `text` is the line
/// asked about and the mode shows readings.
pub(crate) fn shown(terminal: EntityId, text: &str, cx: &App) -> Option<(Kind, SystemOneMode)> {
    let mode = system_one::use_mode(TYPED_LINE.name, cx);
    if !matches!(mode, SystemOneMode::Suggest | SystemOneMode::Act) {
        return None;
    }
    let call = cx.try_global::<Lines>()?.0.get(&terminal)?.call.as_ref()?;
    if call.text != text {
        return None;
    }
    Some((call.kind?, mode))
}

/// The line in front of `terminal` goes to the shell: a call about it waits for its block.
pub(crate) fn entering(terminal: &Entity<Terminal>, cx: &mut App) {
    let line = line_mut(terminal.entity_id(), cx);
    line.settle = None;
    if let Some(call) = line.call.take()
        && let Some(row) = call.row
    {
        line.entering = Some(Entering {
            row,
            text: call.text,
            blocks: call.blocks,
        });
    }
    line.text.clear();
}

/// The line in front of `terminal` went to an agent with Ctrl+Shift+Enter.
pub(crate) fn asked_agent(terminal: &Entity<Terminal>, cx: &mut App) {
    let line = line_mut(terminal.entity_id(), cx);
    line.settle = None;
    line.text.clear();
    if let Some(row) = line.call.take().and_then(|call| call.row) {
        system_one::outcome(&row, "asked the agent".to_string(), cx);
    }
}
