//! PURE — a terminal's commands as blocks anchored in its one scrollback (the three-prong
//! plan's D2).
//!
//! Zed's terminal keeps one grid and one scrollback, so a block does not copy its output: it
//! records where its lines are, as absolute lines counted from the first line the grid ever
//! held, the lines the vendored event loop reports with each hook. [`AnchoredBlocks::apply`]
//! takes each decoded hook with its line:
//! - `InitShell` registers the shell and forgets a staged prompt;
//! - `Precmd` finishes the running block at its line, with its exit code, and stages the next
//!   prompt's metadata and line;
//! - `Preexec` finishes any block still running, without an exit code, and opens a running block
//!   whose output starts at its line, with the staged prompt; its command is verified when the
//!   frame carried the terminal's nonce ([`AnchoredBlocks::with_nonce`]);
//! - `Bootstrapped` changes nothing here.
//!
//! A `Preexec` or `Precmd` before any `InitShell` is refused, as the gpui era's `SessionModel`
//! refuses it. The terminal [`AnchoredBlocks::stamp`]s the blocks as each hook applies, which
//! keeps when each command started and ended beside its block (#491).
//!
//! [`visible_spans`] says which blocks a viewport shows and over which of its rows, for the
//! terminal view to draw them, [`block_scroll`] where to scroll to show the previous or the next
//! block, and [`bottom_shift`] how far down to draw a viewport so its content sits on the bottom
//! edge (T1).
//!
//! A resize that changes the width rewraps the grid and moves every row after the first wrapped
//! one, so [`AnchoredBlocks::rewrap`] carries the anchors across it through a [`RowsView`] of the
//! grid before and after (#544).

use std::ops::Range;
use std::time::{Duration, SystemTime};

use crate::apply::ApplyHookError;
use crate::block::{BlockState, ExitCode, PromptInfo};
use crate::dcs::DcsHook;

/// One command's block, anchored in the terminal's scrollback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchoredBlock {
    /// Its place among the terminal's blocks, from 0.
    pub index: usize,
    /// The command text.
    pub command: String,
    /// Whether the `Preexec` that opened the block carried the terminal's nonce, so the command
    /// is the one the shell's own hook reported and not one that output printed.
    pub command_verified: bool,
    /// [`BlockState::Running`] until a hook finishes it, then [`BlockState::Finished`].
    pub state: BlockState,
    /// The exit code a `Precmd` reported, or none.
    pub exit_code: ExitCode,
    /// The metadata of the prompt the command was typed at.
    pub prompt: PromptInfo,
    /// The absolute line the prompt started on, when the `Precmd` before it was seen.
    pub prompt_line: Option<u64>,
    /// The absolute line the output starts on.
    pub output_start: u64,
    /// The absolute line after the output, once the block finished.
    pub output_end: Option<u64>,
}

impl AnchoredBlock {
    /// The block as Markdown, for a note or a message (#554): a fence holding `$ ` and the
    /// command, then `output`, or a line saying the output is no longer kept; after it one line
    /// with the exit, how long it `took`, and the folder with its branch. The fence is longer than
    /// any run of backticks inside, so the block's text cannot close it.
    #[must_use]
    pub fn markdown(&self, output: Option<&str>, took: Option<Duration>) -> String {
        let body = output.map_or_else(
            || {
                format!(
                    "$ {}\n(the output is no longer in the scrollback)",
                    self.command
                )
            },
            |output| match output.trim_end_matches('\n') {
                "" => format!("$ {}", self.command),
                output => format!("$ {}\n{output}", self.command),
            },
        );
        let longest = body
            .split(|character| character != '`')
            .map(str::len)
            .max()
            .unwrap_or(0);
        let fence = "`".repeat(longest.max(2) + 1);
        let exit = match (self.state, self.exit_code.0) {
            (BlockState::Running | BlockState::Pending, _) => Some("running".to_string()),
            (BlockState::Finished, Some(code)) => Some(format!("exit {code}")),
            (BlockState::Finished, None) => None,
        };
        let folder = self.prompt.pwd.as_ref().map(|pwd| {
            self.prompt
                .git_branch
                .as_ref()
                .map_or_else(|| pwd.clone(), |branch| format!("{pwd} ({branch})"))
        });
        let facts: Vec<String> = [exit, took.map(duration_words), folder]
            .into_iter()
            .flatten()
            .collect();
        if facts.is_empty() {
            format!("{fence}\n{body}\n{fence}\n")
        } else {
            format!("{fence}\n{body}\n{fence}\n{}\n", facts.join(" · "))
        }
    }
}

