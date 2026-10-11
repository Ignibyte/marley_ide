# A usage meter, and resume at the reset — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-715-usage-meter-and-resume-at-reset.md
- **Pipeline spec:** 715-usage-meter-and-resume-at-reset.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-09, on the MonoCode findings: "lets queue up the MonoCode findings
  for potential future use" (finding 2, a usage meter and resume at the reset). Chad, 2026-10-11,
  answering the T3 Code survey's open questions: "2.) yes", usage limits as a state, with Continue
  at Reset and a meter (README items 2 and 12, report 01 items 1 and 6). The ticket moves from
  Deliberate to the Queue, after the tabs batch (#743 to #749) and after #752 and #753.
- **What changed from the ticket as filed on 2026-10-09.** It said an agent at its limit "resumes
  on its own at the reset" and "holds its queued work". Chad's answer takes T3's default instead:
  the button only, continuing by itself a setting, off. Marley holds no queue of work, so nothing
  is held. Agent Panel threads went to Out: ACP reports a thread's tokens, not its account's
  windows. The ticket's Summary and Acceptance say so now.
- **Classification:** feature, prong 2. Marley crates (`marley_agent`, `marley_workbench`,
  `marley_rail`) and two Zed paths that carry Marley rows already (`settings_content`'s
  `marley.rs`, `settings_ui`'s `marley_page.rs`). Size M.
- **The split.** The whole of the ticket's title does not fit one slice. This slice: the meter on
  both agents' rows, and Codex's limited state with Continue at Reset and the setting. The second
  slice, which needs a number: Claude Code's limited state and its continuation, after a check on
  the box (D9, and the spec's Out). If the builder wants this slice smaller still, the meter
  (REQ-001 to REQ-003) can go first on its own.
- **The meter's place.** Chad's 2026-10-11 rule (the tabs-everywhere shelf): "I wanted everything
  to open in the main panel now with a tab", and "everything opens in the main pane now then the
  user can just split panel it out themselves". A meter of two windows per account is a reading,
  not something that opens, so it sits on the row as #640's harness rows have it (D1). A Usage tab
  of token history, T3's item 12, would open as a center tab under that rule; it is its own ticket.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.

### Recall (§18.3)
- **Ledger.**
  - AD-claude-650-marley-joins-a-codex-app-server-of-its-own-per-terminal-001: Marley "sends
    nothing that changes a thread" except an answer (#651). This ticket's `turn/start` is the first
    prompt Marley sends through the App Server, and only on a click or the setting.
  - L-claude-650-a-codex-thread-resume-renames-its-client-001: a client's call can change the
    thread's client name until the next `turn/start`; the continuation's own `turn/start` starts a
    turn under Marley's connection (Risks).
  - AD-claude-651-codex-approvals-answer-in-the-inbox-only-when-clicked-001: an action reaches the
    follow on its channel (`Event::Answer`), checked against the connection's generation; the
    continuation goes the same way.
  - PR-claude-async-answer-carries-question-identity-every-hop-001: an arm carries the terminal,
    the generation, the thread, the failed turn and the reset; the send checks all of them.
  - PR-claude-two-engines-answering-the-same-question-will-diverge-001: one quota reading
    (`Signals::of`) and one drawing for harness and local rows.
  - #640's D6 to D8 (checked values, tokens only, the countdown's words) hold for local rows.
- **Completed pipelines.** #640 (the quota line, the tooltip, the minute redraw), #650 and #651
  (the follow, the channel, the stand-in), #652 and #709 (the shared plugin's reports, its switch),
  #522 (a prompt typed into an idle Claude Code, for the second slice), #540 (resume).
- **The harness** (`/srv/stacks/rustal-harness`, read only): `docs/CODEX_TURNS.md:172-178` reads
  the same notification for the same seat labels; Marley takes its bucket rule, merge and kinds.
- **Brain:** no `rusty` MCP server in this repository's sessions; the promotion runs `brain_ask`.

### Discovery
- **#640's reading and drawing** (`crates/marley_workbench/src/harness.rs:1136-1265`): the label
  keys, `QuotaWindow`, `Signals::of` (percents from 0 to 100, resets in epoch milliseconds, most
  used first). `rail.rs` (at `19dda499a2`) draws a harness row's quota line (`:6837-6844`), its
  tooltip (`signals_tooltip`, `:6928-6958`) and the words (`used_words`, `reset_words`,
  `:6960-6975`); `Signals::of` is read only for harness rows (`:6783`, `:7005`).
- **Claude Code's windows today.** The shared plugin's `turn.complete` (`register.js:232-247`)
  reports `idle`, or `error` for a turn an API error ended, with `measured(rateLimits)`
  (`:62-74`): up to eight windows, a percent from 0 to 10,000 and an optional reset.
  `marley-agent report --quota KIND:PERCENT[:RESETS_AT_MS]` carries them
  (`crates/marley_workbench/bin/marley-agent`, `quota_window`); `Report::labels`
  (`marley_agent/src/report.rs:275-286`) makes `quota.KIND.*` labels and `apply_report`
  (`agent_events.rs:151-225`) puts them on the terminal's seat. Nothing draws them on a local row.
  The plugin loads while `marley.claude_code_shared_plugin` is on (off by default, #709), for
  Claude Code 2.1.287 and later.
- **Codex's windows.** Marley reads no `account/rateLimits/*` today. `heard`
  (`codex_server.rs:1100-1196`) drops a notification that names no thread. At 0.155.1 Codex sends
  `account/rateLimits/updated` to every connection from each token count's windows
  (`app-server/src/bespoke_event_handling.rs:1695-1716`, a broadcast with no thread) and updates
  the windows from a usage-limit error before failing the turn (`core/src/session/turn.rs:1616-1621`).
  `account/rateLimits/read` (`request_processors/account_processor.rs:156`, `:1129-1159`) calls the
  ChatGPT backend through a `BackendClient` on each read.
- **The failure.** `TurnError` (`codex_events.rs:117-121`) keeps `message` only; Codex's schema
  adds `codexErrorInfo`, a string such as `usageLimitExceeded` or `rateLimitExceeded`, or an object
  for HTTP failures. `fold` sets the seat failed with the message (`:344-353`); `seat_words`
  (`:399-418`) gives `failed`.
- **Sending a prompt.** No `turn/start` or `turn/steer` anywhere in Marley; `Client::call`
  (`codex_server.rs:757-790`) is the request path, used for `thread/resume` and others. #651's notes
  ("For the deferred slices") read Codex's TUI drawing a user message another client sent, from its
  source and tests, not live.
- **Needs you.** `inbox_entries` (`rail.rs:6688`), `seat_entry` (`:7060`), `render_inbox_entry`
  (`:2065`, buttons `:2109-2131`), `answer_inbox` (`:2255`); `InboxAnswer`
  (`marley_rail.rs:472`) is a closed set.
- **The settings.** Flat agent settings on `MarleySettingsContent` (`resume_agents`,
  `ask_before_ending_a_working_agent`) and the Agents section of `marley_page.rs`.
- **The box.** Codex 0.162.0 is installed, past #650's tested range (0.155.1 up to 0.158.1,
  `versions.rs:103-114`); `marley.codex_app_server` and `marley.claude_code_shared_plugin` are unset
  in Chad's settings, so neither half of the meter shows there until he turns them on. Claude
  Code's `statusLine` slot is empty on this box (D3 does not use it).
- **Line numbers** in `rail.rs` are at `19dda499a2`; #743's Code phase was editing `rail.rs` while
  this was drafted, so the promotion reads them again.

### Design
- **`marley_agent::codex_events`** (Marley crate):
  - `RateLimits` (the `codex` bucket's two windows, each `used_percent`, `resets_at` in seconds,
    `window_minutes`) and `Notification::RateLimits`, decoded from `account/rateLimits/updated`
    (a snapshot whose `limitId` is another bucket's decodes to nothing).
  - `Input::RateLimits(&RateLimits)` folds into the seat's `quota.KIND.percent_used` and
    `quota.KIND.resets_at_ms`, a window's missing reset kept from the label already there; the kind
    by duration (300 minutes `five_hour`, 10,080 `seven_day`), else `primary` or `secondary`.
  - `TurnError.code`; a failed turn whose code is `usageLimitExceeded` or `rateLimitExceeded` sets
    the labels `limited` and, when known, `limit_resets_at_ms` (the latest reset among the seat's
    windows at 100 % or more that lies after the failure), with the turn's id in `limited_turn`.
    `TurnStarted` clears them. `seat_words` reads `limited · resets in …` or `limited`.
- **`marley_workbench`** (Marley crate):
  - `codex_server.rs`: `heard` folds `account/rateLimits/updated` into the seat;
    `Event::Continue { generation, thread, turn, reset }` on the follow's channel sends
    `turn/start` `{threadId, input: [{type: "text", text: "Continue where you left off."}]}` only
    when the generation, the lead and its idle state match and the seat still reads limited by
    that turn; the outcome is logged, and a refusal leaves the entry with Continue.
  - `limits.rs` (new): the `Arms` global, one arm per terminal with the generation, thread, failed
    turn and reset; `arm`, `cancel`, and a check every 30 seconds against the wall clock
    (`agent_events::now_ms`), so a reset that passed while the machine slept fires at the first
    check after wake; the seat's next turn, a wait, the thread's close, the follow's end and the
    terminal's release drop the arm. With `marley.continue_at_reset` on, a seat that turns limited
    with a known reset is armed as it folds.
  - `harness.rs`: `Signals` read for any seat (its quota half), unchanged in what it accepts.
  - `rail.rs`: a local row's quota line and tooltip from `Signals::of`; the limited row's words and
    warning colour; a Needs you entry for a limited Codex (`InboxTarget::Limit`), its buttons, the
    click to `limits::arm` or the follow's `Event::Continue`.
  - `marley_workbench.rs`: `MarleySettings::continue_at_reset`.
- **`marley_rail`** (Marley crate): `InboxAnswer::ContinueAtReset`, `Continue`, `Cancel`.
- **Zed paths** (rows in `docs/marley/zed-touchpoints.md` widened first):
  `crates/settings_content/src/marley.rs` (`continue_at_reset: Option<bool>`, default false);
  `crates/settings_ui/src/marley_page.rs` (the Agents section's toggle).
- **File manifest:** `marley_agent/src/codex_events.rs`; `marley_rail/src/marley_rail.rs`;
  `marley_workbench/src/{codex_server.rs, limits.rs (new), harness.rs, rail.rs,
  marley_workbench.rs}`; the two Zed paths; `script/e2e/codex-fixture.sh`,
  `script/e2e/715-usage-meter-and-resume-at-reset.sh`; `docs/marley/guide.md`.
- **Ledger rows:** the two settings paths' rows, widened. No new spawn site.

### Visual check plan
| REQ | The scenario sets up and does | The shot |
|---|---|---|
| 001, 002 | Codex from the picker on the stand-in; `work`, `limits five_hour:62:+95m seven_day:31:+3d`, `done`; a second terminal with the stand-in `claude` reporting `five_hour:41` and `seven_day:12` | `715-01-meters` |
| 003 | the pointer on the Codex row | `715-02-tooltip` |
| 004, 005 | `work`, `limited +40s` | `715-03-limited` |
| 006 | Continue at Reset clicked | `715-04-armed` |
| 007 | 45 seconds on | `715-05-continued`; the server log's one `turn/start` from Marley |
| 008 | `work`, `limited +40s`, armed; `work` typed before the reset; 45 seconds on | `715-06-dropped`; no `turn/start` from Marley |
| 009 | `work`, `limited-unknown` | `715-07-no-reset` |
| 010, 011 | the Agents section opened; the toggle off, then on; `work`, `limited +40s` | `715-08-setting` |
| 012, 014 | none: a reset that passes unarmed, and the minute redraw, are read in the diff | review |
| 013 | the server log over the run | no `account/rateLimits/read` |
| 015 | the gate | `script/gates.sh --diff` |

What no scenario reaches: a real usage limit (it needs a model and an exhausted account), and a
machine that sleeps through a reset; the wall-clock check is read in the diff.

### Risks
- **The installed Codex is 0.162.0**, past #650's tested range; the promotion brings the range
  and the stand-in up to date, as #752's and #753's do. The names read here are the same at
  0.155.1 and 0.162.0.
- **The first prompt through the App Server.** Codex's TUI drawing another client's `turn/start`
  as a user message is read from its source and tests (#651's notes), not seen live; the
  scenario's stand-in TUI is not Codex's. A turn Marley starts runs under Marley's connection, and
  Codex gates some features on the TUI's client name (L-claude-650); the continuation asks for
  none of them.
- **gpui timers stop while the machine sleeps** (a monotonic clock); the arms are checked against
  the wall clock every 30 seconds instead of waiting on one long timer.
- **`rateLimitExceeded` can be a short 429** with no full window: the row reads `limited` with no
  reset and no button, which is what Codex knows.
- **A meter needs a turn.** A Codex shows no meter until its first answered turn after Marley
  joined; a Claude Code until its first main turn ends with the shared plugin on (D2, D3).
- **Rows grow by a line**, and every Claude Code row of one account shows the same windows, as
  harness rows of one account do.
- **Two Codex terminals on one account** each read their own server's updates, so each row's
  meter is as fresh as its own last turn.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
