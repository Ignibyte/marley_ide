# TICKET-125 — the git panel as a grid pane [M6 seq-6]

- **Forge ticket:** #125 `24f21f98-4fe6-40c2-b411-115250a10446` (feature, M6 seq-6; sprint #17)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `da81298e-96b8-451e-8285-b9ff6590b4ef`
- **Pipeline doc:** ../../pipeline/active/git-pane.spec.md
- **Status:** closed

## Summary
Move the full git-panel render (change list / stage / commit / empty state) into the Git pane dispatch;
`open_git_pane` (⌘⇧C focus-or-open); retire the M5 side-panel + `git_panel_open` + `right_open`/`viewer_split`.
Shim-only (reuses #124 first_pane_of_kind + the #115/#116 adapters, confinement unchanged). Clears
duplication #3.

## Acceptance
A Git pane renders (no side-panel, live capture); git-write confinement unchanged; FULL gate GREEN. Full EARS
in the spec.