/// How long a command ran, in words: `340 ms` under a second, `2.5 s` after.
fn duration_words(took: Duration) -> String {
    if took < Duration::from_secs(1) {
        format!("{} ms", took.as_millis())
    } else {
        format!("{:.1} s", took.as_secs_f64())
    }
}

/// When a block's command started and, once it finished, when it ended: the times the terminal saw
/// the shell's hooks arrive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockTimes {
    /// When the `Preexec` that opened the block was applied.
    pub started: SystemTime,
    /// When the hook that finished it was applied.
    pub finished: Option<SystemTime>,
}

/// Which shell sent a frame, by the nonce it carried (#526).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Shell {
    /// The terminal's own shell.
    Local,
    /// The shell Marley's `ssh` started on a host.
    Remote(String),
    /// Output that printed a frame, or a shell that signs none.
    Unknown,
}

/// The shell waiting at a terminal's prompt (#526).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptShell<'a> {
    /// The terminal's own shell.
    Local,
    /// The shell Marley's `ssh` started on this host.
    Host(&'a str),
}

impl<'a> PromptShell<'a> {
    /// The host the shell runs on; none for the local shell.
    #[must_use]
    pub const fn host(self) -> Option<&'a str> {
        match self {
            Self::Local => None,
            Self::Host(host) => Some(host),
        }
    }
}

/// A connection Marley's `ssh` announced: the nonce its host's shell signs with, and the host.
#[derive(Debug, Clone)]
struct Connection {
    session: String,
    host: String,
}

/// A terminal's blocks, in order, and the prompt staged for the next one.
#[derive(Debug, Default)]
pub struct AnchoredBlocks {
    blocks: Vec<AnchoredBlock>,
    /// Each block's times, by index, as [`AnchoredBlocks::stamp`] kept them.
    times: Vec<BlockTimes>,
    registered: bool,
    staged: Option<(PromptInfo, u64)>,
    nonce: Option<String>,
    /// Where the command typed at the staged prompt starts, as an absolute line and a column.
    input_start: Option<(u64, usize)>,
    /// The file the shell keeps its history in, when it named one.
    history_file: Option<String>,
    /// The ssh connection whose host's shell the terminal accepts frames from, until the local
    /// shell's next prompt (#526).
    connection: Option<Connection>,
    /// The shell whose prompt is staged.
    prompt_shell: Option<Shell>,
    /// Each block's host, by index; none for the local shell's.
    hosts: Vec<Option<String>>,
    /// How many inputs the terminal noted, for a selection that ends at the next one (#554).
    inputs: u64,
}

impl AnchoredBlocks {
    /// No blocks yet, for a terminal that gave its program `nonce`.
    #[must_use]
    pub fn with_nonce(nonce: String) -> Self {
        Self {
            nonce: Some(nonce),
            ..Self::default()
        }
    }

