# TICKET-235 — Scope-driven top bar (focused-workspace indicator + workspace-scoped actions grouped left)

- **Forge ticket:** #235 (ca7bdb07-de31-484f-8c70-e97881c0a061) (feature, M13 sprint #26)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 5390a623-f206-495b-a62c-16ab0776a173
- **Pipeline doc:** ../../pipeline/active/scope-driven-topbar.spec.md
- **Source ticket:** M13 sprint #26 (chad feedback #6 + #7)
- **Status:** closed

## Summary
Regroup the top bar by SCOPE: add a focused-workspace indicator (the active project's name · branch) so
it's clear the buttons act on the focused workspace (#7), and move the cockpit tabs from the top-right
into the left cluster with the other workspace-scoped actions (#6). The dispatch already targets the
active project (= the focused workspace, #233), so switching focus (⌘⇧]) re-targets the buttons +
updates the indicator with no dispatch change.

## Acceptance
A pure `focused_workspace_indicator(name, branch)` (name · branch, truncated; cov/MSI 100); the top bar
renders the indicator + the cockpit tabs in the left group; and switching the focused workspace updates
the indicator (driven). Full EARS in the pipeline spec. A top-bar switcher + a flex refactor are deferred.
