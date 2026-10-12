# Agent findings as diff notes — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-730-agent-findings-as-diff-notes.md
- **Pipeline spec:** 730-agent-findings-as-diff-notes.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** plannotator finding 5 (`docs/planning/intake/plannotator-findings.md`): "An MCP tool
  (such as `review_note_add`) puts a reviewing agent's or a linter's finding into the diff as a
  note tagged with its source; the user keeps or drops each, then sends the review. Any skill can
  be the review's prompt. Pairs with #717, so one agent reviews another's diff." Chad, 2026-10-11,
  on the finish-line list (`docs/planning/design-notes/finish-line-2026-10-11.md`): "11.) yes". It
  waits under Deliberate with the rest of the review batch until after the finish.
- **Classification:** feature, medium. Zed touch in `crates/editor/src/git.rs` (and the export in
  `editor.rs` if `add_review_note` needs a new public type); the rest in `marley_mcp`,
  `marley_agent` and `marley_workbench`.
- **Recall (§18.3):**
  - AD-claude-522-review-notes-go-to-an-idle-terminal-agent-and-stay-marked-sent-001: the notes,
    the `sent` mark and the picker this extends; Zed's comment rows had no edit or delete buttons,
    which is still true (`git.rs:2916`).
  - AD-claude-704-areas-ask-once-per-session-and-reads-stay-free-001: an acting tool asks once per
    session per caller and project; D2 puts `review_notes_add` in the editors' area.
  - AD-claude-mcp-first-class-control-plane-001: Marley's features are MCP-shaped seams.
  - PR-claude-derive-wire-list-from-single-registry-001: the tool is a `REGISTRY` row, its schema
    in `tool_schemas`'s match; nothing repeats the list.
  - PR-claude-703-an-unactivated-insert-keeps-the-active-tab-001: if the diff is added to a pane
    without activation, the active tab is taken before and restored after.
  - F-claude-600-a-registry-lookup-read-the-workspace-its-caller-was-updating-001: the tool finds
    the project with `project_holding` outside any workspace update, then updates.
  - #522's visual check: in the headless sway the first toolbar click after typing in a note did
    nothing; the scenario clicks the toolbar's count once first, as #522's does.
- **Discovery:**
  - `crates/editor/src/git.rs`: `DiffHunkKey` (162), `StoredReviewComment` (171), `ReviewNote`
    (188), `show_diff_review_overlay` (572: hunk key from the start row's file and anchor, the
    overlay block, focus to its prompt), `submit_diff_review_comment` (712),
    `total_review_comment_count` (764), `unsent_review_notes` (776), `mark_review_notes_sent`
    (810), `add_review_comment` (825), `comments_for_hunk` (1203), `remove_review_comment` (1235),
    the `DeleteReviewComment` handler (1548), `calculate_overlay_height` (2134),
    `render_comments_section` (2749), `render_comment_row` (2816).
  - `crates/editor/src/actions.rs:1040-1064`: `EditReviewComment`, `DeleteReviewComment`,
    `ConfirmEditReviewComment`, `CancelEditReviewComment`; nothing dispatches the first two.
  - `crates/marley_mcp/src/registry.rs`: `Family` (15), `as_str` (49), `is_served` (69), `ToolSpec`
    (93), `REGISTRY` (125; the editor rows 411-455), `tool_schemas` (715);
    `crates/marley_mcp/src/dispatch.rs:243-260` (app families deferred).
  - `crates/marley_workbench/src/mcp.rs`: the grants (108-116), `answer` (516), the gate (525), the
    routes (541-545); `caller_terminal` (863).
  - `crates/marley_workbench/src/editor_tools.rs`: `answer` (38), `after_admit` (64), `open` (470),
    `project_holding` (565).
  - `crates/marley_workbench/src/agent_control.rs`: `Area` (35), `Area::mode` (62), `Level` (85),
    `admit` (161).
  - `crates/git_ui/src/project_diff.rs`: `deploy_at_project_path` (188), `move_to_project_path`
    (309), `editor` (335), `multibuffer` (340).
  - `crates/marley_agent/src/marley_agent.rs`: `ReviewLine` (350), `review_prompt` (371).
- **Decisions:** D1 to D6 in the spec.

