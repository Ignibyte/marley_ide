# TICKET-342 — A column-aware scroll_editor_to (⌘D follow tracks the added cursor)

- **Forge ticket:** #342 `f6df65f0-8c45-4c4f-b973-e5a764c4bddd` (chore, editor/multi-cursor, M22)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `3af09837-6f88-4637-ae52-b0b56f3fe61f`
- **Pipeline doc:** ../../pipeline/active/342-column-aware-scroll-editor-to.spec.md
- **Source:** the polish/debt goal `/work 341,342,343,344,346,347,349,358,359,334` (a #336 follow-up)
- **Status:** closed

## Summary

#336 put the horizontal caret-follow on the shared `scroll_editor_to_row` primitive, but the primitive takes
only a ROW and the follow reads `active_caret()` = the PRIMARY cursor. For ⌘D add-next-occurrence — the one
gesture that scrolls the vertical to a NON-primary (the newly added) cursor — the horizontal then follows the
wrong cursor. Latent (the identity no-op usually hides it), but real. Fix: a `scroll_editor_to(row, target)`
that also steers the horizontal follow toward `target` (a `CharOffset`, so the display column is derived by the
shipped tab-aware `col_of_offset` — not a raw char col); `scroll_editor_to_row(row)` delegates with `None` (the
primary) so its existing callers are byte-identical; ⌘D passes the added member's head.

## Acceptance

⌘D onto an occurrence whose column is off-screen-right scrolls BOTH axes to the ADDED cursor (headless assert on
`scroll_x`); `scroll_editor_to_row`'s other callers (goto-preview, go-to-def, find, keyboard follow) are
unchanged; the identity case still no-ops. Recon confirmed the gap is ⌘D-ONLY (go-to-def + find collapse to a
single caret first → their primary is already the target). No new pure seam — wiring over the #336 tested
primitives; the proof is a headless behavioral test. Full EARS (REQ-ADDED-BOTH-AXES, REQ-DELEGATION-UNCHANGED,
REQ-IDENTITY-NOOP) in the pipeline spec. Verified headlessly (live pixel deferred — chad at the machine).
