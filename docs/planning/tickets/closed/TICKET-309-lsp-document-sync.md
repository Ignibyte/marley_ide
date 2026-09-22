# TICKET-309 — LSP document sync + the UTF-16 position bridge

- **Forge ticket:** #309 5bc7247a-15e9-47f7-a2ce-826b5bfddb65 (feature, M20)
- **Owner:** fabd8254-8504-4118-9e8e-8827ad24687a
- **AAR:** 697adc7c-9c71-4992-9f42-c9097e72b37c
- **Pipeline doc:** ../../pipeline/active/309-lsp-document-sync.spec.md
- **Source ticket:** forge #309 (M20 batch #308-317)
- **Status:** closed

## Summary
Flow every editor Buffer edit to the language server as `textDocument/didChange`, and route every
wire position through ONE pure encoding-aware **position bridge** (char offset ↔ LSP `character`
column, UTF-16 by default — an emoji is 1 char / 2 UTF-16 units). Wire the doc lifecycle
(didOpen/didChange/didSave/didClose) onto #308's `marley_lsp` host. Extend `BufferDelta` with the
inserted text so incremental change events fold from `edits_since`. The bridge is the load-bearing
seam every M20 feature (#310-317) addresses through.

## Acceptance
The bridge maps emoji/BMP/past-EOL/utf-8/utf-32 correctly (REQ-001..003); didOpen-before-didChange
+ didClose (REQ-004); ordered incremental change events replay to the buffer text with monotonic
version (REQ-005); full-text fallback under Full sync (REQ-006); didSave after write (REQ-007). Full
EARS in the pipeline spec.
