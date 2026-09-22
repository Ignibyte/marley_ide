# TICKET-282 — Editor grouped-undo transaction seam

- **Forge ticket:** #282 (99bd47ba-5660-419a-920d-84be6d7baefe) (feature, M17)
- **Owner:** autonomous /goal run (session a2a59fa8)
- **AAR:** 04479f69-abdc-4fb9-a933-524e7230f829
- **Pipeline doc:** ../../pipeline/active/282-grouped-undo.spec.md
- **Source ticket:** M17 follow-up shelf (#282–287, sprint #30); spun from #276 inspect F3 + #272 precedent
- **Status:** closed

## Summary
Multi-edit editor ops today unwind one primitive edit at a time (a 10-line indent = 10 ⌘Z;
replace-all of N = N ⌘Z), and undo restores only the caret, not the anchor. Add an undo
group/transaction seam in `marley_editor` so a bracketed set of edits pops/redoes as ONE step and
restores the `(anchor, caret)` selection pair, then adopt it for the #276 Tab/⇧Tab indent arm and
the #272 `replace_all`.

## Acceptance
One ⌘Z reverts a whole block indent/dedent and a whole replace-all (and ⌘⇧Z re-applies each as one
step), restoring the selection shape (anchor + caret); single-char typed runs still coalesce as
today. Pure seam at cov/MSI 100. Full EARS criteria in the pipeline spec.
