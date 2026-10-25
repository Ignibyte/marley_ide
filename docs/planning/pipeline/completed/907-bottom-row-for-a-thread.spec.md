---
pipeline_id: e4905be0-9ae2-4426-bade-f69ceb8e77cd
ticket: docs/planning/tickets/open/TICKET-907-bottom-row-for-a-thread.md
status: Phase 4 — Complete PASS
title: The bottom row for a thread
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/A-core.md, docs/marley/design-review-2026-10-23/B-rail-chrome.md]
---

## Title
When a thread tab is in front, the bottom row says what that thread is and does, as the Main and
BottomRow boards draw it: the agent's tile and name, model and effort, mode, what the agent does
or asks with how long, a context meter, the files it changed (+/−), and a Full width toggle.
Today the row shows the agent's name, a status word and the title only (`focus_row.rs:197`).

## Scope
### In
- `crates/marley_workbench/src/thread_activity.rs` (Marley crate, new): `ThreadActivity`
  (named fields: the status, what the agent does or asks, and since when) and
  `thread_activity(conversation, cx)`, read from the root thread view. Running: the label of the
  newest tool call still pending or in progress, else "thinking"; since the turn's start.
  Waiting: the pending tool call's label, as the inbox words it (`thread_entry`,
  `rail.rs:9294`), since the turn's start. Done: "N files changed" when the action log holds
  changes, else "idle". Error: "failed". The one helper for this; #910 reuses it for the rail's
  thread rows, so it lives at the crate's root module, where `rail.rs` reaches it too.
