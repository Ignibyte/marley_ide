//! PURE (the rigor center) — the stateful session model and the `apply_hook` transition machine.
//!
//! [`SessionModel`] owns the staged-prompt buffer, the [`ShellSessionId`] → `SessionId` registry,
//! and the [`BlockList`]. `apply_hook` mutates the in-memory model only — no IO — so every
//! transition (R3–R7, R11, R18, R21, R22) is unit-testable without a PTY.

use std::collections::HashMap;

use crate::SessionId;

use crate::block::{BlockList, BlockState, PromptInfo, ShellSessionId};
use crate::dcs::DcsHook;
use crate::styled::StyledLine;

/// Why `apply_hook` refused a hook (R11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyHookError {
    /// A `Preexec`/`Precmd` arrived before any `InitShell` registered the session; the
    /// [`BlockList`] is left unchanged.
    MissingSession,
}

/// The in-memory session state the hook machine drives.
#[derive(Debug)]
pub struct SessionModel {
    session_id: SessionId,
    registry: HashMap<ShellSessionId, SessionId>,
    staged_prompt: Option<PromptInfo>,
    /// #435: the staged run-block identity as a `(tag, command)` pair —
    /// `stage_run_tag` sets it in the same sync region as the spawn's
    /// `write_command`, and a `Preexec` whose command MATCHES takes the tag
    /// onto the block it births (the `staged_prompt` correlation shape,
    /// hardened: an already-buffered earlier command's Preexec mismatches and
    /// leaves the stage for OUR command's Preexec — a tag can only ever bind
    /// to the exact command it was minted for, so a mislabel is impossible by
    /// construction). `InitShell` discards it with the staged prompt
    /// (R7/R22) — a spawn racing a fresh shell's init loses its tag and
    /// births a PLAIN block (fail-safe; the first successful Preexec is
    /// always preceded by the registering `InitShell`).
    staged_run_tag: Option<(String, String)>,
    blocks: BlockList,
    bootstrapped: Option<bool>,
    /// #433: the block LIFECYCLE epoch — monotone PER SESSION, bumped once
    /// per transition (a block BORN at Preexec; a block FINISHED at Precmd or
    /// child-exit). The `DiagnosticStore::publish_epoch` shape for the
    /// terminal lane: a consumer gating on "did any block open/finish since I
    /// last derived?" compares this, never a state count (a count can cancel
    /// — the #430 collision class). A workspace-level consumer SUMMING these
    /// across sessions inherits the LSP twin's accepted window: a session
    /// close can cancel against concurrent bumps elsewhere in one gate read
    /// (self-healing on the next lifecycle event; recorded at #433 inspect).
    /// Every mutation path funnels through THIS model (`pump`'s DCS scan and
    /// `TerminalSession::apply_hook` both call [`Self::apply_hook`]; child
    /// exit calls [`Self::finish_current_if_running`]), so the bumps here are
    /// exhaustive by construction.
    block_epoch: u64,
}

impl SessionModel {
    /// A fresh model for `session_id`, with an empty registry, no staged prompt, and no blocks.
    pub(crate) fn new(session_id: SessionId) -> Self {
        Self {
            session_id,
            registry: HashMap::new(),
            staged_prompt: None,
            staged_run_tag: None,
            blocks: BlockList::default(),
            bootstrapped: None,
            block_epoch: 0,
        }
    }

    /// #433: the monotone block-lifecycle epoch (see the field doc).
    pub(crate) const fn block_epoch(&self) -> u64 {
        self.block_epoch
    }

    /// #435: stage the run-block identity the matching `Preexec` will bind to
    /// its born block (see the field doc — `command` is the exact spawn text
    /// the tag may bind to). The app calls this in the same sync region as
    /// the spawn's `write_command`, AFTER it succeeded. REFUSES to overwrite
    /// a still-staged tag (inspect LOW): two spawns inside one pump window
    /// would otherwise cross-label; losing the second identity (its block
    /// births plain) is fail-safe, mislabeling is not.
    pub(crate) fn stage_run_tag(&mut self, tag: String, command: String) {
        if self.staged_run_tag.is_none() {
            self.staged_run_tag = Some((tag, command));
        }
    }

