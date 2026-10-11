---
pipeline_id: 5e958fce-06d3-475c-b6f4-385572a568dc
ticket: docs/planning/tickets/open/TICKET-765-claude-codes-limited-state.md
status: Phase 4 — Complete PASS
title: Claude Code's limited state and Continue at Reset
type: feature
slice: prong 2; T3 Code survey item 2, MonoCode finding 2; the second of #715's two slices
references:
  - docs/planning/pipeline/completed/715-usage-meter-and-resume-at-reset.spec.md
  - docs/planning/pipeline/completed/750-codex-resumes-and-a-cut-off-turn-continues.spec.md
  - docs/planning/pipeline/completed/709-load-the-shared-claude-code-plugin.spec.md
  - docs/t3code_architecture/01-agents-and-providers.md
---

## Title
A Claude Code session stopped at its usage limit reads `limited · resets in …` on its row instead
of `failed`, and Needs you offers Continue at Reset, which types "Continue where you left off."
into its terminal once the window reopens. #715 did the same for Codex; its D9 left Claude Code
for a check on the box, done at this ticket's plan: Claude Code 2.1.296 ends a turn at its usage
limit with an API error and runs `StopFailure` with `error: "rate_limit"`; it does not hold the
turn until the reset.

## Scope
### In
- **Limited.** A lead `StopFailure` whose `error` is `rate_limit` makes the Claude Code seat limited
  until its next turn or session starts. Its reset is the latest reset among the seat's quota
  windows at 100 % or more that reset after the failure, read as the windows land: the shared
  plugin (#709) reports them at the same turn's end, before or after the frame. No full window
  with a reset means no reset.
- **The row** reads `limited · resets in 1 h 35 m · <prompt>` in the warning colour, or `limited ·
  <prompt>` with no reset; its activity line keeps the error, `rate_limit`, as a failed Claude
  Code's does.
- **Needs you** lists it as "Claude Code · <project>" with #715's entry: `Limited · resets in …
  (15:05)` and Continue at Reset; Continue once the reset has passed; `Continues at 15:05` with
  Cancel while armed; no button without a reset.
- **Continue.** At the armed reset, or at once for Continue, Marley types "Continue where you left
  off." and Enter into the terminal, as #750's Continue does, only while the seat still reads
  limited by the same failure in the same session; once per failure.
- **Guards:** a prompt, a new session, Claude Code leaving the terminal or the terminal closing
  before the reset drops the arm, sending nothing.
- **The setting** `marley.continue_at_reset` covers Claude Code too; its words say "a Claude Code
  or Codex".
- One limited state for both agents: the labels, `Limit` and its reading move from
  `codex_events` to a module of their own in `marley_agent`, and the workbench's arms name their
  agent.
- `script/e2e/765-claude-codes-limited-state.sh`, with a stand-in `claude` that sends the frames and
  the shared plugin's report.
- The guide: Claude Code's limited state.

### Out (explicitly deferred)
- A limit Claude Code reports with another error type (`billing_error`, `overloaded`): not a usage
  limit's state.
- An API-key account's 429 with no windows: it reads `limited` with no reset and no button.
- Continuing a limited Agent Panel thread (ACP reports no account windows), and a harness
  session's (rustal-harness's job).
- Clearing what the user left in Claude Code's composer before typing: Marley cannot see it, and
  the continuation joins it as #522's and #750's deliveries do.

## Reference (§20)
- **T3 Code** (MIT, report 01 §2.8): Limited with its reset, Resume at reset, the guards; the same
  as #715's reference. T3 reaches Claude through its SDK; Marley through the TUI's hooks.
- **#715 and #750** (Marley's own): the limited state, the entry and the arms, and #750's typed
  Continue for a Claude Code a restart cut off.
- **Warp:** N/A — Marley-specific. No Warp surface reads an agent CLI's usage limit, and no Warp
  code was read.

### Prior art
- **Behavior maps:** T3 report 01 §2.8 and §5 open question 1 ("whether Claude's TUI ends the turn
  at its limit"); #715's spec, Out and D9.
- **Published material and the shipped binary:** Claude Code 2.1.296's own bundle, read on the
  box: the `StopFailure` hook input is `{error, error_details, last_assistant_message}`, `error`
  one of `authentication_failed`, `oauth_org_not_allowed`, `account_on_hold`,
  `verification_required`, `billing_error`, `rate_limit`, `overloaded`, `invalid_request`,
  `model_not_found`, `server_error`, `unknown`, `max_output_tokens`, `cloud_credential_error`. A
  429 for a subscriber with rate-limit headers becomes an API error message with `error:
  "rate_limit"` and `apiError: "usage_limit_reached"`; the query loop then returns `api_error` and
  runs `executeStopFailureHooks` with that error. Nothing waits for the reset.
- **Code we already ship:** `claude_events` folds `StopFailure` into `State::Error` with the
  error type in its `error` label; `codex_events::{limit, limit_reset, Limit}` and the labels
  (#715); `limits.rs`, `limit_entry` and the arms (#715); `resume::continue_turn` and
  `terminal_drive::paste_then` (#750); `agent_events::on_frame` and `agent_reports`' report path.
  Nothing in `Cargo.lock` owns the seam.

## UI proof
`script/e2e/765-claude-codes-limited-state.sh`, under `compositor sway` (buttons are clicked). A
stand-in `claude`, cued through `$E2E_WORK/claude.fifo`, prints #519's hook frames and runs
`"$MARLEY_BIN" report` as the shared plugin does; what is typed into it is logged and starts a turn
(a `UserPromptSubmit` frame). Cues: `limited +80s` (a prompt, `StopFailure` `rate_limit`, then a
report `error` with `five_hour` at 100 % resetting in 80 seconds), `limited-unknown` (the same
with `five_hour` at 20 %), `done` (a `Stop`). Shots:
- `765-01-limited`: the row `limited · resets in 1 m · <prompt>` in the warning colour, and Needs
  you "Claude Code · repo" with Continue at Reset.
- `765-02-armed`: Continue at Reset clicked: `Continues at <time>` and Cancel.
- `765-03-continued`: 85 seconds on: the row `working`, the entry gone, the stand-in's terminal
  showing the typed "Continue where you left off."; its log holds it once.
- `765-04-dropped`: limited again and armed, then a prompt typed before the reset: 85 seconds on,
  nothing more typed by Marley.
- `765-05-no-reset`: `limited-unknown`: the row `limited`, the entry with no button.

## Locked-In Decisions
- **D1: the check's answer.** Claude Code ends the turn at its usage limit (`StopFailure`,
  `rate_limit`), so the limited state follows that frame, and the continuation is a new prompt.
- **D2: the reset from the windows.** The `StopFailure` hook carries no reset; the seat's quota
  windows (#709's report at the same turn's end) do. The reset is read from the labels each time,
  after the failure's moment, so a report that lands after the frame still gives it.
- **D3: typed, as #750 types.** The continuation is `paste_then` with Enter into the terminal,
  only while the seat still reads limited by the same failure; a prompt the user sends first
  moves the seat on and drops the arm. Rejected: a Claude Code API to start a turn (none for the
  TUI).
- **D4: one limited state.** `LIMITED_LABEL`, `LIMIT_RESETS_AT_LABEL`, the failure's moment,
  `Limit` and `limit(seat)` live in `marley_agent::usage`, read for both agents; Codex's labels and
  behavior stay as #715 shipped them.
- **D5: one arm per terminal, named by its agent.** An arm keeps the terminal and the view it
  continues; Codex's goes through the App Server follow (#715), Claude Code's is typed.
- **D6: the failure's identity** is the moment Marley folded the `StopFailure` and the session id:
  Claude Code's hooks give no turn id.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a Claude Code turn ends with a `StopFailure` whose error is `rate_limit`, its row shall read `limited` with the time until the reset in place of `failed`, in the warning colour. | Shot `765-01-limited` |
| REQ-002 | WHILE a Claude Code is limited with a known reset, Needs you shall list it with the reset and Continue at Reset. | Shot `765-01-limited` |
| REQ-003 | WHEN Continue at Reset is clicked, the entry shall read "Continues at <time>" and offer Cancel. | Shot `765-02-armed` |
| REQ-004 | WHEN an armed reset passes, Marley shall type "Continue where you left off." and Enter into the terminal, once. | Shot `765-03-continued`; the stand-in's log |
| REQ-005 | IF a prompt or a new session starts, or Claude Code leaves the terminal, before an armed reset, THEN Marley shall type nothing. | Shot `765-04-dropped`; the stand-in's log |
| REQ-006 | IF no full window resets after the failure, THEN the row shall read `limited` and the entry shall offer no Continue at Reset. | Shot `765-05-no-reset` |
| REQ-007 | WHERE `marley.continue_at_reset` is on, the system shall arm a limited Claude Code with a known reset without a click. | Review of the diff |
| REQ-008 | A Codex's limited state, entry and continuation shall behave as #715 shipped them. | Review of the diff |
| REQ-009 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_agent` (`usage.rs`, `claude_events`, `codex_events`); `marley_rail` (the
  re-export); `marley_workbench` (`limits.rs`, `codex_server.rs`, `agent_events.rs`,
  `agent_reports.rs`, `rail.rs`); the setting's words on the Zed paths; the guide; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
