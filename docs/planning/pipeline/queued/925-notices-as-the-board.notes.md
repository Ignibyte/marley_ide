# Toasts, the plan entry and notifications as the board words them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-925-notices-as-the-board.md
- **Pipeline spec:** 925-notices-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 11 (Inbox).
- **Classification / tier:** feature, S–M. Marley crates, plus two small additive hunks in
  `workspace`.
- **Recall (§18.3):**
  - AD-claude-869-toasts-split-one-string-001: a toast is drawn from its one string through
    `MarleyToastLook`; this ticket adds a kind beside the string and keeps the split.
  - AD-claude-752-a-push-follows-each-sources-own-decision-and-four-kinds-switch-it-001 and
    #535's spec: a push carries nothing the agent wrote, and four kinds switch it; the new
    wording keeps both.
  - F-claude-752-a-reporting-codex-pushed-as-claude-001: the agent's name comes from the seat
    (`push::agent_name`), never guessed.
  - F-claude-587-the-answers-toast-shared-the-questions-notification-id-001: a toast's id decides
    what replaces what; kinds do not touch ids.
  - F-claude-903-a-stand-in-not-on-the-terminals-path-001 and PR-claude-687: the stand-in Claude
    Code goes first on the terminals' PATH through the `.bashrc`.
  - L-claude-828-a-theme-check-takes-the-theme-key-out-and-switches-it-live-001 for the Day shot.
- **Discovery:**
  - `crates/workspace/src/workspace.rs:778` `Toast`; `crates/workspace/src/notifications.rs:75–84`
    `MarleyToastLook`, `MarleyToast`, `:209` `show_toast`.
  - `crates/marley_workbench/src/toasts.rs` (`look`, `split`).
  - The 67 `Toast::new` calls in `crates/marley_workbench/src` and its `rusty/` folder.
  - `crates/marley_workbench/src/plan_review.rs`: `Pending` (99), where a plan arrives (the
    `PendingPlans` push), `entry_of` (365).
  - `crates/marley_workbench/src/rail.rs`: `render_inbox_buttons` (2536–2590).
  - `crates/marley_workbench/src/command_watch.rs` (100–155); `notifications.rs` `notify` (206).
  - `crates/marley_workbench/src/push.rs` `push` (137), `post` (237); `crates/marley_agent/src/
    marley_agent.rs` `event_line` (865).
- **Decisions:** D1–D5 in the spec.

### Design
- `workspace`: `pub enum MarleyToastKind { Warning, Success, Error, Question, Neutral }` in
  `notifications.rs` beside `MarleyToast`; `Toast` gains `marley_kind: Option<MarleyToastKind>`
  (`None` in `new`) and `pub fn marley_kind(mut self, kind) -> Self`; `MarleyToastLook`'s function
  takes `(&str, Option<MarleyToastKind>, &App)`. Each hunk carries a `// Marley:` comment.
- `toasts.rs`: `look` maps the kind (given, else `kind_of_words(message)`) to the dot's `Color`.
  `kind_of_words` returns Error for the failure words, Question for a title ending in "?", else
  Neutral. The module doc comment says so.
- Call sites: `close_guard.rs` (Warning, with its Undo); success reports (capture, token written,
  saved, copied) get Success; the Code phase lists each site it changed in Phase 2.
- Plan entry: `Pending` gains `title: String`, set from the request's plan text by
  `plan_title(text)` (first `#` heading, else the first non-empty line, cut at 60 characters with
  "…"). `entry_of` writes the ask with it. In `rail.rs`, a `InboxKind::Plan` entry with no answers
  draws `Button::new(…, "Open the plan").style(ButtonStyle::Filled)` in the accent, calling
  `open_inbox_entry` as Open does.
- Desktop notification: `Tell::Finished` builds the title `format!("{command} finished · exit
  {code}")` and the body from the terminal's `tab_content_text` and `where_you_were(window, cx)`:
  the active item's tab text when the window is active and the item is not this terminal.
  `notify` already takes the title.
- Push: `marley_agent::PushText { title: String, body: String }` and `push_text(project, agent,
  event)`; `post` sets the `Title` header from `title` and the body from `body`; control
  characters filtered as `event_line` does.
- File manifest:
  - Zed crate: `crates/workspace/src/workspace.rs`, `crates/workspace/src/notifications.rs`.
  - Marley crate: `toasts.rs`, the toast call sites, `plan_review.rs`, `rail.rs`,
    `command_watch.rs`, `push.rs`, `crates/marley_agent/src/marley_agent.rs`.
- Ledger: the `crates/workspace/src/notifications.rs` row in `docs/marley/zed-touchpoints.md`
  gains `MarleyToastKind` and the kind argument of `MarleyToastLook` (#925); a
  `crates/workspace/src/workspace.rs` row (or its existing one) gains `Toast::marley_kind` and its
  field. Put back: keep the field `None` wherever upstream builds a `Toast`.

### Visual check plan
- REQ-001: from the palette, `marley: toggle dictation` with voice off (an error, "Could not…" or
  its refusal); the stand-in Claude Code's terminal closed while it works (the Warning with Undo);
  `marley: copy …` or Copy notes (a Success); `workspace: use classic layout` (#869's neutral
  one). `925-01-toasts` shows the four dots. The Code phase confirms which palette entries raise
  each kind and records them.
- REQ-002: the stand-in's "plan" asks a plan whose first line is "# Tidy the notes";
  `925-02-plan` shows the entry; a click on Open the plan, then `925-03-opened`.
- REQ-003: terminal 2 holds the focus with a file open in front; terminal 1 runs `sleep 3; false`
  (Alt+N between them, as #551 does); the scenario prints the logged banner, and the notes quote
  its title and body.
- REQ-004: `marley.push` points at the fake ntfy; the stand-in's turn ends while the window is
  away (#535's FIFO steps); the scenario prints the logged request's `Title` header and body.
- Marley Day: `925-04-day`, the theme switched live.

### Risks
- Changing `MarleyToastLook`'s signature touches only `toasts.rs`; a Zed toast built elsewhere
  passes `None` and keeps today's look except the default dot (grey, not accent).
- A grey default makes some of Marley's current accent toasts quieter; that is the board's intent
  (D2).
- The push title is ASCII by construction (D4); an agent name from Zed's thread (`on_thread`)
  could carry other characters, so the title filters to printable ASCII and falls back to
  "Agent".

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
