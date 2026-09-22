# TICKET-029 — input: command history (↑/↓ recalls previous commands)

- **Forge ticket:** #29 `f6447fdf-d9cf-4543-b67e-79b6aa7230d9` (feature, M1.D — The Daily Driver, seq-2)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `c9c222c0-06fa-499b-8bd4-f545b0da2753`
- **Pipeline doc:** ../../pipeline/active/command-history.spec.md
- **Source ticket:** forge sprint #4 `a029f2bc-dabd-43f5-9f6d-13498f0916d5` (M1.D — The Daily Driver)
- **Status:** closed

## Summary
↑/↓ at the prompt do nothing. Add a pure per-pane `CommandHistory` (bounded ring, dedup-last,
prev/next with a draft-stash, detach-on-edit) on `PaneState`; the shim routes ↑/↓ — only when the
command palette is closed — to recall previous commands into the prompt buffer, and records each
submitted command. Builds on #28's editable line.

## Acceptance
`CommandHistory` at cov 100/MSI 100 (record dedup-last + capacity eviction + cursor reset; prev
stash + clamp-at-oldest; next walk + draft restore + None-at-live; detach re-stash); FULL gate
GREEN [--diff]. Full EARS in the pipeline spec.
