# TICKET-236 — Collapsible workspace rail tree + focused-workspace highlight

- **Forge ticket:** #236 (1c9c86a3-b953-45b2-b68b-8534a1b66ffc) (feature, M13 sprint #26)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** f0bf7067-1fdb-434e-93ee-59496d4c9ac3
- **Pipeline doc:** ../../pipeline/active/collapsible-rail.spec.md
- **Source ticket:** M13 sprint #26 (chad feedback #8 + the rail-highlight ask)
- **Status:** closed

## Summary
The left rail becomes a collapsible disclosure tree: each workspace (project) row gets a ▸/▾ chevron that
hides/shows its tabs (+ nested panes), and the focused workspace (the active project, #233) gets a clear
highlight background (not just today's subtle bright text). Extends the pure `tabs::rail_rows` with a
`collapsed` param + a `RailRow.collapsed` flag; the shim renders the chevron/toggle + the highlight.

## Acceptance
`rail_rows(ws, collapsed)` omits a collapsed project's Tab/Pane rows + marks the Project row collapsed
(cov/MSI 100); the Project row renders a ▸/▾ chevron toggling collapse; the active project row shows a
prominent highlight; driven — collapse hides tabs, the active highlight shows. Full EARS in the spec.
Persisting collapse state + collapsing the Session header are deferred.
