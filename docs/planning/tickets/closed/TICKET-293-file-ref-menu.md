# TICKET-293 — file-ref context menu in terminal output

- **Forge ticket:** #293 `05619daf-a14a-4f7a-8b32-c360155f2815` (feature, M18)
- **Owner:** autonomous /goal run (sprint #31 — M18 Terminal↔Editor Fusion)
- **AAR:** `6d5fbb36-903b-4be8-bcfe-50bcaef68054`
- **Pipeline doc:** ../../pipeline/active/293-file-ref-menu.spec.md
- **Status:** closed

## Summary
Broaden the single left-click→open (#212) with a RIGHT-click context menu on a
`path:line` file ref in block output: **Open in editor**, **Open in split-right**,
**Reveal in file tree**, **Copy path**. Reuses the #166 context-menu infra, #212
`open_file_at`, #246 `split_file_pane`, #190 `resolve_under_root`, and a new pure
`FileTree::reveal(path)` (expand ancestor dirs + return the visible-row index).

## Acceptance
Right-click a `path:line` in terminal output → a 4-item menu; "Open in split-right"
opens that file in a split pane; "Reveal in file tree" scrolls the tree to it.
Full EARS in the pipeline spec.
