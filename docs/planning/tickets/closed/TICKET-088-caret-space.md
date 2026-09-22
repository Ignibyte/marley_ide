# TICKET-088 — caret floats a space after the typed text

- **Forge ticket:** #88 `a406c036-43ea-4adb-b22b-93327514aba7` (bug, Terminal Polish; BACKLOG)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `d67be772-0e94-4563-af71-2b9dec26b719`
- **Pipeline doc:** ../../pipeline/active/caret-space.spec.md
- **Status:** closed

## Summary
The prompt caret bar floats 8px after the last typed char (the input_row `.gap_2()` gaps text↔caret).
FIX: group `before`+caret+`after` in a gapless inner flex, one child of the outer gap_2 row. SHIM render
fix (masked); verified by the gate + self-test/capture. Deps #37 (prompt render) + #28 (buffer/caret).

## Acceptance
The caret sits flush after the typed text (self-test/capture); FULL gate GREEN. Full EARS in the spec.
