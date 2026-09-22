# TICKET-124 — the code viewer as a grid pane [M6 seq-5]

- **Forge ticket:** #124 `efa403e4-ef91-480f-9047-6c4f14aa217d` (feature, M6 seq-5; sprint #17)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `2d71d5d5-4c01-4239-98bf-8d28ce529aa7`
- **Pipeline doc:** ../../pipeline/active/code-pane.spec.md
- **Status:** closed

## Summary
`first_pane_of_kind` + `set_content` (workspace.rs, cov/MSI 100); the CodeView pane renders the full viewer;
`open_code_pane` opens-or-updates a single code pane; retire the M5 #114 side-panel render + the `code_view`
field. Clears duplication #2. Deps #120/#121 + M4 code_view + M5 #114.

## Acceptance
The 2 helpers at cov/MSI 100; a code pane renders (no side panel, live capture); FULL gate GREEN. Full EARS
in the spec.
