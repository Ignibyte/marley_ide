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

### Promotion (2026-10-11)
- Moved from `queued/` to `active/`; the BACKLOG row removed; the ticket in progress. Brain:
  consultation ac0e72a4376d4bb8bc6090059812d2f7, nothing on this seam (a duplicate ask,
  088f777c…, closed with no decision). The ledger: no rule or failure on quotas or rate limits
  since #640.
- **The protocol, against the installed Codex 0.162.0** (`codex app-server generate-ts` and
  `generate-json-schema`, and the binary's strings): `account/rateLimits/updated` carries
  `{rateLimits: RateLimitSnapshot}`, "sparse rolling update"; `RateLimitSnapshot.limitId` is a
  string or null; `RateLimitWindow { usedPercent: int32, windowDurationMins: int64|null,
  resetsAt: int64|null }`; `TurnError { message, codexErrorInfo, additionalDetails, misalignment }`,
  `codexErrorInfo` a string such as `usageLimitExceeded` or `rateLimitExceeded`, or an object for
  HTTP failures; `turn/start` `{threadId, input: [UserInput]}` answered `{turn}`, the text input
  `{type: "text", text, text_elements}` with `text_elements` defaulting to `[]`. All four names are
  in the binary. **New against the draft:** `TurnStartParams` says a `turn/start` sent while a
  turn runs "steers an already-active turn": it adds input to that turn. The continuation is sent
  only while the lead is idle. 0.162.0 is inside #771's tested range (`marley_agent/src/versions.rs`
  112-115, from 0.155.1 before 0.162.1), so no range change.
