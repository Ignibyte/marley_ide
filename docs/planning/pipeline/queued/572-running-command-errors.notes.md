# A running command's error: a notification when a dev server prints an error and keeps running — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-572-running-command-errors.md
- **Pipeline spec:** 572-running-command-errors.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-26, approving the seven ranked uses of the Jev note
  (`docs/planning/design-notes/jev-system-one-2026-09-25.md`). Use 7: "nouls for a new failure
  and for recovered; a notification when a dev server prints an error and keeps running; waits on
  the Warp second pass, finding 3; S–M". The finding (`warp-second-pass-2026-09-25.md`, "A
  command's end, seen from outside its terminal") became #551 for the end and this ticket for the
  failure while running; #551's Out names it. Drafted by the second spec drafter beside #565 and
  #566 to #568, and aligned to #551's and #565's pairs once they were on disk.
- **Classification / tier:** feature, prong 1 (T7b's follow-on). Marley crates, plus the page's
  dropdown for the use's mode, and possibly one hunk in `terminal.rs` (D9). Size S to M.
- **Recall (§18.3):**
  - AD-claude-478 (a notification only for a terminal not in front; a click shows the terminal):
    the same route, the same gate; #551 makes `notify` crate-visible for its own banners.
  - L-claude-478 (`busctl --user monitor` sees a `Notify` and the test closes it): the proof.
  - AD-claude-516 (what Chad sees stays exact; redaction is for what leaves for a model): the
    banner shows the line as printed; only the state's lines are redacted.
  - PR-claude-redact-the-whole-text-before-cutting-it-001: the three lines are redacted whole,
    then cut.
  - L-claude-477 (a quiet foreground process is seen only after output): the agent-CLI check
    reads the foreground after output, which is when this watcher runs anyway.
  - #503's queued D6 (the grid, at most every 500 ms) and D4 (the SSH list); #551's D1 (blocks
    are the source), D3 (never for agent terminals) and D7 (the row's command line); #565's D1
    and D5.
  - Brain: not consulted in this drafting session; promotion asks.
- **Discovery** (at `ca70b6488d`; promotion re-verifies):
  - `crates/terminal/src/terminal.rs`: `apply_shell_hook` (1817-1841, `stamp` at 1834),
    `blocks` (1845), `marley_anchored` (1852), `block_output` (1859-1865),
    `block_output_kept` (1869-1872), `last_n_non_empty_lines` (2635), `foreground_process_command_name`
    (3082), `TaskStatus` (1686), the task's summary line (3426).
  - `crates/marley_terminal/src/anchored.rs`: `AnchoredBlock` (38-58), `BlockTimes` (63-68),
    `AnchoredBlocks::blocks` (143), `stamp` (149-166), `times` (170), `finish_running` (209-219);
    `block.rs`: `BlockState` (29-38), `ExitCode` (41).
  - `crates/marley_workbench/src/notifications.rs`: `init` (30-55), `notify` (59-82), the gate
    (66), the tag (69), `show_sender` (86-100).
  - `crates/marley_workbench/src/rail.rs`: `note_output` (426-438), `terminal_output` (the map),
    `terminal_snapshot` (1717-1784, the plain terminal's title and subtitle at 1749-1759),
    `render_terminal_row` (1319), `row_card` (1982-2024), `thread_status_mark` (2035-2060).
  - `crates/marley_workbench/src/agent_bar.rs`: `agent_in` (129-133).
  - `crates/marley_workbench/src/mcp.rs`: `agent_redactor` (297-303).
  - #551 (queued): `command_watch.rs` (an observer of each `Terminal` entity comparing blocks at
    each notify), `TerminalSnapshot::command` and its row line (`cargo build · running`),
    `duration_label`, `marley.long_command_seconds`, `notify` made `pub(crate)`.
  - #565 (queued): `UseSpec`, `system_one::ask`, `StateBuilder`, `Detail`, `Reading`, the day's
    file, the replay file and `system_one_setting`; `marley.system_one.uses`.
  - `crates/settings_ui/src/marley_page.rs` (6-15, 42-91).
  - `script/e2e/519-claude-code-events-in-the-rail.sh` (the `busctl` monitor into a log;
    L-claude-478) and `script/e2e/547-claude-code-events-slice-2.sh` (`marley_setting`).
- **Decisions:** D1 to D10 in the spec.

### Design
- **The shapes** (`marley_terminal::running_errors`, `regex` sets compiled once):
  - Failure: `^\s*(error|ERROR|Error)\b\s*[:\[]`, `error\[E\d+\]`, `^Traceback \(most recent
    call last\)`, `panicked at`, `\b(Unhandled|Uncaught)\b`, the errno names `EADDRINUSE`,
    `EACCES`, `ECONNREFUSED`, `ENOENT` and `EPERM` as whole words, `Failed to compile|Build
    failed|Compilation failed|could not compile|failed with exit code`,
    `^\s*(✖|✘|×|⨯|FAIL)\b`, `\[vite\] Internal server error`, `Segmentation fault`, `^fatal:`,
    `error TS\d+:`.
  - Recovery: `(C|c)ompiled successfully`, `✓ (built|Compiled)`, `ready in \d+`, `^\s*Finished
    \S`, `webpack compiled`, `No issues found|Found 0 errors`, `Listening on|Server running|
    Local:\s+https?://`, `(hmr|HMR) update|page reload`.
  - Open: `\b(error|fail|failed|exception)\b` inside a line that matched no failure shape,
    `^\s+at .+\(.+:\d+:\d+\)` with no failure shape in the last 40 lines, `^\s*\d+ (error|
    warning)s?\b`, a line of `^\s*[=-]{3,}\s*(error|fail)` banners.
- **The episode.** `Episode::observe(signal, block_running, now) -> Option<Change>`:
  `Quiet` + `Failure` → `Suspect { since: now, line }`; `Suspect` after 5 s with the block still
  running → `Failed { line, at }` and `Change::Flag(line)`; `Suspect` with the block ended →
  `Quiet` (#551's); `Failed` + `Recovery` → `Quiet` and `Change::Recovered(line)`; `Failed` + the
  block's end → `Quiet` silently (the row's mark leaves with the block); `Open` is handed to the
  layer and its reading, in `act`, re-enters as `Failure` or `Recovery`; in `suggest` a
  `new_failure` reading marks with `?` and posts nothing; in `shadow` it is logged.
- **The watcher.** `running_errors::init` subscribes each `TerminalView`'s terminal as
  `notifications::init` does (or rides #551's `command_watch` observer if it has landed), and on
  `Event::Wakeup` schedules a scan for the view at most every 500 ms (a `Task` per view, as
  `rail.rs`'s quiet timers), plus one after 600 ms of quiet. A scan: skip unless
  `blocks().last()` is `Running`, `agent_in` is `None` and the foreground is not in #503's SSH
  list; read the block's output (`block_output`, or D9's hunk from the last absolute line read);
  split into lines; `scan` the new ones; feed the episode; on a `Flag`, set the row's mark (a
  global map by view, as #538's marks) and, unless the view is the focused terminal of the
  active window, `notify`; on `Recovered`, clear and `notify`.
- **The ask.** An `Open` line: `StateBuilder::new(project, detail)` with facts `command` (its
  first word), `block age`, `shapes in the last 40 lines` (names and counts) and texts `before`,
  `line`, `after` (each through the layer's `Mask`, dropped under `Detail::Facts`); the use's
  verdict (`Open`) with it; `UseSpec { name: "running_error", deadline: 2 s }`; one call per
  open line, deduped by the layer's hash; a reading at or above the threshold (0.65 to start)
  acts as a shape in `act`.
- **The row.** `TerminalSnapshot` and `TerminalRow` gain `error: Option<String>` (the first
  line); `terminal_snapshot` reads the map; `render_terminal_row` draws the error mark at the
  row's end and the line in `Color::Error` through `row_card`'s lines, under #551's command line
  (or in its place when #551 has not landed).
- **File manifest.** Marley: `crates/marley_terminal/src/running_errors.rs` (new),
  `marley_terminal.rs` (the module), `crates/marley_terminal/Cargo.toml` (`regex`);
  `crates/marley_workbench/src/running_errors.rs` (new), `marley_workbench.rs` (the module,
  `init`), `notifications.rs` (`notify` crate-visible, if #551 has not made it so), `rail.rs`;
  `crates/marley_rail/src/marley_rail.rs`; `crates/marley_system_one/src/question.rs` (the
  `running_error/1` set) and the workbench's `system_one.rs` (the `UseSpec`);
  `script/e2e/572-running-command-errors.sh`. Zed: `crates/settings_ui/src/marley_page.rs`
  (the use's dropdown), and `crates/terminal/src/terminal.rs` only if D9's hunk is taken.
- **Ledger rows.** The `marley_page.rs` row gains the `running_error` dropdown (#572) before its
  edit; the `terminal.rs` row gains the lines-since hunk if it is taken.

### E2E plan
Fixtures: a scratch repository; a HOME whose `.bashrc` puts `$E2E_WORK/bin` first on the PATH;
`$E2E_WORK/bin/devserver` (Python, signal-driven as the spec says, its pid in a file);
`$E2E_WORK/bin/claude` (prints `error: boom`, sleeps); `busctl --user monitor
org.freedesktop.Notifications` into a log, its `Notify` entries closed by the scenario; the
profile's settings: #565's block enabled on `replay` with the repository listed and
`uses.running_error` rewritten per step by `system_one_setting`;
`$E2E_PROFILE/system_one/replay.jsonl` written by setup (matched on the frame line:
`new_failure` 0.90 for the first, 0.5 for a second occurrence through `repeat`). Two terminals A
and B from the rail's + (and C for the agent case); the focus moved by clicking rows.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-002 | mode `shadow`; in A `devserver`; click B's row; `kill -USR1`; settle 6 | `572-01-error-flag`; the bus log: `Notify` with the title and body |
| REQ-003 | `kill -USR2`; settle 2 | `572-02-recovered`; the bus log |
| REQ-007, REQ-008 | mode `act`; `kill -RTMIN`; settle 6 | `572-03-open-case`; the day's file's row; the bus log |
| REQ-009 | `kill -USR2`; `kill -RTMIN` (the replay's 0.5); settle 6 | `572-04-no-signal`; the day's file; the bus log unchanged |
| REQ-005 | in C `claude`; settle 6 | `572-05-agent-terminal`; the bus log unchanged |
| REQ-006 | click A's row; `kill -USR1`; settle 6 | `572-06-focused`; the bus log unchanged |
| REQ-004 | in B `sh -c 'echo error: bad; sleep 1; exit 1'`; settle 6 | `572-07-exit-not-ours`; the bus log unchanged |
| REQ-010 | mode `off`; `kill -USR2`, `kill -USR1`; settle 6 | `572-08-off`; the logs |
| REQ-011 | the golden set's 515 run | its page shot |

Not reachable by a scenario: a real Vite or cargo-watch (the stand-in prints their lines
verbatim), and #538's private bus before it lands (the monitor stands in, as #519 used it).

### Risks
- A chatty server (thousands of lines a second) makes `block_output` read its whole output every
  500 ms; D9's hunk reads from the last absolute line instead, and promotion measures the
  difference on the box before choosing.
- A server that prints `error` in ordinary log lines (`GET /errors 200`) is the open case by
  design; with the `rules` provider it never flags, which is the safe side.
- A recovery shape that a server never prints (`cargo watch` prints `Finished`; a bare `node
  server.js` prints nothing) leaves the mark until the block ends; the notification still fires
  once, and the mark's line names the error.
- Two notification tickets (#551, #538) touch `notifications.rs` beside this one; the
  crate-visible `notify` is the one seam they share, and whichever lands first adds it.
- The layer must be enabled for the shapes to run at all (#565's REQ-001); a user who wants the
  notification with no model sets the provider to `rules`.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
