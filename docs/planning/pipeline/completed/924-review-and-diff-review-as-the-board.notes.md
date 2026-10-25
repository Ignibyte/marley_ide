# Plan review and diff review as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-924-review-and-diff-review-as-the-board.md
- **Pipeline spec:** 924-review-and-diff-review-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 11, first board pair (Review and DiffReview).
- **Classification / tier:** feature, M. Marley crate, plus one small additive hunk in
  `editor/src/git.rs`.
- **Recall (§18.3):**
  - F-claude-893-a-toolbar-item-read-the-workspace-001: the diff header is a toolbar item; the
    Copy notes row lives in the guide tab, not the toolbar, but must not read the workspace from
    inside an update either.
  - F-claude-865-a-hook-called-from-a-render-read-its-own-view-001: the Viewed addon's header
    slot is called from the editor's render; the counts must come from the repository, never from
    reading that editor.
  - F-claude-903-a-stand-in-not-on-the-terminals-path-001 and PR-claude-687: the stand-in Claude
    Code goes first on the terminals' PATH through the scenario's `.bashrc`.
  - F-claude-877 and F-claude-880: the runner already isolates `CLAUDE_CONFIG_DIR` and tmux;
    #893's scenario sets its own `CLAUDE_CONFIG_DIR` with the plugin installed.
  - AD-claude-729-one-review-tab-for-plans-files-and-replies-001: one tab for three subjects;
    the column's send differs by subject (D3).
  - AD-claude-522-review-notes-go-to-an-idle-terminal-agent-and-stay-marked-sent-001: what
    "ready" means in Send to.
- **Discovery:**
  - `crates/marley_workbench/src/plan_review.rs`: `new` (placeholder, 795), `render_notes`
    (1481), `render_column` (1559), `render_turns` (1718), `send_notes_button` (1833),
    `answer_buttons` (1877), `render` (2048).
  - `crates/marley_workbench/src/turns.rs`: `Turn` (38), `record` (444).
  - `crates/marley_workbench/src/viewed_marks.rs`: `ViewedAddon` (195–228).
  - `crates/marley_workbench/src/guide_tab.rs`: `render_send_to` (346–422).
  - `crates/marley_workbench/src/review_notes.rs`: `readiness` (180), `Pick::Copy`, `init`.
  - `crates/editor/src/git.rs`: `StoredReviewComment` (171), `unsent_review_notes` (788).
- **Decisions:** D1–D6 in the spec.

### Design
- Plan review: `render` gains a foot row under the plan area (`plan_foot`): the answer buttons
  and the hint, over a top border, shown for a plan; `render_column` loses the title, the hint
  and the answer buttons, and ends with Send Review to Agent for a file or a reply (`mt_auto`).
  Send Feedback's label carries `(N)` from `unsent`. For a file or a reply, the plan area opens
  with the tab's title as a `Headline`.
- Notes: a "YOUR NOTES" `Label` (small, muted, uppercase) before the notes; the quote as a
  `div` with `font_buffer`, an accent `border_l_2` and accent text; Remove as a quiet
  `action_button` at the note's foot.
- Turns: `Turn` gains `file_lines: Vec<FileLines>`, a struct with named fields `path` (the
  file's name), `added` and `removed`. `record` runs `git diff --numstat <start> <sha>` through a
  new `worktree_git::numstat(folder, base, head)` (beside `changed_lines`) and parses with
  `git::status::parse_numstat`. The card's state is an enum `TurnState { New, Kept, Undone,
  Failed, Injected }` with its words and pill colours; the files line joins "name +a −r" with
  " · ", truncated.
- Diff header counts: in `ViewedAddon`, from the buffer's file get its project path, find the
  repository through the project's `GitStore` (kept on the addon when it is made, never read
  from the editor in render), and draw `diff_stat_for_path` as two mono labels.
- Send to: the state word's colour by readiness (`Color::Success` ready, `Color::Warning`
  working or asking); the first ready row filled with `element_selected`; a final row "Copy
  notes" · "any agent" whose click copies the notes as `Pick::Copy` does.
- Gutter mark: in `review_notes::init`'s project-diff observer, subscribe to the rhs editor's
  `ReviewCommentsChanged` and set `highlight_gutter::<NotedLines>(editor.review_note_ranges(),
  |cx| cx.theme().colors().text_accent, cx)`.
- File manifest:
  - Marley crate: `plan_review.rs`, `turns.rs`, `worktree_git.rs`, `viewed_marks.rs`,
    `guide_tab.rs`, `review_notes.rs`.
  - Zed crate: `crates/editor/src/git.rs` (`Editor::review_note_ranges`, `// Marley:` comment).
