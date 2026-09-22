# TICKET-311 — LSP hover: types + docs in an overlay card (⌘K / mouse dwell), markdown-lite

- **Forge ticket:** #311 ac07a89b-b4d0-4b25-aa67-687988f575a2 (feature, M20)
- **Owner:** fabd8254-8504-4118-9e8e-8827ad24687a
- **AAR:** fedf1a48-84e2-4122-b959-c0efe8a7f56e
- **Pipeline doc:** ../../pipeline/active/311-lsp-hover.spec.md
- **Source ticket:** forge #311 (M20 batch #308-317)
- **Status:** closed

## Summary
The first interactive LSP round-trip: press ⌘K in an editor tab (or rest the mouse ~400ms over a
symbol) → `textDocument/hover` → a rounded overlay card with the type signature + docs, code
highlighted by the existing tree-sitter/lexer. Builds the GENERAL request→response→consumer path on the
LSP host (today only `initialize` is handled) that #312/#313/#317 reuse. Markdown-lite (fences / inline
code / headings / paragraphs — never panics), a pure dismissal table, a stale-response guard (drop an
answer for a moved-on position), empty→nothing. Surfaces #310's deferred `diagnostic_at_row` +
`Diag.message` (a hovered squiggle shows its message).

## Acceptance
⌘K/dwell requests hover at the caret through the #309 bridge; the card renders markdown-lite with
highlighted fences, clamped to the window; an edit/Esc/caret-move/scroll dismisses it, mouse-into-card
does not; a stale response is dropped; empty→nothing; a hovered diagnostic span shows its message. Full
EARS criteria live in the pipeline spec.
