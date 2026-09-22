# TICKET-279 — Double/triple-click + ⇧-click selection in the terminal

- **Forge:** #279 `520339d5-a2e1-4d9f-b36b-6051cb10b500` (sprint #30, M17)
- **Type:** feature
- **Status:** closed
- **Pipeline:** `docs/planning/pipeline/active/279-click-selection.spec.md` (84ff971f-2aaf-437d-871e-fbcba315e1df)

## Summary
The universal trio on the grid: double-click = the same-class run
under the pointer (a path/URL-friendly word class — `./-_~:@?&#%`
stay inside words), triple-click = the full row, ⇧-click = extend the
head keeping the anchor. Pure bounds fns in text_selection.rs; shim
branches in the grid mouse-down BEFORE the #43 seed; ⌘-click link
precedence untouched; everything feeds the existing #44 copy pipeline
(what highlights is what copies). Word/line drag-extend recorded as a
follow-up if it doesn't drop out of the drag path cheaply.

## Acceptance
Spec REQ-001..004: the class kill list, the select-and-copy pairing,
the extend rule, and the single-click/⌘-click identity.
