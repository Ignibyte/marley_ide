---
pipeline_id: f646e1b9-7258-4253-9491-2791d556cf5c
ticket: forge#282 (99bd47ba-5660-419a-920d-84be6d7baefe) · local docs/planning/tickets/open/TICKET-282-grouped-undo.md
aar_id: 04479f69-abdc-4fb9-a933-524e7230f829
status: Phase 5 — Complete PASS
title: Editor grouped-undo transaction seam — one ⌘Z per block indent/dedent + replace-all
type: feature
milestone: M17
references: []
---

## Title
Today every multi-edit editor op unwinds one primitive edit at a time: a 10-line Tab indent
takes 10 ⌘Z (`UndoHistory::record` coalesces only contiguous single-char inserts), and a
replace-all of N matches takes N ⌘Z. Undo also restores only the caret, never the anchor, so a
grouped op can't restore the selection shape. Ship an undo GROUP/transaction seam in
`marley_editor` so a bracketed set of edits pops/redoes as ONE step and restores the
`(anchor, caret)` selection pair; adopt it for the #276 Tab/⇧Tab indent arm and the #272
`replace_all`.

## Scope
### In
- A group/transaction seam in `marley_editor` undo: bracket N `EditRecord`s into one undo group;
  `undo` pops the whole group (inverting its records newest-first), `redo` re-applies the whole
  group (oldest-first). Single edits remain their own one-record group.
- Each group carries a **selection snapshot** — `sel_before` (restored on undo) and `sel_after`
  (restored on redo), as `(anchor: CharOffset, caret: CharOffset)`. `undo`/`redo` return the
  selection to restore, not just a bare caret.
- Preserve the existing single-char **coalesce** (a typed run stays one step) and the redo-clear-
  on-new-edit invariant.
- Migrate **#272 `replace_all`** (crates/editor/src/find.rs) to bracket its back-to-front loop
  into one group.
- Migrate the **#276 Tab/⇧Tab** indent/dedent arm (crates/marley_app/src/app.rs) to bracket its
  `LineEdit`-apply loop into one group and pass the pre/post selection.
- Wire ⌘Z / ⌘⇧Z (app.rs `"undo"`/`"redo"`) to apply the restored `(anchor, caret)` to the
  editor surface (caret AND anchor), not just the caret.

### Out (explicitly deferred)
- Time/pause-based grouping of typing (single-char coalesce is unchanged; no "new group on pause").
- Cross-buffer / cross-file grouping; a visible undo-history UI / named levels.
- Grouping for the terminal prompt (this is the code editor's `Buffer` history).
- Auto-indent-on-newline (#276 `indent_for_newline`) grouping — it is already one `edit`.

## Reference (§20)
N/A — Marley/IDE-specific undo model. Grouping block-indent and replace-all into a single ⌘Z is a
**universal editor-undo convention** (a multi-line indent is one undo unit everywhere); no
reference-app source was read or translated — the seam is a clean-room reimplementation over the
existing `marley_editor::undo` stack (§20).

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — The pure seam lives in `marley_editor::undo` (+ a thin `Buffer` bracket API). PURE
  cov/MSI 100 on the group stack + selection snapshot. The app arm/wiring is the masked shim.
- D2 — Bracketing model (`begin_group(sel_before)` … `edit()`* … `end_group(sel_after)`) over a
  per-record group id — it fits both call shapes (the indent loop applies a computed set; the
  replace loop iterates matches) without threading an id through every `edit`.
- D3 — Selection is stored in the group as `(anchor, caret)` CharOffsets; the app remains the
  selection owner (it passes its current selection into the bracket and applies the returned one).
  No migration of the editor onto `Buffer::selection()` (out of scope).
- D4 — Coalesce stays, but only WITHIN the currently-open group / the top single-record group — a
  bracketed group never coalesces a later ungrouped keystroke into itself.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user undoes after a block indent/dedent of N lines, the editor shall revert all N line edits in ONE undo step | unit (group of N pops as one) + app-arm integration |
| REQ-002 | WHEN the user undoes after a replace-all of N matches, the editor shall revert all N replacements in ONE step | unit + updated find.rs test |
| REQ-003 | WHEN a grouped edit is undone, the editor shall restore the selection `(anchor, caret)` to its pre-edit state | unit (sel_before, anchor not just caret) |
| REQ-004 | WHEN a grouped edit is redone, the editor shall re-apply the whole group in ONE step and restore the post-edit selection | unit |
| REQ-005 | WHILE the user types a contiguous single-char run, the editor shall keep coalescing it into one undo step (unchanged) | existing coalesce tests stay green |
| REQ-006 | The pure group seam shall reach 100% line coverage and MSI 100 | gate (llvm-cov + mutants) |

## Phase Plan
- **P2 Design** — the `UndoHistory` group restructure (stack of groups vs group-tagged records) +
  the `Buffer` bracket API + selection-snapshot type; the app arm + ⌘Z/⌘⇧Z wiring; the kill-list.
- **P3 Implement** — undo.rs + buffer.rs (pure); find.rs replace_all; app.rs indent arm + undo/redo.
- **P3.5 Inspect** — critics vs the diff (coalesce/group interaction, redo-clear, selection clamp).
- **P4 Validate** — write + RUN tests (unit + arm integration); gate green [diff].
- **P5 Complete** — CHANGELOG + editor architecture doc; AAR; restore the #276/#272 REQ wording; close.
