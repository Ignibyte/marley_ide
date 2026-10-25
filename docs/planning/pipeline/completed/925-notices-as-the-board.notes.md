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
- **Built:** `MarleyToastKind` and the hook's kind argument (`workspace/src/notifications.rs`),
  `Toast::marley_kind` (`workspace/src/workspace.rs`); `toasts::look` maps the kind, else the
  words (failure → Error, a title ending "?" → Question, else Neutral). Kinds set: the close
  guard's Undo and Keep It Open (Warning); the plugin's install and update, a capture with a page
  to open, the Forge token written (Success); a failed push and the Forge token not written
  (Error). The other 60-odd sites build their message at run time ("Could not …" and the like)
  and keep the words' reading.
- `plan_review.rs`: `Pending::title` from `plan_title`; `entry_of` asks "Plan to review:
  “<title>”". `rail.rs`: a plan entry's Open the plan, filled with `prompts::primary_fill`.
- `command_watch.rs`: the title "<command> finished · exit N" and `ran_words` for the body.
- `marley_agent::push_text` / `PushText`; `push.rs` posts its title and body.
- **Deviations:** the plan's Open the plan is a filled `h_flex`, not `ButtonStyle::Filled`, so its
  ink is the theme's background on the accent as the prompts' primary is. The push's title uses
  "Agent" for a name that is not plain ASCII (the risk above), rather than dropping characters.
- **Review:** `FAILURES` misses the push failure's lowercase "could not", so it carries an
  explicit Error. `ran_words` reads the workspace and the terminal, not the view being updated.
- **Gate:** `just gate-diff` 17/17 green.

## Phase 3 — Test
- Scenario `script/e2e/925-notices-as-the-board.sh` (sway), full run green, every check passed.
- `925-02-plan`: Needs you "Claude Code · repo / Plan to review: “Tidy the notes”" with Open the
  plan filled in the accent; the plan's tab opened by its arrival. REQ-002.
- `925-03-opened`: after the terminal was brought forward, Open the plan clicked: Plan · repo in
  front. REQ-002.
- The push log: `title: Turn finished`, body `repo · Claude`. REQ-004.
- The banner log: `Marley|sleep 8; false finished · exit 1|repo — sleep 8 ran 8 s while you were
  in Plan · repo.` REQ-003.
- `925-01-toasts`: the plugin installed (green), could not push (red), Panel Layout presets
  (grey), "Claude Code in repo closes when its turn ends" with Keep It Open (yellow). REQ-001. A
  question's accent dot has no toast a scenario raises; it is the words' rule in `look`.
- `925-04-day`: the same four dots and the plan entry in Marley Day. REQ-001, REQ-002.
- Found while writing the scenario: a stand-in that reports idle through `marley report` before
  its `event.py` frames gets no push for its turns (L-claude-925). Seen, not in scope: the first
  turn logged two identical "Turn finished" pushes; #535's scenario checks one push per turn, so
  it is noted for a regression pass. #535's `last_push` expects the old "Marley" title and the
  old line, and needs the new words when the regression phase runs.
- The `push_text` fallback for a non-ASCII name changed after the run; no shot rests on it, and
  the gate ran again.

## Phase 4 — Complete
- Docs: CHANGELOG; `docs/marley_architecture/marley_workbench.md` (toasts' kinds, the long
  command's words, the push's text, the plan entry); the look plan's W3.4; the two ledger rows.
- Knowledge: L-claude-925.
