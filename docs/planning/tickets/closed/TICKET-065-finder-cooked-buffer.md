# TICKET-065 — cmd-P finder Enter inserts into the cooked buffer

- **Forge ticket:** #65 `e410d50c-a919-4544-b75c-50af2feb865e` (bug, M2.C — a #59 follow-up)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `fc267cda-016c-4d30-bd64-8c46542eaad4`
- **Pipeline doc:** ../../pipeline/active/finder-cooked-buffer.spec.md
- **Source ticket:** forge sprint #11 `fc38f0c0-0704-40e6-9530-3402f8b4821c` (M2.C — The Living Cockpit)
- **Status:** closed

## Summary
Fix the #57 cmd-P finder Enter: it write_bytes the chosen path (invisible in cooked mode — the #59 bug);
route it through `state.buffer.edit(caret..caret, path, Human)` + caret advance (mirror #59's file-click).
SHIM-only (handle_finder_key), masked. No new tests; the self-test is the proof (and finally verifiable —
Enter is a keycode). Deps #57 + #59.

## Acceptance
cmd-P → Enter → the chosen file's path APPEARS at the prompt (visible cooked-buffer insert); no-regression
suite green; FULL gate GREEN. Full EARS in the spec.
