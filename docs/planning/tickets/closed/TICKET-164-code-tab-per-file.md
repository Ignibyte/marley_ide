# TICKET-164 — M10: a code tab per file (dedup by path)

- **Forge ticket:** #164 `72e7d16f-9565-415f-87b0-a4b419dcc2d4` (feature, M10; sprint #21)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `320ef2f6-95d2-4d51-abcd-b3937f6204ae`
- **Pipeline doc:** ../../pipeline/active/code-tab-per-file.spec.md
- **Status:** closed

## Summary
open_or_switch_code keys by PATH: re-opening a file refreshes + switches to its existing tab; a new file
appends its own code tab (title = file name). Pure-only (tabs.rs); replaces #154's single-reused-tab D1.
Deps #154, #161.

## Acceptance
Pure tests at cov/MSI 100 (the path predicate mutation-killed); driven — two files → two rail code tabs,
re-open → no third; FULL gate GREEN.
