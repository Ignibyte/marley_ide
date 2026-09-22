# TICKET-055 — project file listing (walk, skip .git/ignores)

- **Forge ticket:** #55 `11e051df-031d-4b9e-a3be-ce34c34e9d88` (feature, M2.A seq-3)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `f30f381f-069f-482f-8bf4-785624f43db7`
- **Pipeline doc:** ../../pipeline/active/file-listing.spec.md
- **Source ticket:** forge sprint #9 `fcb8d0fe-0630-4f4e-b465-4360fdb40d89` (M2.A — Project, Files & Search)
- **Status:** closed

## Summary
Add to `marley_project`: `should_skip(name)` (curated skip-set) + `FileListing { files, truncated }` +
`list_files_in(root)` (a symlink-safe, bounded recursive walk collecting non-skipped files relative to
root, sorted; a skipped dir prunes its subtree; unreadable dirs skipped, no panic). The bound is
testable via the private `list_files_capped(root, cap)`. Feeds #56 (file tree) + #57 (fuzzy-open).

## Acceptance
should_skip + list_files_in at cov/MSI 100 (skip-set; prune; relative+sorted; cap→truncated;
unreadable no-panic); FULL gate GREEN. Full EARS in the pipeline spec.
