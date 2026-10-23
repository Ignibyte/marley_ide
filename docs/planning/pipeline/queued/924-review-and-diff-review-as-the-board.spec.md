---
pipeline_id: c7cc5032-8976-47ea-a730-7cc27f432f8e
ticket: docs/planning/tickets/open/TICKET-924-review-and-diff-review-as-the-board.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Plan review and diff review as the board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/C-threads.md, docs/planning/pipeline/completed/893-diff-review-header-and-turns.spec.md, docs/planning/pipeline/completed/881-plan-review-turns-and-diff-review.spec.md]
---

## Title
Plan review and the diff review laid out and worded as the Review and DiffReview boards draw
them. The parts exist (#728, #729, #881, #893); they sit in other places or say other words.

## Scope
### In
- **The plan's title as the main column's heading** (`plan_review.rs`, Marley crate):
  `render_column` drops the tab's title (`Label::new(self.title)…Large`, line 1579). For a plan,
  the plan's own first heading, which Zed's Markdown already draws, opens the main column. For a
  file or a reply, the main column opens with the tab's title as a headline.
- **Send Answers beside Send Feedback (N), with the hint** (`plan_review.rs`): `answer_buttons`
  and the hint move from `render_column` to a foot row under the plan in `render`, over a top
  rule. While answers wait: Send Answers (primary) and Send Feedback (N). Otherwise: Approve
  (primary) and Send Feedback (N). N counts the notes and the overall note not sent. The hint at
  the row's right reads "Select a passage and press c to comment".
- **The notes column as the board heads it** (`plan_review.rs`, `render_notes`): a "YOUR NOTES"
  heading in small capitals; each note's quote as a mono chip with an accent rule at its left;
  a "Remove" text button at the note's foot in place of the X; the overall placeholder for a plan
  reads "Anything about the whole plan".
- **Turn cards** (`plan_review.rs`, `render_turns`; `turns.rs`, Marley crate): each card has the
  turn's title, a pill at the right in the state's colours (new, kept, undone, failed, injected),
  and a mono line of its files with their counts ("rail.rs +12 −4 · rail_order.rs +3 −1").
  `Turn` gains the per-file counts, read once when the turn is recorded with
  `git diff --numstat <start> <sha>` and parsed by `git::status::parse_numstat`. A turn with
  `injected` set and no mark shows "injected"; one with no mark shows "new".
- **Send Review to Agent (N) at the column's foot** (`plan_review.rs`): for a file or a reply,
  `send_notes_button` becomes "Send Review to Agent (N)", the column's last child, pushed to its
  foot. It sends as Send Notes does now.
- **Per-file +/− in the diff's file headers** (`viewed_marks.rs`, Marley crate): the Viewed
  addon's `render_buffer_header_trailing_controls` draws the file's added and removed lines
  before the tick, in green and red mono, from the repository's
  `Repository::diff_stat_for_path`.
- **Send to rows** (`guide_tab.rs`, Marley crate, `render_send_to`): the state word in the
  success colour for "ready" and the warning colour for "working" and "asking for permission";
  the first ready row on a filled background; a last row "Copy notes · any agent" that copies
  the notes as the diff header's Copy Notes does.
- **A gutter mark on noted lines** (`review_notes.rs`, Marley crate; `editor/src/git.rs`, Zed
  crate hunk): on `EditorEvent::ReviewCommentsChanged`, Marley sets
  `Editor::highlight_gutter::<NotedLines>` to the notes' ranges in the accent. The Zed hunk is
  `Editor::review_note_ranges`, the anchor ranges of the notes not dropped, beside
  `unsent_review_notes`.

### Out (explicitly deferred)
- "Other…" as a button on a question card: #881 Out (the card's Other field takes a free answer).
- "Changes since main" over the branch diff: #893 Out.
- The guide as an aside beside the diff, in the same view: a layout change of its own (M); a
  follow-up ticket.
- The note's label "Your note · lines 212–214" over a note in the diff: the review lists it with
  the gutter mark; it changes `render_comment_row` in Zed's editor; a follow-up ticket.
- Send Review to Agent on a plan: a plan's notes go with Send Feedback (D3).

## Reference (§20)
Upstream Zed (`editor`, `git_ui`): the project diff and its file headers, the editor's review
comments and its gutter highlights are Zed's and are kept; Marley adds counts and a mark through
the addon slot (#731) and `highlight_gutter`. Plan review and its turns are Marley-specific
(#726, #729, #893) over Claude Code's plan socket.

### Prior art
- `plan_review.rs`: `render_column` (1559–1636) holds the title, the hint (1598), the notes, the
  turns and `answer_buttons` (1877–1910); `render_notes` (1481–1557) draws the italic quote and
  the X; `render_turns` (1718–1800) draws "Turn N · title" and "N files · state";
  `send_notes_button` (1833) is "Send Notes (N)"; `send_answers` (1351) already sends the notes
  with the answers.
- `turns.rs:38` `Turn` already has `injected` (set at 309); `record` (444–515) counts files from
  `Repository::diff_tree`, which gives no line counts.
- `worktree_git.rs:43` `git` and `:652` `changed_lines` run `git diff --numstat` already;
  `git/src/status.rs:586` `parse_numstat` parses it into `GitDiffStat` (Zed crate, reused).
- `project/src/git_store.rs:7014` `Repository::diff_stat_for_path`: per-file counts for the
  working tree, which is what the project diff shows.
- `viewed_marks.rs:200` `ViewedAddon::render_buffer_header_trailing_controls`, the addon slot of
  #731 (`editor/src/editor.rs:801`).
- `ui::DiffStat` (`ui/src/components/diff_stat.rs`), already used by plan review's Changes tab.
- `guide_tab.rs:346` `render_send_to`; `review_notes.rs:180` `readiness`; `diff_header.rs:155`
  Copy Notes; `review_notes.rs` `Pick::Copy`.
- `editor/src/editor.rs:9635` `highlight_gutter`, painted as a narrow rounded bar
  (`element.rs:5589` `paint_gutter_highlights`), used by inline assist
  (`agent_ui/src/inline_assistant.rs:1319`). `editor/src/git.rs:171` `StoredReviewComment` holds
  each note's anchor range, private to the editor.
- `docs/zed_architecture/`: nothing on review comments. The brain: not consulted at queueing; the
  promoting session asks.

## UI proof
Scenario `script/e2e/924-review-and-diff-review-as-the-board.sh` (sway), built from #893's: a
scratch repository with two changed files, the stand-in Claude Code first on the terminals' PATH
through the scenario's `.bashrc`, Marley's plugin hook, `marley.plan_review` on. The stand-in
runs a turn and asks a plan with a question card; the scenario adds a note in the plan and a note
in the diff.
- `924-01-plan`: plan review with a note and an answered question: the plan's heading, Send
  Answers and Send Feedback (1) at the foot with the hint, "YOUR NOTES", the mono quote, Remove,
  the turn card with its pill and per-file counts (REQ-001 to REQ-004).
- `924-02-reply`: a reply under review with a note: Send Review to Agent (1) at the column's
  foot (REQ-005).
- `924-03-diff`: the project diff with a note on a line: +/− in each file header, the gutter
  mark (REQ-006, REQ-008).
- `924-04-guide`: the guide's Send to: the stand-in ready in green, Copy notes · any agent
  (REQ-007).
- `924-05-day`: plan review in Marley Day; `924-06-day-diff`: the diff and the guide in Marley
  Day (REQ-001 to REQ-008).

## Locked-In Decisions
- D1 — Plan review keeps every action it has; only places and words change.
- D2 — Send Feedback (N) sends the notes alone, as now; Send Answers sends the answers with the
  notes, as now. Both show while answers wait.
- D3 — A plan has one way to send its notes, Send Feedback; the column's Send Review to Agent is
  the file's and the reply's send (#729's Send Notes renamed).
- D4 — A turn's per-file counts are read once, when the turn is recorded, and kept on `Turn`;
  turns recorded before this change show their file count only.
- D5 — The gutter mark is Zed's gutter highlight bar in the accent, not a new glyph; it needs no
  change to Zed's element code.
- D6 — The diff header's counts come from git's own numstat for the working tree, the same
  numbers the diff header's total (#893) sums.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN plan review shows a plan, the main column shall open with the plan's title as its heading, and the notes column shall not repeat the tab's title. | `924-01-plan`, `924-05-day` |
| REQ-002 | WHILE answers wait to be sent, plan review shall show Send Answers and Send Feedback (N) side by side under the plan, with "Select a passage and press c to comment" at the row's right. | `924-01-plan`, `924-05-day` |
| REQ-003 | The notes column shall be headed "Your notes", draw each note's quote as a mono chip and offer Remove as a text button. | `924-01-plan`, `924-05-day` |
| REQ-004 | Each turn card shall show the turn's title, a coloured pill for its state (new, kept, undone, failed or injected) and its files with their added and removed lines. | `924-01-plan`, `924-05-day` |
| REQ-005 | WHEN plan review shows a file or a reply, the column's foot shall offer Send Review to Agent (N), which sends the notes as Send Notes did. | `924-02-reply` |
| REQ-006 | Each file header in the project diff shall show the file's added and removed lines beside its Viewed tick. | `924-03-diff`, `924-06-day-diff` |
| REQ-007 | The guide's Send to shall draw "ready" in the success colour and "working" in the warning colour, and end with a "Copy notes · any agent" row that copies the notes. | `924-04-guide`, `924-06-day-diff` |
| REQ-008 | WHEN a line in the project diff has a note, its gutter shall carry a mark in the accent. | `924-03-diff`, `924-06-day-diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `plan_review.rs`, `turns.rs`, `viewed_marks.rs`, `guide_tab.rs`,
  `review_notes.rs`, the `editor/src/git.rs` hunk and its touchpoint row; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
