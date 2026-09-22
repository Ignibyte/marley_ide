# TICKET-297 — multi-cursor gestures + the app shim

- **Forge ticket:** #297 `a8230892-ec68-411c-946e-332e8542c4fd` (feature, M19)
- **Owner:** autonomous /goal run (sprint #32 — M19 Editor power tools)
- **AAR:** `8cecc7af-5a5d-42fc-8bfd-9b81ee7e9feb`
- **Pipeline doc:** ../../pipeline/active/297-multi-cursor-gestures-and-shim.spec.md
- **Depends on:** #296 (multi-cursor core) — SHIPPED at `135b439`
- **Status:** closed

## Summary
#296 gave the editor crate an N-cursor `SelectionSet`, an N-caret edit in one undo unit, and the math for
where the cursors land. But **no key in Marley can create a second cursor**, and the app surface still
carries a single `caret` + `anchor` per open file — so the core is unreachable, invisible, and undrivable.
This ticket closes that: it converts the surface onto the `SelectionSet` (45 call sites), renders N carets
and N selection bands, routes every edit through `edit_at_selections`, teaches motion to move all N (which
needs a per-cursor **goal column** — without it two cursors merge on the first short line and never come
back), and adds the three gestures that create and dismiss cursors: **⌘⌥↑/↓**, **⌘-click**, **Esc**.

Shim and gestures land together by #296's D6: a shim with no gesture is dead weight, and a gesture with no
shim has nothing to drive.

## Acceptance
Open a file → **⌘⌥↓** → two carets render → type a char → **both** lines get it → **⌘Z** → both revert in
one step → **Esc** → back to one caret. Proven on **live pixels**, not mechanism. Full EARS in the spec.

## Known, surfaced up front
- **⌘Z restores all N *edits* but only one *cursor*** — `SelSnapshot` is still a single pair. Phase 2 decides
  whether to grow it (34 mentions, 5 files, a `Copy` derive that ripples) or ship v1 with a follow-up.
- **A confirmed latent bug rides along:** `open_file_at` moves the caret without clearing the anchor, so a
  file:line click with a live selection paints a spurious selection. The conversion fixes it; REQ-008 pins it.
