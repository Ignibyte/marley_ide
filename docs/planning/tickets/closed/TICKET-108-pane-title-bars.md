# TICKET-108 — pane title bars (tabs up top) [M5 seq-2]

- **Forge ticket:** #108 `b8cd2f66-f520-40b6-9324-4017b40011a0` (feature, M5 seq-2; sprint #16)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `b5a055d0-d3de-4df6-83cd-301ea1826534`
- **Pipeline doc:** ../../pipeline/active/pane-title-bars.spec.md
- **Status:** closed

## Summary
`pane_title(kind,name)` + `pane_icon(kind)` (pure, cov/MSI 100) + a title-bar strip atop each pane (icon +
name + ⋮ + × close; the content pane shrinks below it). The Warp "tabs up at the top". Deps seq-1.

## Acceptance
pane_title/pane_icon at cov/MSI 100; each pane shows a top title bar (live capture); FULL gate GREEN.
Full EARS in the spec.