    /// Applies `hook`, which arrived at the absolute line `line`.
    ///
    /// # Errors
    ///
    /// [`ApplyHookError::MissingSession`] for a `Preexec` or `Precmd` before any `InitShell`;
    /// the blocks stay as they were.
    pub fn apply(&mut self, hook: DcsHook, line: u64) -> Result<(), ApplyHookError> {
        let (nonce, hook) = match hook {
            DcsHook::Signed { nonce, hook } => (Some(nonce), *hook),
            hook => (None, hook),
        };
        let shell = self.shell_of(nonce.as_deref());
        match hook {
            DcsHook::InitShell { .. } => {
                // A host's shell has started: the `ssh` block ends there, with no exit code, and
                // its output is the connection's banner (#526).
                if matches!(shell, Shell::Remote(_)) {
                    self.finish_running(line, ExitCode(None));
                }
                self.registered = true;
                self.staged = None;
                self.prompt_shell = None;
                self.input_start = None;
            }
            DcsHook::Preexec(value) => {
                self.require_shell()?;
                self.input_start = None;
                self.finish_running(line, ExitCode(None));
                let (prompt, prompt_line) = self.staged.take().map_or_else(
                    || (PromptInfo::default(), None),
                    |(prompt, at)| (prompt, Some(at)),
                );
                // A command the terminal's shell or its ssh host's reported, not one output
                // printed (#526).
                let shell = self.shell_of(value.nonce.as_deref());
                let command_verified = shell != Shell::Unknown;
                self.prompt_shell = None;
                self.hosts.push(match shell {
                    Shell::Remote(host) => Some(host),
                    Shell::Local | Shell::Unknown => None,
                });
                self.blocks.push(AnchoredBlock {
                    index: self.blocks.len(),
                    command: value.command,
                    command_verified,
                    state: BlockState::Running,
                    exit_code: ExitCode(None),
                    prompt,
                    prompt_line,
                    output_start: line,
                    output_end: None,
                });
            }
            DcsHook::Precmd(value) => {
                self.require_shell()?;
                // The local shell's prompt ends the connection: its nonce is accepted no longer.
                if shell == Shell::Local {
                    self.connection = None;
                }
                self.finish_running(line, value.exit_code);
                self.staged = Some((value.prompt, line));
                self.prompt_shell = Some(shell);
                self.input_start = None;
            }
            DcsHook::Bootstrapped { .. } | DcsHook::Signed { .. } => {}
            // A history file names a file on this machine only when the local shell names it.
            DcsHook::History { file } => {
                if shell == Shell::Local {
                    self.history_file = Some(file);
                }
            }
            // Only the terminal's own shell announces a connection, so output cannot.
            DcsHook::Remote { host, session } => {
                if shell == Shell::Local {
                    self.connection = Some(Connection { session, host });
                }
            }
        }
        Ok(())
    }

    /// Which shell signs with `nonce`: the terminal's, the announced connection's host's, or
    /// neither.
    fn shell_of(&self, nonce: Option<&str>) -> Shell {
        match nonce {
            Some(nonce) if self.nonce.as_deref() == Some(nonce) => Shell::Local,
            Some(nonce) => self
                .connection
                .as_ref()
                .filter(|connection| connection.session == nonce)
                .map_or(Shell::Unknown, |connection| {
                    Shell::Remote(connection.host.clone())
                }),
            None => Shell::Unknown,
        }
    }

    /// The host block `index`'s shell ran on, none for the local shell (#526).
    #[must_use]
    pub fn block_host(&self, index: usize) -> Option<&str> {
        self.hosts.get(index).and_then(Option::as_deref)
    }

