# TICKET-166 — M10: the split context menu

- **Forge ticket:** #166 `d783dfa6-f858-420b-975e-104819cc3a44` (feature, M10; sprint #21)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `a1f30333-325c-43e9-a52d-14b912674ecd`
- **Pipeline doc:** ../../pipeline/active/context-menu.spec.md
- **Status:** closed

## Summary
Right-click a terminal → a menu at the pointer: Split Right / Split Down / Close Pane; Esc/click-away
dismisses, ↑↓+Enter navigate. Pure ContextMenuState + menu_origin (cov/MSI 100); shim overlay + routing;
split_focused_pane gains an axis. Deps #155, #161.

## Acceptance
Pure at cov/MSI 100; driven — the menu at the pointer, Split Down → stacked panes, Close Pane → back to 1,
Esc dismisses; FULL gate GREEN.
