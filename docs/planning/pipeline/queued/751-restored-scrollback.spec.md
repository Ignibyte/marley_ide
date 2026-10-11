---
pipeline_id: 07c3a6ff-8706-42ef-97e2-6e2ff2a16ff6
ticket: docs/planning/tickets/open/TICKET-751-restored-scrollback.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Restored scrollback
type: feature
slice: the finish line (docs/planning/design-notes/finish-line-2026-10-11.md), item 1, restarts that lose nothing, second half (first slice)
references:
  - docs/planning/pipeline/completed/540-session-resume-after-restart.spec.md
  - docs/planning/pipeline/completed/575-terminal-id-across-restore.spec.md
  - docs/planning/pipeline/completed/577-restored-terminal-keeps-its-folder.spec.md
  - docs/planning/pipeline/completed/461-vendor-alacritty-terminal.spec.md
  - docs/t3code_architecture/05-terminal-browser-and-capture.md
  - docs/orca_architecture/05-terminal-and-workspace.md
---

## Title
A local terminal Zed restores after a Marley restart shows the output it held before, dimmed,
above the new shell's first prompt. Each such terminal keeps its recent output, at most 5,000 lines
and 8 MiB, in a file of its own that only the user can read (mode 0600). The file keeps text and
plain formatting only: no terminal query, mode change, title, notification or Marley hook frame is
kept, so nothing in it can answer into the new shell, and no restored row becomes a block.
`marley.restore_scrollback` (on by default) turns it off and deletes the files. Chad, 2026-10-11,
on "Restarts that lose nothing": "1.) this is nice"; T3's open question 7, default on, mode 0600,
a setting to turn it off.

This is the first slice of the item. Sized whole it is L; the blocks half (below, Out) is a second
slice of about the same size, proposed for a number of its own.