    /// The shell waiting at the prompt; none when no prompt is staged or its shell is unknown
    /// (#526).
    #[must_use]
    pub fn prompt_shell(&self) -> Option<PromptShell<'_>> {
        match self.prompt_shell.as_ref()? {
            Shell::Local => Some(PromptShell::Local),
            Shell::Remote(host) => Some(PromptShell::Host(host)),
            Shell::Unknown => None,
        }
    }

    /// Whether `block` may be rerun now: its command was reported by a known shell, it finished,
    /// and that same shell waits at the prompt, so it runs where it ran (#526).
    #[must_use]
    pub fn rerun_offered(&self, block: &AnchoredBlock) -> bool {
        block.command_verified
            && block.state == BlockState::Finished
            && self
                .prompt_shell()
                .is_some_and(|shell| shell.host() == self.block_host(block.index))
    }

    /// The blocks, oldest first.
    #[must_use]
    pub fn blocks(&self) -> &[AnchoredBlock] {
        &self.blocks
    }

    /// Stamps the blocks at `now`, right after a hook applied: a block opened since the last stamp
    /// started now, and one finished since then ended now.
    pub fn stamp(&mut self, now: SystemTime) {
        while self.times.len() < self.blocks.len() {
            self.times.push(BlockTimes {
                started: now,
                finished: None,
            });
        }
        // One block runs at a time, so only the newest can have finished since the last stamp:
        // the walk back stops at the first block that has its end.
        for (block, times) in self.blocks.iter().zip(self.times.iter_mut()).rev() {
            if times.finished.is_some() {
                break;
            }
            if block.state == BlockState::Finished {
                times.finished = Some(now);
            }
        }
    }

    /// When block `index` started and ended, once the terminal stamped it.
    #[must_use]
    pub fn times(&self, index: usize) -> Option<BlockTimes> {
        self.times.get(index).copied()
    }

    /// Whether the shell waits at a prompt: a `Precmd` came, and no `Preexec` since.
    #[must_use]
    pub const fn at_prompt(&self) -> bool {
        self.staged.is_some()
    }

    /// Notes that input was typed with the cursor at the absolute `line` and `column`. The first
    /// such point after a prompt is where the command being typed at it starts.
    pub const fn note_input(&mut self, line: u64, column: usize) {
        self.inputs = self.inputs.wrapping_add(1);
        if self.staged.is_some() && self.input_start.is_none() {
            self.input_start = Some((line, column));
        }
    }

    /// How many inputs [`AnchoredBlocks::note_input`] has noted: a block selected at one count
    /// stays selected until it moves (#554).
    #[must_use]
    pub const fn inputs(&self) -> u64 {
        self.inputs
    }

    /// Where the command being typed at the prompt starts, once a key has been typed there.
    #[must_use]
    pub const fn input_start(&self) -> Option<(u64, usize)> {
        self.input_start
    }

    /// The file the shell keeps its history in, when it named one.
    #[must_use]
    pub fn history_file(&self) -> Option<&str> {
        self.history_file.as_deref()
    }

    const fn require_shell(&self) -> Result<(), ApplyHookError> {
        if self.registered {
            Ok(())
        } else {
            Err(ApplyHookError::MissingSession)
        }
    }

    /// Finishes the running block, if any, with its output ending at `line`.
    fn finish_running(&mut self, line: u64, exit_code: ExitCode) {
        if let Some(block) = self
            .blocks
            .last_mut()
            .filter(|block| block.state == BlockState::Running)
        {
            block.state = BlockState::Finished;
            block.exit_code = exit_code;
            block.output_end = Some(line);
        }
    }

    /// Carries every anchor across a resize that rewrapped the grid from `before` to `after`.
    ///
    /// The anchors are each block's prompt line, output start and output end, the staged
    /// prompt's line, and where the prompt's input started (#544). An anchor already evicted
    /// before stays as it was; one whose logical line the resize dropped off the top comes back
    /// evicted.
    pub fn rewrap(&mut self, before: &RowsView, after: &RowsView) {
        let (before_lines, after_lines) = (before.logical_lines(), after.logical_lines());
        let carry = |line: u64, column: usize| {
            before
                .place_of(&before_lines, line, column)
                .map_or((line, column), |place| after.line_of(&after_lines, place))
        };
        for block in &mut self.blocks {
            if let Some(line) = block.prompt_line.as_mut() {
                *line = carry(*line, 0).0;
            }
            block.output_start = carry(block.output_start, 0).0;
            if let Some(line) = block.output_end.as_mut() {
                *line = carry(*line, 0).0;
            }
        }
        if let Some((_, line)) = self.staged.as_mut() {
            *line = carry(*line, 0).0;
        }
        if let Some((line, column)) = self.input_start.as_mut() {
            (*line, *column) = carry(*line, *column);
        }
    }
}

/// A grid's rows as a rewrap sees them (#544).
///
/// alacritty rewraps logical lines, a row and the rows its last cell's `WRAPLINE` continues into,
/// when the width changes, and keeps the cursor inside its logical line; so a place counted in
/// logical lines from the cursor's, with a character offset inside its own line, survives the
/// rewrap, where an absolute line does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowsView {
    /// The absolute line of the grid's first row.
    pub first: u64,
    /// The cursor's row, counted from the first.
    pub cursor: usize,
    /// Whether each row, from the first to the screen's last, continues into the next.
    pub wraps: Vec<bool>,
    /// The grid's width in columns.
    pub columns: usize,
}

/// Each row's logical line, and the row each logical line starts on.
#[derive(Debug)]
struct LogicalLines {
    line_of_row: Vec<usize>,
    starts: Vec<usize>,
}

/// An anchor as a rewrap keeps it: logical lines from the cursor's (positive above it), and the
/// character offset inside its own logical line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Place {
    above: Distance,
    offset: usize,
}