### Design
- **Zed touch, `crates/editor/src/git.rs`** (each hunk `// Marley:`):
  - `StoredReviewComment.source: Option<SharedString>` and `.proposed: bool` (none and false in
    `new`).
  - `ReviewNote.source: Option<String>`; `unsent_review_notes` skips proposals and fills `source`.
  - `total_review_comment_count` counts neither sent notes nor proposals.
  - `pub fn add_review_note(&mut self, range: Range<Anchor>, comment: String, source: SharedString,
    window, cx) -> Option<usize>`: the hunk key as `show_diff_review_overlay` computes it, the
    hunk's block made when there is none (its prompt not focused), the note stored as a proposal;
    none when the range has no file.
  - `pub fn keep_review_note(id, cx)` and the existing `remove_review_comment` behind Drop.
  - `render_comment_row`: the source as a muted label after the text when there is one; Keep and
    Drop icon buttons for a proposal (Keep calls `keep_review_note`; Drop dispatches
    `DeleteReviewComment`).
- **`crates/marley_mcp/src/registry.rs`** (Marley crate): `Family::Review` (`"review"`, served),
  the `review_notes_add` row (`Tier::Write`, `review.write`), its schema arm.
- **`crates/marley_mcp/src/dispatch.rs`:** `Family::Review` deferred to the app.
- **New `crates/marley_workbench/src/review_tools.rs`** (Marley crate): `answer` for `review_*`;
  `notes_add`: validate (absolute paths, lines from 1, at most 50, comment text cut to a bound),
  admit through `Area::Editors` at `Level::Act`, find each file's project (`project_holding`), open
  or find the project's diff (`deploy_at_project_path` for the first file), wait for each file's
  excerpts (bounded), map lines to anchors (`buffer_point_to_anchor`), refuse lines outside the
  excerpts, `add_review_note` on the right-hand editor; answer with placed and refused notes.
- **`crates/marley_workbench/src/mcp.rs`:** `review.write` in the grants; the `review_` route.
- **`crates/marley_agent/src/marley_agent.rs`:** `ReviewLine.source: Option<String>`; the
  `Source:` line after `User comment:` (escaped as the comment is).
- **`crates/marley_workbench/src/review_notes.rs`:** passes `source` through `prompt`.
- **`docs/marley/guide.md`:** findings in the diff, Keep and Drop; the tool in the MCP tools list.
- **File manifest:** `crates/editor/src/git.rs` (Zed crate); `marley_mcp/src/registry.rs`,
  `marley_mcp/src/dispatch.rs`, `marley_workbench/src/review_tools.rs` (new),
  `marley_workbench/src/mcp.rs`, `marley_workbench/src/review_notes.rs`,
  `marley_workbench/src/marley_workbench.rs`, `marley_agent/src/marley_agent.rs` (Marley crates);
  the guide; the scenario.
