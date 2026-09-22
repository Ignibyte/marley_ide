# TICKET-106 — viewer polish, guards + persistence [M4 FINALE]

- **Forge ticket:** #106 `387b845e-4e88-4bc6-a2b2-e62ac7f7f195` (feature, M4 seq-10 FINALE; sprint #15)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `39e94a27-8066-45bd-b2cd-4dc0dcdba0f9`
- **Pipeline doc:** ../../pipeline/active/viewer-guards.spec.md
- **Status:** closed

## Summary
Guard the viewer against binary/huge files (PURE `is_probably_binary` + `viewer_size_ok`, cov/MSI 100) →
a placeholder flash; a read-only `code.tab_width` setting (mirrors TermCols) the viewer respects. Closes M4.
Deps seq-1..9 + marley_settings.

## Acceptance
The guards + `code.tab_width` at cov/MSI 100; a binary/huge open → placeholder (engine); FULL gate GREEN.
Full EARS in the spec. On close: **close M4 sprint #15**.