    /// Apply one decoded [`DcsHook`], mutating the model (R3–R7, R11, R21, R22).
    ///
    /// - `InitShell` records `shell_session_id` → this session's `SessionId` and discards the
    ///   staged prompt (R7/R22).
    /// - `Preexec` (requires a registered session, else [`ApplyHookError::MissingSession`], R11)
    ///   opens a `Running` block from the consumed staged prompt (R3).
    /// - `Precmd` (requires a registered session, R11) finishes the running block with its exit
    ///   code (R5) and stages the new prompt, replacing any still-staged one (R6/R21).
    /// - `Bootstrapped` records the subshell flag.
    pub(crate) fn apply_hook(&mut self, hook: DcsHook) -> Result<(), ApplyHookError> {
        // This model reads the hook alone, not which shell signed it (#526).
        let hook = hook.into_unsigned();
        match hook {
            DcsHook::InitShell { shell_session_id } => {
                let _previous = self.registry.insert(shell_session_id, self.session_id);
                self.staged_prompt = None;
                self.staged_run_tag = None;
                Ok(())
            }
            DcsHook::Preexec(value) => {
                if self.registry.is_empty() {
                    return Err(ApplyHookError::MissingSession);
                }
                // #435: the staged tag binds ONLY to the command it was
                // minted for (match-bind — see the field doc); a mismatching
                // Preexec leaves the stage in place for the spawn's own.
                let tag_matches = self
                    .staged_run_tag
                    .as_ref()
                    .is_some_and(|(_, cmd)| *cmd == value.command);
                let prompt = self.staged_prompt.take().unwrap_or_default();
                self.blocks
                    .open_running(value.command, Some(self.session_id), prompt);
                if tag_matches {
                    // The block born above IS the current Running block
                    // (`open_running` just pushed it), so the taken tag binds
                    // to exactly it — one take, one assignment, no dead arm.
                    let tag = self.staged_run_tag.take().map(|(tag, _)| tag);
                    if let Some(block) = self.blocks.current_mut() {
                        block.run_tag = tag;
                    }
                }
                self.block_epoch += 1; // #433: a block was BORN
                Ok(())
            }
            DcsHook::Precmd(value) => {
                if self.registry.is_empty() {
                    return Err(ApplyHookError::MissingSession);
                }
                if let Some(block) = self.blocks.current_mut() {
                    block.state = BlockState::Finished;
                    block.exit_code = value.exit_code;
                    self.block_epoch += 1; // #433: a block FINISHED
                }
                self.staged_prompt = Some(value.prompt);
                Ok(())
            }
            DcsHook::Bootstrapped { is_subshell } => {
                self.bootstrapped = Some(is_subshell);
                Ok(())
            }
            DcsHook::History { .. } | DcsHook::Remote { .. } | DcsHook::Signed { .. } => Ok(()),
        }
    }

    /// Replace the current running block's rendered output (R4); a no-op when no block is running.
    pub(crate) fn set_current_output(&mut self, output: Vec<StyledLine>) {
        if let Some(block) = self.blocks.current_mut() {
            block.set_output(output);
        }
    }

    /// Finish the current running block, if any, without touching its exit code (R18 — child exit).
    pub(crate) fn finish_current_if_running(&mut self) {
        if let Some(block) = self.blocks.current_mut() {
            block.state = BlockState::Finished;
            self.block_epoch += 1; // #433: a block FINISHED (child exit)
        }
    }

    /// The session's blocks.
    pub(crate) const fn blocks(&self) -> &BlockList {
        &self.blocks
    }

