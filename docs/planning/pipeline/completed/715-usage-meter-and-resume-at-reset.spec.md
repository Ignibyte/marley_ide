---
pipeline_id: 96dcb201-0928-4d7c-bfd0-6793f94d533a
ticket: docs/planning/tickets/open/TICKET-715-usage-meter-and-resume-at-reset.md
status: Phase 4 — Complete PASS
title: A usage meter, and resume at the reset
type: feature
slice: prong 2; T3 Code survey items 2 and 12, MonoCode finding 2; first of two slices (Codex's limit, both agents' meter)
references:
  - docs/t3code_architecture/README.md
  - docs/t3code_architecture/01-agents-and-providers.md
  - docs/planning/intake/monocode-findings.md
  - docs/planning/pipeline/completed/640-agent-state-source-progress-quota.spec.md
  - docs/planning/pipeline/completed/650-codex-app-server-state.spec.md
  - docs/planning/pipeline/completed/651-codex-approvals-and-prompts.spec.md
  - docs/planning/pipeline/completed/709-load-the-shared-claude-code-plugin.spec.md
  - docs/planning/design-notes/tabs-everywhere-2026-10-11.md
  - docs/planning/pipeline/queued/752-codex-push-and-answers.spec.md
---

## Title
A usage limit becomes a state, with a way to continue, and each agent row carries a usage meter.
A Codex on its own App Server (#650) that hits its limit reads `limited · resets in 1 h 35 m` on
its row and in Needs you instead of `failed`, with Continue at Reset, which sends "Continue where
you left off." through the App Server once the window reopens; a setting, off by default, arms it
for every limit. Claude Code and Codex rows show their account's most-used window and its reset on
a line, every window in the tooltip, as #640's harness rows do. Chad, 2026-10-11: "2.) yes".
This is the first of two slices; the second gives Claude Code the limited state and Continue at
Reset (Out).

## Scope
### In
- **Codex's windows.** The follow reads `account/rateLimits/updated`, which Codex's App Server
  sends with its responses and before a usage-limit failure, for the `codex` bucket (a snapshot
  with no `limitId` or `limitId` `codex`) and passes over any other bucket. Each update is sparse:
  a window it carries replaces the last, keeping a duration or reset the update leaves out. Each
  window lands on the Codex seat as `quota.KIND.percent_used` and `quota.KIND.resets_at_ms` (Codex
  sends `resetsAt` in seconds), its kind `five_hour` or `seven_day` by its duration, otherwise
  `primary` or `secondary`, the names rustal-harness gives the same windows. Marley sends no
  `account/rateLimits/read`.
- **Claude Code's windows**, as they reach the seat today: the shared plugin (#709, while
  `marley.claude_code_shared_plugin` is on) reports `$.session.usage().rateLimits` at each main
  turn's end, and `apply_report` puts them on the seat as the same labels.
- **The meter.** A Claude Code or Codex row whose seat carries quota windows gets #640's quota
  line, `five_hour 62 % · resets in 1 h 35 m`, the most-used window, and #640's tooltip, every
  window most used first with its reset. One drawing for harness, Claude Code and Codex rows, and
  one reading: a percent above 100 reads as 100, a full window, for harness rows too.
- **Limited.** A Codex turn that completes `failed` with `codexErrorInfo` `usageLimitExceeded` or
  `rateLimitExceeded` makes the seat limited. Its reset is the latest reset among the windows at
  100 % or more that reset after the failure, none when such a window has no reset or none is
  full. The row reads `limited · resets in 1 h 35 m` (`limited · resets now` once past), or
  `limited` with no reset, in the warning colour; its activity line keeps Codex's message.
- **Needs you.** A limited Codex is listed as "Codex · <project>" with "Limited · resets in 1 h
  35 m (15:05)" and a Continue at Reset button; once the reset has passed, the button reads
  Continue. Armed, the entry reads "Continues at 15:05" with Cancel. With no reset, the entry has
  no button and a click shows the terminal.
