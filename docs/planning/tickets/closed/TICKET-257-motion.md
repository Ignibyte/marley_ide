# TICKET-257 — Editor keybinding parity — Home/End, ⌘←→, word-wise ⌥←→

- **Forge ticket:** #257 (73daf3cd-5cc6-41a8-8013-7a53700ba122) (feature, M15)
- **Owner:** (autonomous /goal /work 249 to 258)
- **AAR:** e218ff82-b653-4edd-b813-0e381b9d79b5
- **Pipeline doc:** ../../pipeline/completed/motion.spec.md
- **Source ticket:** M15 — The Editable Editor (sprint #28, fa328492); the #242 decomposition (#248–258)
- **Status:** closed

## Summary
Standard mac editor caret motion in the editor — Home/End (line), ⌥←→ (word), ⌘←→ (line), ⌘↑↓ (document), Up/Down
(vertical, same column on the adjacent row) — each with a Shift-variant that EXTENDS the #255 selection. Reuses
the tested `marley_editor::movement` (word/line fns) + a NEW pure `move_up`/`move_down` + a pure `extend_or_go`
generalizing #255's shift-extend to any target. The !platform motions route through the #251 editor key branch;
the ⌘-motions get a new platform-chord branch. Pure seam (move_up/down + extend_or_go) cov/MSI 100. Deps #251/
#255/#250/movement.

## Acceptance
Home/End/⌥←→ move by line/word; Up/Down move vertically (clamped); ⌘←→ go to line start/end and ⌘↑↓ to doc
start/end; the Shift-variant of any motion extends the selection; the pure movement + extend fns are cov/MSI 100;
driven-proven live. Full EARS in the pipeline spec. Deferred: goal-column memory, page-up/down, smart-home,
word-delete.
