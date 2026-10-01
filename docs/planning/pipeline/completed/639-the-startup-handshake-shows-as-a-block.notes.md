# Blocks follow Zed's screen clears, and the startup check is no block — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-639-the-startup-handshake-shows-as-a-block.md
- **Pipeline spec:** 639-the-startup-handshake-shows-as-a-block.spec.md

## Phase 1 — Plan (promoted 2026-10-01)
- **Request:** Chad, 2026-10-01: "lets do 639 and then wait on 445", run autonomously (a session
  goal).
- **Classification / tier:** bug, prong 1 blocks. `marley_terminal` (Marley) and two hunks in
  `crates/terminal/src/terminal.rs` (Zed, row exists).
- **Checklist:** pick ✓, pre-flight ✓, recall ✓, mint ✓, prior art ✓, spec ✓, design ✓.
- **Recall (§18.3):**
  - AD-claude-470: blocks are anchored to absolute lines, `evicted + history + row`.
  - #544's `rewrap`: anchors carried across a change to the grid; a clear needs the same.
  - Brain (`rusty-cli brain ask`, consultation 2100b0956c8b46619a921eb7fd352333): nothing on this
    seam.
- **Calibration (the current build, 2026-10-01, before any change):** `echo one`, `echo two`,
  Escape, Ctrl-Shift-L: the two headers stay drawn over the cleared screen; `echo three` then
  shows `three` under `echo one`'s header. The New Agent picker's Claude Code: the new terminal
  shows `printf '%s%s%s\n' __zed_init_command_ready_ 1 __` as its block, the stand-in's output
  under it, and no `claude` block. So the ticket covers Zed's Clear too: the same
  `clear_saved_screen`, the same missing hook.
- **Discovery:** `clear_saved_screen` (`alacritty.rs:784`) clears the history (Marley's
  `clear_history` counts it as evicted, so absolute numbering goes on), resets the rows above the
  cursor, copies the cursor's line to row 0 and resets the rows below. Its callers:
  `InternalEvent::Clear` (`terminal.rs:2131`) and `clear_for_init_command` (`terminal.rs:2685`).
  Absolute lines: `HookPosition::of(term).absolute_line()`.

### Design
- **`marley_terminal::anchored`:**
  - `ZED_STARTUP_MARKER = "__zed_init_command_ready_"`; in `apply`, a `Preexec` whose command
    holds it finishes a running block, drops the staged prompt and the input start, clears the
    prompt shell, and pushes no block (nor a host).
  - `screen_cleared(cursor_was, cursor_now)`: a line at or below `cursor_was` moves by
    `cursor_now - cursor_was`; a block that ended at or above `cursor_was` is emptied
    (`prompt_line` none, `output_start` and `output_end` at `cursor_now`, a span of nothing); a
    running block whose start is above loses its prompt line and starts its output at
    `cursor_now`; the staged prompt moves to the cursor's new line; an input start on the cursor's
    old line moves with it, any other goes.
- **`terminal.rs` (Zed):** in the Clear arm and `clear_for_init_command`, `HookPosition::of` before
  and after `clear_saved_screen`, and `self.blocks.screen_cleared(..)` off the alternate screen.
- **File manifest:** Marley: `crates/marley_terminal/src/anchored.rs`;
  `script/e2e/639-the-startup-handshake-shows-as-a-block.sh`. Zed: `crates/terminal/src/terminal.rs`
  (its row widened first).

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | sway; trust; click; `echo one`, `echo two`; Escape; Ctrl-Shift-L | `639-01-cleared`: the prompt alone, no header |
| REQ-002 | click; `echo three` | `639-02-after-clear`: one block, `echo three` and `three` |
| REQ-003 | Ctrl-Alt-N, `Claude Code`, Return | `639-03-agent`: the new terminal's block is `claude` with the stand-in's line, no `printf` block |
| REQ-004, REQ-005 | review | the shared path; the running branch |

### Risks
- **Timing:** the startup check's `Precmd` may come before or after Zed's clear; either way no
  block exists for it (D1), and the staged prompt is moved if it came first, anchored right if it
  came after.
- **A user's command holding the marker** would open no block; only Zed types it.

## Phase 2 — Code (2026-10-01)
- **Built:** `marley_terminal::anchored::ZED_STARTUP_MARKER`; `apply`'s guarded `Preexec` arm for
  the startup check (finishes a running block, drops the staged prompt and the input start, opens
  no block or host); `AnchoredBlocks::screen_cleared`; in `terminal.rs`, `HookPosition::of` around
  `clear_saved_screen` in the Clear arm and in `clear_for_init_command`, then
  `screen_cleared(was, now)` off the alternate screen; the scenario; the touchpoint row widened.
