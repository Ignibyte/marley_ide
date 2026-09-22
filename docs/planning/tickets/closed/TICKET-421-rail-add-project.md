# TICKET-421 — Add-project affordance in the rail

- **Ticket:** LOCAL #421 (feature, M31)
- **Tags:** rail, projects, simple-rail
- **Created:** 2026-08-12
- **Status:** closed (2026-08-12 — shipped: the header ＋ + AddProject menu + the two shipped verbs; 2137 tests green; driven live; GATE GREEN [diff] 15/15)

## Summary

Projects can only be added via ⌘O or the empty-workspace launcher card (app.rs:8500-8522,
:10640-10715; spec sweep finding — there is NO palette door, "open-project" lives only in the
keymap) — the rail itself has no affordance (the ＋ exists only on section headers and creates
tabs). Per #418's design: an add-project verb in the rail
(placement per the settled React design — Workspace header ＋ or a trailing "Add project…"
row) wiring to the EXISTING `open_project_picker` / `new_empty_workspace` paths. No new
project machinery. Recall pin F-#236: project add/close shifts indices — any index-keyed
view state (collapsed_projects et al.) must stay correctly mapped.

## Acceptance

Headline: a project can be added from the rail alone (picker opens, chosen folder appears as
a project row, active project switches to it), matching #418's React design 1:1; existing
open paths unchanged. Full EARS in the queued spec
(`docs/planning/pipeline/queued/421-rail-add-project.spec.md`).