- **Continue.** At the armed reset, or at once for Continue, Marley sends `turn/start` on the lead
  thread with the text "Continue where you left off.", once per (terminal, connection generation,
  thread, failed turn, reset), and only while the thread is idle: Codex 0.162 takes a `turn/start`
  sent during a turn as more input to that turn. Codex's TUI draws it as a user message.
- **Guards**, after T3 Code's `UsageLimitRecoveryWorker.ts:11-70`: no arm when the reset was not
  after the failure; an arm drops, sending nothing, when any turn starts on the thread, the thread
  waits on a request, the thread closes, Marley's connection ends or the terminal closes. A reset
  that passed while the machine slept fires at the next tick, once.
- **The setting** `marley.continue_at_reset`, off by default, in the Agents section of the Marley
  settings page ("Continue Limited Agents at Reset"): on, every limited Codex with a known reset is
  armed without a click. The button stays either way.
- **Redraws:** the countdown redraws each minute while a reset lies ahead, as #640's does.
- `script/e2e/715-usage-meter-and-resume-at-reset.sh`; the stand-in Codex (`codex-fixture.sh`)
  gains cues for rate-limit updates and a usage-limit failure.
- The guide: the meter, the limited state, Continue at Reset, the setting.

### Out (explicitly deferred)
- **Claude Code's limited state and Continue at Reset: the second slice, which needs a number.**
  It waits on a check on the box: whether Claude Code 2.1.296's TUI ends a turn at its usage limit
  (a `StopFailure` with `rate_limit`, then the shared plugin's `error` report with the windows) or
  holds it until the window reopens, as T3 Code's docs say its SDK path does (report 01, open
  question 1). If it ends the turn, the reset comes from the seat's full window and the
  continuation is typed into the terminal while the session is idle (#522's path).
- A Usage tab with tokens per day, project and model read from the CLIs' transcripts (T3 item 12,
  report 01 item 6). It would open as a center tab under Chad's 2026-10-11 rule; it is its own
  ticket.
