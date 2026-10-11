# Claude Code's limited state and Continue at Reset — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-765-claude-codes-limited-state.md
- **Pipeline spec:** 765-claude-codes-limited-state.spec.md

## Phase 1 — Plan
- **Request:** #715's second slice, split on 2026-10-11 when Chad asked to "spec all" of the
  survey list; Chad's "2.) yes" on usage limits as a state with Continue at Reset (T3 README item
  2). The goal: "once the specs are finished lets drive on building the remaining features."
- **Classification / tier:** feature, prong 2, size S to M. Marley crates, and the setting's
  words on two Zed paths already carrying Marley rows (`settings_content`'s `marley.rs`,
  `settings_ui`'s `marley_page.rs`) and `default.json`.
- **Checklist:** no TaskCreate in this harness; the phase checklist lives here. Pre-flight clean
  (cargo, gate, e2e, shear, hooks, no active pipeline, README marker, cargo idle).
- **Recall (§18.3):**
  - #715 (completed today): the limited state, `limit_entry`, the arms in `limits.rs`, the
    setting; its D9 named this slice's check. F-claude-715 (Cancel against the setting) holds for
    any agent: `cancel` records the limit as handled.
  - PR-claude-keys-after-a-paste-go-in-a-later-write-001: the Enter after the prompt goes through
    `terminal_drive::paste_then`, as #750's `continue_turn` does; that path was checked against a
    reader of bracketed pastes in #594 and #750, so this scenario's stand-in may read lines.
  - PR-claude-596 (look up across windows before updating one): Continue from Needs you runs in
    the rail's listener, inside the window's update, so the arm keeps the terminal's weak handle
    and its view's id rather than finding them through `cx.windows()`.
  - Brain: consultation f2bbc83b301740e787fcbed8fdebdebd, nothing on this seam.
- **The check on the box** (the ticket's first step), Claude Code 2.1.296's bundle
  (`~/.local/share/claude/versions/2.1.296`), read with Python's `re` over the binary:
  - The `StopFailure` hook input: `hook_event_name: "StopFailure"`, `error` (the enum `Zn`:
    `authentication_failed`, `oauth_org_not_allowed`, `account_on_hold`, `verification_required`,
    `billing_error`, `rate_limit`, `overloaded`, `invalid_request`, `model_not_found`,
    `server_error`, `unknown`, `max_output_tokens`, `cloud_credential_error`), `error_details`,
    `last_assistant_message`.
  - A 429 for a subscriber with rate-limit info becomes `Is({content, error: "rate_limit",
    apiError: "usage_limit_reached", apiErrorParams: {rate_limit_info}, quotaLimits})`; other 429s
    are `rate_limit` too.
  - The query loop: `if (h?.isApiErrorMessage) { … return L2e(re ?? h, p), {reason: "api_error",
    errorKind: h.error, quotaLimits: h.quotaLimits} }`, where `L2e` is
    `executeStopFailureHooks`, with `error = e.error ?? "unknown"`. The turn ends; nothing holds it
    until the reset.
- **Discovery:**
  - `claude_events.rs`: `StopFailure` (`fold`, ~470) → `end_turn(State::Error)` and `ERROR_LABEL`
    = the error type; `seat_line` (292) gives `failed · <prompt>` for an error; `HookEvent.error`
    (102).
  - `agent_events.rs`: `on_frame` (275) folds a frame and calls `after_fold`; `apply_report`
    (159) puts a report's labels on the seat, called from `on_frame` for a held report and from
    `agent_reports.rs` (289) for a new one.
  - `resume.rs`: `continue_turn` (267) types `RESUME_PROMPT` with `terminal_drive::paste_then` and
    Enter while the seat is idle.
  - #715's code (`e8ea3e97b7`): `codex_events::{LIMITED_LABEL, LIMIT_RESETS_AT_LABEL, Limit,
    limit, limit_reset}`; `marley_rail::Limit` re-exports `codex_events::Limit`;
    `limits::{Limited, Arms, arm, cancel, forget, continue_now, on_seat}`, keyed by the terminal
    entity; `codex_server::continue_lead`; `rail.rs` `usage_of` (Codex only), `codex_line`,
    `limit_entry`, `InboxTarget::Limit { view, terminal, limited }`, the inbox's `limited` lookup
    through `generation_of` (Codex only).

### Design
- **`marley_agent::usage`** (new, Marley crate): `LIMITED_LABEL` (the failure's identity),
  `LIMITED_AT_LABEL` (its moment), `LIMIT_RESETS_AT_LABEL`, `Limit { turn, resets_at_ms }`,
  `limit(seat)` (a seat in `State::Error` with `LIMITED_LABEL`: the reset label when set, else the
  latest reset after `LIMITED_AT_LABEL` among the full windows), `limit_reset(labels, after_ms)`.
  `codex_events` uses them (its fold still sets the reset at the failure); `marley_rail` re-exports
  `usage::Limit`.
- **`claude_events`**: a lead `StopFailure` with `rate_limit` sets `LIMITED_LABEL` to its moment
  and `LIMITED_AT_LABEL`; `UserPromptSubmit` and `SessionStart` clear both. `seat_line` says
  `limited` for a limited seat.
- **`marley_workbench`**:
  - `limits.rs`: `Limited` gains `agent` and names the session (Codex's thread, Claude Code's
    session id); `Limited::of_claude(seat)`; an arm keeps the view's id and the terminal's weak
    handle; `send` types the prompt for Claude Code (`paste_then`, Enter) after checking the seat
    still reads the same limit, and goes through the follow for Codex; `on_claude_seat(view,
    terminal, seat)` for the frame and report paths.
  - `agent_events.rs` (`on_frame`) and `agent_reports.rs`: after a Claude Code seat moves, hand it
    to `limits::on_claude_seat`.
  - `codex_server.rs`: `on_seat` takes the view and the terminal.
  - `rail.rs`: `usage_of` reads `usage::limit` for any agent; the limited line's reset words come
    after `limited` for both agents; the inbox's limited lookup covers Claude Code (kind
    `Terminal`, its icon by name); `InboxTarget::Limit` keeps the terminal's weak handle.
  - The setting's words (Zed rows widened): `settings_content`'s doc, `default.json`'s comment,
    `marley_page.rs`'s description.
- **File manifest:** `crates/marley_agent/src/{usage.rs (new), marley_agent.rs, claude_events.rs,
  codex_events.rs}`; `crates/marley_rail/src/marley_rail.rs`;
  `crates/marley_workbench/src/{limits.rs, codex_server.rs, agent_events.rs, agent_reports.rs,
  rail.rs}`; `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `assets/settings/default.json` (Zed paths); `script/e2e/765-claude-codes-limited-state.sh`;
  `docs/marley/guide.md`, the in-app guide.
- **Ledger rows:** the three settings rows widened with the words' change. No new spawn site.

### Visual check plan
| REQ | The scenario sets up and does | The shot |
|---|---|---|
| 001, 002 | the stand-in `claude` started; `limited +80s` cued | `765-01-limited` |
| 003 | Continue at Reset clicked | `765-02-armed` |
| 004 | 85 seconds on | `765-03-continued`; the stand-in's log holds the prompt once |
| 005 | `done`; `limited +80s`; armed; a prompt typed; 85 seconds on | `765-04-dropped`; the log holds no second continuation |
| 006 | `done`; `limited-unknown` | `765-05-no-reset` |
| 007, 008 | none: the setting's path and Codex's are #715's, read in the diff | review |

What no scenario reaches: a real usage limit (it needs a spent account), and the frame and the
report landing in the other order (the reset is read from the labels each time, D2).

### Risks
- **The composer.** Marley types into Claude Code's prompt, which may hold text the user left; the
  continuation joins it (the spec's Out), as #522's and #750's deliveries do.
- **The report's timing.** With the shared plugin off, a Claude Code reports no windows: the row
  reads `limited` with no reset and the entry has no button.
- **A held report's overlay** (#652) sets the state from the report: a `rate_limit` frame with a
  held `idle` report would read idle. The shared plugin reports `error` for a turn an API error
  ended (`register.js` `turn.complete`), so a limited turn stays failed.

## Phase 2 — Code
- **Built.**
  - `marley_agent::usage` (new): `LIMITED_LABEL`, `LIMIT_RESETS_AT_LABEL`, `LIMITED_AT_LABEL`,
    `QUOTA_PREFIX`, `Limit`, `limit(seat)` (the fixed reset, else the latest full window's reset
    after `LIMITED_AT_LABEL`), `limit_reset(labels, after_ms)`. `codex_events` uses them as #715
    shipped; `marley_rail` re-exports `usage::Limit`.
  - `claude_events`: a lead `StopFailure` with `rate_limit` (`LIMIT_ERROR`) sets `LIMITED_LABEL`
    and `LIMITED_AT_LABEL` to the fold's moment; `UserPromptSubmit` and `SessionStart` clear
    `LIMIT_LABELS`; `seat_line` reads `limited`.
  - `limits.rs`: `Limited { agent, generation, session, turn, resets_at_ms }` with `of_codex` and
    `of_claude`, `LimitKey` (generation, session, turn) for staleness and the handled set; an
    `Armed` keeps the view's id and the terminal's weak handle; `send` goes through the follow for
    Codex and `type_continuation` (`paste_then`, Enter, after the seat check) for Claude Code;
    `arm`, `continue_now` and `on_seat` take the terminal and the view.
  - `agent_events::on_frame` and `agent_reports` hand a Claude Code seat to `limits::on_seat`;
    `codex_server`'s fold passes `Limited::of_codex`.
  - `rail.rs`: `usage_of` reads `usage::limit` for any agent; `with_limit_reset` puts the reset
    after `limited` for both agents (out of `codex_line`); the inbox finds a limited Claude Code;
    `limit_entry` names the agent and its kind; `InboxTarget::Limit` keeps the terminal's weak
    handle.
  - The setting's words on the three Zed paths, rows widened first.
- **Deviations from the design:** none in substance. `Limited`'s identity for the arms is a
  `LimitKey` that leaves out the reset, since a Claude Code's reset is read from labels a later
  report may change.
- **Review against the criteria.**
  - The click in Needs you runs inside the window's update (PR-claude-596), so the arm takes the
    terminal's weak handle from the entry, built outside it; `type_continuation` reads the seat by
    the view's id and needs no window lookup.
  - A held report's overlay (#652) would keep a reported state over a frame: the stand-in, like
    the shared plugin, reports `working` at each prompt and `error` at the limited turn's end.
  - `is_continuation` matches only the compaction's opening, so the typed "Continue where you
    left off." starts a turn and clears the limit.
  - REQ-007 (the setting arms a Claude Code): `on_seat` arms any `Limited` with a reset under
    `ContinueAtReset::Always`, for both agents. REQ-008: Codex keeps `of_codex`, the fixed reset
    label, the follow's checks and `turn/start`.
  - Clippy found `similar_names`, `needless_pass_by_value` and `needless_borrow`; all fixed.
- **Gate:** `script/gates.sh --diff` green, every gate and the receipt, after both guides.

## Phase 3 — Test
- **Scenario:** `script/e2e/765-claude-codes-limited-state.sh` under `compositor sway`, a stand-in
  `claude` cued through `claude.fifo` that prints hook frames and reports as the shared plugin
  does; typed lines go to `typed.log` and start a turn. Two checks, both pass: one continuation
  typed, and still one after the dropped arm.
- **Shots:**
  - `765-01-limited` (REQ-001, REQ-002): the row `limited · resets in 1 m · Refacto…` in the
    warning colour, `rate_limit` under it, `5 h 100 % · resets in 1 m`; Needs you "Claude Code ·
    repo", `Limited · resets in 1 m (17:06)`, Continue at Reset.
  - `765-02-armed` (REQ-003): `Continues at 17:06`, Cancel.
  - `765-03-continued` (REQ-004): the row `working · Continue where you l…`, the entry gone; the
    terminal shows the typed "Continue where you left off." and the stand-in's `user: Continue
    where you left off.`
  - `765-04-dropped` (REQ-005): armed, then "Fix the tests" typed: `working · Fix the tests`, no
    entry, and no second continuation in `typed.log`.
  - `765-05-no-reset` (REQ-006): `limited · Refactor the parser`, the line moved to `7 d 40 % ·
    resets in 2 d 23 h`; the entry `Limited` with no button.
- **Not driven:** REQ-007 and REQ-008 (review), a real usage limit, and the report landing before
  the frame (D2 reads the reset from the labels each time).
- **Fix from the shots:** the spec said the activity line keeps the turn's message; it shows the
  error type, as any failed Claude Code row does. The spec now says so; no code change.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md`; `docs/marley/guide.md` (Claude Code's limited state under Usage
  limits); the in-app guide's `#usage-limits`; `docs/marley_architecture/marley_workbench.md`
  (`src/limits.rs`), `marley_agent.md` (`src/usage.rs`), `marley_rail.md` (`usage::Limit`). The
  three Zed rows (59, 63, 68) name the setting's new words.
- **Knowledge:** L-claude-765-claude-code-ends-a-turn-at-its-usage-limit-001;
  AD-claude-765-claude-code-is-limited-on-rate-limit-and-continued-by-typing-001. No F: the
  review and the visual check found no bug.
- **Brain:** consultation f2bbc83b301740e787fcbed8fdebdebd closed with a decision (follow-up
  2026-11-11).
