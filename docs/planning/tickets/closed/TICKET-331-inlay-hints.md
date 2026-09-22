# TICKET-331 — LSP inlay hints: inline type + parameter annotations (the display-map ticket)

- **Forge ticket:** #331 38801e5a-b739-43a0-ae71-a5060c8be6b1 (feature, M21)
- **Owner:** claude (this session)
- **AAR:** 040d8d7f-9dc3-4370-a6d9-34ffb80fd975
- **Pipeline doc:** ../../pipeline/completed/331-inlay-hints.spec.md
- **Source ticket:** M21 "The workspace IDE" batch (#322-331) — the LAST of the batch
- **Status:** closed

## Summary
`let s = String::new()` renders as `let s: String = String::new()` — the muted `: String` is rust-analyzer's,
not the file's. THE render-model ticket: the editor's buffer→display mapping must learn about phantom text
that occupies columns but belongs to no buffer char, or caret/click/drag/selection/squiggles all desync.
**The plan verified the risk is containable to ONE pure function:** `LineLayout` already decouples `display`
(what's drawn) from `col_starts` (which maps only BUFFER chars), so a phantom is just "emit into `display` +
advance `col`, push no `col_starts` entry" — which makes the caret land on the code side and a click snap to
an anchor FOR FREE, and makes every rider correct with no change. Plus a pure `parse_inlay_hints` (both label
shapes, kind, padding, #309 encoding bridge), the #311 request loop + a `workspace/inlayHint/refresh` arm
(already-routed plumbing), a muted render run, and `editor.inlay_hints` + a palette toggle.

## Acceptance
`line_layout_with_inlays` at cov/MSI 100 over the truth table (hint at BOL/mid-line/EOL, adjacent hints,
hint+tab, wide-glyph neighbors, click-inside-phantom both halves) PLUS the byte-identity property (an empty
inlay slice ≡ today's `line_layout`, so OFF is provably the pre-ticket path). `parse_inlay_hints` at cov/MSI
100 (both label shapes, kinds, padding, malformed-skip, the emoji encoding pin). Headless: the viewport
request + stale-drop, the refresh arm re-fetches + replies null, the caret past a hint lands on the right
char, a click in a phantom snaps to its anchor, toggle off → the pre-ticket layout. Live pixel drive
env-blocked (screen locked) → units+mechanism. Full EARS in the pipeline spec.
