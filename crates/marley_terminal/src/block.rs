//! PURE — the per-command [`Block`] value types and the ordered [`BlockList`].
//!
//! A `Block` holds its rendered output as an owned `Vec<String>` (one entry per screen line),
//! NOT a live grid, so [`Block::output_text`] is a pure join with no PTY read (R19). `BlockList`
//! owns the sole id/index allocator: its `open_running` is the *only* place a `BlockId`
//! or `BlockIndex` is minted (R1/R2).

use crate::SessionId;

use crate::styled::StyledLine;

/// A `Block` identity, unique within its [`TerminalSession`](crate::TerminalSession) (R1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(u64);

/// A `Block`'s position, strictly increasing in command-execution order (R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockIndex(usize);

/// The id a shell **self-reports** in its `InitShell` bootstrap hook.
///
/// Correlation only — there is no `next()`/allocation API. `apply_hook` maps it to the session's
/// [`crate::SessionId`]; it is never the value stamped onto a [`Block`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShellSessionId(pub(crate) u64);

/// The lifecycle state of a [`Block`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockState {
    /// Opened but not yet executing.
    Pending,
    /// Executing — output is being appended.
    Running,
    /// Finished — the command completed and its exit code is set.
    Finished,
}

/// The process exit code of a finished command — `None` when no numeric code is available
/// (for example, terminated by a signal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExitCode(pub Option<i32>);

/// The prompt metadata a shell reports for a block, staged from one `Precmd` and consumed by the
/// next `Preexec` (R3/R6). Every field is optional; absent fields stay `None`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PromptInfo {
    /// The working directory the command ran in.
    pub pwd: Option<String>,
    /// The active git branch, if any.
    pub git_branch: Option<String>,
    /// The active Python virtualenv, if any.
    pub virtual_env: Option<String>,
    /// The active Node version, if any.
    pub node_version: Option<String>,
}

/// One command's record: identity, the command text, lifecycle [`BlockState`], exit code, prompt
/// metadata, and the rendered output cells (held as owned lines, never a live grid).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The session-unique identity (R1).
    pub id: BlockId,
    /// The execution-order position (R2).
    pub index: BlockIndex,
    /// The session's [`crate::SessionId`] — never the shell-reported id (R7); `None` until
    /// an `InitShell` hook has registered the session.
    pub session_id: Option<SessionId>,
    /// The command text from the opening `Preexec` hook.
    pub command: String,
    /// The lifecycle state.
    pub state: BlockState,
    /// The exit code, set when a `Precmd` finishes the block (R5).
    pub exit_code: ExitCode,
    /// The prompt metadata initialized from the staged-prompt buffer (R3).
    pub prompt: PromptInfo,
    /// #435: the run-block identity — `Some(tag)` IFF this block was born from
    /// a runnable spawn (the #434 gutter ▶ or an identity-aware rerun), staged
    /// via [`crate::TerminalSession::stage_run_tag`] and consumed by the birth
    /// `Preexec` (the staged-prompt correlation, R3's twin). OPAQUE here — the
    /// app owns the encoding (this crate stays toolchain-free); session-scoped
    /// by construction (the `BlockList` is never serialized).
    pub run_tag: Option<String>,
    /// Rendered output, one styled line per screen line. Private: mutated only through the model.
    output: Vec<StyledLine>,
}

/// What a block-copy action copies (R29) — the selector for [`Block::copy_text`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockCopy {
    /// The command line.
    Command,
    /// The command's output.
    Output,
}

impl Block {
    /// The text a block-copy action copies (R29): the command line, or the plain output
    /// (`output_text`, so a block-action copy agrees with a drag-copy of the same output).
    pub fn copy_text(&self, what: BlockCopy) -> String {
        match what {
            BlockCopy::Command => self.command.clone(),
            BlockCopy::Output => self.output_text(),
        }
    }

    /// The command to re-run this block (R30): `Some(command)` IFF the block has FINISHED and its
    /// command is non-empty; `None` for a still-running/pending block or an empty command (not
    /// re-runnable). The app resends it (`write_command`) only when the session is idle (#40).
    pub fn rerun_command(&self) -> Option<String> {
        if self.state == BlockState::Finished && !self.command.is_empty() {
            Some(self.command.clone())
        } else {
            None
        }
    }

