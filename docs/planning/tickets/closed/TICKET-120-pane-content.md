# TICKET-120 — PaneContent: typed pane content [M6 seq-1 FOUNDATION]

- **Forge ticket:** #120 `ac00c9c1-3da7-4053-92a6-911fbdc340bb` (feature, M6 seq-1 FOUNDATION; sprint #17)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `4719c07e-3767-42f0-b74b-46dc231a0c8d`
- **Pipeline doc:** ../../pipeline/active/pane-content.spec.md
- **Status:** closed

## Summary
Refactor `PaneState<S>` → typed `PaneContent<S> { Terminal(TerminalPane<S>) | FileTree | CodeView | Git }`
(workspace.rs, cov/MSI 100) + `kind()`/`terminal()`/`terminal_mut()` accessors + `open_pane(content)`; the
split/close/focus algebra unchanged. Mechanically migrate app.rs's ~117 terminal-field accesses; the terminal
keeps working (live-captured). The enabler for the whole Warp layout. Deps M5 #107. LARGE, no visual change.

## Acceptance
The new workspace surface at cov/MSI 100; existing pane tests pass; the terminal still renders live; FULL gate
GREEN. Full EARS in the spec.