- Ledger: the `crates/editor/src/git.rs` row in `docs/marley/zed-touchpoints.md` gains
  `Editor::review_note_ranges` (#924), the anchor ranges of the notes not dropped, for Marley's
  gutter mark; put back by keeping it while `StoredReviewComment` stays private.

### Visual check plan
- REQ-001: the stand-in asks a plan whose text starts with "# Tidy the notes"; `924-01-plan`
  shows that heading at the top of the main column and no "Plan · repo" label in the column.
- REQ-002: a question card answered with Accept recommended and a note added with c;
  `924-01-plan` shows Send Answers and Send Feedback (1) under the plan, the hint at the right.
- REQ-003: the same shot: "YOUR NOTES", the quote in mono with the accent rule, Remove.
- REQ-004: the stand-in's "turn" wrote `todo.txt`; `924-01-plan` shows the turn card with "new"
  and "todo.txt +1 −0".
- REQ-005: a reply opened for review (#729's Review Last Reply on the stand-in's terminal), a
  note added; `924-02-reply` shows Send Review to Agent (1) at the column's foot.
- REQ-006: the project diff opened; `924-03-diff` shows "+2 −2" on `notes.txt` and "+1 −1" on
  `other.txt` beside Viewed.
- REQ-007: `review_guide_open` from the stand-in MCP client; `924-04-guide` shows the stand-in's
  row with "ready" in green and the Copy notes row.
- REQ-008: a note added on line 10 of `notes.txt`; `924-03-diff` shows the accent bar in its
  gutter.
- Marley Day: `924-05-day` and `924-06-day-diff` after switching the theme live
  (L-claude-828).

### Risks
- The gutter highlight shares the gutter with the diff hunk bars; the accent bar may sit over the
  green one. If it does, the Test phase records it and the mark moves into the line-number
  column with a follow-up.
- `numstat` for a turn runs once per turn; a large turn's output is small (one line a file).
- Moving the answer buttons out of the column changes where #881's and #893's scenarios click;
  they are not re-run (no regression per ticket, §7).

## Phase 2 — Code
- **Built:** `editor/src/git.rs` `review_note_ranges`; `plan_review.rs` (`render_foot`, the
  headline, `render_column`, `render_notes`, `turn_state`, `turn_files`, the placeholder, Send
  Review to Agent); `turns.rs` `Turn::file_counts`; `turn_git.rs` `FileCount`, `numstat_in`;
  `viewed_marks.rs` the gutter mark (`NotedLines`); `guide_tab.rs` the state colours, the filled
  row and `copy_notes_row`.
- **Deviations:**
  - The per-file +/− in the diff's headers was already drawn by Zed (beside Open File); a first
    version added Marley's own beside Viewed, which repeated it, and was taken out. REQ-006 is met
    by Zed's own counts.
  - The gutter mark's subscription lives in `viewed_marks.rs`, which already observes each
    project diff, not in `review_notes.rs`.
  - The scenario reviews `brief.md` for REQ-005 (a file), not a reply: both take the same column.
- **Review:** plan review keeps every action (D1); Send Feedback still sends the notes alone and
  Send Answers the answers with them (D2); a turn recorded before this change shows its file count
  (D4).
- **Gate:** `just gate-diff` 17/17 green.

## Phase 3 — Test
- **Scenario:** `script/e2e/924-review-and-diff-review-as-the-board.sh`, sway, 6 shots, each read.
  - `924-03-diff`: notes.txt's line 12 with the accent gutter mark and the kept note; each header's
    +/− (Zed's) and Viewed. REQ-006, REQ-008.
  - `924-04-guide`: Send to: "Claude Code · repo" on a filled row, "ready" in green, then "Copy
    notes · any agent". REQ-007.
  - `924-01-plan`: "Tidy the notes" opens the plan; the question answered; Send Answers and Send
    Feedback (1) at the foot with the hint; YOUR NOTES with "notes" as a mono chip, "Start at
    line 9", Remove; Turn 1 with its "new" pill and "todo.txt +1 −0". REQ-001 to REQ-004.
  - `924-02-file`: "Review · brief.md" over the text, the note, Send Review to Agent (1) at the
    column's foot. REQ-005.
  - `924-05-day`, `924-06-day-diff`: the same in Marley Day.
- **Fix in Test:** the duplicate header counts (above); coordinates from the probe.

## Phase 4 — Complete
- CHANGELOG, `marley_workbench.md`, the look plan's review row, the `editor/src/git.rs` ledger row.
- Knowledge appended: L-claude-924-zeds-diff-header-already-counts-a-files-lines-001.
- Brain: decision recorded.
