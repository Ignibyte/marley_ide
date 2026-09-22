# TICKET-119 — fix sticky/ambiguous search focus [BUG]

- **Forge ticket:** #119 `f4bb512a-f31d-4d91-ad59-db32bd2f818b` (bug; unsprinted M5 follow-up)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `e7a700d7-c54f-4001-b0d8-94a7c1a14f3d`
- **Pipeline doc:** ../../pipeline/active/focus-fix.spec.md
- **Status:** closed

## Summary
The session-search / commit-message / top-search focus flags weren't mutually exclusive or cleared on
outside-click → chad-reported "funky search". Fix: a `clear_input_focus` helper (masked shim) + call it
before setting any focus true + on terminal-pane click. No new pure surface.

## Acceptance
Focus is mutually exclusive + cleared on a terminal click (code review); FULL gate GREEN. Full EARS in the spec.
