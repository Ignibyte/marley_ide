# TICKET-239 — Remove the "MARLEY" session header from the left rail

- **Forge ticket:** #239 (e5fde138-6d0f-4b55-91a5-60ac3d9f9d50) (chore, M14 sprint #27)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 9d29dd82-6701-474a-88a1-61e1f8b59c52
- **Pipeline doc:** ../../pipeline/completed/rail-remove-session-header.spec.md
- **Source:** chad live feedback #1 (2026-07-10) — the M14 round (#238-247); 2nd ticket.
- **Status:** closed

## Summary
The left rail's first row is a `RailLevel::Workspace` all-caps header ("MARLEY") — the top-level session name.
With one session it's redundant (the status bar + project row + tabs identify the workspace). Remove the
Workspace-level rail row (pure `tabs::rail_rows` + the shim render arm + the now-unused `RailLevel::Workspace`
variant), leaving Project / Tab / Pane rows unchanged.

## Acceptance
`rail_rows` emits no Workspace row (output begins with the first Project row); Project/Tab/Pane rows unchanged;
the rail renders no caps session header (driven capture). Pure seam cov/MSI 100. Full EARS in the spec.
