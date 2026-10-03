---
pipeline_id: 95acc0ae-4ebb-4d42-aec1-2b57b9cb4494
ticket: docs/planning/tickets/open/TICKET-640-agent-state-source-progress-quota.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "A harness session's state source, progress and quota on its rail row"
type: feature
slice: prong 2, C1 (the harness's read side); rustal-harness MREQ-005 to MREQ-007, its D164
references: [docs/planning/design-notes/herdr-and-hermes-2026-10-02.md, docs/planning/pipeline/completed/534-harness-sessions-in-the-rail.spec.md, docs/marley/fleet-contract.md]
---

## Title
Marley reads three label conventions rustal-harness sends on its sessions (its MREQ-005 to
MREQ-007, D164) and shows them on each session's row in the rail's Harness section: where the
state came from (`state.source`), with a state read off a screen drawn weaker and kept out of
the approvals inbox; how far the agent is (`progress.percent`, `progress.activity`); and how much
of its account's quota each window has used and when it resets (`quota.KIND.percent_used`,
`quota.KIND.resets_at_ms`), tokens only.

## Scope
### In
- **The reading:** a pure reading of a session's labels in `marley_workbench::harness`:
  `state.source` (`protocol`, `reported`, `detected`, or any other word); `progress.percent` (a
  number from 0 to 100) and `progress.activity` (its first line); for each window KIND,
  `quota.KIND.percent_used` (0 to 100) and `quota.KIND.resets_at_ms` (epoch milliseconds); and
  `quota.account`, the account's name. A value that does not parse is left out.
- **Weaker:** a session whose source is `detected`, or a word Marley does not know, is drawn as
  the rail draws a stale row, for its state only: a muted dot, and the source's word at the end of
  its state line. Its title keeps its colour.
- **No approvals from a weaker state:** such a session waiting on a question shows the question on
  its row and gets no entry in the approvals inbox. A source of `protocol` or `reported`, or none
  (today's harness sends none), keeps #534's behavior.
- **Progress:** a line `42 % · Running the tests` under the state line, either part alone when only
  one is sent.
- **Quota:** a line for the window with the most used: its kind as sent, the percent, and the time
  until it resets (`five_hour` `62 % · resets in 1 h 35 m`).
- **The tooltip:** on a row that carries any of the labels, the state's source in words, every
  quota window most used first with its reset, and the account.
- **Redraws:** a change of the labels redraws the row (the harness's `upsert` already replaces
  them); the countdown is drawn again each minute while a reset lies ahead.
- **The row's height** follows its line count, so a row with a state, a progress and a quota line
  has room for three.
- `script/e2e/640-agent-state-source-progress-quota.sh`, with a stand-in `rh mcp` it writes.

### Out (explicitly deferred)
- **The Fleet panel and `marley.work/v1`.** Harness sessions do not reach the panel: it reads
  `marley.work/v1` stores, the pseudo provider and the host collector, and nothing in `fleet.rs`,
  `fleet_providers.rs` or `marley_sdk` reads `marley_fleet` sessions. Follow-up, one ticket: the
  optional `state_source`, `progress` and `usage.quota` on AgentList and AgentDetail in
  `marley_sdk` and `fleet-contract.md`, drawn on the Fleet panel's row and snapshot the same way.
- The session's tab (`HarnessView`) and an explain view for a detected state (herdr's `agent
  explain`); detection is the harness's (its TICKET-095).
- Marley's own terminals' rows: their typed state comes from Claude Code's hooks, and Marley
  builds no screen rules.
