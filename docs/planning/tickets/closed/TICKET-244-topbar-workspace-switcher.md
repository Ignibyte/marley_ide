# TICKET-244 — Make the top-bar focused-workspace indicator a click-to-switch popover

- **Forge ticket:** #244 (3f762a4d-e498-4375-82ba-dfc38cdf511b) (feature, M14 sprint #27)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** c3aa9110-dc55-4647-8679-d53252dc0b44
- **Pipeline doc:** ../../pipeline/completed/topbar-workspace-switcher.spec.md
- **Source:** the M14 round; 7th ticket. A #235 follow-on (independent of the #242 editable editor).
- **Status:** closed

## Summary
The #235 top-bar focused-workspace indicator ("name · branch") is display-only. Make it interactive: click it
to open a popover listing the open workspaces; click a row to switch the focused workspace. Reuses the shipped
#233 `switch_project` + `sync_active_project` + `persist_grid` rail-click idiom and the #166 `context_menu`
overlay pattern; a pure `titlebar::workspace_switcher_rows` display model carries the tested logic. Switches the
active PROJECT within today's single Workspace container — it does NOT touch the multi-workspace re-architecture.

## Acceptance
Clicking the indicator opens/closes a popover of the open workspaces; clicking a row switches the focused
workspace and dismisses; the backdrop dismisses; the active row is highlighted. Pure model cov/MSI 100 (or
coverage+regression if 0 viable mutants); driven-validated on the running app. Full EARS in the pipeline spec.
