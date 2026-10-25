# The bottom row for a thread — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-907-bottom-row-for-a-thread.md
- **Pipeline spec:** 907-bottom-row-for-a-thread.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 2 (the bottom row says what is in front), the thread part.
- **Classification / tier:** feature, M. Marley crate (`marley_workbench`) plus three small
  accessor hunks in a Zed crate (`agent_ui`). Build order 906, 905, 907, 908, 909, 910, 911:
  #905's tint and agent glyph land first; #908 and #909 add the other screens to the same
  `Focus` enum after this one; #910 reads `thread_activity`.
- **Recall (§18.3):**
  - F-claude-833 and PR-claude-833: a status item keeps `flex_none` and caps its long parts; check
    it beside Zed's language-server message.
  - PR-claude-831: a status item reads nothing of the workspace in `new`.
  - F-claude-865: a Marley function a Zed view calls from its render never reads that view. Not
    this shape (the row reads the thread view from its own render), but the accessors take
    `&self` and read only the view's own fields and selectors.
  - F-claude-893: a toolbar item never reads its workspace in `set_active_pane_item`. The row's
    `set_active_pane_item` reads only the item and the conversation it holds.
  - L-claude-825 and #864's scenario: a scripted ACP agent writes a file through
    `fs/write_text_file`, so the action log holds it. L-claude-508: a stand-in agent asks for
    permission.
  - F-claude-903: stand-ins first on the terminals' PATH. This scenario needs no terminal agent;
    the scripted agent comes in through `MARLEY_ASSISTANT_ADAPTER`.
  - L-claude-552: a read that redraws must redraw only on a change. The row observes the thread
    view, whose turn timer notifies each second only while a turn runs.
- **Discovery:**
  - `crates/marley_workbench/src/focus_row.rs`: `render_thread` (197), `set_active_pane_item`
    (335), `chip`, `muted`.
  - `crates/marley_workbench/src/thread_tab.rs`: `render_details` (697), `follow_details_thread`
    (673), `context_section` (963), `humanize` (1069).
  - `crates/marley_workbench/src/rail.rs`: `thread_entry` (9294), `one_line` (9349),
    `RowState::dot` and `line` (11014-11030).
  - `crates/agent_ui/src/conversation_view/thread_view.rs`: `TurnFields` (698), `current_model_id`
    (1282), `current_mode_id` (1288), `render_thinking_control` (5840), `render_marley_text_width`
    (6431).
  - `crates/agent_ui/src/config_options.rs`: `current_value_name` (395).
  - `crates/agent_ui/src/mode_selector.rs`: the name lookup in `render` (133).
- **Decisions:** D1 to D6 in the spec. Open points settled by default: the mode shows as the agent
  names it until the Plan / Act ticket; the activity text is capped at about 320 px, as the waiting
  chip is (`ASK_WIDTH`).

- **Promoted 2026-10-25:** seams re-read after #905: `focus_row.rs` `render_thread` and
  `set_active_pane_item` unchanged; `thread_tab.rs` `follow_details_thread` :673,
  `render_details` :697, `humanize` :1069; `thread_view.rs` `TurnFields` :699 (pub, the view's
  `turn_fields` pub), `current_model_id` :1282, `render_thinking_control` :5841 (native effort
  from `LanguageModelEffortLevel { name, value }`); `config_options.rs` `current_value_name`
  :395 (on `ConfigOptionSelector`), `find_option_name` :1105; `mode_selector.rs` `render` :133.
  #905's `item_kind::AgentMark` gives the tile.

