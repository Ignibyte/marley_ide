# TICKET-245 — Persist the per-workspace rail collapse-state across restart

- **Forge ticket:** #245 (07560d41-0da6-4b6f-8af2-187f27c4fa37) (feature, M14 sprint #27)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** b1f18e31-d587-4589-a563-8655830b5a45
- **Pipeline doc:** ../../pipeline/completed/persist-rail-collapse.spec.md
- **Source:** the M14 round; 6th ticket. A #236 follow-on (independent of the #242 editable editor).
- **Status:** closed

## Summary
The rail's per-project collapse-state (`collapsed_projects`, #236) is in-memory only, so a collapsed project
reopens on restart. Persist it keyed by the stable project ROOT (not the shifting index): a `CollapsedProjects`
`Vec<String>` setting mirroring `Recents` + a `persist_collapsed` fn + pure `collapsed_roots` (save) /
`collapsed_indices` (restore) helpers. Persist on toggle + close; restore on boot.

## Acceptance
The pure mapping helpers round-trip (out-of-range/unknown roots skipped, cov/MSI 100); toggling persists (by
root) + boot restores; collapse → quit → relaunch → still collapsed (driven). Full EARS in the spec.