    /// The block's rendered output as a single plain string (R19): each line's run texts
    /// concatenated, the lines joined by `\n`, trailing blank lines trimmed. Byte-identical to the
    /// pre-color model — the styled runs are flattened here. No PTY read.
    pub fn output_text(&self) -> String {
        self.output
            .iter()
            .map(|line| line.iter().map(|run| run.text.as_str()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end()
            .to_string()
    }

    /// The block's rendered output as styled lines (R20b) — the per-run fg/bg/flags the render
    /// paints. `output_text` is the plain projection of the same data.
    pub fn output_styled(&self) -> &[StyledLine] {
        &self.output
    }

    /// Replace the block's rendered output lines (model-internal; used by the render passthrough).
    pub(crate) fn set_output(&mut self, output: Vec<StyledLine>) {
        self.output = output;
    }
}

/// An ordered collection of [`Block`]s and the sole [`BlockId`]/[`BlockIndex`] allocator.
#[derive(Debug, Default)]
pub struct BlockList {
    blocks: Vec<Block>,
    next_id: u64,
    next_index: usize,
}

impl BlockList {
    /// The number of blocks.
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Whether the list holds no blocks.
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// The block at `index`, or `None` if no block has that index.
    pub fn get(&self, index: BlockIndex) -> Option<&Block> {
        self.blocks.get(index.0)
    }

    /// The most recent re-runnable command (R30) — the LAST block, in execution order, whose
    /// [`Block::rerun_command`] is `Some` (Finished + non-empty). Drives cmd-R; `None` when no
    /// finished command exists.
    pub fn last_rerunnable(&self) -> Option<String> {
        self.blocks.iter().rev().find_map(|b| b.rerun_command())
    }

    /// The current block — the most recently opened block IFF it is still `Running` (R2); `None`
    /// once it has finished.
    pub fn current(&self) -> Option<&Block> {
        self.blocks
            .last()
            .filter(|b| b.state == BlockState::Running)
    }

    /// Iterate the blocks in execution order.
    pub fn iter(&self) -> impl Iterator<Item = &Block> {
        self.blocks.iter()
    }

    /// Open a new `Running` block with `command`/`session_id`/`prompt`, minting the next
    /// [`BlockId`] and [`BlockIndex`]. This is the SOLE id/index allocation point (R1/R2).
    pub(crate) fn open_running(
        &mut self,
        command: String,
        session_id: Option<SessionId>,
        prompt: PromptInfo,
    ) {
        let id = BlockId(self.next_id);
        self.next_id += 1;
        let index = BlockIndex(self.next_index);
        self.next_index += 1;
        self.blocks.push(Block {
            id,
            index,
            session_id,
            command,
            state: BlockState::Running,
            exit_code: ExitCode(None),
            prompt,
            run_tag: None,
            output: Vec::new(),
        });
    }

    /// A mutable reference to the current block — the most recently opened block IFF it is still
    /// `Running`; `None` otherwise.
    pub(crate) fn current_mut(&mut self) -> Option<&mut Block> {
        self.blocks
            .last_mut()
            .filter(|b| b.state == BlockState::Running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A three-block list `[a, b, c]`, all `Running`, opened in order.
    fn three() -> BlockList {
        let mut bl = BlockList::default();
        let sid = Some(SessionId::from(1));
        bl.open_running("a".into(), sid, PromptInfo::default());
        bl.open_running("b".into(), sid, PromptInfo::default());
        bl.open_running("c".into(), sid, PromptInfo::default());
        bl
    }

    #[test]
    fn empty_list_len_and_is_empty() {
        let bl = BlockList::default();
        assert_eq!(bl.len(), 0); // kills len -> 1
        assert!(bl.is_empty()); // kills is_empty -> false
    }

    #[test]
    fn open_running_allocates_exact_ids_and_indices() {
        // The allocator mints ids [0,1,2] and indices [0,1,2] — EXACT, in order. `+=`→`*=` would
        // freeze the counter at 0 (all ids 0); `+=`→`-=` underflows on the second alloc (panic).
        let bl = three();
        assert_eq!(bl.len(), 3); // kills len -> 0
        assert!(!bl.is_empty()); // kills is_empty -> true
        let ids: Vec<BlockId> = bl.iter().map(|b| b.id).collect();
        let indices: Vec<BlockIndex> = bl.iter().map(|b| b.index).collect();
        assert_eq!(ids, vec![BlockId(0), BlockId(1), BlockId(2)]);
        assert_eq!(indices, vec![BlockIndex(0), BlockIndex(1), BlockIndex(2)]);
        // iter() yields all three (kills `empty()` and `once(..)` iter mutants).
        assert_eq!(bl.iter().count(), 3);
    }

    #[test]
    fn open_running_stamps_fields() {
        // open_running -> () would push nothing; assert the pushed block's fields.
        let mut bl = BlockList::default();
        bl.open_running(
            "echo".into(),
            Some(SessionId::from(9)),
            PromptInfo {
                pwd: Some("/w".into()),
                ..Default::default()
            },
        );
        let b = bl.get(BlockIndex(0)).expect("block 0 exists");
        assert_eq!(b.command, "echo");
        assert_eq!(b.state, BlockState::Running);
        assert_eq!(b.session_id, Some(SessionId::from(9)));
        assert_eq!(b.exit_code, ExitCode(None));
        assert_eq!(b.prompt.pwd, Some("/w".to_string()));
    }

    #[test]
    fn get_returns_the_indexed_block_or_none() {
        let bl = three();
        // get(1) is the SECOND block (kills get -> None and the leak-default mutant).
        let second = bl.get(BlockIndex(1)).expect("index 1 exists");
        assert_eq!(second.command, "b");
        assert_eq!(second.id, BlockId(1));
        assert_eq!(second.index, BlockIndex(1));
        // out of range → None.
        assert!(bl.get(BlockIndex(99)).is_none());
    }

    #[test]
    fn current_is_last_iff_running() {
        // Running last → current() is Some(the last); finishing it → None. This kills BOTH
        // `current -> None` / `-> Some(leak default)` and the `state == Running`→`!=` mutant.
        let mut bl = three();
        assert_eq!(bl.current().expect("running").command, "c");
        assert!(bl.current_mut().is_some());
        bl.current_mut().expect("running").state = BlockState::Finished;
        assert!(bl.current().is_none());
        assert!(bl.current_mut().is_none());
    }

    #[test]
    fn output_text_joins_and_trims() {
        // set_output then output_text: lines joined by \n, trailing blank lines trimmed (R19).
        // Kills set_output -> () and output_text -> String::new()/"xyzzy".
        let mut bl = BlockList::default();
        bl.open_running("x".into(), Some(SessionId::from(1)), PromptInfo::default());
        let b = bl.current_mut().expect("running");
        b.set_output(crate::styled::plain_lines(&["line1", "line2", ""]));
        assert_eq!(bl.get(BlockIndex(0)).unwrap().output_text(), "line1\nline2");
    }

    // ── R30 — rerun_command: Some(command) iff Finished + non-empty; else None ──
    #[test]
    fn rerun_command_finished_nonempty() {
        let mut bl = BlockList::default();
        bl.open_running(
            "ls -la".into(),
            Some(SessionId::from(1)),
            PromptInfo::default(),
        );
        bl.current_mut().expect("running").state = BlockState::Finished;
        assert_eq!(
            bl.get(BlockIndex(0)).unwrap().rerun_command(),
            Some("ls -la".to_string())
        );
    }

    #[test]
    fn rerun_command_running_is_none() {
        // A still-RUNNING block (with a NON-EMPTY command — pins the `&&`, not just the empty guard)
        // is not re-runnable.
        let mut bl = BlockList::default();
        bl.open_running(
            "ls -la".into(),
            Some(SessionId::from(1)),
            PromptInfo::default(),
        );
        assert_eq!(bl.get(BlockIndex(0)).unwrap().rerun_command(), None);
    }

    #[test]
    fn rerun_command_empty_is_none() {
        // A Finished block with an EMPTY command is not re-runnable.
        let mut bl = BlockList::default();
        bl.open_running(
            String::new(),
            Some(SessionId::from(1)),
            PromptInfo::default(),
        );
        bl.current_mut().expect("running").state = BlockState::Finished;
        assert_eq!(bl.get(BlockIndex(0)).unwrap().rerun_command(), None);
    }

    // ── R30 — last_rerunnable: the most recent finished command; skips running; None if none ──
    #[test]
    fn last_rerunnable_finds_most_recent() {
        let mut bl = BlockList::default();
        let sid = Some(SessionId::from(1));
        bl.open_running("first".into(), sid, PromptInfo::default());
        bl.current_mut().unwrap().state = BlockState::Finished;
        bl.open_running("second".into(), sid, PromptInfo::default());
        bl.current_mut().unwrap().state = BlockState::Finished;
        // The LAST finished command, not the first.
        assert_eq!(bl.last_rerunnable(), Some("second".to_string()));
    }

    #[test]
    fn last_rerunnable_skips_running() {
        let mut bl = BlockList::default();
        let sid = Some(SessionId::from(1));
        bl.open_running("done".into(), sid, PromptInfo::default());
        bl.current_mut().unwrap().state = BlockState::Finished;
        bl.open_running("running".into(), sid, PromptInfo::default()); // still Running
        // Skips the running block → the previous finished one.
        assert_eq!(bl.last_rerunnable(), Some("done".to_string()));
    }

    #[test]
    fn last_rerunnable_none_when_no_finished() {
        let mut bl = BlockList::default();
        bl.open_running(
            "running".into(),
            Some(SessionId::from(1)),
            PromptInfo::default(),
        );
        assert_eq!(bl.last_rerunnable(), None);
    }

    // ── R29 — copy_text: the Command arm returns the command; the Output arm returns output_text ──
    #[test]
    fn copy_text_command() {
        let mut bl = BlockList::default();
        bl.open_running(
            "ls -la".into(),
            Some(SessionId::from(1)),
            PromptInfo::default(),
        );
        assert_eq!(
            bl.get(BlockIndex(0)).unwrap().copy_text(BlockCopy::Command),
            "ls -la"
        );
    }

    #[test]
    fn copy_text_output() {
        let mut bl = BlockList::default();
        bl.open_running(
            "cmd".into(),
            Some(SessionId::from(1)),
            PromptInfo::default(),
        );
        let b = bl.current_mut().expect("running");
        b.set_output(crate::styled::plain_lines(&["file1", "file2"]));
        let block = bl.get(BlockIndex(0)).unwrap();
        // The Output arm is EXACTLY output_text (so a block-action copy agrees with a drag-copy).
        assert_eq!(block.copy_text(BlockCopy::Output), "file1\nfile2");
        assert_eq!(block.copy_text(BlockCopy::Output), block.output_text());
    }

    // The load-bearing SWAP guard (inspect F1): MSI only emits whole-fn mutants for copy_text, so a
    // Command↔Output arm swap is invisible to the gate — this asserts BOTH arms on a block where the
    // command DIFFERS from the output, so a swap changes both results and fails.
    #[test]
    fn copy_text_arms_distinct() {
        let mut bl = BlockList::default();
        bl.open_running(
            "echo hi".into(),
            Some(SessionId::from(1)),
            PromptInfo::default(),
        );
        let b = bl.current_mut().expect("running");
        b.set_output(crate::styled::plain_lines(&["hi"]));
        let block = bl.get(BlockIndex(0)).unwrap();
        assert_eq!(block.copy_text(BlockCopy::Command), "echo hi");
        assert_eq!(block.copy_text(BlockCopy::Output), "hi");
        assert_ne!(
            block.copy_text(BlockCopy::Command),
            block.copy_text(BlockCopy::Output)
        );
    }

    #[test]
    fn output_styled_returns_the_runs() {
        // output_styled exposes the per-line runs the render paints (R20b) — distinct from the
        // flattened output_text. Kills `output_styled -> &[]`.
        let mut bl = BlockList::default();
        bl.open_running("x".into(), Some(SessionId::from(1)), PromptInfo::default());
        let b = bl.current_mut().expect("running");
        b.set_output(crate::styled::plain_lines(&["hello", "world"]));
        let styled = bl.get(BlockIndex(0)).unwrap().output_styled();
        assert_eq!(styled.len(), 2);
        assert_eq!(styled[0][0].text, "hello");
        assert_eq!(styled[1][0].text, "world");
    }
}
