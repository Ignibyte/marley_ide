# TICKET-102 — a read-only git-diff seam

- **Forge ticket:** #102 `d0450d67-eb2a-49eb-b807-d47bd193fe3d` (feature, M4 seq-6; sprint #15)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `14643a8e-1d40-422e-bf3f-10a58728c152`
- **Pipeline doc:** ../../pipeline/active/git-diff-seam.spec.md
- **Status:** closed

## Summary
Parse `git diff` into `Vec<FileDiff>` (PURE `parse_diff` in git_diff.rs, cov/MSI 100) + a masked read-only
`git diff` spawn + a ⌘⇧D working-diff overlay (colored +/-/context). Deps #004 + #53.

## Acceptance
parse_diff at cov/MSI 100 (file/hunk/line/path/empty); ⌘⇧D shows the colored working diff (live git spawn/
engine); FULL gate GREEN. Full EARS in the spec.
