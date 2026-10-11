# Restored blocks — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-764-restored-blocks.md
- **Pipeline spec:** 764-restored-blocks.spec.md

## Phase 1 — Plan
- **Request:** split from #751 on 2026-10-11 (Chad, "spec all" of the survey list; item 1,
  "Restarts that lose nothing": "1.) this is nice"). #751's notes sketched it: records beside the
  log with offsets, redrawn as blocks marked restored with no Rerun. The ticket was minted then; this
  plan writes its pipeline pair.
- **Classification:** feature, M: one record path, one restore path, and the readers that must not
  take history for the present.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **Recall (§18.3):**
  - AD-claude-464: a block is three absolute lines into the grid and its metadata; its output is
    read from the grid, never stored. Restored blocks are the same, at the lines the replay drew.
  - AD-claude-474 and PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001: only
    a verified command is offered Rerun; showing a field needs no check. A restored block is never
    run again.
  - AD-claude-628: the header takes a verified, non-pending block's prompt rows; AD-claude-630: its
    branch is the repository's at the block's start, recorded on the main thread.
  - AD-claude-639: blocks follow Zed's clears by a counted move; Zed's Clear is not in the byte
    stream, so a kept file still holds what it cleared.
  - #751 (AD-claude-751, F-claude-751-a-restored-files-two-sessions-ran-together-001): the kept
    file, its caps cut at a line's start, `SESSION_END` between sessions, the writer's batching.
  - Brain: consultation 1fbc8edd3b78434899022c850a1e6cb5, nothing on this seam.
