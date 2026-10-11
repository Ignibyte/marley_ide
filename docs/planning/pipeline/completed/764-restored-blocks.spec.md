---
pipeline_id: 9a32a3ae-8856-45dc-9035-62cb7325d6ed
ticket: docs/planning/tickets/open/TICKET-764-restored-blocks.md
status: Phase 4 — Complete PASS
title: Restored blocks
type: feature
slice: the finish line (docs/planning/design-notes/finish-line-2026-10-11.md), item 1, restarts that lose nothing, second half (second slice)
references:
  - docs/planning/pipeline/completed/751-restored-scrollback.spec.md
  - docs/planning/pipeline/completed/751-restored-scrollback.notes.md
  - docs/planning/pipeline/completed/464-zeds-terminal-keeps-anchored-blocks.spec.md
  - docs/t3code_architecture/05-terminal-browser-and-capture.md
---

## Title
After a restart, the commands a terminal ran before come back as blocks over #751's dimmed output:
each finished command with its bar and its exit pill, and Marley's header (its folder, and
"restored" where the branch goes) in place of its prompt, as the terminal drew it. A restored block
is history: it offers no Rerun, no Reinput, no "Ask the agent", no English hint, publishes no
diagnostics, is not the rail row's last command, and an agent reading the terminal's blocks is told
it is restored. The first command typed after the restore makes a live block as before. The second
slice of the finish line's first item; #751 restored the output (Chad, 2026-10-11: "1.) this is
nice").

## Scope
### In
- **Records in the kept file.** While a local terminal's output is kept (#751), each Marley shell
  hook that opens or closes a block (`preexec`, `precmd`) adds a record to the kept bytes, at the
  place it fell: a DCS frame of Marley's own (`ESC P p mark;… ESC \`) that no replay draws. A
  command's record holds its text, whether the terminal's nonce verified it, and the time; a
  prompt's holds the exit code, the folder and the time. No nonce is written. A hook on the
  alternate screen adds nothing, as it opens no block live.
- **Blocks drawn again.** A restored terminal's builder feeds the replay in pieces cut at the
  records and takes the cursor's absolute line at each, so each record lands where the replay put
  its rows; the records become finished blocks (command, verification, exit code, folder, times,
  prompt line, output start and end), added to the terminal's blocks before any live hook. A
  command still running at the quit becomes a block with no exit code. A record whose command
  left the caps leaves nothing; a command whose prompt left them has no prompt rows.
- **Marked restored.** `AnchoredBlocks` counts its leading restored blocks (`is_restored`). The
  header's place line reads the folder and `restored`; `record_branches` records no branch for a
  restored block.
- **History, not current.** A restored block offers no Rerun and no Reinput (`rerun_offered`);
  `send_block`'s "Ask the agent", `english`'s hint on 127, `failures` (#623's diagnostics), the
  rail row's command line, System One's "last command" and `typed_line`'s last exit skip it.
  `terminal_blocks` lists it with `restored: true`, its tool description saying what that means.
- `docs/marley/guide.md` and the in-app guide: restored blocks.
- `script/e2e/764-restored-blocks.sh`.

### Out (explicitly deferred)
- The branch a restored block ran on: no shell script sends `git=`, and the branch the header
  shows today is the repository's at the block's start, read on the main thread. The header says
  `restored` there instead.
- Zed's Clear (and the clear before an agent's command) is a grid operation, not bytes: the kept
  file still holds what it cleared, so a restore shows that output and its blocks.
