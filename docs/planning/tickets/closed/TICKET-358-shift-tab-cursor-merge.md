# TICKET-358 — ⇧Tab merges two cursors inside the same row's indentation (2→1)

- **Forge ticket:** #358 `ae7dbf9e-c8ef-4b48-87fb-5ad1ac5f0349` (bug, editor/multi-cursor/M22, #307 follow-up, low-priority, from-inspect)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `d08e90f8-d95a-45e0-a8e8-a35d4d62fd67`
- **Pipeline doc:** ../../pipeline/completed/358-shift-tab-cursor-merge.spec.md
- **Source ticket:** the polish/debt goal `/work 341,342,343,344,346,347,349,358,359,334` (a #307 follow-up, from its inspect)
- **Status:** closed — shipped LOCAL: documented-correct merge + guards (Opt 3 was a no-op, no behavior change). GATE GREEN [diff] 15/15.

## Summary
Two carets that both sit INSIDE the same row's leading indentation collapse to one cursor on ⇧Tab (dedent). Buffer
`"aa\n    bb\ncc\n"` with carets at offsets 4 and 6 (columns 1 and 3, both within row 1's four-space indent) → ⇧Tab →
`"aa\nbb\ncc\n"` with one cursor at offset 3. `rebase_through`'s clamp maps any position inside a removed span to the
span's (shifted) start, so both carets land at the new line start and `SelectionSet::from_selections` merges them
(same-offset carets ARE one cursor by the `overlaps` `<=` caret law). Filed low-priority because of the #297
silent-cursor-loss scar — but here it happens on an EDIT, so ⌘Z restores the two-cursor set.

## Resolution
**Opt 1 — accept + document + regression-guard (the recon dissolved the fork).** The merge is topologically forced:
after removing the 4 indent chars, columns 1 and 3 no longer exist, so there is nowhere distinct for the two carets
to go — one cursor at the new line start is the only correct outcome (VS Code does the same), and on an edit ⌘Z
restores. The ticket's Opt 3 (`line_start + (col - stripped).max(0)`) is a **no-op** on the real `rebase_through` (an
inside caret always has `col < remove`, so the `.max(0)` is always 0 = the current clamp; an outside caret never hits
the clamp arm). Opt 2 (co-located cursors) contradicts `from_selections`. So the shippable slice is a documenting
headless drive (the merge + ⌘Z restores the 2-cursor set) + a regression guard (3 carets on 3 separate rows keep all
3) + a doc note.

## Acceptance
2 carets inside one row's indent → ⇧Tab → exactly 1 cursor at the new line start; ⌘Z restores the 2-cursor set. 3
carets on 3 separately-indented rows → ⇧Tab → all 3 survive. Full EARS in the pipeline spec.
