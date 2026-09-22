# TICKET-114 — the code viewer as a side-by-side panel [M5 seq-8]

- **Forge ticket:** #114 `861a81d5-1115-4c12-bbc5-22325f1940f7` (feature, M5 seq-8; sprint #16)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `0c16ff56-2e29-4a17-8f80-ed19bafedd9b`
- **Pipeline doc:** ../../pipeline/active/viewer-pane.spec.md
- **Status:** closed

## Summary
`viewer_split(center_w) -> (terminal_w, viewer_w)` (pure, cov/MSI 100) + the M4 viewer repositioned from a
centered modal to a right-side panel (terminal shrinks left; titled, closable). Deps seq-1/2 + M4 code_view.
(Side-panel, not a PaneGroup pane — the session-per-pane refactor is deferred.)

## Acceptance
viewer_split at cov/MSI 100; the viewer sits beside the terminal (live capture); FULL gate GREEN. Full EARS
in the spec.
