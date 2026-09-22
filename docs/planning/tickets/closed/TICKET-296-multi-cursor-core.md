# TICKET-296 — multi-cursor core (the deferred "M1.B")

- **Forge ticket:** #296 `2fb2b40a-d84c-4846-9a20-8265b0532ef8` (feature, M19)
- **Owner:** autonomous /goal run (sprint #32 — M19 Editor power tools) — THE KEYSTONE
- **AAR:** `d68a1852-579a-4110-90ce-7683688f7c11`
- **Pipeline doc:** ../../pipeline/active/296-multi-cursor-core.spec.md
- **Status:** closed

## Summary
`SelectionSet` documents its own deferral verbatim: *"M1.A always holds exactly one
member (`single`); the multi-member sort/merge constructor (`from_selections`) is
deferred to M1.B."* This lands M1.B: an ordered+disjoint N-cursor set, an N-caret
edit that reuses the shipped back-to-front `replace_all` sweep (no rebasing needed)
inside ONE `begin/end_undo_group` (#282), and the pure forward shift that says where
the N carets land afterward.

Every later M19 ticket (#297 gestures, #298 ⌘D-as-cursor, #299 comment, #300 move-line,
#303 delete-ops) is multi-cursor aware and depends on this.

## Acceptance
Two carets + a typed char → two inserts, both carets land correctly, and ⌘Z reverts
BOTH in one step. Full EARS in the pipeline spec.