- **Deviations:** none. A first `cargo check` failed on `missing_docs`: the constant had landed
  between `AnchoredBlock`'s doc comment and its struct; moved above the comment.
- **Dry run (before the gate):** `639-01-cleared` showed the prompt alone, `639-02-after-clear`
  one block for `echo three`, `639-03-agent` the `claude` block with the stand-in's line, against
  the calibration run on the build before the change (the two headers over the cleared screen,
  `three` under `echo one`'s header, the `printf` block).
- **Review:**
  - REQ-001/002: every finished block ends at or above the cursor's old line (`output_end` is the
    next prompt's line), so a clear empties them all; an empty span (`output_start ==
    output_end`, no prompt line) is left out of `visible_spans` and `prompt_rows`, so nothing draws.
  - REQ-003/004: the check's own `Preexec` opens no block whichever side of Zed's clear its
    `Precmd` lands on; if that `Precmd` came first, `screen_cleared` moves the staged prompt to the
    top row, so the agent's command opens its block with the right prompt line.
  - REQ-005: a running block keeps going: a start above the cursor's old line becomes the new
    line, its prompt line goes.
  - Indices never change (D2); `hosts` stays aligned with `blocks`, since the check pushes neither.
  - The alternate screen is left alone: its grid is not the blocks' (AD-claude-470).
- **Gate:** `script/gates.sh --diff` GATE GREEN (17 of 17), the first run, at `nice 19`.

## Phase 3 — Test (2026-10-01)
- **The first run for the record went red on REQ-003** (`639-03-agent`): no `printf` block, but
  `$ claude` drawn as plain rows, no header and no gutter, where the dry run had drawn the
  `claude` block. **A real bug of the first design (F-…):** the shell's hook frames are parsed on
  the PTY thread, each with the absolute line it fell on, and applied later on the main thread;
  Zed's clear runs on the main thread in between. The startup check's `Precmd`, parsed before the
  clear and applied after it, staged the prompt at a line from before the clear, below where the
  agent's command then started, so its block's span was empty. The dry run had applied the frame
  in time.
- **Fix (back to Code):** the vendored grid counts Zed's clears (`marley_clears`,
  `marley_count_clear`, called by `clear_saved_screen`); `HookPosition::clears` carries the count
  at the hook; `AnchoredBlocks` keeps the last clears (count, cursor before, cursor after, eight at
  most), and `apply_shell_hook` takes the hook's line through `line_now`, which carries a line
  parsed before a clear across it as `screen_cleared` carries the anchors. `vendor/README.md`
  lists the two hunks; the `alacritty.rs` and `terminal.rs` rows name them.
- **Gate again:** `script/gates.sh --diff` GATE GREEN (17 of 17), the first run after the fix.
- **Scenario, three runs on the fixed build** (`script/e2e.sh
  script/e2e/639-the-startup-handshake-shows-as-a-block.sh`, a headless sway, each exit 0; focus
  report: no Hyprland window, sway stopped with the run's Marley):
  - `639-01-cleared` (REQ-001), all three: the prompt `$ ` alone after Ctrl-Shift-L, no header.
  - `639-02-after-clear` (REQ-002), all three: one block, `…/repo · main` over `echo three`, its
    output `three`, then the prompt.
  - `639-03-agent` (REQ-003), all three: the new terminal's block is `…/repo · main` over `claude`,
    with `the stand-in Claude Code runs` under it; no `printf … __zed_init_command_ready_` block.
- REQ-004 and REQ-005 rest on the review (#540's resume takes the same
  `write_init_command_after_startup` path).

## Phase 4 — Complete (2026-10-01)
- **Docs (§21):** `CHANGELOG.md` (Fixed); `docs/marley_architecture/terminal_blocks.md`
  (`screen_cleared`, `line_now`, the clear count, `ZED_STARTUP_MARKER`); `vendor/README.md` (the two
  hunks). Touchpoint rows checked: `crates/terminal/src/terminal.rs` (the two call sites and
  `apply_shell_hook`) and `crates/terminal/src/alacritty.rs` (`marley_count_clear`).
- **Ledger:** F-claude-639-a-hook-parsed-before-a-clear-was-applied-after-it-001,
  PR-claude-639-a-change-to-the-grid-on-the-main-thread-carries-the-hooks-in-flight-001,
  AD-claude-639-blocks-follow-zeds-clears-by-a-counted-move-001.
- **Brain:** `brain decide` on consultation 2100b0956c8b46619a921eb7fd352333
  (`decisions/marleys-blocks-follow-zeds-screen-clears-by-a-counted-move`).
- **Ticket:** closed; archived; committed on `marley/workbench-shell`. #445 stays held (Chad).

