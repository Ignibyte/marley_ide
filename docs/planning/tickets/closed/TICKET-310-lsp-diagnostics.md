# TICKET-310 — LSP diagnostics: squiggles + gutter + counts, merged into the M18 lane

- **Forge ticket:** #310 d82fad8b-f689-440c-9824-6fb61c1cff6a (feature, M20)
- **Owner:** fabd8254-8504-4118-9e8e-8827ad24687a
- **AAR:** e7c2437d-2428-4afb-b1d1-c6ae46a50790
- **Pipeline doc:** ../../pipeline/active/310-lsp-diagnostics.spec.md
- **Source ticket:** forge #310 (M20 batch #308-317)
- **Status:** closed

## Summary
The first user-visible LSP payoff: consume `textDocument/publishDiagnostics` (REPLACE/CLEAR per uri),
map each diagnostic's range to buffer offsets via #309's position bridge, MERGE with the M18
(#289-291) failed-terminal-block gutter rows into one lane, paint squiggle underlines by severity,
show `N errors, M warnings` in the status bar, and let F8 walk the merged set. `lsp-types` re-added,
confined to the parse seam; the app consumes a pure Marley `Diag`.

## Acceptance
publish REPLACE/empty-CLEARS (REQ-001); range→CharOffset span emoji-correct (REQ-002); merge +
dedupe + severity rank (REQ-003); multi-line/zero-width underline runs (REQ-004); status counts
(REQ-005); F8 over merged (REQ-006); live: introduce a compiler error → squiggle+gutter+count, fix
it → clears (REQ-007). Full EARS in the pipeline spec.