- **Discovery (an Explore pass at `bab9fbc1ee`):**
  - Hooks: `terminal.rs:2044` → `apply_shell_hook` 2058-2085 (alternate screen skipped 2063,
    `line_now` 2067, `decode_frame` 2070, `blocks.apply` 2075, `stamp(now)` 2077).
    `marley_terminal::dcs`: `decode_frame` 370, `DcsHook` 56-92 (`Precmd { exit_code, prompt }`,
    `Preexec { command, nonce }`, `Signed`); no prompt-start or output-start hooks. bash sends
    `precmd;exit;pwd;nonce` (`marley.bash:100`) and `preexec;command;nonce` (107-114); zsh and fish
    the same two. A finished block: `prompt_line` = the previous precmd's line, `output_start` = the
    preexec's, `output_end` and the exit code = the next precmd's.
  - `AnchoredBlock` (`anchored.rs:43-65`): index, command, `command_verified`, state, `exit_code`,
    prompt, `prompt_line`, `output_start`, `output_end`; tests build it as a literal.
    `AnchoredBlocks` (186-214): `blocks`, `times`, `hosts` in lockstep; `apply` 244-326, `stamp`
    398-415 (times for blocks without them, at `now`), `rerun_offered` 364-370.
  - The builder: nonce 1328-1335, restored id 1351-1361, `new_term` 1477, the #751 replay and
    recorder 1488-1496, `spawn_event_loop` 1569-1575, `blocks` built 1668-1681 (nonce, agents out
    of history, a task's block), `subscribe` drains the hook channel 1726-1733.
  - Drawing: spans `terminal_element.rs:2523-2536`, wash 2725, bar 2735, pill 2750; header
    `prompt_rows` (`anchored.rs:770-788`) and `block_headers.rs:119-182` (`Branches` by entity and
    index, `record_branches` 88-115); hover actions 2808-2921, Rerun from `rerun_offered` (1810)
    gated again at 2831-2833; Reinput in `blocks.rs:240-255`, 295-311.
  - Readers that would take a restored block as current: `failures::read_finished`
    (`failures.rs:138-190`, from index 0 at view creation, publishing #623 diagnostics);
    `send_block::chip` (160-172, the last block failed and at the prompt); `english::asks_on_127`
    (307-318); the rail's `command_snapshot` (`rail.rs:8370-8404`); System One's last command
    (`system_one.rs:1341-1368`); `typed_line` last exit (335-342). `terminal_blocks` (`mcp.rs`
    971-1043, schema `registry.rs:1908-1969`, description 150-162) lists them with no flag.
    `command_watch` reads the last index at view creation, after the builder's blocks exist.
  - The sink: `advance_with_hooks` (`marley_hooks.rs:84-101`) calls it with passthrough pieces
    only; the hook arm sends `Event::ShellHook`. `RawDcs` holds the selector and the payload.
  - Nothing in `marley_terminal` derives serde; `marley_dcs` has no dependencies.

### Design
- **The record (`marley_terminal::history`, Marley crate):** `Mark::Command { at_ms, verified,
  command }` and `Mark::Prompt { at_ms, exit: Option<i32>, pwd: Option<String> }`; `Mark::frame()`
  gives `ESC P p mark;kind=command;at=…;verified=1;command=<hex> ESC \` (or
  `kind=prompt;at;exit;pwd=<hex>`), `Mark::read(payload)` the reverse, `None` for anything else.
  Hex keeps `;`, `=` and newlines out, so a cut at a line's start never splits a record.
- **The restore's pieces (`marley_terminal::history`):** `restore(kept) -> Vec<Piece>`, `Piece =
  Draw(Vec<u8>) | Mark(Mark)`: a `DcsScanner` over the kept bytes; passthrough runs through one
  `Allowlist::dimming` (its state carried across records), starting with `marley_dcs::history::DIM`
  and ending with `SESSION_END`; a `p` frame that reads as a mark ends the current draw piece. It
  replaces `marley_dcs::history::replay` in the restore path.
- **The restored blocks (`marley_terminal::anchored`):** `Restoring`, fed `(Mark, line)`: a prompt
  mark finishes the running block (its line, exit and time) and stages the prompt's folder and
  line; a command mark opens a block from the staged prompt at its line, finishing one still
  running without an exit code; `finish(line)` ends a block still running at the end without an
  exit code and gives the blocks with their times. `AnchoredBlocks::restore(blocks)` pushes them
  (blocks, times, hosts as local) on an empty list and counts them; `is_restored(index)`;
  `rerun_offered` is false for them.
- **The vendored sink (Marley-owned for the ledger):** the hook arm of `advance_with_hooks` also
  feeds the sink the frame as it came (`RawDcs::to_bytes`, a new `marley_dcs` method).
- **Zed's `terminal` (rows exist):** `MarleyTerminalHistory::restored` returns `Option<Vec<Piece>>`
  and `recorder` takes the nonce; the builder feeds the pieces through one `Processor`, reads
  `HookPosition::of(&term).absolute_line()` at each mark into a `Restoring`, finishes it at the end,
  and after building `blocks` calls `blocks.restore(...)`.
- **`marley_workbench::scrollback`:** the recorder runs a `DcsScanner` over what the sink gives:
  passthrough through the allowlist as now; a frame decoded with `decode_frame`; a `Preexec` or
  `Precmd` (inside `Signed` too) becomes a `Mark` frame in the kept bytes, verification from the
  live nonce, unless the allowlist is on the alternate screen (`Allowlist::on_alternate_screen`).
  `restored` returns `history::restore`.
- **The readers (`marley_workbench`, `terminal_view`):** `failures` starts after the restored
  blocks; `send_block::chip`, `english::asks_on_127`, `rail::command_snapshot`, System One's last
  command and `typed_line`'s last exit skip a restored last block; `block_headers` records no branch
  for a restored block and shows `restored` in its place; `blocks.rs`' Reinput already follows
  `rerun_offered`.
- **`terminal_blocks` (`marley_workbench/src/mcp.rs`, `marley_mcp/src/registry.rs`):** a `restored`
  field on a restored block, and the tool's description.
- **File manifest:** `crates/marley_dcs/src/{marley_dcs.rs, history.rs}`,
  `crates/marley_terminal/src/{history.rs, anchored.rs}`, `crates/marley_workbench/src/{scrollback.rs,
  failures.rs, send_block.rs, english.rs, rail.rs, system_one.rs, typed_line.rs, block_headers.rs,
  mcp.rs}`, `crates/marley_mcp/src/registry.rs` (Marley crates); `vendor/alacritty_terminal/src/
  marley_hooks.rs` and `vendor/README.md`; `crates/terminal/src/terminal.rs` (Zed crate, row
  exists); `docs/marley/guide.md`, the in-app guide; `script/e2e/764-restored-blocks.sh`,
  `script/e2e/browser-fixture.sh` (the `blocks` line prints `restored`).

### Visual check plan
| REQ | The scenario does | The shot or check |
|---|---|---|
| — | `echo one`, `ls missing-file`, `echo two` in the first terminal | 764-01-before |
| 001, 002, 005 | a second terminal `sleep 600`; quit, launch | 764-02-restored: three blocks, bars and pills, headers with `restored`; the rail row with no command line |
| 003 | the pointer over the failed restored block | 764-03-hover: no Rerun, no "Ask the agent" |
| 007, 005 | `echo fresh`, pointer over it | 764-04-fresh: a live block, Rerun shown, the rail row `echo fresh` |
| 008 | the second terminal | 764-05-running: `sleep 600`'s block without a pill |
| 006 | `mcp_agent blocks` | the three restored `True`, `echo fresh` not |
| 009 | `grep -c mark;kind=command` and no `nonce` in the kept file | the check |
| 004 | review | `failures` starts after the restored blocks |
Not reached by a scenario: a diagnostic from a restored failure (no parser output to show without a
compiler error format; the review covers it), the alternate screen's hooks (no shell prompts there).

### Risks
- **Readers missed.** Any new reader of "the last block" must skip restored ones; the review
  checks every `blocks().last()` site.
- **Records in a hand-edited file** can make a restored block appear with a header; it is display
  only, and offers nothing that acts.
- **Block indices** shift by the restored count for the session's live blocks; marks keyed by
  index (bookmarks, branches) start empty, so nothing points at a wrong block.
- **A clear before the quit** restores what it cleared (Out).

## Phase 2 — Code
- **Built:**
  - `marley_dcs`: `RawDcs::to_bytes` (a frame as it came); `history::DIM` public;
    `Allowlist::on_alternate_screen`; `history::replay` removed (the restore builds its own pieces).
  - The vendored `advance_with_hooks` feeds the sink each hook frame (`frame.to_bytes()`) before
    reporting it; `vendor/README.md`'s bullet names it.
  - `marley_terminal::history`: `Mark::{Command, Prompt}` with `frame` and `read` (a `p` frame
    whose payload starts `mark;`, the text hex-encoded); `Piece::{Draw, Mark}`; `restore(kept)`.
  - `marley_terminal::anchored`: `Restoring` (`mark`, `finish`); `AnchoredBlocks::restore`,
    `is_restored`, `restored_count`, `live`; `rerun_offered` false for a restored block; `stamp`
    skips the restored blocks.
  - Zed's `terminal`: `MarleyTerminalHistory::restored` gives pieces and `recorder` takes the nonce;
    the builder feeds the pieces through one `Processor`, places each mark with
    `HookPosition::of(&term).absolute_line()` in a `Restoring`, and calls `blocks.restore(...)`
    after building the list.
  - `marley_workbench::scrollback`: the recorder runs a `DcsScanner`; `mark_of` turns a `Preexec`
    (not Zed's startup check) or a `Precmd` (inside `Signed` too) into a mark, verified against the
    live nonce, none on the alternate screen; `restored` returns `history::restore`.
  - Readers on the live blocks: `send_block::chip`, `english::asks_on_127`, the rail's
    `command_snapshot`, System One's `check_asking`, `typed_line`'s last exit, `failures`
    (starting after the restored count), `block_headers::record_branches`; the header shows
    `restored` for a restored block.
  - `terminal_blocks`: `restored` on each entry, a cut-off restored block's `duration_ms` null; the
    schema and the description in `marley_mcp::registry`.
  - The scenario's helper (`browser-fixture.sh`) prints `restored` on each `blocks` line.
  - Guides: `docs/marley/guide.md` and the in-app guide.
- **Deviations from the plan:**
  - `restored` is set on `block_entry`'s answer rather than passed in: an eighth parameter fails
    clippy's `too_many_arguments`.
  - `AnchoredBlocks::restored_count` joins `is_restored` and `live`, for `failures`' start.
  - `marley_dcs::history::replay` is gone rather than kept unused.
- **Review of the diff:**
  - Caught while writing it: `stamp` walks back to the first block with an end, and a restored
    block the quit cut off has none, so the first live hook would have stamped it finished now; the
    walk skips the restored blocks. `terminal_blocks` gives such a block no duration for the same
    reason.
  - No path runs or retypes a restored block: `rerun_offered` gates the element's Rerun (checked
    again on verification) and Reinput in `blocks.rs`.
  - `command_watch` reads the last index when the view is made, after the builder's blocks exist,
    so no restored block posts a "done" notification.
  - Marks go only between whole kept sequences: the allowlist holds a partial escape until it ends,
    so a mark never splits one; and they hold no newline, so the cap's cut at a line's start never
    splits a mark.
  - An ssh host's commands in a local terminal are verified live by the connection's nonce (#526);
    the recorder knows only the terminal's, so they come back unverified, without a header.
  - Provenance: Marley's own crates and Zed's builder; nothing from Warp.
- **Gate:** `just gate-diff` RED on spelling alone (typos read the scenario's `grep -aci` as
  `acpi`; the flags are now apart), then GREEN, 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/764-restored-blocks.sh` (`compositor sway`); the first run passed every
  check and every shot showed its criterion, so no coordinate moved.
- **Shots:**
  - `764-01-before`: three live blocks, `echo one` (✓), `ls missing-file` (red bar, `exit 2`),
    `echo two` (✓), each header `…/repo · main`; the rail row `echo two · done · 0 s`.
  - `764-02-restored` (REQ-001, REQ-002, REQ-005): after the relaunch the same three blocks over
    the dimmed output, bars green, red, green and pills ✓, `exit 2`, ✓; each header `…/repo ·
    restored`; the old waiting prompt `$` dim below them and the new prompt bright; the rail row
    `repo — bash` with no command line.
  - `764-03-hover` (REQ-003): the pointer over the restored failed block shows Bookmark, Find, Save
    as Workflow, Filter and Copy, and no Rerun; no "Ask the agent" chip.
  - `764-04-fresh` (REQ-005, REQ-007): `echo fresh` is a live block, its header `…/repo · main`;
    hovered, its actions end with Rerun; the rail row reads `echo fresh · done · 0 s`.
  - `764-05-running` (REQ-008): the second terminal: `sleep 600`'s block, header `…/repo ·
    restored`, a grey bar and no pill.
- **Checks:** `command marks 4, nonces 0` (REQ-009); `terminal_blocks`: blocks 0-2 `restored
  True`, `echo fresh` `restored False`, and the second terminal's `sleep 600` `exit None, restored
  True` (REQ-006).
- REQ-004 by the review (`failures` starts after the restored blocks).
- **Not reached by a scenario:** a hook on the alternate screen (no shell prompt draws there); a
  diagnostic from a restored failure (no compiler-format output in the scenario).
- **Seen, not in scope:** a restored header's command is drawn at full brightness over the dimmed
  output, the header being Marley's own element.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Added: earlier commands come back as blocks); `docs/marley/guide.md` and
  the in-app guide (the restored-output article); `docs/marley_architecture/marley_workbench.md`,
  `marley_dcs.md` and `terminal_blocks.md`; `docs/marley/zed-touchpoints.md`, the row of
  `crates/terminal/src/terminal.rs` widened; `vendor/README.md` names the hunk.
- **Ledgers:** AD-claude-764-a-restored-block-is-placed-from-a-mark-in-the-kept-bytes-001,
  L-claude-764-blocks-added-from-elsewhere-keep-out-of-the-live-walks-001.
- **Brain:** consultation 1fbc8edd3b78434899022c850a1e6cb5, `brain decide`.
- Ticket closed, pipeline archived, committed.