- A progress bar or quota colours (Orca's yellow at 60 % and red at 80 %); text only here.
- Answering a question from the inbox (C4); when it comes, a weaker state still gets no answer.
- The keys and the filter for the Harness section (still outside the rail's model, #534's D4).
- Marking MREQ-005 to MREQ-007 answered: that is the harness's file, in its repository.

## Reference (§20)
N/A — Marley-specific. The three labels are rustal-harness's convention for Marley's session
envelope (its MREQ-005 to MREQ-007 and D164), and the rows are Marley's Harness section (#534).
Warp has no counterpart: its managed agents run on Warp's servers, which the once-over ruled out,
as #534 records. No Zed crate draws an agent's state source, progress or quota. Upstream Zed is
used as #534 uses it: `context_server`'s MCP client, unchanged. The weaker drawing follows the
rail's own stale row (#534: a muted dot and a word).

### Prior art
- **Behavior maps.** The design note `herdr-and-hermes-2026-10-02.md`, Part 1: item 2, herdr's
  split between an agent that reports (the authority, screen rules off; herdr
  `src/detect/mod.rs:323-333` at `5d78d05`) and a detected state; item 3, `blocked` never comes
  from a fallback, and a detected state is "a weaker status on the rail row, never driving the
  approvals inbox". Orca's map, `docs/orca_architecture/01-agents-and-sessions.md` §2.12: a
  mini-bar for the tightest window and a percent per window, reset countdowns listed highest usage
  first, last numbers kept and marked stale; the row's most-used window and the tooltip's order
  come from it. `docs/warp_architecture/subsystems/04-agent-ai-mcp.md` and `docs/zed_architecture/`:
  nothing on an agent's quota, progress or state source.
- **Published material.** The harness's `docs/planning/MARLEY_REQUESTS.md` (MREQ-005 to MREQ-007),
  `docs/DECISIONS.md` D164, `docs/research/HERDR.md` feature 9 and `docs/FLEET.md` (the envelope's
  label table), read 2026-10-02. Claude Code's rate-limit windows (five-hour and seven-day, each
  with a percent and a reset) as MREQ-007 reads them from its mods API (`$.session.usage()`,
  2.1.287) and Orca's map from its `statusLine` `rate_limits`; not re-read for this draft. The MCP
  specification's `tools/call` with `structuredContent`, as #534 uses it.
- **Code we already ship.** `marley_workbench::harness` (#534, #632): the follow loop, `seed`,
  `answer`'s `resync_required`, `shown_prompt`, the minute bump; `rail.rs`: `harness_row` (the
  stale row: muted dot and title, `· stale`), `harness_entries`, `row_card` and `RowLine`, whose
  end slot is never cut; `marley_fleet::apply`, whose `Upsert` replaces a seat's labels; the
  rail's other weaker idioms, a `?` after a reading's guess (`marley_agent::stall`, `stop_kind`)
  and a dashed chip (`risk.rs`), not chosen since neither marks a row; the label constants of
  `marley_agent::claude_events` (#566's precedent: a label's convention lives with the crate that
  reads it); `marley_rail::waited_words` (`now`, `3 m`, `1 h 5 m`) and `fleet::how_long`;
  `ui::Indicator`, `ui::Tooltip`, `ui::ProgressBar` (the Fleet panel's meters, not chosen for a
  row). `marley_sdk`'s work types carry no source, progress or quota. Nothing in `Cargo.lock` is
  needed: the countdown is arithmetic on epoch milliseconds.