/// How many logical lines an anchor sits above or below the cursor's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Distance {
    Above(usize),
    Below(usize),
}

impl RowsView {
    /// Each row's logical line and each logical line's first row.
    fn logical_lines(&self) -> LogicalLines {
        let mut line_of_row = Vec::with_capacity(self.wraps.len());
        let mut starts = Vec::new();
        for row in 0..self.wraps.len() {
            let continued = row
                .checked_sub(1)
                .and_then(|previous| self.wraps.get(previous))
                .copied()
                .unwrap_or(false);
            if !continued {
                starts.push(row);
            }
            line_of_row.push(starts.len().saturating_sub(1));
        }
        LogicalLines {
            line_of_row,
            starts,
        }
    }

    /// Where `line` and `column` sit as a rewrap keeps it, or `None` for a line outside the
    /// view (already evicted, or past the last row).
    fn place_of(&self, lines: &LogicalLines, line: u64, column: usize) -> Option<Place> {
        let row = usize::try_from(line.checked_sub(self.first)?).ok()?;
        let logical = *lines.line_of_row.get(row)?;
        let start = *lines.starts.get(logical)?;
        let cursor_logical = *lines.line_of_row.get(self.cursor)?;
        let above = if logical <= cursor_logical {
            Distance::Above(cursor_logical - logical)
        } else {
            Distance::Below(logical - cursor_logical)
        };
        let offset = (row - start)
            .saturating_mul(self.columns.max(1))
            .saturating_add(column);
        Some(Place { above, offset })
    }

    /// The absolute line and column of `place` in this view. A logical line above the view's
    /// first row, which the rewrap dropped, comes back as the line before it, so it reads as
    /// evicted; one past the last row comes back on the last.
    fn line_of(&self, lines: &LogicalLines, place: Place) -> (u64, usize) {
        let evicted = (self.first.saturating_sub(1), 0);
        let Some(&cursor_logical) = lines.line_of_row.get(self.cursor) else {
            return evicted;
        };
        let target = match place.above {
            Distance::Above(count) => match cursor_logical.checked_sub(count) {
                Some(target) => target,
                None => return evicted,
            },
            Distance::Below(count) => cursor_logical
                .saturating_add(count)
                .min(lines.starts.len().saturating_sub(1)),
        };
        let Some(&start) = lines.starts.get(target) else {
            return evicted;
        };
        let end = lines
            .starts
            .get(target + 1)
            .copied()
            .unwrap_or(lines.line_of_row.len());
        let columns = self.columns.max(1);
        let rows_in_line = end.saturating_sub(start).max(1);
        let row = start + (place.offset / columns).min(rows_in_line - 1);
        let line = self
            .first
            .saturating_add(u64::try_from(row).unwrap_or(u64::MAX));
        (line, place.offset % columns)
    }
}

/// Where a block sits in a viewport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockSpan {
    /// The block's place among the terminal's blocks.
    pub index: usize,
    /// The viewport rows the block covers, counted from the top row, 0.
    pub rows: Range<usize>,
    /// Whether the block's first line is in the viewport.
    pub starts_in_view: bool,
    /// Whether the block is running or finished.
    pub state: BlockState,
    /// The block's exit code, if it has one.
    pub exit_code: ExitCode,
}

/// The blocks a viewport shows, in order, each with the rows it covers there.
///
/// The viewport is `screen_lines` rows from the absolute line `top`. A block starts at its
/// prompt's line, or at its output's when no prompt was seen; it ends before its `output_end`,
/// or, while it runs, after the absolute line `cursor_line`, where its output is still being
/// written. A block with no line in the viewport is left out.
#[must_use]
pub fn visible_spans(
    blocks: &[AnchoredBlock],
    top: u64,
    screen_lines: usize,
    cursor_line: u64,
) -> Vec<BlockSpan> {
    let bottom = top.saturating_add(u64::try_from(screen_lines).unwrap_or(u64::MAX));
    let row = |line: u64| usize::try_from(line - top).ok();
    blocks
        .iter()
        .filter_map(|block| {
            let start = block.prompt_line.unwrap_or(block.output_start);
            let end = block
                .output_end
                .unwrap_or_else(|| cursor_line.saturating_add(1));
            let (first, last) = (start.max(top), end.min(bottom));
            if first >= last {
                return None;
            }
            Some(BlockSpan {
                index: block.index,
                rows: row(first)?..row(last)?,
                starts_in_view: start >= top,
                state: block.state,
                exit_code: block.exit_code,
            })
        })
        .collect()
}

