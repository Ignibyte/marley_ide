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
//!   whose output starts at its line, with the staged prompt;
//! - `Bootstrapped` changes nothing here.
//!
//! A `Preexec` or `Precmd` before any `InitShell` is refused, as the gpui era's `SessionModel`
//! refuses it.

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

/// A terminal's blocks, in order, and the prompt staged for the next one.
#[derive(Debug, Default)]
pub struct AnchoredBlocks {
    blocks: Vec<AnchoredBlock>,
    registered: bool,
    staged: Option<(PromptInfo, u64)>,
}

impl AnchoredBlocks {
    /// Applies `hook`, which arrived at the absolute line `line`.
    ///
    /// # Errors
    ///
    /// [`ApplyHookError::MissingSession`] for a `Preexec` or `Precmd` before any `InitShell`;
    /// the blocks stay as they were.
    pub fn apply(&mut self, hook: DcsHook, line: u64) -> Result<(), ApplyHookError> {
        match hook {
            DcsHook::InitShell { .. } => {
                self.registered = true;
                self.staged = None;
            }
            DcsHook::Preexec(value) => {
                self.require_shell()?;
                self.finish_running(line, ExitCode(None));
                let (prompt, prompt_line) = self.staged.take().map_or_else(
                    || (PromptInfo::default(), None),
                    |(prompt, at)| (prompt, Some(at)),
                );
                self.blocks.push(AnchoredBlock {
                    index: self.blocks.len(),
                    command: value.command,
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
                self.finish_running(line, value.exit_code);
                self.staged = Some((value.prompt, line));
            }
            DcsHook::Bootstrapped { .. } => {}
        }
        Ok(())
    }

    /// The blocks, oldest first.
    #[must_use]
    pub fn blocks(&self) -> &[AnchoredBlock] {
        &self.blocks
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