### Design
- `thread_activity.rs` (Marley crate, new): `pub(crate) struct ThreadActivity { status:
  marley_rail::ThreadStatus, doing: SharedString, since: Option<Instant> }` with accessors, and
  `pub(crate) fn thread_activity(conversation: &Entity<ConversationView>, cx: &App) ->
  Option<ThreadActivity>`. It walks the root thread's entries from the end for the newest
  `ToolCall` whose status is pending or in progress; uses `pending_tool_call` for a wait; reads
  `turn_fields.turn_started_at`; counts `changed_buffers` for done. Elapsed words: under a minute
  "N s", then "N min", then "N h M min", in one function here (#910 shows the same words).
- `thread_tab.rs` (Marley crate): `pub(crate) struct ChangedFile { name, added, removed, path }`
  from the details panel's tuple (named fields, per the style section), `pub(crate) fn
  changed_files(thread, cx) -> Vec<ChangedFile>`, and `humanize` made `pub(crate)`.
- `focus_row.rs` (Marley crate): `Focus::Thread` keeps the conversation; `item_events` becomes a
  small `Vec<Subscription>` (conversation, root thread view, thread, action log). `render_thread`
  draws the parts in the board's order. The tile comes from #905's shared helper. The Full width
  toggle is a `Button` with `toggle_state`, whose click calls `settings::update_settings_file` with
  the project's `Fs`.
- `thread_view.rs` (Zed crate): `pub struct MarleySessionFacts { pub model: Option<SharedString>,
  pub effort: Option<SharedString>, pub mode: Option<SharedString> }` and `pub fn
  marley_session_facts(&self, cx: &App) -> MarleySessionFacts`.
- `config_options.rs` (Zed crate): `pub fn value_name_of(&self, category:
  &acp::SessionConfigOptionCategory) -> Option<String>`.
- `mode_selector.rs` (Zed crate): `pub fn mode_name(&self) -> Option<SharedString>`; `render`
  calls it, so the lookup is not repeated.
- Ledger rows in `docs/marley/zed-touchpoints.md`: extend the `thread_view.rs` row with
  `marley_session_facts`; new rows for `config_options.rs` (`value_name_of`) and `mode_selector.rs`
  (`mode_name`), each "an accessor the bottom row reads; keep the method at a merge".

### Visual check plan
- REQ-001, REQ-002, REQ-003, REQ-005, REQ-006: `907-01-working`. The scripted agent's
  `session/new` returns config options (model, thought level, mode); its first prompt sends
  `usage_update`, an in-progress tool call "Edit notes.md" and a file write, then waits on a flag
  file. The shot reads the row's parts left to right.
- REQ-004: `907-01-working` and `907-02-ticked` (a few seconds apart, the time moved);
  `907-03-waiting` (a second prompt asks for permission; the chip turns needs-you with the ask);
  `907-04-done` (the flag released, the turn ended; idle with the file count).
- REQ-007: `907-05-full-width`. A click on Full width; the thread's text widens and the toggle is
  lit. The scenario starts with `agent.limit_content_width` true.
- REQ-008: `907-06-day`. The theme switched live to Marley Day (L-claude-828), the working state
  held.
- REQ-009: the diff review.
- Not reachable in a scenario: a real Claude Code session's model names. The scripted agent stands
  in, as in #825 and #864.

### Risks
- The scripted agent's config options: Zed shows a config options view in place of the mode
  selector when an agent sends them (`thread_view.rs:5148`). If the ACP version the stand-in
  speaks does not carry them, fall back to `modes` and `models` in `session/new` for mode and
  model, and record effort as seen only on a native thread.
- The row grows wide. With a long model name and the language-server message it can crowd the
  bar; each part truncates and the row is checked beside that message (PR-claude-833).
- Observing the thread view redraws the row each second during a turn. The row is small; if the
  profile shows cost, observe only while the status is running.

## Phase 2 — Code
- **Built:** `thread_activity.rs`; `focus_row.rs` (`render_thread`, `follow_thread`,
  `render_full_width`, `activity_chip`, `context_meter`, `files_changed`); `thread_tab.rs`
  (`ChangedFile`, `changed_files`, `humanize` shared); `item_kind::tile` moved out of
  `tab_tiles.rs`; Zed hunks: `MarleySessionFacts` and `marley_session_facts` (`thread_view.rs`,
  re-exported from `agent_ui.rs`), `ConfigOptionsView::value_name_of`, `ModeSelector::mode_name`
  (which `render` now calls). Ledger rows written first.
- **Deviations:** `follow_thread` is generic over the view type: `ThreadView` has no export. The
  row's needs-you part leaves out the front thread's own request
  (F-claude-907-the-row-repeated-the-front-threads-wait-001).
- **Review:** the row reads the conversation, its thread view, thread and action log in its own
  render, never the workspace in an update; each Zed hunk is an accessor with a `// Marley:`
  comment (REQ-009). Google style: `ChangedFile` with named fields replaces the details panel's
  four-field tuple; `MarleySessionFacts` has named fields.
- **Gate:** `just gate-diff` 17/17 green.

## Phase 3 — Test
- **Scenario:** `script/e2e/907-bottom-row-for-a-thread.sh` (sway): Marley's entry on a scripted
  ACP agent with model, thought-level and mode config options, a usage update, a client write of
  notes.md, an "Edit notes.md" tool call held open on a flag file, then a permission request.
- `907-01-working`: `M` tile, "Marley", "Opus 5.5 · High", "Act", "Working · Edit notes.md · 6 s"
  on amber, Context bar "48k of 200k", "1 file · +4 −0", Full width (REQ-001 to REQ-006).
- `907-02-ticked`: "· 14 s" (REQ-004).
- `907-03-done`: "Idle · 1 file changed" on grey (REQ-004, REQ-006).
- `907-04-waiting`: "Needs you · Edit README.md · 5 s" in pink; no repeated request after Full
  width (REQ-004).
- `907-05-full-width`: the text runs the window's width, Full width lit (REQ-007).
- `907-06-day`: Marley Day (REQ-008).
- The first full run showed the request twice (fixed, above). The details panel still reads "its
  default model" for an agent that names its model through config options; it reads
  `current_model_id`, which this ticket left alone.

## Phase 4 — Complete
- CHANGELOG, `docs/marley_architecture/marley_workbench.md`, three ledger rows.
- Knowledge: F-claude-907-the-row-repeated-the-front-threads-wait-001.
