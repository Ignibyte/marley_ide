# TICKET-053 — project model: discover the project root

- **Forge ticket:** #53 `c8e50f7b-316e-4724-8c98-0f2580aadfb7` (feature, M2.A — Project, Files & Search, seq-1)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `150002f3-f519-48de-bbe3-00911ea86b29`
- **Pipeline doc:** ../../pipeline/active/project-model.spec.md
- **Source ticket:** forge sprint #9 `fcb8d0fe-0630-4f4e-b465-4360fdb40d89` (M2.A — Project, Files & Search)
- **Status:** closed

## Summary
The M2 moat foundation: a new `marley_project` crate with `Project { root, name, is_git }` +
`Project::discover_in(start)` — walk up for the NEAREST `.git` ancestor (dir OR file); no-git →
root=start, is_git=false. A pure `name_for` (basename + `/` fallback) keeps both branches testable.
cov/MSI 100 via tmp-tree tests. #54/#55/#56 build on this. No UI.

## Acceptance
`discover_in` + `name_for` at cov 100/MSI 100 (nearest .git; .git-file worktree; no-git fallback;
basename + `/` fallback); FULL gate GREEN. Full EARS in the pipeline spec.
