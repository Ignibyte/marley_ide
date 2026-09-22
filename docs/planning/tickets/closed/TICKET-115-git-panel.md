# TICKET-115 — the git changes panel (source control) [M5 seq-9]

- **Forge ticket:** #115 `a3897c5f-fecf-48fb-a199-8f1327434739` (feature, M5 seq-9; sprint #16)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `18d512ca-f7d2-4770-b47e-54d4af83e9b0`
- **Pipeline doc:** ../../pipeline/active/git-panel.spec.md
- **Status:** closed

## Summary
`parse_status(porcelain)` → `Vec<ChangeRow>` + `change_summary` (git_diff.rs, cov/MSI 100) + a ⌘⇧C
right-side git panel (masked `git status --porcelain` → the per-file change list + "No open changes").
Row-click opens the diff. Deps #102 + #004 + #114. Commit flow is #116.

## Acceptance
parse_status/change_summary at cov/MSI 100 (XY/staged/status/untracked/empty); ⌘⇧C shows the git panel
(live capture); FULL gate GREEN. Full EARS in the spec.
