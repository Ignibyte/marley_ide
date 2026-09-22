# TICKET-325 — Workspace symbols (⌘T): jump to any symbol in the project by name

- **Forge ticket:** #325 e3000b55-3787-4d70-8470-156fa935d128 (feature, M21)
- **Owner:** claude (this session)
- **AAR:** d18ca6f9-f7e7-439b-8b7d-47ed4b8bd232
- **Pipeline doc:** ../../pipeline/active/325-workspace-symbols.spec.md
- **Source ticket:** M21 "The workspace IDE" batch (#322-331)
- **Status:** closed

## Summary
⌘T opens a name query and lists matching symbols from anywhere in the workspace (server-filtered + fuzzy
via `workspace/symbol`); Enter jumps to the definition. The cross-file sibling of #304's in-file ⌘⇧O.
Server-filtered (each keystroke re-queries, debounced one pump tick, #313 park; the response replaces the
list; a stale answer for an older query is dropped). Pure marley_lsp parse of BOTH `SymbolInformation` and
`WorkspaceSymbol` shapes (missing range → line 0). Multi-root: queries every Ready host + merges.

## Acceptance
⌘T (with a Ready `workspaceSymbolProvider`) opens a picker + sends `workspace/symbol {query}` per keystroke;
the parser normalizes both shapes + missing-range + skips malformed; the list replaces + drops stale-query
answers + caps with "+N more"; Enter reuses #312 open_and_place_caret + NavStack (⌃- returns), a failed open
flashes + moves nothing; no host/provider → quiet flash; the handshake advertises workspace.symbol.symbolKind
(upgrade). Full EARS criteria in the pipeline spec.