- Remote terminals, tasks and display-only terminals, which keep nothing (#751).
- Bookmarks, block filters and sends made before the restart (#559's marks are in memory).
- Blocks for agents' commands run through `terminal_run` read as such after a restore; they are
  restored blocks like any other.

## Reference (§20)
Warp (blocks): Session Restoration brings back "your windows, tabs, panes, and recent Blocks" when
Warp relaunches (docs.warp.dev/terminal/sessions/session-restoration, cited by #540), and Warp keeps
its blocks as structured rows in its local SQLite store, apart from the bytes
(`docs/warp_architecture/subsystems/06-platform-settings-infra.md:89-90`, `:170`). Marley matches
the visible part: a restored terminal shows its earlier commands as blocks over their output. T3
Code's report 05 §3 item 1 (MIT): blocks come back from saved records, never from replayed hook
bytes.

### Prior art
- **The code we ship:**
  - `marley_terminal::anchored::AnchoredBlocks` already models a block as three absolute lines
    into the grid with its command, verification, exit code, prompt and times (AD-claude-464);
    restored blocks are the same values at the lines the replay measured, so every element that
    draws a block (bars, pills, headers, the sticky header, copy, filter) draws them unchanged.
  - `vendor/alacritty_terminal/src/marley_hooks.rs` `advance_with_hooks` already sees each hook
    frame in stream order between the passthrough pieces the #751 sink gets; the sink only needs
    the frame too.
  - `marley_dcs::DcsScanner` finds `p` frames read by read; it splits the kept file into drawn
    bytes and records at the restore, so no second scanner is written.
  - `marley_terminal::decode_frame` decodes the live frame on the recorder's thread; the records'
    own payload is `key=value` pairs like the hooks' plain encoding, with the text hex-encoded so a
    record holds no `;`, `=` or newline.
  - `HookPosition::of(&term).absolute_line()` is how the event loop measures a hook's line; the
    builder measures each record the same way.
- **Behavior maps:** T3 report 05 §2.1 and §3 item 1 (records apart from bytes); Orca report 05
  §2.2 (a cold restore redraws scrollback, no blocks); Warp's docs above.
- **Published material:** none beyond ECMA-48 for the DCS form.

## UI proof
`script/e2e/764-restored-blocks.sh`, `compositor sway`, a scratch repository, a HOME whose
`.bashrc` sets a plain prompt and puts the scenario's `bin` first.
- The first terminal: `echo one`, `ls missing-file` (exit 2), `echo two`; a second terminal runs
  `sleep 600` at the quit.
- Shots:
  - `764-01-before`: the first terminal's three blocks before the quit.
  - `764-02-restored`: after a relaunch: the three blocks over the dimmed output, bars and pills
    (green, red with 2, green), each header naming the folder and `restored`; the rail row shows no
    command line.
  - `764-03-hover`: the pointer over the restored failed block: its actions with no Rerun, and no
    "Ask the agent" chip.
  - `764-04-fresh`: `echo fresh` typed: a live block with its branch in the header; hovered, Rerun
    shows; the rail row reads `echo fresh`.
  - `764-05-running`: the second terminal: `sleep 600`'s block with no pill.
- Checks: `terminal_blocks` through the run's MCP agent lists the three with `"restored": true`
  and `echo fresh` without it; the kept file holds three command records and no nonce.

## Locked-In Decisions
- **D1 — Records ride in the kept bytes,** at the place their hook fell, as DCS frames of Marley's
  own. Rejected: a `.blocks.jsonl` beside the log with byte offsets (offsets drift with every
  compaction, which cuts the head, and the writer batches); absolute lines (the replay's width and
  cut differ from the session's).
- **D2 — The sink gets the hook frame,** re-encoded as it came, in stream order; the recorder finds
  it with its own `DcsScanner`, decodes it, and writes the record. The vendored hunk stays one
  call. Rejected: a second callback or an enum for the sink (every sink site changes); positions
  from the event loop (they are grid lines, which the file has none of).
- **D3 — Verification is decided when the record is written,** from the terminal's live nonce,
  which the recorder is given; no nonce is written to disk.
- **D4 — Positions are measured during the replay,** in the builder, as hooks' are: a piece of
  drawn bytes, then the cursor's absolute line at each record.
- **D5 — Restored blocks lead the list,** indices `0..n`, counted by `AnchoredBlocks`; no field is
  added to `AnchoredBlock`, whose literals the tests build.
- **D6 — History, never current:** no action that would run the command again or put it back,
  nothing that reads "the last command" or "the latest failure", and an agent told `restored`.
  Showing (bars, pills, headers, copy, filter, sticky header) needs no check (PR-474).
- **D7 — No branch is guessed:** the header says `restored` where the branch goes.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley relaunches and restores a local terminal whose kept output holds finished commands, the system shall draw each as a block over its output, with its bar and its exit pill. | Shot 764-02 |
| REQ-002 | A restored block whose command the shell's own hook reported shall show Marley's header in its prompt rows, naming its folder and `restored`. | Shot 764-02 |
| REQ-003 | The system shall offer no Rerun, no Reinput, no "Ask the agent" and no English hint on a restored block. | Shot 764-03; review |
| REQ-004 | The system shall publish no diagnostics from a restored block. | Review of the diff |
| REQ-005 | The rail row, System One's last command and the last exit shall ignore restored blocks. | Shots 764-02, 764-04; review |
| REQ-006 | `terminal_blocks` shall mark each restored block `restored: true`. | The scenario's check |
| REQ-007 | The first command after the restore shall make a live block, with Rerun offered. | Shot 764-04 |
| REQ-008 | WHEN a command was running at the quit, its restored block shall show no exit code. | Shot 764-05 |
| REQ-009 | The kept file shall hold no nonce, and no record for a hook on the alternate screen. | The scenario's check; review |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the records (`marley_terminal::history`), the restore's pieces and the restored
  blocks (`history`, `anchored`), the sink's hook frame (vendored, `marley_dcs`), the builder's
  piecewise replay (Zed's `terminal`), the recorder and the readers (`marley_workbench`), the
  `terminal_blocks` field (`marley_mcp` schema, `mcp.rs`), the guides; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), touchpoint rows checked, ledger capture
  (§19), close the ticket, archive, commit.
