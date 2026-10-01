# A block's path links resolve against the block's own folder — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-619-block-path-links-at-the-blocks-folder.md
- **Pipeline spec:** 619-block-path-links-at-the-blocks-folder.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, first batch (#619 to #623): T2 and the rest of T4.
- **Recall (§18.3):**
  - `cwd_at_line` falls back to the current folder once the scrollback is full and for remote terminals; that fallback is where a block's own folder matters.
  - `process_hyperlink` runs without the terminal's lock, so the block lookup needs its inputs passed in.
  - Plan D6 and the fusion note name block-scoped links as the remaining half of the terminal-editor link.
- **Discovery:** one Explore sweep for T2 and T4 (2026-09-30) over the block model, the
  terminal's links, the failure shapes, the task spawn path and the diagnostics store; the spec's
  Prior art cites what applies here, and the Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-09-30)
- **Promoted** from `queued/`; the seams re-verified: `process_hyperlink` (`terminal.rs:2176`) takes
  `history_size` from its two callers, `InternalEvent::FindHyperlink` (2143) and
  `ProcessHyperlink` (2166), which hold the terminal's lock; `cwd_at_line` (3356) falls back to the
  current folder for a remote terminal, an empty history, or `history_size >=
  scrolling_history`. Blocks count absolute lines: `evicted_lines + history_size + line`
  (`marley_hooks::HookPosition::absolute_line`); `block_lines(block, cursor_line)` (anchored.rs:685);
  `PromptInfo.pwd: Option<String>`; `AnchoredBlocks::block_host(index)` is none for the local
  shell.
- **Recall:** the queued notes' bullets stand. The brain (`rusty-cli brain ask`, consultation
  4cff6b9ad004497e89e60bf4a1c5db3e): nothing on this seam.

### Design
- **`crates/marley_terminal/src/anchored.rs`:** `AnchoredBlocks::folder_at(line, cursor_line) ->
  Option<&str>`: the local block (no `block_host`) whose `block_lines` holds the absolute `line`,
  and its `prompt.pwd`.
- **`crates/terminal/src/terminal.rs`** (Zed crate, `// Marley:` hunks): both callers compute the
  match's absolute line from the locked grid (`evicted_lines + history_size + line`) and ask
  `self.blocks.folder_at`, unless the terminal is remote or the alternate screen shows;
  `process_hyperlink` takes that folder (`Option<PathBuf>`) and uses it before `cwd_at_line`.
- **Ledger:** the `crates/terminal/src/terminal.rs` row in `docs/marley/zed-touchpoints.md` gains
  the clause.
- **Manifest:** `anchored.rs` (Marley), `terminal.rs` (Zed), the touchpoints row, the scenario.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | `repo/a/src/main.rs` and `repo/b/src/main.rs` with their own text; `terminal.max_scroll_history_lines` 100; in `b`, `seq 1 300` fills the history; `cd ../a; printf 'src/main.rs:2:5\n'; cd ../b`; Ctrl+click on the link | `opened.png`: `a/src/main.rs` at line 2 |
| REQ-002 | the pointer on the link with Ctrl held | `hover.png`: the tooltip with `a/src/main.rs` |
| REQ-003 | review only | — |

### Risks
- A block whose command changes folder itself (`cd x && make`) resolves against the folder it
  started in; that is the block's folder as the shell reported it.

## Phase 2 — Code (2026-09-30)
- **Built:** `AnchoredBlocks::folder_at(line, cursor_line)` (`anchored.rs`); in `terminal.rs`,
  `marley_block_folder(term, line)` (the absolute line from `HookPosition`, none for a remote
  terminal or the alternate screen), called by both `FindHyperlink` and `ProcessHyperlink`, and
  `process_hyperlink`'s new `block_folder` used before `cwd_at_line`. The ledger row's clause.
- **Deviations:** none.
- **Review:** the hyperlink's `Point.line` is the grid's line (negative in the history), the
  coordinate `cwd_at_line` already takes; the absolute line is `evicted_lines + history_size +
  line`, as hooks count it; a link in a running block uses the cursor's line as the block's end.
- **Clippy found:** a stray `.0` on the line (the terminal crate's `Point.line` is an `i32`).
- **Gate:** `just gate-diff` GREEN, 17 PASS (the scenario written before it).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/619-block-path-links-at-the-blocks-folder.sh` (sway), three runs.
- **Run 1:** stopped at launch: `terminal_env` refuses settings with a `terminal` block of their
  own; the scenario sets `HOME` through `set_setting terminal.env` instead.
- **Run 2:** stopped at `click_with ctrl` (the runner names it `CTRL`). Its first shots showed the
  layout; `hover.png` had the link underlined but no tooltip.
- **Run 3, the shots read:**
  - `printed.png`: `seq 1 300` up to 300 (the history full at 100), then `cd ../a`, the `printf`
    block with `src/main.rs:2:5`, `cd ../b`, the prompt; each block with its pill.
  - `hover.png` (REQ-002): Ctrl held, the link underlined, and the tooltip
    `…/work.…/repo/a/src/main.rs:2:5`, `a`'s file though the shell is in `b`. Zed builds the
    tooltip once it found the link and shows it on a move, so the scenario points, waits and
    moves four pixels.
  - `opened.png` (REQ-001): after Ctrl+click, the editor on `a/src/main.rs` ("The main.rs in a."),
    the cursor at 2:5 (the status bar reads 2:5).
- **REQ-003** (outside a block, or remote, as Zed does): reviewed; with the history full,
  `cwd_at_line` alone gives the shell's folder now, `b`.
- **Log:** rust-analyzer's "Failed to discover workspace" in the scratch repository; not in scope.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md`; the guide (Blocks); `terminal_blocks.md`; the plan's T2 row; the
  touchpoints row (Code).
- **Knowledge:** L-claude-619-a-scenario-that-sets-terminal-settings-cannot-use-terminal-env-001.
- **Brain:** consultation 4cff6b9ad004497e89e60bf4a1c5db3e closed with `brain decide`.