/// The scroll offset that shows the previous or the next block from a viewport's top.
///
/// The viewport's top is `display_offset` lines above `screen_top`, the absolute line of the
/// live screen's top. Going back, the target is the last block starting above that line; going
/// forward, the first starting below it. The offset puts the target's first line at the top:
/// 0 when that line is on the live screen, and at most `history`. `None` when no block starts
/// that way.
#[must_use]
pub fn block_scroll(
    blocks: &[AnchoredBlock],
    screen_top: u64,
    display_offset: usize,
    history: usize,
    forward: bool,
) -> Option<usize> {
    let top = screen_top.saturating_sub(u64::try_from(display_offset).unwrap_or(u64::MAX));
    let mut starts = blocks
        .iter()
        .map(|block| block.prompt_line.unwrap_or(block.output_start));
    let target = if forward {
        starts.find(|&start| start > top)?
    } else {
        starts.rev().find(|&start| start < top)?
    };
    let offset = usize::try_from(screen_top.saturating_sub(target)).unwrap_or(usize::MAX);
    Some(offset.min(history))
}

/// How many rows down to draw a viewport, so the live screen's last used row sits on the bottom
/// edge.
///
/// `empty_bottom_rows` is how many rows of the live screen lie below its content. Scrolled back
/// by `display_offset` rows, the shift is that much smaller, so the history appears above the
/// content instead of the content moving. The alternate screen keeps its own layout: none.
#[must_use]
pub const fn bottom_shift(
    empty_bottom_rows: usize,
    display_offset: usize,
    alt_screen: bool,
) -> usize {
    if alt_screen {
        0
    } else {
        empty_bottom_rows.saturating_sub(display_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ShellSessionId;
    use crate::dcs::{PrecmdValue, PreexecValue};

    fn init() -> DcsHook {
        DcsHook::InitShell {
            shell_session_id: ShellSessionId(1),
        }
    }

    fn preexec(command: &str) -> DcsHook {
        DcsHook::Preexec(PreexecValue {
            command: command.to_string(),
            nonce: None,
        })
    }

    fn precmd(exit: i32, pwd: &str) -> DcsHook {
        DcsHook::Precmd(PrecmdValue {
            exit_code: ExitCode(Some(exit)),
            prompt: PromptInfo {
                pwd: Some(pwd.to_string()),
                ..PromptInfo::default()
            },
        })
    }

    fn pwd(pwd: &str) -> PromptInfo {
        PromptInfo {
            pwd: Some(pwd.to_string()),
            ..PromptInfo::default()
        }
    }

    /// A block whose output starts at `output_start` and, once it finished, ends before
    /// `output_end`.
    fn spanned(
        index: usize,
        prompt_line: Option<u64>,
        output_start: u64,
        output_end: Option<u64>,
    ) -> AnchoredBlock {
        AnchoredBlock {
            index,
            command: format!("command {index}"),
            command_verified: false,
            state: if output_end.is_some() {
                BlockState::Finished
            } else {
                BlockState::Running
            },
            exit_code: ExitCode(output_end.map(|_| 0)),
            prompt: PromptInfo::default(),
            prompt_line,
            output_start,
            output_end,
        }
    }

    fn span(index: usize, rows: Range<usize>, starts_in_view: bool, finished: bool) -> BlockSpan {
        BlockSpan {
            index,
            rows,
            starts_in_view,
            state: if finished {
                BlockState::Finished
            } else {
                BlockState::Running
            },
            exit_code: ExitCode(finished.then_some(0)),
        }
    }

    #[test]
    fn a_viewport_shows_the_rows_of_each_block_it_holds() {
        // A viewport of 10 rows from line 100.
        let blocks = [
            spanned(0, Some(80), 81, Some(90)),
            spanned(1, Some(95), 97, Some(104)),
            spanned(2, Some(104), 105, Some(108)),
            spanned(3, None, 108, Some(112)),
            spanned(4, Some(112), 113, None),
        ];
        assert_eq!(
            visible_spans(&blocks, 100, 10, 115),
            [
                // Started above: its rows from the top, and its first line out of view.
                span(1, 0..4, false, true),
                span(2, 4..8, true, true),
                // No prompt seen: it starts at its output.
                span(3, 8..10, true, true),
            ]
        );
        // The running block runs to the cursor's line; with the viewport lower it shows.
        assert_eq!(
            visible_spans(&blocks, 110, 10, 115),
            [span(3, 0..2, false, true), span(4, 2..6, true, false)]
        );
    }

    #[test]
    fn a_block_with_no_line_in_the_viewport_is_left_out() {
        let blocks = [
            spanned(0, Some(10), 11, Some(20)),
            spanned(1, None, 30, Some(30)),
            spanned(2, Some(50), 51, Some(60)),
        ];
        // Above, empty, and below a viewport of lines 25 to 34.
        assert!(visible_spans(&blocks, 25, 10, 40).is_empty());
        assert!(visible_spans(&[], 0, 10, 0).is_empty());
    }

    #[test]
    fn the_block_keys_scroll_to_each_blocks_start_from_the_viewports_top() {
        // Blocks starting at lines 10, 40 and 70, and one with no prompt seen at 95; the live
        // screen's top at line 100, with 90 lines of history above it.
        let blocks = [
            spanned(0, Some(10), 11, Some(40)),
            spanned(1, Some(40), 41, Some(70)),
            spanned(2, Some(70), 71, Some(95)),
            spanned(3, None, 95, Some(96)),
        ];
        // From the live screen, back to the block at 95, then 70, 40 and 10.
        assert_eq!(block_scroll(&blocks, 100, 0, 90, false), Some(5));
        assert_eq!(block_scroll(&blocks, 100, 5, 90, false), Some(30));
        assert_eq!(block_scroll(&blocks, 100, 60, 90, false), Some(90));
        // And forward again; the last goes back to the live screen.
        assert_eq!(block_scroll(&blocks, 100, 90, 90, true), Some(60));
        assert_eq!(block_scroll(&blocks, 100, 30, 90, true), Some(5));
        assert_eq!(block_scroll(&blocks, 100, 5, 90, true), None);
        // A block whose start left the history is reached at the history's top.
        assert_eq!(block_scroll(&blocks, 100, 60, 50, false), Some(50));
    }

    #[test]
    fn with_no_block_that_way_the_block_keys_do_nothing() {
        let blocks = [spanned(0, Some(100), 101, Some(103))];
        // The only block starts on the live screen: nothing above, nothing below.
        assert_eq!(block_scroll(&blocks, 100, 0, 40, false), None);
        assert_eq!(block_scroll(&blocks, 100, 0, 40, true), None);
        assert_eq!(block_scroll(&[], 100, 0, 40, true), None);
        // A block below the viewport on the live screen is reached by going to the bottom.
        assert_eq!(block_scroll(&blocks, 100, 20, 40, true), Some(0));
    }

    #[test]
    fn a_command_becomes_a_finished_block_over_its_output_lines() {
        let mut blocks = AnchoredBlocks::default();
        for (hook, line) in [
            (init(), 0),
            (precmd(0, "/a"), 0),
            (preexec("echo hi"), 1),
            (precmd(0, "/a"), 2),
        ] {
            assert_eq!(blocks.apply(hook, line), Ok(()));
        }
        assert_eq!(
            blocks.blocks(),
            [AnchoredBlock {
                index: 0,
                command: "echo hi".to_string(),
                command_verified: false,
                state: BlockState::Finished,
                exit_code: ExitCode(Some(0)),
                prompt: pwd("/a"),
                prompt_line: Some(0),
                output_start: 1,
                output_end: Some(2),
            }]
        );
        // The next block takes the prompt the last `Precmd` staged.
        assert_eq!(blocks.apply(preexec("false"), 3), Ok(()));
        let next = &blocks.blocks()[1];
        assert_eq!((next.index, next.prompt_line), (1, Some(2)));
        assert_eq!((next.state, next.output_end), (BlockState::Running, None));
    }

    #[test]
    fn the_shift_puts_the_content_on_the_bottom_edge_until_the_view_scrolls_past_it() {
        assert_eq!(bottom_shift(20, 0, false), 20);
        // Scrolling back takes a row off the shift for each row, and never goes below none.
        assert_eq!(bottom_shift(20, 3, false), 17);
        assert_eq!(bottom_shift(20, 20, false), 0);
        assert_eq!(bottom_shift(20, 25, false), 0);
        // A full screen, and the alternate screen, are drawn as they are.
        assert_eq!(bottom_shift(0, 0, false), 0);
        assert_eq!(bottom_shift(20, 0, true), 0);
    }

    #[test]
    fn a_blocks_command_is_verified_by_the_terminals_nonce_alone() {
        let preexec_with = |command: &str, nonce: Option<&str>| {
            DcsHook::Preexec(PreexecValue {
                command: command.to_string(),
                nonce: nonce.map(String::from),
            })
        };
        let verified = |blocks: &AnchoredBlocks| -> Vec<bool> {
            blocks
                .blocks()
                .iter()
                .map(|block| block.command_verified)
                .collect()
        };
        let mut blocks = AnchoredBlocks::with_nonce("00ff".to_string());
        for (hook, line) in [
            (init(), 0),
            (preexec_with("ls", Some("00ff")), 1),
            (preexec_with("forged", Some("0fff")), 2),
            (preexec_with("forged", None), 3),
        ] {
            assert_eq!(blocks.apply(hook, line), Ok(()));
        }
        assert_eq!(verified(&blocks), [true, false, false]);
        // A terminal that gave no nonce verifies nothing, a frame without one included.
        let mut blocks = AnchoredBlocks::default();
        for (hook, line) in [
            (init(), 0),
            (preexec_with("ls", None), 1),
            (preexec_with("ls", Some("00ff")), 2),
        ] {
            assert_eq!(blocks.apply(hook, line), Ok(()));
        }
        assert_eq!(verified(&blocks), [false, false]);
    }

    #[test]
    fn hooks_before_the_shell_registers_are_refused() {
        let mut blocks = AnchoredBlocks::default();
        assert_eq!(
            blocks.apply(preexec("ls"), 0),
            Err(ApplyHookError::MissingSession)
        );
        assert_eq!(
            blocks.apply(precmd(0, "/"), 0),
            Err(ApplyHookError::MissingSession)
        );
        assert!(blocks.blocks().is_empty());
    }

    #[test]
    fn a_command_with_no_precmd_after_it_ends_where_the_next_one_starts() {
        let mut blocks = AnchoredBlocks::default();
        for (hook, line) in [(init(), 0), (preexec("vim"), 1), (preexec("ls"), 9)] {
            assert_eq!(blocks.apply(hook, line), Ok(()));
        }
        let first = &blocks.blocks()[0];
        assert_eq!(first.state, BlockState::Finished);
        assert_eq!(first.exit_code, ExitCode(None));
        assert_eq!(first.output_end, Some(9));
        // With no `Precmd` staged, a block has no prompt line and an empty prompt.
        assert_eq!(
            (first.prompt_line, &first.prompt),
            (None, &PromptInfo::default())
        );
    }

    #[test]
    fn a_new_shell_forgets_the_staged_prompt_and_bootstrapped_changes_nothing() {
        let mut blocks = AnchoredBlocks::default();
        for (hook, line) in [
            (init(), 0),
            (precmd(0, "/old"), 0),
            (init(), 1),
            (DcsHook::Bootstrapped { is_subshell: true }, 1),
            (preexec("pwd"), 2),
        ] {
            assert_eq!(blocks.apply(hook, line), Ok(()));
        }
        let block = &blocks.blocks()[0];
        assert_eq!(
            (block.prompt_line, &block.prompt),
            (None, &PromptInfo::default())
        );
    }

    #[test]
    fn a_precmd_with_nothing_running_only_stages_the_prompt() {
        let mut blocks = AnchoredBlocks::default();
        assert_eq!(blocks.apply(init(), 0), Ok(()));
        assert_eq!(blocks.apply(precmd(1, "/"), 4), Ok(()));
        assert!(blocks.blocks().is_empty());
        assert_eq!(blocks.apply(preexec("ls"), 5), Ok(()));
        assert_eq!(blocks.blocks()[0].prompt_line, Some(4));
    }
}