- Snooze until reset (T3), and a push for a limited agent (#752's kinds).
- Zed agent threads' limits (Claude Code and Codex through ACP): a thread reports its token use
  (`AcpThreadEvent::TokenUsageUpdated`), not its account's windows.
- Harness sessions: their quota line is #640's; continuing one at its reset is rustal-harness's
  job.
- Reset credits and `account/rateLimitResetCredit/consume`: a private endpoint and account
  pooling, which T3's own survey lists under "What not to take".
- `account/rateLimits/read`: Codex fetches it from its backend on each call (D2).

## Reference (§20)
- **T3 Code** (MIT, report 01 §2.8 and §2.12): a thread stopped by a usage limit reads Limited with
  its reset, Resume at reset schedules "Continue where you left off.", Auto-resume is a setting,
  and one recovery is keyed on (thread, run, reset) with a guard against a reset already past
  (`UsageLimitRecoveryWorker.ts:11-70`); Codex's reset is the latest of its exhausted windows
  (`provider/codexUsageLimits.ts:237-250`, `mergeCodexRateLimits` at `:168-186`). Marley keeps the
  state, the button, the off-by-default setting, the guards and the text; the meter is a row line,
  not T3's Usage page.
- **Orca** (MIT, `docs/orca_architecture/01-agents-and-sessions.md` §2.12, via #640): the tightest
  window first, resets listed most used first.
- **#640's harness rows** (Marley's own): the quota line, the tooltip and the minute redraw this
  ticket extends to local rows.
- **Warp:** N/A — Marley-specific. No Warp surface reads a CLI agent's account windows, and no
  Warp code was read.

### Prior art
- **Behavior maps.** T3 report 01 §2.8 (Limited, Resume at reset, snooze, the guards; "Codex comes
  first: its App Server gives the reset time and takes `turn/start`"), §2.12 (the Usage page and
  where its numbers come from), §3 items 1 and 6, §5 open questions 1 and 2; the README's items 2
  and 12 and open question 2 ("the button only; auto-continue is a setting, off"). MonoCode
  finding 2 (`docs/planning/intake/monocode-findings.md`): Claude's and Codex's five-hour and
  weekly use, resume at the reset. Orca report 01 §2.12 and §3 item 10 (Claude's `rate_limits`
  through a status line, one slot the user may own).
- **Published material.** Codex's App Server protocol generated from the installed 0.162.0:
  `AccountRateLimitsUpdatedNotification { rateLimits: RateLimitSnapshot }` ("sparse rolling
  update"), `RateLimitSnapshot { limitId, primary, secondary, rateLimitReachedType, … }`,
  `RateLimitWindow { usedPercent, resetsAt (seconds), windowDurationMins }`, `TurnError {
  message, codexErrorInfo }` with `usageLimitExceeded` and `rateLimitExceeded`, `TurnStartParams {
  threadId, input[] }`. At 0.155.1: `account/rateLimits/updated` is sent from each token count's
  windows (`app-server/src/bespoke_event_handling.rs:1695-1716`), and a usage-limit error updates
  them before the turn fails (`core/src/session/turn.rs:1616-1621`); `turn/start` at
  `app-server-protocol/src/protocol/common.rs:1038`. rustal-harness's `docs/CODEX_TURNS.md:172-178`
  (its TICKET-104, D174): the `codex` bucket, the sparse merge, and the kinds `five_hour`,
  `seven_day`, `primary`, `secondary`.
- **Code we already ship.**
  - `crates/marley_workbench/src/harness.rs:1136-1147` (the `quota.` label keys), `:1204-1265`
    (`QuotaWindow`, `Signals`, `Signals::of`); `crates/marley_workbench/src/rail.rs:6775-6850`
    (`harness_row`, its quota line at `:6837-6844`), `:6928-6958` (`signals_tooltip`), `:6960-6975`
    (`used_words`, `reset_words`). `Signals::of` is read for harness rows only (`:6783`, `:7005`).
  - `crates/marley_agent/src/report.rs:112-121` (`QuotaWindow`), `:275-286` (the `quota.` labels);
    `crates/marley_workbench/src/agent_events.rs:151-225` (`apply_report`, labels kept on the
    seat); `crates/marley_workbench/claude_shared_plugin/hooks/register.js:62-74` (`measured`),
    `:232-247` (`turn.complete` reports `error` or `idle` with the windows).
  - `crates/marley_workbench/src/rail.rs:7758-7866` (`terminal_snapshot`: the row's line from
    `codex_line`, `:7972`, or `claude_events::seat_line`), `:5486-5545` (`render_terminal_row`).
  - `crates/marley_agent/src/codex_events.rs:322-397` (`fold`; a failed turn at `:344-353`),
    `:117-121` (`TurnError`, message only), `:399-418` (`seat_words`).
  - `crates/marley_workbench/src/codex_server.rs:1100-1196` (`heard`: a notification about no
    thread is dropped), `:757-790` (`Client::call`), `:888-985` (`follow`, its tick).
  - `crates/marley_workbench/src/rail.rs:6688` (`inbox_entries`), `:7060` (`seat_entry`),
    `:2065-2131` (`render_inbox_entry`), `:2255` (`answer_inbox`); `crates/marley_rail/src/marley_rail.rs:457-530`.
  - `crates/settings_content/src/marley.rs` (flat agent settings such as `resume_agents`) and
    `crates/settings_ui/src/marley_page.rs` (the Agents section).
  Nothing in `Cargo.lock` owns the seam: the countdown is arithmetic on epoch milliseconds, and the
  timer is gpui's executor.

## UI proof
`script/e2e/715-usage-meter-and-resume-at-reset.sh`, under `compositor sway` (buttons are clicked
and the pointer rests on a row). Fakes only: #650's stand-in `codex` (`codex-fixture.sh`,
`MARLEY_CODEX`, `marley.codex_app_server` on in the run's copy) with new cues `limits
five_hour:62:+95m seven_day:31:+3d` (an `account/rateLimits/updated`), `limited +80s` (an update
with `five_hour` at 100 % resetting in 80 seconds, then `turn/completed` `failed` with
`usageLimitExceeded`), and `limited-unknown` (`rateLimitExceeded` with no full window); its server
logs every `turn/start` with its client and text. A stand-in `claude` that runs `"$MARLEY_BIN"
report idle --quota five_hour:41:<ms> --quota seven_day:12:<ms>` as the shared plugin does.
Shots (the reset is `+80s`, so the countdown reads `resets in 1 m`):
- `715-01-meters`: a Codex row with `5 h 62 % · resets in 1 h 34 m` and a Claude Code row with
  `5 h 41 % · resets in 2 h 9 m` (`5 h` and `7 d` are the row's short words for `five_hour` and
  `seven_day`).
- `715-02-tooltip`: the pointer on the Codex row: "Codex usage", both windows by name, most used
  first, with resets.
- `715-03-limited`: after `limited +80s`: the row `limited · resets in 1 m`, not `failed`, in the
  warning colour, and the Needs you entry with Continue at Reset.
- `715-04-armed`: Continue at Reset clicked: "Continues at <time>" and Cancel.
- `715-05-continued`: 85 seconds on: the row `working`, the entry gone; the server log's one
  `turn/start` from Marley with "Continue where you left off."; the TUI prints it as a user message.
- `715-06-dropped`: a second `limited +80s`, armed, then `work` cued before the reset: the entry
  gone, and 85 seconds on no second `turn/start` from Marley in the log.
- `715-07-no-reset`: `limited-unknown`: the row `limited`, the entry with no button.
- `715-08-setting-off` and `715-08-setting`: the Agents section's "Continue Limited Agents at
  Reset", off, then on.
- `715-09-armed-by-setting`: a third `limited +80s` arms with no click ("Continues at <time>").
- `715-10-cancelled`: Cancel clicked with the setting on, then a rate-limit update folds the seat
  again: the entry back to Continue at Reset, not armed again.

## Locked-In Decisions
- **D1:** The meter is a line on the agent's rail row and its tooltip, #640's drawing reused for
  local rows. Not a center tab: Chad's 2026-10-11 rule is that everything that opens, opens as a
  tab in the main pane (`docs/planning/design-notes/tabs-everywhere-2026-10-11.md`), and a reading
  on a row opens nothing; a Usage tab of token history would be a tab under that rule and is its
  own ticket. Not the status bar: #746 is moving its right end.
- **D2:** Codex's windows come only from the `account/rateLimits/updated` Codex already sends;
  Marley sends no `account/rateLimits/read`, which Codex answers from its backend, so Marley adds
  no request to OpenAI's servers. A Codex that has not answered a turn since Marley joined shows no
  meter. One vocabulary with rustal-harness: the `codex` bucket, the sparse merge, the kinds
  `five_hour`, `seven_day`, `primary`, `secondary`.
- **D3:** Claude Code's windows come from the shared plugin's reports, which already land on the
  seat. Rejected: a status line script (one slot the user owns, Orca's caveat); the transcripts
  (they hold tokens, not limits); `api.anthropic.com/api/oauth/usage` (a private endpoint, T3's
  "What not to take").
- **D4:** Limited is Codex's own word for the failure (`usageLimitExceeded`, and
  `rateLimitExceeded`, as T3 Code classes both), not a reading of its message. The reset is the
  latest among the full windows, since all must reopen before a continuation can run (T3's
  `codexUsageLimitResetAt`).
- **D5:** The button only; continuing by itself is the setting `marley.continue_at_reset`, off by
  default (T3 README open question 2's default, which Chad's "2.) yes" took).
- **D6:** The continuation is a `turn/start` on Marley's own connection, the first prompt Marley
  sends through the App Server; nothing is typed into the TUI. Its text is T3's, "Continue where
  you left off.", the same #750 sends after a relaunch. Rejected: typing it with Enter through
  #522's delivery, as #750 does: it would join whatever the user left in the TUI's composer, which
  Marley cannot see, and `turn/start` does not touch the composer. #750 types because no
  `turn/start` path existed when it was drafted (its D7); once this one exists, #750's Continue
  for a Codex on its App Server could take it.
- **D7:** One continuation per (terminal, connection generation, thread, failed turn, reset), and
  nothing once the user or Codex moved the thread on (T3's guards); an arm lives in memory and does
  not survive a Marley restart, whose thread is a new connection.
- **D8:** A limited Codex is listed in Needs you, as Chad's decision asks, beside the agents that
  wait on the user; its entry leaves when the thread's next turn starts.
- **D9:** Claude Code's limited state waits for the second slice: whether its TUI ends the turn at
  the limit is unchecked, and its continuation is a typed prompt, a different path.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Codex's App Server sends a rate-limit update for the `codex` bucket, the Codex row shall show the most-used window's kind, its percent used and the time until it resets, on a line of its own. | Shot `715-01-meters` |
| REQ-002 | WHILE a Claude Code seat carries quota windows from the shared plugin's report, its row shall show the same line. | Shot `715-01-meters` |
| REQ-003 | WHEN the pointer rests on a Claude Code or Codex row with quota windows, the rail shall show every window, most used first, with its reset. | Shot `715-02-tooltip` |
| REQ-004 | WHEN a Codex turn fails with `usageLimitExceeded` or `rateLimitExceeded`, its row shall read `limited` with the time until the reset in place of `failed`. | Shot `715-03-limited` |
| REQ-005 | WHILE a Codex is limited with a known reset, Needs you shall list it with the reset and a Continue at Reset button. | Shot `715-03-limited` |
| REQ-006 | WHEN Continue at Reset is clicked, the entry shall read "Continues at <time>" and offer Cancel. | Shot `715-04-armed` |
| REQ-007 | WHEN an armed reset passes, Marley shall send one `turn/start` on that thread with the text "Continue where you left off.". | Shot `715-05-continued`; the server log |
| REQ-008 | IF a turn starts on the thread, or it waits on a request, or it closes, or the connection ends, before an armed reset, THEN Marley shall drop the arm and send nothing. | Shot `715-06-dropped`; the server log |
| REQ-009 | IF no full window resets after the failure, THEN the row shall read `limited` and the entry shall offer no Continue at Reset. | Shot `715-07-no-reset` |
| REQ-010 | WHERE `marley.continue_at_reset` is on, the system shall arm every limited Codex with a known reset without a click. | Shot `715-08-setting` |
| REQ-011 | `marley.continue_at_reset` shall be off by default, and the Agents section of the Marley settings page shall show it. | Shot `715-08-setting`; review |
| REQ-012 | WHEN the reset has passed and nothing is armed, the entry's button shall read Continue and send the continuation at once. | Review of the diff |
| REQ-013 | Marley shall send no `account/rateLimits/read` to Codex's App Server. | The server log over the run |
| REQ-014 | WHILE a shown reset lies ahead, the rail shall draw its countdown again at least once a minute. | Review of the diff |
| REQ-015 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes: re-read #650's range against
  the installed Codex, #752's changes to the follow and the inbox, and the shared plugin's report
  path; `brain_ask`.
- **P2 Code** — `marley_agent` (`codex_events`: the rate-limit snapshot, its merge and labels, the
  turn error's code, the limited state and its words); `marley_workbench` (`codex_server`: the
  notification, `turn/start`; a new `limits.rs` for the arms and their guards; `harness.rs`'s
  `Signals` and `rail.rs`'s quota line for local rows, the limited row, the Needs you entry and its
  buttons; `marley_workbench.rs` for the setting); `marley_rail` (the entry's answers); the Zed
  settings paths with their ledger rows; the stand-in's cues; the guide; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot and the server log.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
