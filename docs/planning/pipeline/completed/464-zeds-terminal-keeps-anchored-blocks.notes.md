# Zed's terminal keeps anchored blocks — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-464-zeds-terminal-keeps-anchored-blocks.md
- **Pipeline spec:** 464-zeds-terminal-keeps-anchored-blocks.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (Chad: "ok lets go for it").
- **Request:** T0, continued after #462.
- **Pre-flight:** no active pipeline, README marker present.
- **Recall (§18.3).**
  - #462's AD: the event and its raw positions.
  - `marley_terminal`'s `SessionModel` transitions (R3-R7, R11): `Preexec` and `Precmd`
    need a registered shell, `Precmd` stages the next prompt, and `InitShell` forgets it.
  - Brain: to be asked with the decision at Complete.
- **Discovery.**
  - `bounds_to_string(start, end)` walks lines inclusive and strips the final newline
    (`term/mod.rs:558-570`).
  - `build_test_terminal_with_arguments` runs a program on a real PTY as a task
    (`terminal.rs:3716`).

### Design
- **`anchored.rs`:** `AnchoredBlock { index, command, state, exit_code, prompt, prompt_line,
  output_start, output_end }` and `AnchoredBlocks { blocks, registered, staged }` with
  `apply(DcsHook, u64) -> Result<(), ApplyHookError>` and `blocks()`. The block and prompt
  types come from `block.rs` (`BlockState`, `ExitCode`, `PromptInfo`), and `ApplyHookError`
  from `apply.rs`.
- **Zed:**
  - A `blocks: marley_terminal::AnchoredBlocks` field in both `Terminal` literals.
  - The `ShellHook` arm: skip the alternate screen, decode, apply, log a refusal, and notify
    on a change.
  - `blocks()`, and `block_output()`: lock, map each absolute line to a grid `Line` with
    `evicted_lines` and `history_size`, `None` below the oldest held line, and read the range
    with `bounds_to_string`. A running block reads to the cursor.
- **File manifest.**
  - Marley: `crates/marley_terminal/src/{anchored.rs, marley_terminal.rs}`.
  - Zed: `crates/terminal/{Cargo.toml, src/terminal.rs}`, the root `Cargo.toml`
    (`[workspace.dependencies]`), `Cargo.lock`.
  - Ledger: rows for `crates/terminal/Cargo.toml` (new) and `terminal.rs` and `Cargo.toml`
    (updated).

### Test plan
| REQ | Test |
|---|---|
| 001 | PTY: `/bin/sh -c` printing `init`, `preexec` in one `printf`, then `echo hi`, then `printf` of `precmd;exit=0`: a Finished block, exit 0, output "hi" |
| 002 | PTY: one `printf` carrying the frames and `hi\r\n` together: the same |
| 003 | `anchored.rs` unit tests: a command's lines, prompt and exit; refusals before `InitShell`; a `Preexec` with no `Precmd` before it; `InitShell` forgetting the staged prompt; `Bootstrapped`; indexes |
| 004 | a PTY test whose terminal keeps a small history, with output past it: `None` for the first block |
| 005 | `script/gates.sh --diff` |

Negative checks: the `ShellHook` arm ignoring hooks again (REQ-001 and REQ-002 fail); the end
anchored at the start (the output text fails); the eviction ignored in the mapping (REQ-004
fails).

### Risks
- **The task terminal's own lines.** A task terminal may print a header or a summary line. The
  block's range excludes them, since it is anchored at the hooks.
- **The history size in a test.** The terminal's scrollback setting is Zed's (10000 by
  default). REQ-004 needs a small one set in the test's settings.

## Phase 2 — Code (2026-09-23)
- **Checklist:** [x] `anchored.rs` · [x] the exports · [x] the ledger rows first · [x] Zed's
  `Terminal` and `alacritty.rs` · [x] fmt and clippy.
- **Built.**
  - `marley_terminal::anchored` (public, like the crate's other modules): `AnchoredBlock` and
    `AnchoredBlocks::{apply, blocks}`, with 5 unit tests. It re-exports `marley_dcs::RawDcs`.
  - The root `[workspace.dependencies]` gains `marley_terminal`, and the `Cargo.toml` ledger row
    now names all eight members and six workspace dependencies, `marley_dcs` from #462 among
    them.
  - `crates/terminal`:
    - `marley_terminal` among its dependencies;
    - the `blocks` field in the struct and both of its literals;
    - the `ShellHook` arm calling `apply_shell_hook`: skip the alternate screen, decode, apply,
      notify, or log a refusal;
    - `blocks()`, and `block_output()`.
  - `alacritty.rs` gets `absolute_lines_text`, kept with the crate's other alacritty helpers
    since `terminal.rs` reaches alacritty's types only through that module.
- **Deviation: REQ-004's test.** The PTY test planned for it failed first: all 500 lines were
  still held. Task terminals always get `MAX_SCROLL_HISTORY_LINES`
  (`terminal.rs:1202-1207`), whatever the setting says. A unit test of
  `absolute_lines_text` over a `Term` with a three-line screen and two lines of history
  replaces it, and also covers a running block, an end past the screen, and empty ranges.

## Phase 3 — Test (2026-09-23)
- **REQ-001:** `marley_shell_hooks_leave_a_finished_block_with_its_output`. `/bin/sh` on a real
  PTY prints `init` and `preexec`, runs `echo hi`, and prints `precmd;exit=0`. The result is a
  Finished block with exit 0 whose output is "hi".
- **REQ-002:** `marley_one_write_with_the_whole_command_leaves_the_same_block`, with one
  `printf` carrying the frames and `hi\r\n`.
- **REQ-003:** the five `anchored` unit tests.
- **REQ-004:** `absolute_lines_text_reads_held_lines_and_none_once_evicted`.
- **Negative checks** (restored by sha256 checksum):
  - N1, the arm ignoring hooks again: both PTY tests fail.
  - N2, a block's end anchored at its start: two model tests and both PTY tests fail.
  - N3, eviction ignored in the mapping: the mapping test fails.
- **Live drive:** N/A. Nothing is drawn until T1.
- **Gate:** `script/gates.sh --diff`, scope the Marley crates plus `terminal` and
  `marley_terminal`: GATE GREEN [diff], 20 of 20. 560 tests ran over the scope, and the copy's
  own tests ran in gate:3. Coverage was 100% of lines (2837) and functions (294). The receipt
  matches.
- **Pre-existing:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist:** [x] document · [x] knowledge · [x] close the ticket · [x] archive · [x] commit.
- **Docs:** `CHANGELOG.md` (Added), `docs/marley_architecture/terminal_blocks.md`
  (`anchored.rs`), `docs/marley/three-prong-plan.md` (the T0 row), and the three ledger rows.
- **Knowledge:** `AD-claude-464-blocks-are-anchored-ranges-read-from-the-grid-001` and
  `L-claude-464-zeds-task-terminals-keep-the-maximum-history-001`. No `F-` block: the
  eviction test's failure was a wrong assumption in the test, not a bug.
- **Brain:** consultation `ca8707c62c33421da9e2d43e90e99823` closed with a decision, follow-up
  by 2026-10-07.
- **Ticket:** #464 closed. #463, the shell scripts, is the last of T0.
