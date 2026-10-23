# An agent's answers as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-914-thread-answers-as-the-board.md
- **Pipeline spec:** 914-thread-answers-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 5 (the agent thread).
- **Classification / tier:** feature, M. Mostly `agent_ui` hunks under Marley's hooks, one new
  Marley file.
- **Recall (§18.3):**
  - F-claude-865-a-hook-called-from-a-render-read-its-own-view-001: a Marley function called from
    the thread view's render never reads that view; `answer_header` takes an `AnswerFacts` value.
  - #864's notes: `MarleyThreadLook` is a `Copy` struct of function pointers set in
    `search_looks.rs`; adding a field means naming it there.
  - #825's notes: the bubble is a Marley hunk around Zed's entry editor; editing a past message
    goes through that editor, so it must stay an editor.
  - L-claude-828-a-theme-check-takes-the-theme-key-out-and-switches-it-live-001: the Day shot
    switches the theme live in the same run.
  - F-claude-903-a-stand-in-not-on-the-terminals-path-001: nothing here starts a CLI agent; the
    scripted ACP agent runs through `MARLEY_ASSISTANT_ADAPTER`.
- **Discovery:**
  - `crates/agent_ui/src/conversation_view/thread_view.rs`: `MarleyThreadLook` (:574),
    `marley_card_label` (:583), `render_entry` (:6906), the RUN slot (:8837), the EDIT label
    (:11007), `render_marley_chips` (:7771), `render_send_button` (:6126),
    `render_marley_text_width` (:6428), `turn_fields` (:698), `current_model_name` (:12497).
  - `crates/agent_ui/src/message_editor.rs:2057-2080` (the text style) and the entry editor's
    constructor in `entry_view_state.rs`.
  - `crates/agent_ui/src/agent_ui.rs:523` (`MarleyThreadHost`, for the "Marley hosts" check).
  - `crates/marley_workbench/src/search_looks.rs:35`, `home_page.rs:925` (`cli_glyph`).
- **Decisions:** D1–D5 in the spec.

### Design
- `AnswerFacts { agent_id: SharedString, agent_name: SharedString, model: Option<SharedString>,
  effort: Option<SharedString>, state: Option<AnswerState> }` and `enum AnswerState { Working {
  elapsed: Duration }, NeedsYou }`, public in `thread_view.rs` and re-exported from `agent_ui`
  beside `MarleyThreadLook`. `MarleyThreadLook` gains `answer_header: fn(&AnswerFacts, &App) ->
  AnyElement`.
- In `render_entry`, `is_answer_start` = top-level, not a user message, and the previous entry is
  a user message (or none). The header goes above `primary` in the same `v_flex`. The newest
  answer is the one after the last user message; it gets a state while `thread.status()` is
  `Generating` (`NeedsYou` when a tool call waits for authorization).
- `marley_card_label(kind: CardLabel, cx)` with `enum CardLabel { Edit, Run }` picks the fill
  and ink; the RUN slot adds the result chip from the terminal's exit status; the edit header adds
  the counts from the call's first diff.
- `marley_workbench/src/answer_header.rs` (Marley crate, new): `pub(crate) fn answer_header(facts:
  &AnswerFacts, cx: &App) -> AnyElement`; the glyph from the agent id (Claude Code's and Codex's
  ACP ids, Marley's assistant id, `agent::ZED_AGENT_ID`), the elapsed time as "N s" under a
  minute and "N min" after (`marley_terminal::duration_label` if it reads so, else its own).
- Ledger rows in `docs/marley/zed-touchpoints.md`: `thread_view.rs`'s row gains the header, the
  filled labels with the result chip and counts, the pill chips, the send square and the text
  button (#914); a new row for `message_editor.rs` (the entry editor's UI font while Marley hosts
  threads). The `agent_ui.rs` row names the re-export.
- File manifest: Zed crate: `thread_view.rs`, `message_editor.rs`, `entry_view_state.rs`,
  `agent_ui.rs`. Marley crate: `answer_header.rs` (new), `search_looks.rs`,
  `marley_workbench.rs` (the module).

### Visual check plan
- REQ-001: the prompt sent; settle two seconds; `914-01-working`; read the tile, name, model and
  the "Working · N s" state.
- REQ-001 to REQ-006: settle until the answer ends; type a word in the message box;
  `914-02-answer`; read the header (no state), the EDIT label's fill and counts, the RUN label's
  fill, "passed" and time, the chips' shape, the bubble's font and fill, the send square and
  "Full width".
- REQ-005: click "Full width"; `914-03-column`.
- All in Day: switch the theme live; `914-04-day`.

### Risks
- The scripted agent may not report a model; then the header shows the name only. Add a model to
  the scripted agent's `session/new` answer (ACP's session models) in the scenario.
- A thread that opens with a tool call and no text still needs its header above the call: the
  rule keys on the entry after the user message, whatever its kind.
- The UI font in the entry editor changes its line height; check a long message wraps inside the
  75 % bubble in the shot.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