- **Seams checked again** at `a8c9b94b0c` (an Explore pass over all thirteen):
  - `harness.rs` 1142-1153 (the `quota.` keys), `QuotaWindow` 1210 (`percent_used: u8`), `Signals`
    1222, `Signals::of(labels)` 1233 reads a bare labels map, so any seat's; its callers are
    harness.rs:982 and rail.rs:7338 and 7560, harness rows only. `percent_of` (1308) drops a value
    outside 0 to 100.
  - `harness_row` rail.rs:7330, its quota line 7392-7398, `signals_tooltip` 7485, `used_words`
    7515, `reset_words` 7524. The countdown's minute redraw is the harness poll's `bump_minute`
    (harness.rs:303, 947-988), harness rows only; a terminal row's `minute_timer` (rail.rs:156,
    armed at 2638-2676 from `AgentEvents::next_quiet_change`) counts working seats' quiet times,
    never a reset.
  - `terminal_snapshot` rail.rs:8334 (the subtitle `codex_line` at 8550 or `seat_line`, the
    activity line), `render_terminal_row` 5989 (the subtitle drawn muted, then the activity, the
    command, a running error in the error colour, #750's chip); `RowLine` 9697, `CardLine` 9744,
    `row_card` 9766.
  - `marley_rail.rs`: `TerminalSnapshot` 171 and `TerminalRow` 649 carry no quota; `InboxKind`
    468 (`Codex` exists), `InboxAnswer` 483, `InboxEntry` 542. `InboxTarget` is in rail.rs:941.
    `inbox_entries` rail.rs:7243 (a Codex with requests gives `codex_entry` 7640, else a waiting
    seat gives `seat_entry` 7615); `render_inbox_entry` 2394; `answer_inbox` 2594 →
    `codex_decision` 7700 → `codex_server::answer`. The inbox redraws every 30 s while it holds an
    entry (`inbox_timer`, 2218-2225).
  - `codex_events.rs`: `TurnError` 123 (`message` only), `Notification` 153, `decode` 206 (any
    other method is `Ok(None)`), `Input` 323, `fold` 337: a failed turn (362) sets `error` and
    `turn_failed`, state `Error`; `TurnStarted` clears them; `Input::Lead` with a new thread id
    drops every label (346-349). `seat_words` 417. Nothing reads rate limits or `codexErrorInfo`.
  - `codex_server.rs`: `heard` 1209 drops a notification with no `threadId` (debug log only),
    `Client::call` 760 (5 s timeout), `Event` 627, `follow` 992 (generation at 1047, a 1 s tick at
    1053), `answered` 1372 (the generation check), `Lead` 808 `{id, idle, resumed, cwd}`,
    `tell_turns` 893, `push_change` 962. No `turn/start` anywhere in `crates/` or `script/`.
  - `report.rs`: `QuotaWindow.percent_used: f64` from 0 to 10,000, written `{:.1}`;
    `apply_report` (agent_events.rs:159) keeps the seat's `quota.*` labels; `now_ms` 262.
    `register.js` `measured` 62 passes the plugin's percent through raw.
  - Settings: `resume_agents` and `ask_before_ending_a_working_agent` in `settings_content`'s
    `marley.rs` 36 and 46; `default.json` 1713 and 1720; `agents_section() -> [SettingsPageItem;
    19]` (`marley_page.rs:170`); `MarleySettings` 377, a flat bool read at 706. Ledger rows:
    `zed-touchpoints.md` 59, 63 and 68.
  - The stand-in (`codex-fixture.sh`): cues through `handle` 587 and `play` 289; `notify` 143;
    every method logged to `server.log` with its client (372); `turn/start` falls to "method not
    found" (446); no rate-limit notification.
- **What differs from the draft, and where the design takes it:**
  - **Over 100 percent.** A Codex at its limit sends 100 or more, and the plugin's percent may
    pass 100, but `percent_of` drops such a window, so a full window would vanish from the meter
    just when it matters. `Signals` reads a percent above 100 as 100 (the window is full), for
    harness rows as well: one reading (PR-claude-two-engines-…). A negative or non-finite value is
    still left out (#640's D6).
  - **The merge lives with the follow.** Labels are keyed by kind, and an update that leaves out a
    window's duration leaves its kind unknown, so `Following` keeps the `codex` bucket's merged
    snapshot by slot (`RateLimits::merge`, in `codex_events`), and each update folds the whole
    merged snapshot into the seat (`Input::RateLimits`), which replaces the seat's `quota.*`
    labels. A new thread's `Lead` drops every label, so the follow folds the snapshot again after
    it.
  - **`heard`** routes `account/rateLimits/updated` before its thread filter.
  - **The minute redraw for terminal rows:** the rail's `minute_timer` also arms for the next
    minute while any terminal row shows a reset ahead (a quota window's or a limit's).
  - **The limited row's colour:** the subtitle is drawn muted today, so `TerminalSnapshot` gains
    the seat's quota windows (`marley_rail::QuotaWindow`, which `harness.rs` then uses, one type)
    and `limit: Option<Limit { resets_at_ms }>`; a limited row's subtitle is drawn in the warning
    colour.
  - **The entry's clock** ("15:05") is `chrono::Local`, as `threads_page.rs` and `rail.rs:612`
    format times.
  - **The send** checks the lead is idle as well as the generation, the thread and the seat's
    limited turn (0.162's steering, above).
  - **The stand-in** gains `turn/start` (logged with its client and text, a turn started with its
    `userMessage` item, the TUI printing it), the cues `limits`, `limited` and `limited-unknown`,
    and broadcasts the update to every connection, as Codex does.

## Phase 2 — Code
- **Built.**
  - `marley_agent::codex_events`: `RATE_LIMITS_UPDATED`, `RateLimitWindow`, `RateLimits` (`decode`
    passes over another bucket, `merge` keeps a duration or reset the update leaves out, `windows`
    names each by length), `TurnError.code` from `codexErrorInfo` (a word, or an HTTP kind's
    name) and `is_limit`, `LIMITED_LABEL` and `LIMIT_RESETS_AT_LABEL`, `Input::RateLimits` (the
    seat's `quota.*` labels replaced, still when nothing changed), `limit_reset` (the latest
    future reset among the full windows; none when a full window has no reset), `Limit` and
    `limit(seat)`, `seat_words` → `limited`.
  - `marley_rail`: `Usage { quota, limit }` on `TerminalSnapshot` and `TerminalRow`,
    `QuotaWindow` (now the one type; `harness.rs` re-exports it), `Limit` re-exported from
    `codex_events`; `InboxAnswer::{ContinueAtReset, Continue, Cancel}` with words and tooltips.
  - `marley_workbench`:
    - `limits.rs` (new): `Limited`, the `Arms` global (armed, handled, checking), `arm`,
      `cancel`, `forget`, `continue_now`, `on_seat`, and a wall-clock check at the earliest
      reset, 30 s at most.
    - `codex_server.rs`: the follow publishes its answer channel at its start; `generation_of`,
      `continue_lead`, `Event::Continue`; `heard` merges `account/rateLimits/updated` before its
      thread filter; `follow_lead` folds the merged windows after a new lead; `Following::fold`
      hands the seat to `limits::on_seat`; `continued` sends `turn/start` only when the
      generation, the idle lead, no open request and the seat's limited turn all still match;
      `forget` where the follow and the terminal end.
    - `harness.rs`: `used_percent_of` (above 100 reads as 100, for quota windows only; the
      progress percent stays strict through `percent_of`, both over `whole_percent`).
    - `rail.rs`: `usage_of`, `codex_line` with the limit's reset, `quota_line`, `kind_words`,
      `window_words`, `usage_parts`, the limited subtitle in the warning colour, the row's
      tooltip, `next_reset_change` in the minute timer, `InboxTarget::Limit`, `limit_entry`,
      `clock_words`, the answers, the observed `Arms` global, `terminal_row_end` split out.
    - `marley_workbench.rs`: `MarleySettings.continue_at_reset` (`ContinueAtReset::{OnClick,
      Always}`).
  - Zed paths, rows widened first: `settings_content`'s `continue_at_reset`, `default.json`'s
    `false`, `marley_page.rs`'s toggle (the Agents array grown to twenty).
  - `codex-fixture.sh`: `turn/start` (a turn with its text as the user's message, or logged as
    steering during a turn), `rate_limits` broadcast to every connection, the cues `limits`,
    `limited`, `limited-unknown`, and the TUI printing another client's user message.
- **Deviations from the design.**
  - `continue_at_reset` is an enum, not a bool: `MarleySettings` is at clippy's four-bool limit.
  - `Limit` lives in `codex_events` and carries the failed turn; `marley_rail` re-exports it,
    since `marley_rail` depends on `marley_agent` and not the other way.
  - The row line writes `five_hour` and `seven_day` as `5 h` and `7 d` (`kind_words`): the first
    shot cut the kind to `five…` at the rail's width while the percent and the reset fit. The
    tooltip keeps the names. Harness rows take the same line.
  - Arms are keyed by the terminal entity (as `CodexRequests` is), not the view.
- **Review against the criteria.**
  - Found: with the setting on, Cancel only dropped the arm and the next fold of the same limit
    armed it again, so Cancel could not hold. `cancel` now records the dropped limit in `handled`
    beside the continued ones, which the setting never arms again; the scenario gained
    `715-10-cancelled` for it.
  - Entity re-entrancy: `limits` reads and writes its global from `cx.update`, never inside an
    entity's update; `continued` reads the seat in `cx.update` before the call.
  - The continuation is logged when refused or failed; the entry keeps its button.
  - No `account/rateLimits/read` is sent. Clippy found `struct_field_names`, `too_many_lines` in
    `render_terminal_row` (the end and the usage parts split out), `struct_excessive_bools` (the
    enum) and `needless_pass_by_ref_mut`.
- **Gate:** `just gate-diff` green, every gate and the receipt, after Test's Cancel fix and both guides.

## Phase 3 — Test
- **Scenario:** `script/e2e/715-usage-meter-and-resume-at-reset.sh` under `compositor sway`,
  #650's stand-in Codex at 0.162.0 and a stand-in `claude` that sends a `SessionStart` frame and
  the shared plugin's `report idle --quota …`. Three checks, all pass on the final run:
  one `turn/start` by Marley with "Continue where you left off.", still one after the dropped
  arm, and no `account/rateLimits/read`.
- **Shots, final run:**
  - `715-01-meters` (REQ-001, REQ-002): Codex `idle · 15k tokens`, `5 h 62 % · resets in 1 h 34 m`;
    Claude Code `idle`, `5 h 41 % · resets in 2 h 9 m`.
  - `715-02-tooltip` (REQ-003): "Codex usage", `five_hour 62 % · resets in 1 h 34 m`,
    `seven_day 31 % · resets in 2 d 23 h`.
  - `715-03-limited` (REQ-004, REQ-005): the row `limited · resets in 1 m · 12k tok…` in the
    warning colour, "You've hit your usage limit. Try…" under it, `5 h 100 % · resets in 1 m`;
    Needs you "Codex · repo", `Limited · resets in 1 m (16:33)`, Continue at Reset.
  - `715-04-armed` (REQ-006): `Continues at 16:33`, Cancel.
  - `715-05-continued` (REQ-007): the row `working`, the entry gone; Codex's terminal shows
    `user: Continue where you left off.`; the log's one `turn/start by marley`.
  - `715-06-dropped` (REQ-008): armed, then `work` cued; 85 s on the row `working`, no entry, and
    still one `turn/start` from Marley.
  - `715-07-no-reset` (REQ-009): `limited · 12k tokens`, "Rate limit reached…", the line moved to
    `7 d 31 % · resets in 2 d 23 h`; the entry `Limited` with no button.
  - `715-08-setting-off`, `715-08-setting` (REQ-011): the settings search on "Continue Limited":
    Marley → Agents, the toggle off, then on with its reset arrow.
  - `715-09-armed-by-setting` (REQ-010): a third limit, `Continues at 16:47` and Cancel with no
    click.
  - `715-10-cancelled`: Cancel, then a rate-limit update: `Limited · resets in 1 m (16:47)` with
    Continue at Reset, not armed again.
- **Not driven** (review): REQ-012 (`limit_entry` gives Continue once the reset has passed with
  nothing armed; `continue_now` sends at once), REQ-014 (`next_reset_change` arms the minute
  timer for the next change of any shown countdown; the inbox redraws every 30 s), a reset passed
  during sleep (the arms are checked against the wall clock), a real usage limit (it needs a model
  and a spent account).
- **Calibration and scenario fixes:** the first full run put Claude Code's tab in front for
  `715-05`, so the scenario clicks the Codex row first; the settings search needed a click in its
  box; `+40s` read `resets now` (under a minute, #640's words), so the reset is `+80s`.
- **Pre-existing, not in scope:** the toast "Ctrl-Alt-N opened Marley's New Agent picker; the
  program in this terminal did not get the key." shows after New Agent from a focused terminal.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md`; `docs/marley/guide.md` (Usage limits under Codex's App Server,
  and `continue_at_reset` in the settings example); the in-app guide's `#usage-limits`;
  walkthrough stop 5.17; `docs/marley_architecture/marley_workbench.md` (usage limits in the App
  Server section, `src/limits.rs`, the rail), `marley_agent.md` (rate limits and the limited
  state), `marley_rail.md` (`Usage`, the answers); `docs/marley/borrowing.md` (T3 item 2,
  MonoCode). The three Zed rows in `zed-touchpoints.md` (59, 63, 68) name what shipped.
- **Knowledge:** F-claude-715-cancel-could-not-hold-against-the-setting-that-arms-001;
  L-claude-715-a-codex-turn-start-during-a-turn-steers-it-001;
  AD-claude-715-a-limited-codex-is-a-state-and-continues-through-its-app-server-on-a-click-001.
- **Brain:** consultation ac0e72a4376d4bb8bc6090059812d2f7 closed with a decision (follow-up
  2026-11-11); the duplicate 088f777c… closed with no decision.
- **Next slice:** #765, Claude Code's limited state, is the next ticket in the queue.