- **Ledger rows (`docs/marley/zed-touchpoints.md`):** the `crates/editor/src/git.rs` row gains
  `source`, `proposed`, `ReviewNote.source`, `add_review_note`, `keep_review_note`, Keep and Drop
  and the source label (#730). Why: findings from tools land as proposals the user keeps or drops.
  On merge: keep them; if upstream adds note deletion or authors, move onto it.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | `git: diff`; the client calls `review_notes_add` (source `scenario-lint`, lines 3, 5 and 1) | 730-01-asked: the question |
| 002, 003, 004 | Clicks Allow for This Session; saves the client's answer | 730-02-proposed: two proposals with source, Keep and Drop; no count; the answer names line 1 |
| 005, 006 | Clicks Keep on line 3's note, Drop on line 5's | 730-03-kept: one kept note with its source; Send Review to Agent (1) |
| 007 | Send Review to Agent; Enter on the ready row | 730-04-sent: the echo with `Source: scenario-lint` |

### Risks
- A file not yet loaded into the diff (the diff loads 16 files at a time) has no excerpts when the
  call arrives; the tool waits a bounded time for each file before it refuses it as not in the diff.
- The overlay's height is computed from the count of comments; a proposal's buttons must fit the
  two lines a comment row gets, or the height counts them.
- An agent's comment can be long; the row wraps it, and the tool cuts it at a bound it states.

## Phase 1 — Plan (promoted 2026-10-12)
- **Promotion:** the queued pair moved to `active/`; the ticket is in progress; its Backlog row is
  gone. The brain (consultation a4df09371d8e4d4ebd5721845c20dd00) had nothing on the seam.
- **Seams re-verified** (an Explore pass at `8190786f44`): as the queued notes say, with these
  differences, which change the design below:
  - `show_diff_review_overlay(display_range, window, cx)` (git.rs 572) keys the block by the
    range's first line and always focuses its prompt; `add_review_comment` (825) is `pub(super)`
    and draws no block; `refresh_diff_review_overlay_height` (2510) resizes a block, and
    `ReviewCommentsChanged` does not. `render_comment_row` (2816) is a static fn with no editor
    handle; `render_diff_review_overlay` (2567) has one.
  - Nothing announces that the project diff has loaded: `DiffMultibuffer::refresh` loads files 16
    at a time and ends in `cx.notify()`. `ProjectDiff::editor()` is a `SplittableEditor`
    (`rhs_editor()`), and `act_as::<Editor>` answers the same editor #522 reads.
  - `MultiBufferSnapshot::anchor_in_excerpt` is `None` outside the diff's excerpts, the check D5
    needs. `review_prompt` now feeds #732's `review_message`.
- **Design changes:**
  - `add_review_note` calls `show_diff_review_overlay` for the note's rows and gives the focus
    back to what had it, so the upstream function is not touched; it stores the note as a proposal
    under the same key and resizes the block.
  - Keep and Drop act through the editor handle, not a dispatched action: a dispatched action goes
    to the focused element, which may be a terminal. `render_comment_row` gains the handle as an
    argument.
  - `review_tools` waits for each file's excerpts with a bounded retry (three seconds), since the
    admission question may already have used most of an app call's 30 seconds.
  - The family is `review` with the verb `notes_add` (wire name `review_notes_add`).

## Phase 2 — Code

- **Built:** Zed's `crates/editor/src/git.rs` (its ledger row first): `source` and `proposed`,
  `ReviewNote.source`, proposals out of the count and the send, `add_review_note`,
  `keep_review_note`, `drop_review_note`, and `render_comments_section` and `render_comment_row`
  taking the editor handle for the source label and Keep and Drop. `marley_mcp`: `Family::Review`,
  `review_notes_add`, `review_schemas`, the deferred arm, the registry test (68 tools).
  `marley_workbench`: `review_tools.rs` (new), the `review.write` grant and the `review_` route,
  `review_notes::prompt` passing the source. `marley_agent`: `ReviewLine.source` and the
  `Source:` line.
- **Deviations:** the source line reads `Source: "<name>"`, quoted and escaped as the comment is,
  rather than bare; the design changes recorded at promotion.
- **The review found** nothing to fix. Seen, left as upstream does: a hunk's comment box stays
  open, empty, after its last note is dropped, as it does when the user deletes their own last note.
- **Clippy:** a borrow of `call.tool` across its move (cloned first) and `needless_pass_by_ref_mut`
  twice. Fixed at the source. A language server's `cargo check --workspace` was running on the
  repository when clippy was due; the run waited for it.
- **Gate:** `script/gates.sh --diff` green, 17 of 17, on the code the scenario ran; green again after the in-app guide's row (the receipt hashes it, L-claude-730-the-receipt-hashes-the-in-app-guide-001).

## Phase 3 — Test

- **Scenario:** `script/e2e/730-agent-findings-as-diff-notes.sh` under `compositor sway`: #522's
  set-up with a twenty-line `notes.txt`, lines 10 and 12 changed; `mcp_agent` calls
  `review_notes_add` (source `scenario-lint`, lines 10, 12 and 1). The first run stopped at 730-02
  to place Keep (1525,337) and Drop (1566,529); the second passed every check.
- **730-01-asked (REQ-001):** "e2e wants to use review_notes_add", "put 3 review notes from
  scenario-lint into the project diff", Allow for This Session, Always for This Project, Deny.
- **730-02-proposed (REQ-002 to REQ-004):** the Uncommitted Changes diff in front; under line 10
  "Ten should stay in lower case" and under line 12 "Twelve may stay as it is", each with
  `scenario-lint`, Keep and Drop; the toolbar has no Send Review to Agent. The answer: placed 10
  and 12, refused 1 with "the lines are not shown in the project diff".
- **730-03-kept (REQ-005, REQ-006):** line 10's note without buttons, still `scenario-lint`; line
  12's gone; Send Review to Agent (1).
- **730-04-sent (REQ-007):** the stand-in read #732's message, `File: notes.txt`, `Line: 10`,
  `User comment: "Ten should stay in lower case"`, `Source: "scenario-lint"`; nothing of line 12.

## Phase 4 — Complete

- **Documented:** CHANGELOG (Added); the guide (Findings from tools; the tool in Agent Control's
  list) and the in-app guide's Review notes table; `marley_mcp.md`, `marley_agent.md`,
  `marley_workbench.md` ("Findings into the project diff"); the touchpoint row of
  `crates/editor/src/git.rs` describes what shipped.
- **Knowledge:** L-claude-730-the-receipt-hashes-the-in-app-guide-001,
  AD-claude-730-a-tools-findings-are-proposals-in-the-project-diff-001. No failure found.
- **Brain:** `brain decide` on the consultation recorded at promotion.
