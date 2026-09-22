# TICKET-128 — open Files/Code/Git panes on demand (the openers) [M6]

- **Forge ticket:** #128 `3671a317-44bc-4677-b9c1-1a245cb9958c` (feature, M6; sprint #17)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `2c2ea08f-1dce-4e1b-8824-71d3597cb0f2`
- **Pipeline doc:** ../../pipeline/active/openers.spec.md
- **Status:** closed

## Summary
`OpenAction` + `open_or_focus(target)` (workspace.rs, cov/MSI 100) + `open_files_pane` + a "Files" palette
command; DRY the code/git openers through `open_or_focus`. Makes every pane kind user-openable. Deps
#120/#124/#125.

## Acceptance
open_or_focus at cov/MSI 100; a "Files" command opens a FileTree pane (code-reviewed; render proven); FULL
gate GREEN. Full EARS in the spec.
