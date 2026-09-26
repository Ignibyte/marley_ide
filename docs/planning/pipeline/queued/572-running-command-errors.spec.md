---
pipeline_id: 739b7853-05b6-489b-b0ca-1478f9c4db2a
ticket: docs/planning/tickets/open/TICKET-572-running-command-errors.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "A running command's error: a notification when a dev server prints an error and keeps running"
type: feature
slice: prong 1 T7b's follow-on (after #478 and #551, the long command's end); use 7 of the System One layer, on #565
references: [docs/planning/design-notes/jev-system-one-2026-09-25.md, docs/planning/design-notes/warp-second-pass-2026-09-25.md, docs/planning/pipeline/queued/565-system-one-layer.spec.md, docs/planning/pipeline/queued/551-command-end-from-outside.spec.md, docs/planning/pipeline/completed/478-terminal-notifications.spec.md, docs/planning/pipeline/queued/538-notifications-with-content.spec.md, docs/planning/pipeline/queued/503-terminal-urls-open-in-the-browser.spec.md, docs/marley/three-prong-plan.md]
---

## Title
A plain terminal's running block is watched for new lines. The error shapes any build tool prints
open a failure episode when the block is still running five seconds after the line; the recovery
shapes close it. An episode posts one desktop notification titled with the command when the
terminal is not in front, a second when it recovers, and the rail row carries a red mark with the
first error line until then. A System One question goes through #565's layer only for lines the
shapes leave open, and may mark, never more. It pairs with #551: #551 says a command ended; this
says a command that did not end failed.

## Scope
### In
- **`marley_terminal::running_errors`** (new, pure): `scan(lines) -> Vec<Signal>`, one of
  `Failure(line)`, `Recovery(line)` or `Open(line)`, by the shapes in the notes; `Episode`, the
  state machine (`Quiet`, `Suspect { since, line }`, `Failed { line, at }`) that turns signals and
  the block's state into `Flag`, `Recovered` or nothing, with the five-second grace of D3.
