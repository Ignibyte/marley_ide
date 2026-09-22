# TICKET-107 — typed panes (PaneKind) [M5 seq-1 FOUNDATION]

- **Forge ticket:** #107 `99ad0ddd-b6be-46a2-b6b5-5d987ac0fb08` (feature, M5 seq-1; sprint #16)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `c2c6b7cc-185d-4241-b21c-e4e9a375114c`
- **Pipeline doc:** ../../pipeline/active/typed-panes.spec.md
- **Status:** closed

## Summary
`PaneKind{Terminal,FileTree,CodeView,Git}` + `PaneState<S>.kind` (default Terminal, preserved across
split/close) + `PaneKind::label`. Pure in workspace.rs (cov/MSI 100). The foundation for the Warp
typed-pane grid. Deps M1.B.

## Acceptance
PaneKind + the kind field at cov/MSI 100 (label/default/preservation); FULL gate GREEN. Full EARS in the spec.