    /// #433 (cross-crate TEST SCAFFOLDING — hidden, `_for_test`): mint one
    /// COMPLETED block through the REAL hook machine (register if needed →
    /// stage `pwd` via Precmd → Preexec births → plain-styled output →
    /// Precmd finishes with `exit`), so app-level drives get deterministic
    /// failed blocks without a PTY. Rides [`Self::apply_hook`] verbatim —
    /// the epoch bumps exactly as live traffic would (+2 per seeded block).
    ///
    /// # Errors
    ///
    /// Whatever [`Self::apply_hook`] refuses. The session is registered before the first hook, so
    /// its `MissingSession` refusal does not arise here.
    #[doc(hidden)]
    pub fn seed_finished_block_for_test(
        &mut self,
        command: &str,
        pwd: Option<&str>,
        exit: Option<i32>,
        output: &str,
    ) -> Result<(), ApplyHookError> {
        use crate::block::ExitCode;
        use crate::dcs::{PrecmdValue, PreexecValue};
        if self.registry.is_empty() {
            // Register DIRECTLY, not via the InitShell arm — the live arm
            // wipes staged state (R7/R22), which would silently eat a tag a
            // drive staged for the FIRST seeded block of a fresh session
            // (inspect MED). Epoch-neutral either way (InitShell never bumps).
            let _previous = self
                .registry
                .insert(ShellSessionId(u64::MAX), self.session_id);
        }
        // Stage the pwd the next block will be born with (finishes nothing —
        // no running block yet on this path — so no spurious bump).
        self.apply_hook(DcsHook::Precmd(PrecmdValue {
            exit_code: ExitCode(None),
            prompt: PromptInfo {
                pwd: pwd.map(str::to_string),
                ..PromptInfo::default()
            },
        }))?;
        self.apply_hook(DcsHook::Preexec(PreexecValue {
            command: command.to_string(),
            nonce: None,
        }))?;
        self.set_current_output(
            output
                .lines()
                .map(|l| {
                    vec![crate::styled::StyledRun {
                        text: l.to_string(),
                        fg: alacritty_terminal::vte::ansi::Color::Named(
                            alacritty_terminal::vte::ansi::NamedColor::Foreground,
                        ),
                        bg: alacritty_terminal::vte::ansi::Color::Named(
                            alacritty_terminal::vte::ansi::NamedColor::Background,
                        ),
                        flags: alacritty_terminal::term::cell::Flags::empty(),
                        hyperlink: None,
                    }]
                })
                .collect(),
        );
        self.apply_hook(DcsHook::Precmd(PrecmdValue {
            exit_code: ExitCode(exit),
            prompt: PromptInfo::default(),
        }))
    }

    /// The prompt context a `Precmd` staged for the LIVE prompt (before the next command's `Preexec`
    /// consumes it) — `None` until the first precmd, and again after a command starts. Used to render
    /// the cwd/git segments on the input row (R43).
    pub(crate) const fn current_prompt(&self) -> Option<&PromptInfo> {
        self.staged_prompt.as_ref()
    }

    /// The session's [`crate::SessionId`].
    pub(crate) const fn session_id(&self) -> SessionId {
        self.session_id
    }