- `crates/marley_workbench/src/focus_row.rs` (Marley crate): `render_thread` draws, in order:
  - the agent tile (#905's per-kind tint and agent glyph: CC, CX, M, R) and the agent's name;
  - the model and its effort ("<model> · <effort>"), effort only when the agent reports one;
  - the mode's name as the agent reports it;
  - the activity chip: a state dot and "<state word> · <what> · <elapsed>" on the state's
    background (working amber, needs-you pink, failed red, idle muted), its text capped;
  - "Context <bar> <used> of <max>", when the agent reports token use;
  - "<N> files · +<added> −<removed>", when the action log holds changes;
  - a Full width toggle.
  The thread's title leaves the row (the tab carries it).
- `focus_row.rs` `set_active_pane_item`: for a thread tab, also observe the root `ThreadView`
  (its one-second turn timer ticks the elapsed time), its `AcpThread` (token use, tool calls) and
  its `ActionLog` (changed files), as `ThreadTab::follow_details_thread` does.
- `crates/marley_workbench/src/thread_tab.rs` (Marley crate): the details panel's changed-file
  read and `humanize` become `pub(crate)` helpers both the panel and the row call, so the two
  agree.
- `crates/agent_ui/src/conversation_view/thread_view.rs` (Zed crate hunk):
  `ThreadView::marley_session_facts(&self, cx) -> MarleySessionFacts`, a struct of three
  optional names: the model (the model selector's active model, or the native thread's model), the
  effort (the native thread's thinking effort, or the agent's thought-level config option) and
  the mode (the mode selector's mode, the native thread's profile, or the agent's mode config
  option).
- `crates/agent_ui/src/config_options.rs` (Zed crate hunk): `ConfigOptionsView::value_name_of(category)`,
  the current value's name of the first option in a category, which `current_value_name` already
  computes for one selector.
- `crates/agent_ui/src/mode_selector.rs` (Zed crate hunk): `ModeSelector::mode_name`, the lookup
  its `render` already does.
- The Full width toggle flips `agent.limit_content_width` in the user's settings through
  `update_settings_file`, as #822's button in the composer does; it is lit while the text runs the
  full width.

### Out (explicitly deferred)
- The bottom row for an agent CLI in a terminal (AgentTerminal board: agent chip, model,
  "bypass", the waiting chip with "Enter runs", Turns (N), context): its facts come from the
  seat's hook events, not from an ACP thread. A follow-up ticket.
- "Act · bypass" as two words: the board joins the Plan / Act switch with the permission mode.
  The Plan / Act switch is its own ticket (the agent thread, build order 5); until then the row
  shows the mode's name as the agent reports it.
- The per-kind tint and the agent glyphs themselves: #905 lands them in `crates/marley_workbench/src/item_kind.rs`; this ticket reuses them.
- The rail's thread row line: #910, on `thread_activity`.
- A thread in the Agent Panel (not a center tab): the focus item follows the active pane item,
  and the panel is not one. Unchanged.
- The answer's own header (agent, model, effort, state at the right; A-core Main): the agent
  thread ticket (build order 5).

## Reference (§20)
Upstream Zed (`agent_ui`): the facts are the ones Zed's thread view already shows in its
composer: the model selector's model, the thinking-effort selector, the mode selector or the
agent's config options, the turn timer (`TurnFields::turn_started_at`), the token use
(`AcpThread::token_usage`) and the action log's changed buffers. The row keeps Zed's values and
words for them and only draws them in one more place. The layout is Marley-specific (the Main
and BottomRow boards).

### Prior art
- `focus_row.rs:197` `render_thread`: name, `marley_rail::thread_status` word, title. Gap holds
  at HEAD. `set_active_pane_item` (`focus_row.rs:335`) observes only the `ConversationView`.
- `thread_tab.rs:697` `render_details` (#825, #864) already reads the model id
  (`current_model_id`), `AcpThread::token_usage`, `action_log().changed_buffers` with
  `action_log::DiffStats::single_file`, and draws the context bar (`context_section`,
  `thread_tab.rs:963`) with `humanize` (`thread_tab.rs:1069`). `follow_details_thread`
  (`thread_tab.rs:673`) observes the thread and its action log: the pattern the row copies.
- `thread_view.rs:698` `TurnFields` (pub): `turn_started_at`, set by `initialize_turn`
  (`thread_view.rs:1480`), which notifies the view every second while a turn runs, and by
  `sync_reported_activity` from `AcpThread::activity_started_at` for agents that report activity.
- `thread_view.rs:1282` `current_model_id` gives the id, not the name; `model_selector_popover.rs:35`
  `active_model` gives `AgentModelInfo` with its name. `thread_view.rs:5840`
  `render_thinking_control` reads `thinking_effort` on native threads only.
  `config_options.rs:395` `current_value_name` and the `ThoughtLevel`, `Model`, `Mode`
  categories (`config_options.rs:506-528`) cover ACP agents that send config options.
  `mode_selector.rs:133` finds the mode's name in its `render`.
- `thread_view.rs:6431` `render_marley_text_width` (#822): flips `agent.limit_content_width`
  through `update_settings_file`. The board's Full width (`Main.dc.html`: `aria-label="Text
  width: full"`) is the same toggle.
- `rail.rs:9294` `thread_entry`: the waiting tool call's label through `pending_tool_call` and
  `one_line`; `marley_rail.rs:444` `thread_status`; `marley_rail.rs:623` `waited_words`.
- The boards: `Main.dc.html` and `BottomRow.dc.html` (agent chip, "Opus 5.5 · High",
  "Act · bypass", the working chip "Working · editing thread_view.rs · 12 min", "Context" bar
  "48k of 200k", "3 files · +35 −2", "Full width").
- docs/zed_architecture/ and docs/warp_architecture/: nothing on a status-bar thread summary.
  No crate owns this seam beyond `agent_ui` and `acp_thread`, read above.

## UI proof
Scenario `script/e2e/907-bottom-row-for-a-thread.sh` (sway). Setup: #864's scripted ACP agent
as `MARLEY_ASSISTANT_ADAPTER`, its `session/new` answer extended with config options (a model, a
thought level, a mode) and its prompt handling sending a `usage_update`, an in-progress
`tool_call` "Edit notes.md" and an `fs/write_text_file`, then holding the turn until the
scenario touches a file; a second prompt asks for permission (#508's stand-in shape). No real
agent CLI on the PATH.
- `907-01-working`: the thread in front, mid-turn: tile and name, model · effort, mode, the
  working chip with "Edit notes.md" and its time, the context meter, "1 file · +N −M", Full width
  (REQ-001 to REQ-006).
- `907-02-ticked`: the same, seconds later: the elapsed time moved (REQ-004).
- `907-03-waiting`: the permission prompt pending: the chip in the needs-you color with the ask
  (REQ-004).
- `907-04-done`: the turn ended: the chip reads idle with the files changed (REQ-004, REQ-006).
- `907-05-full-width`: Full width clicked: the thread's text runs the full width and the toggle
  is lit (REQ-007).
- `907-06-day`: `907-01-working` in Marley Day (REQ-008).

## Locked-In Decisions
- D1 — The row draws only what Zed's thread view already knows (model, effort, mode, turn timer,
  token use, action log); it computes nothing new about the agent.
- D2 — `thread_activity` is the one source of "what the agent does or asks, and since when";
  #910's rail row reads the same helper, so the row and the rail never disagree.
- D3 — Full width flips the same setting as #822's composer button; there is one width setting,
  not two.
- D4 — A fact the agent does not report is left out, not shown as "unknown" (effort for a model
  without one, context for an agent that sends no usage).
- D5 — The thread's title leaves the row: the board does not draw it, and the tab carries it.
- D6 — The row keeps `flex_none` and caps the activity text with `max_w` and a truncating label
  (PR-claude-833).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a thread tab is in front, the bottom row shall lead with the agent's tile, tinted and lettered by its agent, and the agent's name. | `907-01-working` |
| REQ-002 | WHILE a thread tab is in front, the bottom row shall show the model's name, followed by its effort when the agent reports one. | `907-01-working` |
| REQ-003 | WHILE a thread tab is in front and the agent reports a mode, the bottom row shall show the mode's name. | `907-01-working` |
| REQ-004 | WHILE a thread tab is in front, the bottom row shall show a chip with a state dot and what the agent does or asks, in the state's color, with the time since the turn started while it works or waits. | `907-01-working`, `907-02-ticked`, `907-03-waiting`, `907-04-done` |
| REQ-005 | WHILE a thread tab is in front and the agent reports its token use, the bottom row shall show "Context", a bar and "<used> of <max>". | `907-01-working` |
| REQ-006 | WHILE a thread tab is in front and the thread has changed files, the bottom row shall show the file count and the lines added and removed. | `907-01-working`, `907-04-done` |
| REQ-007 | WHEN the user clicks Full width in the bottom row, the system shall flip `agent.limit_content_width`, and the toggle shall show lit while the text runs the full width. | `907-05-full-width` |
| REQ-008 | The thread's bottom row shall draw in Marley Day as in Marley Night. | `907-06-day` |
| REQ-009 | The Zed hunks shall be accessors only, each with a `// Marley:` comment and a row in `docs/marley/zed-touchpoints.md`. | diff review |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `thread_activity.rs`, `focus_row.rs`, the shared helpers in `thread_tab.rs`,
  the three Zed accessors and their ledger rows; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
