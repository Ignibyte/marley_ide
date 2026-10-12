---
pipeline_id: 5d3550a1-f154-459f-8df1-59292c99d91d
ticket: docs/planning/tickets/open/TICKET-731-viewed-marks-in-the-diff.md
status: Phase 4 — Complete PASS
title: Viewed marks in the project diff
type: feature
slice: the review batch after the finish (docs/planning/intake/plannotator-findings.md item 6; docs/planning/design-notes/finish-line-2026-10-11.md row 11)
references:
  - docs/orca_architecture/02-worktrees-and-review.md
  - docs/planning/pipeline/completed/522-review-notes-to-the-agent.spec.md
---

## Title
Each file in the project diff gets a Viewed tick in its header. A ticked file folds, stays ticked
after a restart, and loses the tick and unfolds when its diff changes, as when an agent rewrites
it. Chad, 2026-10-11, on the finish-line list: "11.) yes".

## Scope
### In
- A **Viewed** checkbox at the end of each file's header in the project diff (`git: diff`), in the
  unified and the split view and in the sticky header.
- Ticking it folds the file; unticking unfolds it.
- The mark is kept per repository and file with a fingerprint of the file's diff (its base text and
  its current text). It survives a restart.
- When a marked file's diff changes (a save, a reload from disk, a commit that moves its base), the
  mark clears and the file unfolds. A file that left the diff and comes back with the same diff
  comes back ticked.

### Out (explicitly deferred)
- The branch diff (`BranchDiff`, #511's Review): its base is a ref, so its marks need their own key.
- A count of viewed files in the toolbar.
- Marks on hunks or lines; review notes (#522) stay as they are.

## Reference (§20)
Upstream Zed, `git_ui::ProjectDiff` and the editor's buffer headers: the header's fold chevron, its
path and its stage checkbox (`GitPanelAddon`) are kept as they are, and the Viewed tick is drawn
after them through the editor's `Addon` seam. The behavior follows GitHub's per-file Viewed box in a
pull request (published), and Orca's mobile app, whose per-file reviewed marks reset when the
file's diff identity changes (`docs/orca_architecture/02-worktrees-and-review.md` §2.8, citing
`mobile/src/session/mobile-diff-review-state.ts`).

### Prior art
- **The code we ship:**
  - `editor::Addon` (`crates/editor/src/editor.rs:786-818`): `render_buffer_header_controls` (789)
    and `extend_buffer_header_context_menu` (799); `Editor::register_addon` (11175).
  - The header draws only one addon's controls:
    `addons.values().filter_map(render_buffer_header_controls).take(1)`
    (`crates/editor/src/element/header.rs:806-814`), and ProjectDiff's `GitPanelAddon` fills it
    with the stage checkbox (`crates/git_ui/src/git_panel.rs:9463-9486`, 7993-8046; registered at
    `crates/git_ui/src/project_diff.rs:250`). `render_buffer_header` (`header.rs:627`) serves the
    block header, the folded buffer and the sticky header (`header.rs:184`), and split view passes
    the right-hand editor (`crates/editor/src/split_editor_view.rs:572, 655`).
  - Folding: `fold_buffer`, `unfold_buffer`, `is_buffer_folded` (`crates/editor/src/fold.rs:618,
    663, 678`), in memory only.
  - Reaching the editor: `ProjectDiff::editor` (`project_diff.rs:335`) and
    `SplittableEditor::rhs_editor` (`crates/editor/src/split.rs:558`); `ProjectDiff::multibuffer`
    (340).
  - Change signals: `BufferDiffEvent::DiffChanged` (`crates/buffer_diff/src/buffer_diff.rs:1602-1613`);
    DiffMultibuffer subscribes per file (`crates/git_ui/src/diff_multibuffer.rs:451-474`) and
    refreshes on status changes (109-127). Fingerprint parts: `BufferDiffSnapshot::base_text_string`
    (`buffer_diff.rs:343`) and the buffer's text.
  - Persistence: `db::kvp::KeyValueStore::scoped` (`crates/db/src/kvp.rs:89-140`); Marley's
    `launch.rs:335-360` keys a sha256 per folder and name, the model for a fingerprint that asks
    again when it changes.
  - No viewed or reviewed concept exists in `git_ui`, `editor` or `agent_ui`; `action_log`'s
    unreviewed edits belong to an agent thread, not a git diff.
- **Behavior maps:** Orca report 02 §2.8 (above); `docs/zed_architecture/subsystems/03-editor-multibuffer.md`
  (excerpts and buffer headers).
- **Published material:** GitHub's "Viewed" checkbox in a pull request's Files tab, which collapses
  the file and clears when the file changes.

## UI proof
`script/e2e/731-viewed-marks-in-the-diff.sh`, under `compositor sway`: a scratch repository with two
committed files, `alpha.txt` and `beta.txt`, each with one line changed; `git: diff` from the
palette.

Shots:
- `731-01-ticked`: Viewed clicked on `alpha.txt`'s header: its box ticked, the file folded,
  `beta.txt` open below it.
- `731-02-restart`: Marley quit and started again on the same profile, `git: diff`: `alpha.txt`
  still ticked and folded.
- `731-03-changed`: the scenario rewrites a line of `alpha.txt` from outside, as an agent would:
  its box unticked and the file unfolded.

## Locked-In Decisions
- **D1:** the tick is drawn by a Marley addon on the diff's right-hand editor, through one additive
  Zed touch: `Addon` gains `render_buffer_header_trailing_controls` (default none), which the header
  draws after the existing controls. Rejected: relaxing `.take(1)` (which addon comes first would
  depend on a hash map's order); extending `GitPanel`'s controls (a Marley concept inside Zed's git
  panel); a context-menu entry only (no mark to see).
- **D2:** ticking folds the file, unticking unfolds it (GitHub's behavior). Rejected: a tick that
  leaves the file open (the point of a mark is that the file gets out of the way).
- **D3:** the fingerprint is a sha256 of the diff's base text and the file's current text, kept in
  the key-value store under `marley-viewed`, keyed by the repository's folder and the file's path.
  Rejected: the buffer's version (in memory, gone at a restart); the git blob id (no public API for
  the working file's).
- **D4:** a mark clears on any change to the file's diff, whoever made it. Telling an agent's write
  from the user's is not needed: either means the file wants another look.
- **D5:** the project diff only; the branch diff waits.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The project diff shall draw a Viewed checkbox in each file's header. | Shot 731-01 |
| REQ-002 | WHEN the user ticks a file's Viewed box, the system shall fold that file and keep the other files as they were. | Shot 731-01 |
| REQ-003 | WHEN Marley starts again and the project diff shows a file marked viewed whose diff has not changed, the system shall show it ticked and folded. | Shot 731-02 |
| REQ-004 | WHEN a marked file's diff changes, the system shall untick it and unfold it. | Shot 731-03 |
| REQ-005 | WHEN the user unticks a file, the system shall unfold it and forget its mark. | Review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — the touchpoint row first; the trait method and its call in `header.rs`; a new
  `viewed_marks.rs` in `marley_workbench` (the addon, the marks, the store); the guide; a review of
  the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
