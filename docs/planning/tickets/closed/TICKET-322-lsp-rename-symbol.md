# TICKET-322 — LSP rename symbol (F2): prepareRename + the multi-file WorkspaceEdit engine

- **Forge ticket:** #322 25a0742a-b2a2-4543-b261-c3b85772f1f9 (feature, M21)
- **Owner:** claude (this session)
- **AAR:** <uuid — recorded in the notes>
- **Pipeline doc:** ../../pipeline/active/322-lsp-rename-symbol.spec.md
- **Source ticket:** M21 "The workspace IDE" batch (#322-331), the foundation ticket
- **Status:** closed

## Summary
F2 renames a symbol everywhere it lives (including files not open) as one action per file, and — the
strategic point — builds the ONE `WorkspaceEdit` apply path that #323 code-actions and (later) the
import-organizing half of #314 formatting reuse. Rename/code-actions/formatting all converge on this
applier; #322 builds it. It rides the #311 request→response recipe (its fourth consumer) with a new
`textDocument/prepareRename` + `textDocument/rename` pair and a hand-parsed `WorkspaceEdit`, applying
open buffers through the `Buffer`+undo-group model (`EditOrigin::Agent`) and closed files through new
read→apply→write machinery.

## Acceptance
F2 on a symbol used across two files renames both — the open one through its buffer (one ⌘Z reverts
it), the closed one written to disk byte-exactly — with a quiet flash when no rename is possible and
never a partial/corrupt write. Full EARS criteria (REQ-001..009) live in the pipeline spec.
