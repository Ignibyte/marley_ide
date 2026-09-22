# TICKET-142 — cwd + git branch in the title bar [M8 seq-6]

- **Forge ticket:** #142 `1e361d9f-4f16-4f03-b824-1d509091528d` (feature, M8; sprint #19)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `b9ba6060-ab35-4a89-9b43-9a00b5486518`
- **Pipeline doc:** ../../pipeline/active/titlebar-context.spec.md
- **Status:** closed

## Summary
Show the abbreviated cwd (`~/…/Marley`) + the current git branch in the unified titlebar (#138). Pure
`titlebar.rs` (abbreviate_path, branch_from_git_head parsing `.git/HEAD`, titlebar_label compose); the shim
reads `HOME` + `{project_root}/.git/HEAD` and renders the label left of the search. Deps #138 + M2 status_bar.

## Acceptance
The 3 pure fns at cov/MSI 100; the titlebar shows `~/…/Marley · main` (live capture); FULL gate GREEN.
Full EARS in the spec.