## UI proof
`script/e2e/640-agent-state-source-progress-quota.sh` (`compositor sway`: it rests the pointer on
a row for its tooltip). Setup: a stand-in `rh mcp`, a small stdio MCP server in Python the scenario
writes (as #633's stand-in `rusty-mcp`), answering `initialize`, `fleet_snapshot`, `fleet_events`
and `session_read` as the harness does, from a fixture file the scenario writes and later
rewrites; `marley.harness` names it. Six sessions: `build` (protocol, working, 40 % and an
activity, `five_hour` 62 % and `seven_day` 31 %, account `work`), `review` (reported, 75 % and an
activity, waiting on "Merge the branch?"), `watcher` (detected, working), `asker` (detected,
waiting on "Allow the edit?"), `plain` (no labels, idle), `odd` (source `guessed`, a percent of
140, a two-line activity, a quota percent of `lots`, an account with an `@`). Shots:
- `640-01-rows`: the six rows: watcher and asker weaker with `detected`, odd with `guessed`,
  build, review and plain as #534 draws them; asker's question on its row; build's progress and
  quota lines; odd with its activity's first line and no percent or quota;
- `640-02-inbox`: the inbox with review's question and not asker's;
- `640-03-tooltip`: the pointer on build's row: its source, both windows most used first, the
  account;
- `640-04-moved`: the fixture rewritten (build at 80 % with a new activity and `five_hour` at
  70 %; asker's source `reported`): build's new values, asker drawn declared, and its question in
  the inbox.

## Locked-In Decisions
- D1 — Labels, as D164 settled: the harness keeps serving the pinned `marley_fleet` envelope, so
  Marley reads `Session.labels` and adds no envelope field.
- D2 — The reading lives with its reader, in `marley_workbench::harness`, as #566's labels live in
  `marley_agent`; `marley_fleet` names no label (its charter: no substrate's vocabulary).
- D3 — Only `protocol` and `reported` are declared. `detected`, and any word Marley does not know,
  is drawn weaker and enters no inbox: Marley cannot vouch for a source it does not know. No label
  keeps #534's behavior, since the harness serves declared state only today.
- D4 — Weaker is the stale row's treatment on the state alone: a muted dot and a word in the
  line's end slot. The title keeps its colour, since the session is live.
- D5 — One line for progress and one for quota, each only when sent; the row shows the most-used
  window, and the tooltip carries every window, the source and the account.
- D6 — Values are checked: a percent is a finite number from 0 to 100, rounded to a whole one; a
  reset is an integer of epoch milliseconds; an activity is its first line. Anything else is left
  out, never guessed.
- D7 — Tokens only, money out (fleet contract, Settled 3): of `quota.` Marley reads each window's
  `percent_used` and `resets_at_ms`, and `account`; every other `quota.` key is ignored. An account
  name with an `@` is not shown. A window's kind is shown as sent, never mapped.
- D8 — The countdown reads `marley_rail::waited_words`'s words under a day (`now`, `35 m`,
  `1 h 35 m`) and `3 d 4 h` beyond; a reset already past reads `resets now`.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a harness session's `state.source` is `detected`, the rail shall draw its row's dot muted and the word `detected` at the end of its state line. | Shot `640-01-rows` |
| REQ-002 | WHILE a session's `state.source` is a word other than `protocol`, `reported` or `detected`, the rail shall draw its row as a detected one, with that word in place of `detected`. | Shot `640-01-rows` |
| REQ-003 | WHILE a session's `state.source` is `protocol` or `reported`, or absent, the rail shall draw its state as #534 draws it. | Shot `640-01-rows` |
| REQ-004 | WHILE a session drawn weaker waits on a question, the rail shall show the question on its row and the approvals inbox shall hold no entry for it. | Shots `640-01-rows`, `640-02-inbox` |
| REQ-005 | WHEN a waiting session's `state.source` becomes `protocol` or `reported`, the rail shall draw it as declared and list its question in the inbox within five seconds. | Shot `640-04-moved` |
| REQ-006 | WHILE a session carries a `progress.percent` from 0 to 100, its row shall show the percent as a whole number on a line of its own. | Shot `640-01-rows` |
| REQ-007 | WHILE a session carries `progress.activity`, its row shall show the activity's first line after the percent. | Shot `640-01-rows` |
| REQ-008 | WHILE a session carries one or more quota windows, its row shall show the most-used window's kind, its percent used and the time until it resets. | Shot `640-01-rows` |
| REQ-009 | IF a progress or quota percent is not a number from 0 to 100, THEN the rail shall leave that value out and draw the rest of the row. | Shot `640-01-rows` |
| REQ-010 | WHEN the pointer rests on a row whose session carries any of the labels, the rail shall show a tooltip naming the state's source, every quota window most used first with its reset, and the account. | Shot `640-03-tooltip` |
| REQ-011 | WHEN the harness publishes new progress or quota values for a session, its row shall show them within five seconds. | Shot `640-04-moved` |
| REQ-012 | The rail shall show no `quota.` label other than a window's percent used and reset, and no account name that contains `@`. | Review |
| REQ-013 | WHILE a shown reset lies ahead, the rail shall draw its countdown again at least once a minute. | Review |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes. At promotion: re-read the harness's
  `MARLEY_REQUESTS.md`, `FLEET.md` and TICKET-091 for the label names and the account's key;
  `brain_ask`.
- **P2 Code** — the reading in `harness.rs`; the row's lines, mark and tooltip, the inbox's rule
  and the row's height in `rail.rs`; the minute bump; a review of the diff; `script/gates.sh
  --diff` green.
- **P3 Test** — the visual check: write the stand-in and the scenario, run it, read every shot.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md` (the labels the
  harness module reads), the guide's harness section, the plan's C1 row, `fleet-contract.md`'s
  Later (the three optional fields), ledger capture (§19), close the ticket, archive, commit.