    /// Whether a `Bootstrapped` hook has been applied, and the subshell flag it reported.
    #[must_use]
    pub const fn bootstrapped(&self) -> Option<bool> {
        self.bootstrapped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{Block, ExitCode};
    use crate::dcs::{PrecmdValue, PreexecValue};

    fn model() -> SessionModel {
        SessionModel::new(SessionId::from(7))
    }

    /// The block at execution position `n` (the `BlockIndex` field is private to `block`, so the
    /// tests reach blocks by iteration order rather than constructing an index).
    fn block_at(m: &SessionModel, n: usize) -> &Block {
        m.blocks().iter().nth(n).expect("block at position exists")
    }

    fn init(m: &mut SessionModel, id: u64) {
        m.apply_hook(DcsHook::InitShell {
            shell_session_id: ShellSessionId(id),
        })
        .expect("InitShell never fails");
    }

    fn preexec(m: &mut SessionModel, command: &str) -> Result<(), ApplyHookError> {
        m.apply_hook(DcsHook::Preexec(PreexecValue {
            command: command.into(),
            nonce: None,
        }))
    }

    fn precmd(
        m: &mut SessionModel,
        exit: ExitCode,
        prompt: PromptInfo,
    ) -> Result<(), ApplyHookError> {
        m.apply_hook(DcsHook::Precmd(PrecmdValue {
            exit_code: exit,
            prompt,
        }))
    }

    // ── #435 — the staged run tag: match-bind, take-once, refuse-overwrite ──
    #[test]
    fn staged_run_tag_match_binds_take_once_and_refuses_overwrite() {
        let mut m = model();
        init(&mut m, 1);
        m.stage_run_tag("test:a".into(), "cargo test a".into());
        // A second stage before consumption is REFUSED (fail-safe: lost
        // identity beats wrong identity).
        m.stage_run_tag("test:b".into(), "cargo test b".into());
        // An already-buffered UNRELATED command's Preexec mismatches — it
        // births PLAIN and leaves the stage for the spawn's own Preexec.
        preexec(&mut m, "ls").expect("registered → ok");
        assert_eq!(block_at(&m, 0).run_tag, None, "mismatch binds nothing");
        preexec(&mut m, "cargo test a").expect("ok");
        assert_eq!(
            block_at(&m, 1).run_tag.as_deref(),
            Some("test:a"),
            "the matching Preexec binds the staged tag (never the refused b)"
        );
        // Take-once: the SAME command again consumes nothing.
        preexec(&mut m, "cargo test a").expect("ok");
        assert_eq!(block_at(&m, 2).run_tag, None, "the stage was consumed");
    }

    #[test]
    fn init_shell_discards_the_staged_run_tag() {
        let mut m = model();
        init(&mut m, 1);
        m.stage_run_tag("main".into(), "cargo run".into());
        // A shell (re)init wipes staged state (R7/R22) — the spawned block
        // births PLAIN (fail-safe; never a mislabel).
        init(&mut m, 2);
        preexec(&mut m, "cargo run").expect("ok");
        assert_eq!(block_at(&m, 0).run_tag, None);
    }

    // The inspect-MED pin: the seed registers DIRECTLY (not via InitShell),
    // so a drive can stage on a FRESH session and the tag lands on the first
    // seeded block; the second seed consumes nothing (take-once).
    #[test]
    fn seed_after_stage_lands_the_tag_on_the_first_seeded_block() {
        let mut m = model();
        m.stage_run_tag("test:x".into(), "cargo test x".into());
        m.seed_finished_block_for_test("cargo test x", Some("/w"), Some(101), "boom")
            .unwrap();
        assert_eq!(block_at(&m, 0).run_tag.as_deref(), Some("test:x"));
        m.seed_finished_block_for_test("cargo test x", Some("/w"), Some(0), "")
            .unwrap();
        assert_eq!(block_at(&m, 1).run_tag, None, "take-once");
        // Both seeds completed through the real machine regardless of tags.
        assert_eq!(m.blocks().len(), 2);
        assert_eq!(block_at(&m, 1).state, BlockState::Finished);
    }

    // ── R7 — InitShell registers the session; opened blocks are stamped with the session's id ──
    #[test]
    fn r7_init_registers_and_blocks_carry_session_id() {
        let mut m = model();
        init(&mut m, 1);
        preexec(&mut m, "ls").expect("registered → ok");
        assert_eq!(m.blocks().len(), 1); // kills blocks() -> Box::leak(Default BlockList)
        let b = block_at(&m, 0);
        // The block carries the SESSION id (SessionId::from(7)), never the shell-reported id.
        assert_eq!(b.session_id, Some(SessionId::from(7)));
        assert_eq!(b.command, "ls");
    }

    // ── R3 — Preexec opens a Running block from the consumed staged prompt ──
    #[test]
    fn r3_preexec_opens_running_and_consumes_staged() {
        let mut m = model();
        init(&mut m, 1);
        precmd(
            &mut m,
            ExitCode(Some(0)),
            PromptInfo {
                pwd: Some("/home".into()),
                git_branch: Some("main".into()),
                ..Default::default()
            },
        )
        .unwrap();
        preexec(&mut m, "ls").unwrap();
        let b = block_at(&m, 0);
        assert_eq!(b.command, "ls");
        assert_eq!(b.state, BlockState::Running);
        assert_eq!(b.prompt.pwd, Some("/home".to_string()));
        assert_eq!(b.prompt.git_branch, Some("main".to_string()));
        // The staged prompt was CONSUMED: a second Preexec gets the default (empty) prompt.
        preexec(&mut m, "pwd").unwrap();
        assert_eq!(block_at(&m, 1).prompt, PromptInfo::default());
    }

    // ── the unwrap_or_default None-branch — Preexec with nothing staged ──
    #[test]
    fn preexec_without_staged_uses_default_prompt() {
        let mut m = model();
        init(&mut m, 1);
        preexec(&mut m, "q").unwrap();
        assert_eq!(block_at(&m, 0).prompt, PromptInfo::default());
    }

    // ── R4 — set_current_output writes ONLY the running block ──
    #[test]
    fn r4_set_current_output_targets_running_only() {
        let mut m = model();
        init(&mut m, 1);
        preexec(&mut m, "ls").unwrap();
        m.set_current_output(crate::styled::plain_lines(&["out1", "out2"]));
        assert_eq!(block_at(&m, 0).output_text(), "out1\nout2");
        // Finish the block; set_current_output is now a no-op (no running block).
        precmd(&mut m, ExitCode(Some(0)), PromptInfo::default()).unwrap();
        m.set_current_output(crate::styled::plain_lines(&["ignored"]));
        assert_eq!(block_at(&m, 0).output_text(), "out1\nout2");
    }

    // ── R27 — current_prompt exposes the precmd-staged LIVE context (init BEFORE precmd) ──
    #[test]
    fn current_prompt_is_the_staged_context() {
        let mut m = model();
        init(&mut m, 1);
        assert!(m.current_prompt().is_none()); // nothing staged yet
        precmd(
            &mut m,
            ExitCode(Some(0)),
            PromptInfo {
                pwd: Some("/home/c".into()),
                git_branch: Some("main".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let info = m.current_prompt().expect("staged after precmd");
        assert_eq!(info.pwd.as_deref(), Some("/home/c"));
        assert_eq!(info.git_branch.as_deref(), Some("main"));
        preexec(&mut m, "ls").unwrap(); // the command starts → the staged prompt is consumed
        assert!(m.current_prompt().is_none());
    }

    // ── R5 — Precmd finishes the running block with its EXACT exit code (004 trap) ──
    #[test]
    fn r5_precmd_finishes_with_exact_exit_code() {
        let mut m = model();
        init(&mut m, 1);
        preexec(&mut m, "x").unwrap();
        precmd(&mut m, ExitCode(Some(3)), PromptInfo::default()).unwrap();
        let b = block_at(&m, 0);
        assert_eq!(b.state, BlockState::Finished);
        assert_eq!(b.exit_code, ExitCode(Some(3)));
    }

    // ── R6 — Precmd stages the next prompt's four fields ──
    #[test]
    fn r6_precmd_stages_next_prompt() {
        let mut m = model();
        init(&mut m, 1);
        precmd(
            &mut m,
            ExitCode(None),
            PromptInfo {
                pwd: Some("/p".into()),
                git_branch: Some("br".into()),
                virtual_env: Some("ve".into()),
                node_version: Some("18".into()),
            },
        )
        .unwrap();
        // The staged prompt surfaces on the next opened block.
        preexec(&mut m, "y").unwrap();
        assert_eq!(
            block_at(&m, 0).prompt,
            PromptInfo {
                pwd: Some("/p".into()),
                git_branch: Some("br".into()),
                virtual_env: Some("ve".into()),
                node_version: Some("18".into()),
            }
        );
    }

    // ── R21 — a second Precmd replaces the still-staged prompt ──
    #[test]
    fn r21_second_precmd_replaces_staged() {
        let mut m = model();
        init(&mut m, 1);
        precmd(
            &mut m,
            ExitCode(Some(0)),
            PromptInfo {
                pwd: Some("/first".into()),
                ..Default::default()
            },
        )
        .unwrap();
        precmd(
            &mut m,
            ExitCode(Some(0)),
            PromptInfo {
                pwd: Some("/second".into()),
                ..Default::default()
            },
        )
        .unwrap();
        preexec(&mut m, "z").unwrap();
        assert_eq!(block_at(&m, 0).prompt.pwd, Some("/second".to_string()));
    }

    // ── R22 — InitShell discards any still-staged prompt ──
    #[test]
    fn r22_init_discards_staged() {
        let mut m = model();
        init(&mut m, 1);
        precmd(
            &mut m,
            ExitCode(Some(0)),
            PromptInfo {
                pwd: Some("/x".into()),
                ..Default::default()
            },
        )
        .unwrap();
        init(&mut m, 2); // re-init discards the staged prompt
        preexec(&mut m, "c").unwrap();
        assert_eq!(block_at(&m, 0).prompt, PromptInfo::default());
    }

    // ── R11 — Preexec/Precmd before any InitShell → MissingSession; blocks UNCHANGED ──
    #[test]
    fn r11_hooks_without_session_are_rejected() {
        let mut m = model(); // no InitShell
        assert_eq!(preexec(&mut m, "x"), Err(ApplyHookError::MissingSession));
        assert_eq!(m.blocks().len(), 0);
        assert_eq!(
            precmd(&mut m, ExitCode(Some(0)), PromptInfo::default()),
            Err(ApplyHookError::MissingSession)
        );
        assert_eq!(m.blocks().len(), 0);
    }

    // ── R18 — finish_current_if_running finishes WITHOUT touching the exit code ──
    #[test]
    fn r18_finish_current_if_running() {
        let mut m = model();
        init(&mut m, 1);
        preexec(&mut m, "x").unwrap();
        assert_eq!(block_at(&m, 0).state, BlockState::Running);
        m.finish_current_if_running();
        let b = block_at(&m, 0);
        assert_eq!(b.state, BlockState::Finished);
        assert_eq!(b.exit_code, ExitCode(None)); // untouched
        // No-op when nothing is running (must not panic).
        m.finish_current_if_running();
        assert_eq!(block_at(&m, 0).state, BlockState::Finished);
    }

    // ── session_id() returns the constructed id (its Default mutant is unviable) ──
    #[test]
    fn session_id_returns_constructed_id() {
        let m = SessionModel::new(SessionId::from(42));
        assert_eq!(m.session_id(), SessionId::from(42));
    }

    // ── the Bootstrapped 3-state white-box (F3) ──
    #[test]
    fn bootstrapped_three_state_white_box() {
        let mut m = model();
        assert_eq!(m.bootstrapped(), None); // fresh
        m.apply_hook(DcsHook::Bootstrapped { is_subshell: true })
            .unwrap();
        assert_eq!(m.bootstrapped(), Some(true));
        m.apply_hook(DcsHook::Bootstrapped { is_subshell: false })
            .unwrap();
        assert_eq!(m.bootstrapped(), Some(false));
    }

    // ── #433 — the block-lifecycle epoch: one bump per transition, exactly ──

    #[test]
    fn epoch_bumps_once_per_born_and_once_per_finish() {
        let mut m = model();
        assert_eq!(m.block_epoch(), 0);
        init(&mut m, 1);
        assert_eq!(
            m.block_epoch(),
            0,
            "InitShell is not a lifecycle transition"
        );
        preexec(&mut m, "cargo test").expect("registered");
        assert_eq!(m.block_epoch(), 1, "a block was BORN");
        precmd(&mut m, ExitCode(Some(1)), PromptInfo::default()).expect("registered");
        assert_eq!(m.block_epoch(), 2, "the block FINISHED");
        // A bare re-prompt (no running block) stages but does NOT bump.
        precmd(&mut m, ExitCode(Some(0)), PromptInfo::default()).expect("registered");
        assert_eq!(m.block_epoch(), 2, "no running block → no transition");
        m.apply_hook(DcsHook::Bootstrapped { is_subshell: false })
            .expect("ok");
        assert_eq!(m.block_epoch(), 2, "Bootstrapped never bumps");
    }

    #[test]
    fn epoch_missing_session_preexec_does_not_bump() {
        let mut m = model();
        assert_eq!(preexec(&mut m, "x"), Err(ApplyHookError::MissingSession));
        assert_eq!(m.block_epoch(), 0, "a refused hook is not a transition");
    }

    #[test]
    fn epoch_child_exit_finish_bumps_once_and_buffered_precmd_does_not_double() {
        let mut m = model();
        init(&mut m, 1);
        preexec(&mut m, "sleep 999").expect("registered");
        assert_eq!(m.block_epoch(), 1);
        m.finish_current_if_running();
        assert_eq!(m.block_epoch(), 2, "child exit finished the block");
        m.finish_current_if_running();
        assert_eq!(m.block_epoch(), 2, "already finished → no second bump");
        // The buffered Precmd arriving AFTER the child-exit finish: current_mut
        // filters Running, so the double-finish never double-bumps.
        precmd(&mut m, ExitCode(Some(137)), PromptInfo::default()).expect("registered");
        assert_eq!(m.block_epoch(), 2, "the #433 double-finish guard");
    }

    // ── #433 — the cross-crate test seed rides the REAL machine ──

    #[test]
    fn seed_finished_block_mints_via_real_transitions() {
        let mut m = model();
        m.seed_finished_block_for_test(
            "cargo build",
            Some("/proj"),
            Some(101),
            "error: boom\nsrc/lib.rs:3:1: expected `;`",
        )
        .unwrap();
        assert_eq!(
            m.block_epoch(),
            2,
            "born + finished — live traffic's exact bumps"
        );
        let b = block_at(&m, 0);
        assert_eq!(b.command, "cargo build");
        assert_eq!(b.prompt.pwd.as_deref(), Some("/proj"));
        assert_eq!(b.exit_code, ExitCode(Some(101)));
        assert_eq!(b.state, BlockState::Finished);
        assert_eq!(b.output_text(), "error: boom\nsrc/lib.rs:3:1: expected `;`");
        // A second seed reuses the registration; epochs keep climbing.
        m.seed_finished_block_for_test("cargo build", Some("/proj"), Some(0), "")
            .unwrap();
        assert_eq!(m.block_epoch(), 4);
        assert_eq!(block_at(&m, 1).exit_code, ExitCode(Some(0)));
    }
}
