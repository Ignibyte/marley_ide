# TICKET-113 — the file explorer (icons + dimming) [M5 seq-7]

- **Forge ticket:** #113 `f35494de-7d89-479b-a3e2-2f8736a76fb7` (feature, M5 seq-7; sprint #16)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `7f60d2e5-5276-4607-a70b-c7d7b3a4a487`
- **Pipeline doc:** ../../pipeline/active/file-explorer.spec.md
- **Status:** closed

## Summary
`file_icon(name)` (extension → glyph) + `entry_is_dimmed(name)` (dotfiles) in a new file_tree_view.rs
(cov/MSI 100); the left-dock Files tree renders file-type icons + dims dotfiles. Deps #56. (Gitignore signal
absent → dotfile dimming; Files stays in the dock — movable-pane deferred to the seq-8 dispatch.)

## Acceptance
file_icon/entry_is_dimmed at cov/MSI 100; the Files tree shows type icons + a dimmed dotfile (live capture);
FULL gate GREEN. Full EARS in the spec.
