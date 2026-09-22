# TICKET-323 — LSP code actions + quick fix (⌘.): the picker + codeAction/resolve

- **Forge ticket:** #323 056e3f20-fa74-4d16-9d3e-6631daf27dda (feature, M21)
- **Owner:** claude (this session)
- **AAR:** cbce96e2-8719-472c-8073-b5c01645fe0f (submitted; F-VAL-1 + PR-claude-lsp-advertise-client-capability-001)
- **Pipeline doc:** ../../pipeline/completed/323-lsp-code-actions.spec.md
- **Source ticket:** M21 "The workspace IDE" batch (#322-331)
- **Status:** closed

## Summary
⌘. at the caret asks `textDocument/codeAction` and offers the server's quick-fixes + assists (auto-import
— the #313 cut — plus rust-analyzer's assist catalog) in a picker; accepting applies the fix through the
#322 WorkspaceEdit engine, resolving a deferred edit via `codeAction/resolve` first when needed. The
apply is #322's engine unchanged (the whole point of that foundation); the new work is the codeAction
request, the Command-vs-CodeAction parse, the resolve round-trip, and the picker.

## Acceptance
Delete a `use` line → a squiggle → ⌘. → "import X" → accept → the use line is back on disk, applied
through the #322 engine; a Command-only action is skipped and counted; no provider → a quiet flash. Full
EARS criteria (REQ-001..008) live in the pipeline spec.