## Scope
### In
- **Which terminals:** local interactive terminals, the ones with a `MARLEY_TERMINAL_ID` (#520,
  #575): the center's and the terminal panel's. Not task, display-only or remote terminals.
- **What is kept:** the bytes the terminal's parser sees, after Marley's hook frames are taken out
  (#462), through an allowlist: printable text, CR, LF, BS and HT; SGR; cursor motion; erase,
  insert and delete; scroll and the scroll region. Everything else is dropped, among it every query
  and report T3 strips (device status and cursor reports, DA, DECRQM, XTVERSION, the kitty keyboard
  query, DECRQSS, XTGETTCAP, the OSC 10/11/12 colour queries), every mode change, every OSC (titles,
  notifications, clipboard, links), DCS, APC, PM and BEL. Bytes written while the alternate screen
  is on are dropped, so a full-screen program's screen never comes back (Orca's rule).
- **The caps:** the newest 5,000 lines and at most 8 MiB, whole lines dropped at the head, never a
  split character (T3's caps, report 05 §2.1).
- **The file:** `<data dir>/terminal_history/<terminal id>.log`, the folder made 0700 and the file
  created 0600, written off the main thread within a second of the output going quiet, and at the
  quit (awaited, as #575's save is). An append-only file compacted to the caps once it passes twice
  them.
- **The restore:** when Zed builds a terminal for a restored id whose file exists and the setting is
  on, the builder (already off the main thread) reads the file, passes it through the allowlist
  again, and writes it into the grid with every cell dim (SGR 2 kept on through each SGR), ending
  with a reset and a new line, before the terminal's event loop starts reading the PTY. The new
  shell's first prompt is drawn below it at full brightness.
- **No blocks from the restore:** restored rows carry no block, pill, header or Rerun; the first
  command after the restore makes a block as usual.
- **The file goes** when the user closes the terminal, when the setting is turned off (every file,
  at once), and at start for an id no saved terminal holds (as #540's rows go). A quit keeps it.
- **The setting:** `marley.restore_scrollback: Option<bool>`, default `true`, in
  `MarleySettingsContent`, `default.json` and the Marley page's Terminal section.
- `docs/marley/guide.md`: what comes back, where it is kept, and the setting.
- `script/e2e/751-restored-scrollback.sh`.

### Out (explicitly deferred)
- **Blocks redrawn from saved records (the second slice, needs a number):** each finished block's
  command, exit code, folder, host and times saved beside the log with the log offsets where its
  prompt and output began and ended, and redrawn on the restored rows as pills and headers marked
  restored, with no Rerun. The design sketch is in the notes. Size M.
- Colours of a full-screen program, its screen, and a grid snapshot of what was on screen
  (alacritty has no serializer; Orca report 05 §2.2).
- Remote terminals (#543, #641: tmux on the host keeps their history), task and display-only
  terminals, the Agent Panel's terminal threads.
- Redacting the log (#516): the log holds what was printed; mode 0600 and the setting are the guard,
  T3's default for its open question 7.
- A marker line ("restored at …") between the old output and the new prompt: the dim says it.
- Keeping processes alive across a restart: the harness's (#632).

## Reference (§20)
Warp (terminal, blocks): Session Restoration brings back "your windows, tabs, panes, and recent
Blocks" when Warp relaunches (docs.warp.dev/terminal/sessions/session-restoration, cited by #540),
and Warp keeps its blocks as structured rows in its local SQLite store, apart from the bytes
(`docs/warp_architecture/subsystems/06-platform-settings-infra.md:89-90`, `:170`). Marley matches
the user-visible part this slice owns: a restored terminal shows its earlier output above a new
prompt. Blocks from saved records are the second slice. The byte rules are T3 Code's (MIT; report 05
§2.1: bounded history, queries stripped before storing and again on replay) and Orca's cold restore
(report 05 §2.2: a replay guard, a dead full-screen program's screen dropped).

### Prior art
- **The code we ship (the highest-yield leg):**
  - `crates/terminal/src/terminal.rs`: the builder runs in `cx.background_spawn` (1687), opens the
    PTY (1522) and only then starts the event loop (`spawn_event_loop`, 1539), so a replay written
    into `term` between the two lands above everything the shell prints, with no main-thread IO
    and no gate. The restored id arrives at 1335–1343 (#575). `write_raw_output` (2480) feeds bytes
    through a plain `Processor` with no hooks, which is why replayed hook frames could never make a
    block. The answers to a query go back to the PTY through the term's listener and
    `process_event` (`PtyWrite` 1966, `ColorRequest` 2002, `TextAreaSizeRequest` 1967,
    `ClipboardLoad` 1957), asynchronously, so a flag around the replay cannot stop them: the
    allowlist must.
  - `vendor/alacritty_terminal/src/marley_hooks.rs`: `advance_with_hooks` (74), called from
    `event_loop.rs:155` in `pty_read`, already takes Marley's frames out and feeds the remaining bytes
    to `marley_dcs::NotificationScanner` (#478); the recorder sits beside it.
  - `crates/marley_dcs` (no dependencies, usable from the vendored crate): `DcsScanner`, read by
    read with a carry; the allowlist is the same kind of scanner.
  - `crates/terminal_view/src/terminal_element.rs:929-932`: a cell with `Flags::DIM` (SGR 2) draws
    its foreground at 0.7 alpha; dim cells keep the flag through scroll, rewrap and clear, where an
    overlay would have to follow them (PR-claude-639-a-change-to-the-grid-on-the-main-thread-carries-the-hooks-in-flight-001).
  - `crates/terminal_view/src/persistence.rs` keeps a terminal's folder and custom title only (410–456);
    `marley_workbench/src/terminal_ids.rs` (`known_ids` 53, `cleanup` 99) and `resume.rs`'s `init`
    show how a Marley store follows Zed's restore and cleanup by terminal id.
  - Files written 0600 today: `agent_activity.rs:275-282` (`append_in(dir, …)`),
    `marley_mcp/src/discovery.rs:39-50`; folders made 0700: `agent_reports.rs:143-148`.
  - Settings: `resume_agents` in `settings_content/src/marley.rs:31-36`, `default.json:1713`,
    `marley_page.rs:218-240`; the Terminal section at `marley_page.rs:592`.
- **Behavior maps:** T3 report 05 §2.1 (`Manager.ts:96-99` caps, `846-930` the ring, `1024-1066`
  the stripped sequences and why, `1775-1798` tail-only reads) and §3 item 1 (blocks from saved
  records, never from replayed bytes; mode 0600; a setting); Orca report 05 §2.2 (scrollback on
  disk, the cold restore's replay guard and dropped alternate screen) and §3 item 3.
- **Published material:** ECMA-48 and xterm's control sequences (which sequences are queries and
  which only draw); the Warp docs page above.

## UI proof
`script/e2e/751-restored-scrollback.sh`, `compositor sway`, a scratch repository, a HOME whose
`.bashrc` sets a plain prompt and puts the scenario's `bin` first. The runner's `--user-data-dir`
keeps the logs under the run's profile.
- In the terminal: `printf` of red and green lines; `seq 1 20000`; `echo before`; then `bin/ask`, a
  script that prints a cursor-position query (`ESC[6n`), a DA query (`ESC[c`) and a background
  colour query (`ESC]11;?BEL`) and reads its own answers away, so the live shell stays clean.
- A second terminal running `less README.md` at the quit.

Shots:
- `751-01-before`: the first terminal before the quit.
- `751-02-restored`: after a relaunch: the red and green lines and `before` dimmed above a new
  prompt at full brightness, no pill on the restored rows.
- `751-03-clean`: `echo clean` typed at the new prompt runs as typed and makes a block; nothing
  else at the prompt.
- `751-04-cap`: the restored terminal scrolled to its top: the first restored number is past
  15000.
- `751-05-fullscreen`: the second terminal: the lines before `less`, dimmed; no trace of `less`'s
  screen.
- `751-06-off`: `marley.restore_scrollback` set false between a quit and a launch: an empty
  terminal.
Checks (`expect`): the log is mode 600 in a folder of mode 700; closing the second terminal removes
its log; with the setting off every log is gone.

## Locked-In Decisions
- **D1 — Raw bytes through an allowlist, not a grid snapshot.** The bytes are cheap to keep as they
  come and reproduce colours and plain formatting; the allowlist removes every query and mode
  change at once. Rejected: a grid serializer (alacritty has none; a snapshot costs a lock of the
  grid on each save); T3's blocklist of queries alone (it keeps titles, notifications and the
  clipboard, which a replay would fire again).
- **D2 — Filtered twice:** when written and when read, so a file from an older build or edited by
  hand cannot answer into the new shell (T3 applies its replay protection twice).
- **D3 — Replayed in the builder before the event loop starts.** The builder already runs off the
  main thread and owns the order. Rejected: replaying from the view after it appears (the shell's
  first prompt can come first); a flag that drops answers during the replay (answers reach the PTY
  through an asynchronous channel after any flag is cleared).
- **D4 — Dim through SGR 2 on every restored cell.** Rejected: an overlay over the rows above a
  boundary line (it must follow clears, rewraps and scrolls, PR-claude-639).
- **D5 — No block from restored bytes:** frames never reach the file, and the replay's processor has
  no hooks (PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001). Blocks come
  back only from records Marley saved itself, in the second slice.
- **D6 — On by default, mode 0600, a setting that also deletes** (T3's open question 7 default).
  Turning it off removes every file at once, since the concern is what the file holds.
- **D7 — T3's caps,** 5,000 lines and 8 MiB, both enforced, so one long line cannot defeat them.
- **D8 — Keyed by `MARLEY_TERMINAL_ID`,** which a restored terminal keeps (#575); the file goes with
  the terminal, never at a quit.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley relaunches and restores a local terminal whose output was kept, the system shall show that output above the new shell's first prompt. | Shot 751-02 |
| REQ-002 | The restored output shall be drawn dim, and the new prompt and what follows it at full brightness. | Shots 751-02, 751-03 |
| REQ-003 | WHEN the earlier session printed terminal queries, the restored terminal's new prompt shall show nothing the queries would answer, and a typed command shall run as typed. | Shot 751-03 |
| REQ-004 | The system shall keep at most the newest 5,000 lines and 8 MiB of a terminal's output. | Shot 751-04 |
| REQ-005 | WHEN a full-screen program was running at the quit, the system shall restore the output before it and none of its screen. | Shot 751-05 |
| REQ-006 | Restored rows shall carry no block, pill or header; the first command after the restore shall make a block. | Shots 751-02, 751-03 |
| REQ-007 | The system shall create each kept file with mode 0600 in a folder of mode 0700. | The scenario's check |
| REQ-008 | WHEN the user closes a terminal, the system shall delete its kept output. | The scenario's check |
| REQ-009 | WHERE `marley.restore_scrollback` is false, the system shall keep no output, restore none, and delete every kept file. | Shot 751-06; the check |
| REQ-010 | The system shall keep no output of task, display-only or remote terminals. | Review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes, re-verified at promotion.
- **P2 Code** — the allowlist and the dim pass in `marley_dcs`; the ring, the caps and the file store
  (`*_in(dir)`) in `marley_terminal`; the recorder beside the notification scanner in
  `vendor/alacritty_terminal` (`marley_hooks.rs`, `event_loop.rs`, with `vendor/README.md`); the
  builder's replay and the recorder's hand-over in Zed's `terminal` crate (`terminal.rs`,
  `alacritty.rs`); the hook global, the deletions and the setting in `marley_workbench`; the three
  settings files; the guide; the scenario; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), touchpoint rows checked, ledger capture
  (§19), close the ticket, archive, commit.