- **`marley_workbench::running_errors`** (new, beside #551's `command_watch`): for each local
  terminal whose newest block is `Running`, whose foreground is not an agent CLI
  (`agent_bar::agent_in`) and not an SSH client (#503's D4 list), on its output at most every
  500 ms (as #503's D6 reads the grid): the block's lines since the last read, scanned; an `Open`
  line asked through `system_one::ask` with the set `running_error/1` (nouls `new_failure` and
  `recovered`), the use's own verdict handed with it, and the state of D7; the episode per
  terminal; the outcome line for each call (D8).
- **The notification**, through `notifications::notify` (crate-visible since #551; the focus
  gate of #478, the click that shows the terminal): `<project>: <command> printed an error` with
  the first error line as printed, cut to 179 characters on a character boundary with `…`
  (#538's cut); on recovery `<project>: <command> recovered` with the recovery line; one of each
  per episode.
- **The rail**: under #551's command line (`devserver · running`), the first error line in the
  error color with a red mark at the row's end (`IconName::Close`, `Color::Error`, as a failed
  thread's), until recovery or the block's end; the filter matches it.
- **Settings**: the use registered with #565 (`UseSpec { name: "running_error", deadline: 2 s }`),
  its mode in `marley.system_one.uses` (`off` by default), its dropdown on the Marley page's
  System One section.
- `script/e2e/572-running-command-errors.sh`.

### Out (explicitly deferred)
- #551's own scope: the notification when a command ends after the threshold, the running command
  on the row's line, the last command's result with its mark, the password wait.
- Agent terminals (#538's banners) and SSH terminals (#526's blocks; #503's D4 rule).
- Jumping to the error's `file:line:col` at the block's cwd (plan T2); filtering the block (#528).
- A "quiet server" signal (no output for N minutes): a server at rest is quiet by right.
- Zed tasks' `TaskStatus` (a task ends and reports its exit code; nothing here reads tasks).
- User shape lists; the words of the banner beyond the block's own lines.

## Reference (§20)
- **Warp:** desktop notifications when a command finishes after `long_running_threshold` or waits
  for a password, only while Warp is not in front (docs.warp.dev/terminal/more-features/notifications/),
  and a tab-bar mark for a command that exited with an error
  (docs.warp.dev/terminal/appearance/tabs-behavior/), as `warp-second-pass-2026-09-25.md`'s
  finding 3 records them and #551 takes them. Warp notifies on an end; nothing in its docs
  notifies while a command runs, and the finding's hard part names this ticket's gap: "A dev
  server never finishes, so its useful signal is a failure printed while it runs, which takes
  judgment (use 7)". Marley keeps Warp's gate (not in front) and its mark, and adds the running
  case. Nothing of Warp's was read.
- **Upstream Zed:** `TaskStatus` and the `finished with exit code: N` summary line for a spawned
  task (`crates/terminal/src/terminal.rs:1686`, `:3426`), kept as they are: Zed judges a task by
  its exit code, which a running server never gives. Zed's `hyperlinks.rs` finds `panicked at`
  paths for clicking (`crates/terminal/src/alacritty/hyperlinks.rs:895`), not for status.
- **Orca:** none. Report 05's terminal survey watches output for advertised URLs (#503), not for
  failures.

### Prior art
- **Behavior maps and reports.** The Warp second pass, finding 3 ("Marley today: blocks carry the
  command, exit code, start and end, and only the terminal's drawing and the MCP tools read them";
  "agent terminals keep their own banners (#538) and skip this one"); #551's D1 (blocks are the
  source), D3 (never for agent terminals), D4 (#478's rule for "in front") and its Out ("A failure
  printed by a command that keeps running (a dev server): use 7"); #503's D4 and D6 for the SSH
  rule and the grid read. The Jev note's use 7 ("nouls for a new failure and for recovered; a
  notification when a dev server prints an error and keeps running"), its rule 6 (uses that read
  program output "only display, rank and route") and its rule 7 (masked, listed projects only).
  `docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md` §2 (why Zed has no blocks) and
  §8 (the failed-block wedge). #565's layer (`UseSpec`, `ask`, `StateBuilder`, `Reading`, the
  providers, the day's file).
- **Published material.** The shapes as their tools print them: rustc's `error[E0308]:` and
  `error:`, cargo's `Finished` and `error: could not compile`, Python's `Traceback (most recent
  call last):`, Node's `Error:` with `    at fn (file:line:col)` frames and `EADDRINUSE`, Vite's
  `ready in N ms`, `[vite] Internal server error` and `✓ built in`, webpack's `compiled
  successfully` and `Failed to compile`, Next's `⨯` and `✓ Compiled`, tsc's `error TS2322:` and
  `Found 0 errors`. Zed's terminal already knows the `panicked at` form.
- **The code we already ship.** `Terminal::blocks()` (`crates/terminal/src/terminal.rs:1845`),
  `block_output` (`:1859`, `absolute_lines_text(term, output_start, output_end)`, with no end for a
  running block), `block_output_kept` (`:1869`), `apply_shell_hook` and `stamp` (`:1817-1841`),
  `last_n_non_empty_lines` (`:2635`), `foreground_process_command_name` (`:3082`);
  `AnchoredBlock` (`crates/marley_terminal/src/anchored.rs:38-58`: `state`, `output_start`,
  `output_end: Option`), `BlockTimes` (`:63-68`), `BlockState::Running` (`block.rs:29-38`);
  `notifications::notify` (`crates/marley_workbench/src/notifications.rs:59-82`), its gate (`:66`),
  `show_sender` (`:86-100`), `Event::MarleyNotification` emitted at `terminal.rs:1806`; #551's
  queued `command_watch.rs` (an observer of each `Terminal` entity comparing blocks at each
  notify), its row line and `duration_label`; the rail's `note_output` (`rail.rs:426`) and
  `terminal_output`, the last output's instant per view; `agent_bar::agent_in`
  (`agent_bar.rs:129-133`); `thread_status_mark`'s error mark (`rail.rs:2050-2054`); the `regex`
  crate; `mcp::agent_redactor` (`mcp.rs:297`); `marley_terminal::apply`'s own fixture text
  `error: boom` (`apply.rs:632`). Does a crate we build own the seam? Nothing scans output for a
  status; Zed's `task` crate judges by exit code, `marley_terminal` owns the block whose lines
  are read, #551 the watch and the row line, #565 the ask.

## UI proof
UI-AFFECTING: a desktop notification, a mark and a line on the rail's terminal rows.
`script/e2e/572-running-command-errors.sh` (`compositor sway`: the focused case needs Marley's
window active). Fixtures: a fake `devserver` first on the PATH (Python: prints `ready in 120 ms`,
then on `SIGUSR1` three lines starting `error: Failed to compile ./src/App.tsx`, on `SIGUSR2`
`Compiled successfully.`, on `SIGRTMIN` `GET /api/errors 200 12ms` and a bare
`    at Object.<anonymous> (server.js:12:5)`, and keeps running); a fake `claude` that prints
`error: boom` and sleeps; a second terminal to hold the focus; `busctl --user monitor
org.freedesktop.Notifications` into a log (L-claude-478); #565's layer enabled on `replay` with
the scratch repository listed and `uses.running_error` rewritten per step by
`system_one_setting`; the replay file at `$E2E_PROFILE/system_one/replay.jsonl` (the bare frame:
`new_failure` 0.90; a second such frame later: 0.5). Shots:
- `572-01-error-flag`: the mode `shadow`; `devserver` in terminal A, the focus in B, `SIGUSR1`,
  6 s: A's row with the red mark and `error: Failed to compile ./src/App.tsx`; the bus log's
  `Notify` titled `repo: devserver printed an error` with that body.
- `572-02-recovered`: `SIGUSR2`: the mark gone; the `Notify` titled `repo: devserver recovered`.
- `572-03-open-case`: the mode `act`, `SIGRTMIN`, 6 s: the mark and the frame line; a `Notify`.
- `572-04-no-signal`: `SIGUSR2`, then `SIGRTMIN` again (the replay's 0.5): no mark, no `Notify`;
  the day's file has the row.
- `572-05-agent-terminal`: `claude` in terminal C prints `error: boom`: no mark on C, no `Notify`.
- `572-06-focused`: the focus in A, `SIGUSR1`: the mark, and no `Notify` in the log.
- `572-07-exit-not-ours`: in B, `sh -c 'echo error: bad; sleep 1; exit 1'`: no mark from this
  ticket and no `Notify` (the block ended inside the grace).
- `572-08-off`: the mode `off`, `SIGUSR1` again: nothing.
Checks: the bus log's `Notify` calls by title and body; the day's file's rows name
`running_error/1` and carry two redacted lines around the open line and no others; in `572-03`
the Decisions view lists the call.

## Locked-In Decisions
- D1 — "local first and then jev second" (Chad, 2026-09-26). The shapes decide; only an open
  line is asked, and a reading may open or close an episode's mark, nothing else. With the
  `rules` provider, the project unlisted (`Refused`), the provider unreachable (`Unavailable`) or
  the reading `NoSignal`, the shapes alone run, which is the whole feature for most servers.
- D2 — Off by default, with its own switch and its own mode, and no provider named: "we need
  probably every aspect of this configurable and turned off / on where the system will use or
  wont use it. Otherwise this becomes a jev required system" (Chad). The switch is the use's mode
  in `marley.system_one.uses` (#565's shape); `rules` runs the shapes with no model at all.
- D3 — "Keeps running" is the trigger: a failure line in a block still `Running` five seconds
  later. A block that ends inside the grace is #551's (its exit notification names the failure);
  the two never post for one event.
- D4 — Plain terminals only: an agent CLI in the foreground (#538's banners) or an SSH client
  (#503's D4: that output is another machine's) is never watched, as #551's D3 leaves them out.
- D5 — One banner per episode and one on recovery; the mark stays until recovery or the block's
  end; #478's gate (not in front, as #551's D4 reads it) and #538's per-project cooldown when it
  exists. A new episode needs a recovery or a new block between.
- D6 — The words are the block's own lines: the first failure line as printed, cut at 180 (#538's
  rule); the banner never carries the model's reading, and the row's mark carries a `?` when it
  came from a reading in `suggest`. What Chad sees stays exact (AD-claude-516).
- D7 — The state is the open line with one line before and one after, through #516's redactor as
  the layer's `Mask`, and facts (the shapes matched in the last 40 lines, their counts, the
  command's first word, the block's age); only for a listed project; `Detail::Facts` for a
  metadata-only project. No other output.
- D8 — The outcome line: within ten minutes of a flag, whether the terminal took the focus, the
  block ended, or a recovery came; a flag no one looked at and nothing recovered from is the
  likely false positive the golden report counts.
- D9 — The scan reads the grid, not the byte stream (#503's D6): the block's lines since the last
  read, at most every 500 ms per terminal and once more when the output goes quiet. `block_output`
  reads a running block's whole output; if that proves too coarse, a Marley hunk beside it in
  `crates/terminal/src/terminal.rs` reads from an absolute line to the end (its row exists);
  promotion decides.
- D10 — Modes as #565 defines them: `off` watches nothing; `shadow` runs the shapes and logs the
  model; `suggest` marks a reading's failure with `?` and posts no banner for it; `act` treats a
  reading as a shape.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE the use is not `off`, WHEN a plain terminal's running block prints a line of a failure shape and is still running five seconds later, and the terminal is not in front, the system shall post one desktop notification titled with the project and the command whose body is that line. | Shot `572-01-error-flag`; the bus log |
| REQ-002 | WHILE an episode is open, the terminal's rail row shall show a red mark and the first error line. | Shot `572-01-error-flag` |
| REQ-003 | WHEN the block prints a line of a recovery shape, the system shall clear the mark and post one notification saying the command recovered. | Shot `572-02-recovered`; the bus log |
| REQ-004 | WHEN the block ends within five seconds of a failure line, the system shall post nothing and mark nothing. | Shot `572-07-exit-not-ours`; the bus log |
| REQ-005 | WHERE the terminal's foreground is an agent CLI or an SSH client, the system shall watch nothing in it. | Shot `572-05-agent-terminal`; the bus log |
| REQ-006 | WHILE the terminal is in front, the system shall show the mark and post no banner. | Shot `572-06-focused`; the bus log |
| REQ-007 | WHERE the project is listed and the mode is not `off`, WHEN a line the shapes leave open prints, the system shall ask the `running_error` set once with two redacted lines and the facts. | The day's file's row |
| REQ-008 | WHERE the mode is `act`, WHEN `new_failure` reads at or above its threshold, the system shall open the episode as for a shape. | Shot `572-03-open-case`; the bus log |
| REQ-009 | WHEN the reading is in the band, no signal, refused or unavailable, the system shall change nothing. | Shot `572-04-no-signal`; the day's file |
| REQ-010 | WHERE the mode is `off`, the system shall post nothing, mark nothing and make no call. | Shot `572-08-off`; the bus log; the day's file |
| REQ-011 | WHEN the Marley settings page opens, its System One section shall show the use's mode. | The 515 scenario's page shot |
| REQ-012 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec; the design and the e2e plan in the notes. At promotion: #565 and #551
  have shipped (the ask API, the replay file, `command_watch`, the row line, `notify` made
  crate-visible), and whether #538's private bus fixture exists to replace the `busctl` monitor;
  `brain_ask`.
- **P2 Code:** the ledger row first (`crates/settings_ui/src/marley_page.rs`, the use's dropdown;
  the `crates/terminal/src/terminal.rs` row only if D9's hunk is taken); `running_errors` in
  `marley_terminal` and in the workbench; the `running_error/1` set in
  `marley_system_one::question`; the notification, the row and the filter. fmt and clippy clean;
  a review of the diff.
- **P3 Test:** write and run the scenario and read every shot; rerun #551's scenario (its rows
  and notifications must hold with this use off); `just regress` (484 and 544 sit in the set and
  shoot the rail); `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/terminal_blocks.md`,
  `marley_workbench.md` and `marley_system_one.md`; the plan's T7 row; ledger capture; close the
  ticket, archive, commit.
