# TICKET-161 — M10: tab close (rail × + ⌘W tab semantics + the last-terminal guard)

- **Forge ticket:** #161 `7056f380-b37e-4cbc-b476-c66c7cc894ae` (feature, M10; sprint #21)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `8ad2463e-a192-428f-b143-9af0a1e0460d`
- **Pipeline doc:** ../../pipeline/active/tab-close.spec.md
- **Status:** closed

## Summary
Tabs can be closed: an × per rail tab row; ⌘W closes the focused pane as today but closes the TAB at its last
pane (or a cockpit/code active tab); closing the last TERMINAL tab is refused (TabError::LastTerminal — absorbs
#159's guard, keeping workspace()'s invariant enforced). Pure guard + Workspace::project_mut at cov/MSI 100;
the × render/click + ⌘W branch masked. Deps #150-153.

## Acceptance
The pure guard at cov/MSI 100; driven — × closes a tab / refuses the last terminal (flash); ⌘W closes the tab
at 1 pane; FULL gate GREEN. Full EARS in the spec.
