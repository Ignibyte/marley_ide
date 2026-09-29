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
- **Recall at promotion (2026-09-29):**
  - #551 shipped the neighbour: `command_watch.rs` (a watch per `TerminalView` keyed by entity id,
    `observe_in` on the terminal and `Event::TitleChanged`), `CommandSnapshot` and `RowLine` (a
    line's text, `state` and color) in the rail, `notifications::notify` and `mark_unread`
    crate-visible, and L-claude-551-an-agents-block-outlives-the-agent-001: test the block's own
    command with `agent_kind_of` as well as `agent_in`.
  - #565's layer as it stands: `system_one::ask(UseSpec, &Asking, cx) -> Task<Asked>`,
    `record` for a verdict the use settled (a `rules` row whatever the provider), `outcome`,
    `use_mode`, `project_of` and `project_name`; `Asking { subject, project, folders, local,
    facts, texts, verdict }`, each text masked by `state_for` (#516's rules and the key);
    `noul_verdict`; `Reading::Model(reads)` with `Signal::Noul { holds, probability }` past the
    band. The replay rows match on `set` and a `match` text the state holds, answer once unless
    `repeat`. #569's `stall.rs` is the pattern: rules first, `record`, the model for the open case.
  - `notify` keeps #538's five-second cooldown per project and the tag `marley-terminal-<id>`, so
    a recovery banner replaces the error's; the scenario spaces banners past the cooldown.
  - `links::SSH_CLIENTS` and `over_ssh` (#503's D4) are the SSH rule; `over_ssh` is private.
  - Brain (consultation f4c1b435): nothing on this seam.
- **Seams re-verified** (at `cc8a66509d`): `Terminal::block_output` (`terminal.rs:1929`) reads a
  block from `output_start` to `output_end` or the cursor through
  `alacritty::absolute_lines_text` (`alacritty.rs:975`); no reader starts at an arbitrary line,
  and no accessor gives the cursor's absolute line, which `HookPosition::of(&term)
  .absolute_line()` computes (`terminal.rs:2425`). `Event::Wakeup` is emitted on output
  (`terminal.rs:1842`). `agent_bar::agent_in`; `notifications.rs:110-201` (`looking_at`,
  `mark_unread`, `banner_allowed`, `notify`); the rail's `observe_global_in::<Attention>`
  (`rail.rs:537`), `terminal_match`, `render_terminal_row`'s `.children(mark).children(flag)`
  (`rail.rs:3602`), `thread_status_mark`'s `Close` in `Color::Error`; the Marley page's use
  dropdowns (`marley_page.rs:572-720`) and `default.json`'s `uses` block (`:1747`).
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
- **The shapes and the episode** (`marley_terminal::running_errors`, new, pure; `regex` sets
  compiled once in a `LazyLock`):
  - `scan(line) -> Option<Shape>`: `Failure` for `^\s*(error|ERROR|Error)(\[\w+\])?\s*:`,
    `error TS\d+:`, `^Traceback \(most recent call last\)`, `panicked at`,
    `\b(Unhandled|Uncaught)\b`, `\b(EADDRINUSE|EACCES|ECONNREFUSED|ENOENT|EPERM)\b`,
    `Failed to compile|Build failed|Compilation failed|could not compile|failed with exit code`,
    `^\s*(✖|✘|×|⨯|FAIL)\s`, `\[vite\] Internal server error`, `Segmentation fault`, `^fatal:`;
    `Recovery` for `[Cc]ompiled successfully`, `✓ (built|Compiled)`, `ready in \d+`,
    `^\s*Finished \S`, `webpack compiled`, `No issues found|Found 0 errors`,
    `Listening on|Server running|Local:\s+https?://`, `(hmr|HMR) update|page reload`; `Open` for a
    whole word `error`, `fail`, `failed`, `exception` or `fatal` (any case) on a line no failure
    shape matched. Failure is tried first, then recovery, then open.
  - `Episode`: `Quiet`, `Suspect { since, line, questioned }`, `Failed { line, questioned }`.
    `failure(line, questioned, now)` opens a suspect from quiet; `recovery(line)` closes a suspect
    silently and a failed one with `Change::Recovered { line, told }`; `due()` is when the
    suspect's grace (5 s) ends; `tick(now)` turns an elapsed suspect into `Failed` with
    `Change::Flag { line, questioned }`; `ended()` drops everything, with `Change::Cleared` from
    `Failed`; `mark()` is the failed line and whether it is questioned.
- **The Zed hunk** (`crates/terminal/src/terminal.rs`, beside `block_output`):
  `marley_lines_since(from) -> Option<(String, u64)>`: the main screen's lines from absolute line
  `from` up to the cursor's, and the cursor's absolute line; `None` on the alternate screen or once
  `from` has left the scrollback.
- **The watcher** (`marley_workbench::running_errors`, new, beside `command_watch.rs`): a global
  `RunningErrors` of `Watched { block, next_line, recent: VecDeque<(Option<Shape>)> (40),
  episode, call, scan: Option<Task<()>>, grace: Option<Task<()>> }` by the view's entity id,
  released with the view, observed by the rail. `init` observes each new `TerminalView`: on
  `Event::Wakeup` it schedules one scan 500 ms later unless one is pending; `observe_in` on the
  terminal checks the watched block's end. A scan runs only while `use_mode("running_error")` is
  not `off`, the last block is `Running`, `agent_in` is `None`, the block's command names no agent
  (`agent_kind_of`), and the foreground is not an SSH client (`links::ssh_in`, split out of
  `over_ssh`). It reads `marley_lines_since(next_line)` (the block's `output_start` at a new
  block), scans each complete line, feeds `Failure` and `Recovery` to the episode, and asks the
  layer about each `Open` line: `Asking { subject: the view, facts: command (first word), block
  age, failure and recovery lines in the last 40, texts: before, line, after (the next line of the
  same read) }`; the answer, in `act`, is a failure; in `suggest`, a questioned one; in `shadow`,
  logged only. A suspect schedules the grace timer at `due()`. On `Flag`: `record` a `rules` row
  (a shape) or keep the model's row (a reading), keep its id as `call`, `mark_unread`, and unless
  questioned `notify(view, "<project>: <command> printed an error", line)`. On `Recovered`:
  `record`, the outcome `recovered` for the call, and when `told`, `notify(.., "<project>:
  <command> recovered", line)`. The block's end clears the mark with the outcome `ended`; focus on
  the terminal while flagged logs `seen` once. `off` drops the watch's state.
- **The rail.** `marley_rail::TerminalSnapshot` and `TerminalRow` gain `running_error:
  Option<RunningError { line, questioned }>`; `terminal_snapshot` reads the global;
  `render_terminal_row` adds the line in `Color::Error` under #551's command line and, at the
  row's end, `IconName::Close` in `Color::Error` (with a `?` label when questioned) whose tooltip is
  the line; `terminal_match` falls back to the error line.
- **The use.** `marley_system_one::RUNNING_ERROR_SET` (`running_error/1`: nouls `new_failure` and
  `recovered`) and `RUNNING_ERROR` (`UseSpec { name: "running_error", deadline: 2 s }`);
  `default.json`'s `uses.running_error: "off"` and its comment; the Marley page's Running Error
  dropdown.
- **File manifest.** Marley: `crates/marley_terminal/src/running_errors.rs` (new),
  `marley_terminal.rs`; `crates/marley_workbench/src/running_errors.rs` (new),
  `marley_workbench.rs`, `links.rs` (`ssh_in`), `rail.rs`; `crates/marley_rail/src/marley_rail.rs`;
  `crates/marley_system_one/src/marley_system_one.rs`. Zed: `crates/terminal/src/terminal.rs`,
  `crates/settings_ui/src/marley_page.rs`, `assets/settings/default.json`. Script:
  `script/e2e/572-running-command-errors.sh`.
- **Ledger rows.** The `terminal.rs` row gains `marley_lines_since`; the `marley_page.rs` row the
  Running Error dropdown; the `default.json` row `uses.running_error`.

### E2E plan
The spec's `## UI proof` lists the steps and shots. Checks beside the shots: the private bus's log
by summary and body; the day's `calls-*.jsonl` rows for `"use":"running_error"` (the rules rows,
the replay rows with `line: worker 3: job exception`, the masked token, `before:` and `after:`);
the user's bus has no `Notify`. Banners are spaced past the five-second cooldown.

Not reachable by a scenario: a real Vite or cargo-watch (the stand-in prints their lines
verbatim); a banner's click (#478's path, unchanged); a remote project's terminal (no PTY here).

### Risks
- A line that wraps past the terminal's width is read as grid rows joined by the grid's own
  text; a read that ends inside a wrapped line reads its first rows now and the rest next time,
  and a shape split across the two is missed. Servers' error lines fit a row as a rule.
- A server that prints `error` in ordinary log lines is the open case by design; with `rules` it
  never flags.
- A recovery shape a server never prints leaves the mark until the block ends; the banner still
  fires once and the mark names the error.
- The layer must be on for the shapes to run (#565's REQ-001): a user who wants the banner with no
  model sets the provider to `rules`.

## Phase 2 — Code
- **Built to the manifest.** `marley_terminal::running_errors` (`scan`, `Shape`, `Episode`,
  `Change`, `GRACE`; the shapes as three `RegexSet`s compiled once, a pattern that does not
  compile logged and matching nothing). `Terminal::marley_lines_since` in `terminal.rs` (the
  ledger row first). `RUNNING_ERROR_SET` and `RUNNING_ERROR` in `marley_system_one`.
  `links::ssh_in`, split out of `over_ssh`. `marley_rail::RunningError` on the terminal's
  snapshot and row. The rail: the error line in `Color::Error` under #551's command line, the
  `Close` mark (a `?` beside it when questioned) with the line as its tooltip, the filter falling
  back to the line, and a refresh on the `ErrorMarks` global. `marley_workbench::running_errors`
  (the watcher). `default.json`'s `uses.running_error: "off"` and the Marley page's Running Error
  dropdown.
- **Deviations.** Two globals, not one: the watch's state changes at every read, so the rail
  observes `ErrorMarks`, touched only when a mark changes (links.rs's note on `default_global`
  says why). D8 as the promotion rewrote it: the first of `seen`, `recovered` or `ended` is the
  flag's call's outcome, with no timer. The banner's unread mark and the banner itself are skipped
  for a questioned flag and while the terminal is in front. An open line at the start or the end
  of a read sends no empty `before` or `after`. `SwitcherRow::Terminal` and
  `SwitcherEntry::Terminal` hold a `Box<TerminalRow>`: the new field put the row past clippy's
  `large_enum_variant` threshold.
- **Review.** Re-entrancy: every read, grace timer and answer runs in `update_in` on the view and
  reads the terminal, a different entity; the observed global is written only through
  `set_mark`. The block's end is checked on each terminal notify through `try_global` (no write),
  so a quiet check costs a lookup. Clippy's reds fixed at the source: `too_many_lines` on
  `Rail::new` (the observers moved into `observe_marks`) and on `render_terminal_row` (the marks
  into `terminal_marks`), `needless_pass_by_value` (a `by_reading` flag), `needless_pass_by_ref_mut`
  on the window, two redundant closures.
- **Gate.** Run 1 red on gate:14 only: the module's public doc linked the private `SCAN_DELAY`
  (`rustdoc::private_intra_doc_links`); the doc now says 500 ms. Run 2: `GATE GREEN [diff]`, 16
  passed, 0 failed, the receipt written.

## Phase 3 — Test
- **Scenario** `script/e2e/572-running-command-errors.sh` under `compositor sway`, on a private
  bus whose server logs `app|summary|body`, #565's layer on `replay` with the scratch repository
  listed. Run 1 passed every check. The empty `after:` in the band's call led to the change above;
  run 2 then failed only the user-bus check: another program on the desktop posted a `Notify`
  during the run, and the check counted any sender. It now counts Marley's name only, as #535's
  does (L-claude-572 below). Run 3 passed all 17 checks.
- **Every shot read** (run 3; rail crops scaled 2.5x):
  - `572-01-error-flag`: terminal 1's row reads `devserver · running`, then `error: Failed to
    compile…` in red, the `×` mark and the unread dot; terminal 2 in front has no line. Log:
    `Marley|repo: devserver printed an error|error: Failed to compile ./src/App.tsx`; the day's
    file has the `rules` row with `new_failure`. REQ-001, REQ-002.
  - `572-02-recovered`: the mark and the red line gone, the dot kept until seen. Log: `repo:
    devserver recovered|Compiled successfully.` REQ-003.
  - `572-03-suggest`: `× ?` and `worker 3: job exception…`; no banner. The replay row, mode
    `suggest`, holds `before: auth: signed in with [redacted: github token]`, `line: worker 3: job
    exception, retrying`, `after: worker 3: next try in 5 s`, and no trace of the token. REQ-007.
  - `572-04-open-case`: `act`: the `×` without `?` and the open line; the banner names it. REQ-008.
  - `572-05-no-signal`: after a recovery, `cache: fail count reset` (0.50, in the band): no mark,
    no banner; its row logged, with no `after:`. REQ-009.
  - `572-05b-agent-and-ssh`: terminal 2 ran the `claude` and `ssh` stand-ins, each printing
    `error: boom` and running seven seconds: no mark on its row (it reads #551's `ssh · done · 7
    s`), no banner. REQ-005.
  - `572-06-focused`: terminal 1 in front: its row marked with the error line, no dot, no banner.
    REQ-006.
  - `572-07-exit-not-ours`: terminal 2's `sh -c 'echo error: bad; sleep 1; exit 1'`: #551's red
    `exit 1 · 1 s` and no mark; no banner. REQ-004.
  - `572-08-off`: `off`, terminal 2 in front, `SIGUSR1`: no mark, no banner, no new row. REQ-010.
  - `572-09-setting`: the settings search for `Running Error` shows it under Marley › System One,
    at `Off`, with its description. REQ-011.
- **Not reachable:** a real Vite or cargo-watch (the stand-in prints their lines); a banner's
  click (#478's path, unchanged); a remote project's terminal.

## Phase 4 — Complete
- **Docs.** CHANGELOG Added; the plan's T7 row; `marley_workbench.md` (a section for
  `running_errors.rs`), `marley_rail.md` (`TerminalSnapshot::running_error`, the boxed switcher
  row), `terminal_blocks.md` (`running_errors`, `marley_lines_since`), `marley_system_one.md`
  (the set and its consumer); the touchpoint rows for `terminal.rs`, `marley_page.rs` and
  `default.json` describe what shipped.
- **Knowledge.** AD-claude-572-a-running-commands-error-by-shapes-then-the-model-001,
  L-claude-572-a-user-bus-check-names-marley-001,
  L-claude-572-an-observed-global-written-at-every-wakeup-redraws-the-rail-001. No F: nothing
  the review or the visual check found was a product bug. Brain: `brain_decide` on consultation
  f4c1b435.
- **Closed** the ticket, archived the pair, committed.
