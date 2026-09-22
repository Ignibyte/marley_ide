# TICKET-036 — Block chrome: command cards + exit-status indicator

- **Forge ticket:** #36 `a3b9ca14-91d1-4f9b-9cc7-002d5839fee9` (feature, M1.E — The Warp Look, seq-3)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `2adddcce-a9a6-4a95-a047-c7bc0543f367`
- **Pipeline doc:** ../../pipeline/active/block-chrome.spec.md
- **Source ticket:** forge sprint #5 `cf1ba6de-3af6-4158-813e-b300e596c831` (M1.E — The Warp Look)
- **Status:** closed

## Summary
Blocks render as bare text with no status. Add a pure `block_status` module (`StatusKind` +
`exit_status_kind(BlockState, ExitCode)` + `status_indicator(StatusKind, &ThemeColors)`) and render a
styled command HEADER row (exit-status indicator ✓/✕/○ colored by success/danger/border + the
command) with a block separator. Warp's signature look. Pure decision cov/MSI 100; the card layout is
shim. Deps #34/#35 done.

## Acceptance
`exit_status_kind` + `status_indicator` at cov 100/MSI 100 (every state/exit combo incl. Finished+None
→ Failure; each kind → exact glyph + color role); the header render (masked multi-block visual — green
✓ + red ✕, chad-verified); FULL gate GREEN. Full EARS in the pipeline spec.
