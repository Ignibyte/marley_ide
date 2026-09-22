# TICKET-121 — typed-pane render dispatch [M6 seq-2]

- **Forge ticket:** #121 `108e9743-4c3b-4518-a0d6-b0d86c13b3ca` (feature, M6 seq-2; sprint #17)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `a37488ef-1119-4808-b6e2-223d478f14a6`
- **Pipeline doc:** ../../pipeline/active/pane-render.spec.md
- **Status:** closed

## Summary
The per-pane render dispatches on `state.kind()` — a FileTree/CodeView/Git pane renders a real body (not an
empty frame) below its title bar, reusing the existing pure fns. A temporary boot default opens a Files pane
beside the terminal so the dispatch is capturable (seq-3 replaces it). Shim-only. The first VISIBLE M6 step.

## Acceptance
Boot renders [terminal | Files pane] side by side, each titled, the Files pane showing the icon'd tree (live
capture); FULL gate GREEN. Full EARS in the spec.
