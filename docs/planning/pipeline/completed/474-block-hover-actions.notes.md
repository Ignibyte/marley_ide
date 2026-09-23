# Copy and rerun on a hovered block — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-474-block-hover-actions.md
- **Pipeline spec:** 474-block-hover-actions.spec.md

## Phase 1 — Plan (2026-09-23)
- **Request:** the plan's T1 hover copy and rerun, the last stage-one piece after #470 and #473;
  Chad's goal "lets continue on those".
- **Recall.** #470's pill as an `AnyElement` from `prepaint_as_root`;
  L-claude-470-a-pty-test-child-that-exits-can-lose-its-last-bytes-001 (the test scripts
  sleep); L-claude-470-a-script-edit-skips-the-ledger-hook-001 (rows before scripted edits).
- **Design.** `marley_pills` becomes `marley_blocks`: for each span starting in view, a `div`
  over its rows (`group("marley-block-{index}")`), its first row an `h_flex` justified to the end
  with the buttons (`visible_on_hover`, `occlude`) and the pill. Copy and Rerun capture the
  terminal and the block. `rerun` is offered when the last block is finished.
- **Files.** Zed: `crates/terminal_view/src/terminal_element.rs`, `terminal_view.rs` (tests);
  ledger rows first.

## Phase 2 — Code (2026-09-23)
- **Built.**
  - `terminal_element.rs` (Zed): `LayoutState::marley_pills` became `marley_blocks`. `prepaint`
    builds a `marley_block` for each span starting in view and lays it out at the block's first
    row with the block's height (`prepaint_as_root`); `paint` paints them after the text, where
    the pills were. `marley_block` is a `div` with the group `marley-block-{index}` over the
    block's rows; its first row, justified to the end, holds the actions (`visible_on_hover`)
    and the pill. Copy reads `Terminal::block_output` when clicked. Rerun sends
    `\x15{command}\r` through `Terminal::input`, and is drawn only while the last block is
    finished and the command is verified and not empty. Each button sits in
    `marley_keep_from_terminal`.
  - The nonce (D4): `marley_terminal::shell_integration::{NONCE_VARIABLE, new_nonce}` (128 bits
    from `rand`, the workspace's 0.9, now a dependency of `marley_terminal`);
    `PreexecValue::nonce`, decoded from the frame's `nonce=` field; `AnchoredBlocks::with_nonce`
    and `AnchoredBlock::command_verified`. In Zed's `TerminalBuilder::new`, a local terminal's
    program gets `MARLEY_SHELL_NONCE` and its blocks the same nonce. Both scripts copy it to
    `__MARLEY_NONCE`, unset it before the user's files run, and add it to each `preexec` frame.
- **Deviations from the plan.**
  - D3: the plan had the actions `occlude`. The first driven click copied nothing: an occluding
    hitbox stops the hit test, so the block's hitbox behind it is no longer hovered, the group's
    hover ends, and the button hides as the pointer reaches it
    (F-claude-474-the-occluding-buttons-hid-under-the-pointer-001). A wrapper stops the left
    press instead; the terminal's listeners were registered before the block painted, so they
    run after the wrapper's.
  - D4, added: the review's security lens found that Rerun sent the recorded command of any
    finished block, and any output can print frames
    (F-claude-474-rerun-would-have-run-a-command-that-output-printed-001). Rerun is this
    ticket's, so its fix is too: the spec gained D4 and REQ-005 and REQ-006, and the gate became
    REQ-007.
- **Review of the diff**, against each criterion:
  - correctness: Copy clones the block in prepaint and reads the terminal when clicked, so a
    running block copies what it holds by then. The Rerun condition is the last frame's: a
    command that started between that frame and a click on it would take the rerun as typed
    input, a window of one frame.
  - re-entrancy: the click handlers update the terminal entity, not the element being painted.
  - security: the spoofing gap above, fixed. The nonce replaces any `MARLEY_SHELL_NONCE` in the
    terminal's environment settings, and remote terminals get none. `Terminal` has no `Debug`,
    and no log line prints its blocks.
  - provenance: no Warp source; the nonce follows VS Code's published shell-integration
    protocol, with no code taken from it.
  - upstream discipline: each Zed hunk is additive with its `// Marley:` comment; the rows for
    `terminal.rs`, `terminal_element.rs` and `terminal_view.rs` were written before the edits.
  - Left as it is: a middle or right press on a button reaches the terminal, as it would
    anywhere in it; the wrapper stops the left press, the one a button acts on.

## Phase 3 — Test (2026-09-23)
- **Tests.**
  - `terminal_view`, over a real PTY through `marley_hook_terminal`; `MARLEY_TWO_BLOCKS`'s command
    frames carry `$nonce`, which a script sets:
    - `marley_a_hovered_blocks_buttons_copy_and_rerun_it` (REQ-001 to REQ-003): mouse reporting
      on, asserted; over the passing block its Copy shows and the failed block's does not; a
      press on the failed block's output released over Copy reaches the program as a press and
      a release report and copies nothing; Copy puts `oops` on the clipboard and writes nothing
      to the PTY; Rerun writes exactly `\x15false\r`.
    - `marley_no_rerun_while_a_block_runs` (REQ-004), with verified commands, so the running
      block alone hides Rerun.
    - `marley_no_rerun_for_a_command_that_output_printed` (REQ-005): `nonce=forged`.
  - `terminal` (REQ-006): the bash and zsh tests assert that a typed command's block is verified
    and that `printenv MARLEY_SHELL_NONCE` exits 1;
    `marley_only_a_local_terminal_gives_its_program_a_nonce` runs `/bin/sh` local and remote.
  - `marley_terminal`: `decode_hook_preexec_carries_its_nonce`,
    `a_blocks_command_is_verified_by_the_terminals_nonce_alone`,
    `a_nonce_is_32_hex_digits_and_new_each_time`.
- **Results:** `marley_terminal` 166 of 166; `terminal`'s `marley_` tests 9 of 9, a real bash and
  zsh among them; `terminal_view`'s `marley_` tests 8 of 8. Clippy is clean on the three crates,
  and fmt was applied.
- **A blind check, found and fixed.** The first test of the press read
  `Terminal::take_input_log`, which records only `input()`; a mouse report goes out through
  `write_to_pty`, which `take_pty_write_log` records. Removing the wrapper passed it
  (L-claude-474-the-input-log-misses-mouse-reports-001). On the PTY write log, with a press on
  the output as the control, each check below bites.
- **Negative checks**, each file restored by sha256:
  1. the wrapper's press stop removed: FAIL, `no mouse report`, `left: [[27, 91, 77, 32, 248, 34]]`;
  2. a release stop added to the wrapper: FAIL, `left: [Some(32)]`, `right: [Some(32), Some(35)]`;
  3. `visible_on_hover` removed: FAIL at `copy-1` `is_none()`;
  4. Rerun without `command_verified`: FAIL in `marley_no_rerun_for_a_command_that_output_printed`;
  5. Rerun without the at-a-prompt condition: FAIL in `marley_no_rerun_while_a_block_runs`;
  6. bash without `unset MARLEY_SHELL_NONCE`: FAIL, `left: ExitCode(Some(0))`; bash's frame
     without the nonce: FAIL, `assertion failed: block.command_verified`;
  7. zsh, the same two: each FAIL;
  8. the builder giving remote terminals the nonce: FAIL, `got: NONCE_LENGTH=32.`;
  9. `command_verified` without `self.nonce.is_some()`: FAIL, `left: [true, false]`; with only
     it: FAIL, `left: [true, true, true]`;
  10. the decoder dropping the nonce: FAIL, `nonce: None`.
  Before the test's rewrite, Rerun offered while a block ran and Rerun without Ctrl-U each
  failed as well.
- **Found in Test:** the release stop
  (F-claude-474-a-press-elsewhere-lost-its-release-over-a-button-001), fixed. Pre-existing, not
  in scope: the PTY tests of #463 and #465 install the scripts in the user's real data
  directory, `~/.local/share/marley/shell_integration` (TICKET-475). A mutated script sat there
  only until the next run; the last run put the tree's back (`cmp` matched both).
- **Live drive** (`just shot` with #470's seed, on a hidden workspace, no input sent), on the
  binary built at 16:36 with the whole change: three blocks, a check on `echo hello`, `exit 2`
  and a red wash on `ls /nope`, `running` and a blue wash on `sleep 60`; no button beside any
  pill with no pointer over the window, and the pills where #470 drew them. The 16:10 build,
  before the nonce, showed the same. While it ran, `/proc/<pid>/environ` (presence only, never
  the value): both of Marley's bash shells started with `MARLEY_SHELL_NONCE`, and the `sleep`
  each one's `.bashrc` ran had none. Hovering needs a pointer over the window, which would be
  input into Chad's session: not driven, and the driven tests cover it.
- **Gate:** `just gate-diff`, scope `terminal`, `marley_terminal` and `terminal_view`: 20 passed,
  0 failed, `GATE GREEN [diff]`, coverage 100% of lines on `marley_terminal`.

## Phase 4 — Complete (2026-09-23)
- **Docs:** `CHANGELOG.md` (#474); `docs/marley_architecture/terminal_blocks.md` (the block
  element, the wrapper, the nonce); the three-prong plan (T1b shipped, and D5 on the nonce);
  the ledger rows for `terminal.rs`, `terminal_element.rs` and `terminal_view.rs`, checked
  against what ships.
- **Knowledge:** AD-claude-474-hover-actions-live-on-one-element-per-block-001,
  AD-claude-474-a-blocks-command-is-trusted-only-with-the-terminals-nonce-001,
  L-claude-474-an-occluding-child-ends-its-groups-hover-001,
  L-claude-474-the-input-log-misses-mouse-reports-001,
  F-claude-474-the-occluding-buttons-hid-under-the-pointer-001,
  F-claude-474-a-press-elsewhere-lost-its-release-over-a-button-001,
  F-claude-474-rerun-would-have-run-a-command-that-output-printed-001,
  PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001.
- **Brain:** consultation 9557336ae60f4325bc715adaac98239d, asked at Complete, since Plan
  skipped it; nothing came back on this seam. Decided:
  `decisions/block-hover-actions-with-rerun-only-for-a-command-the-shells-own-hook-reported`,
  follow-up by 2026-10-07.
- **Tickets:** #474 closed. #475 minted at the top of the Queue (the shell tests' data
  directory). #466 notes that fish's frames carry the nonce too.
