---
pipeline_id: 5974daab-c835-46f4-ab8d-ca30284e5a6d
ticket: forge#118 (2bfc66bc-19f0-49b3-9d6c-4bd781ab7c27) · local docs/planning/tickets/open/TICKET-118-layout-persist.md
aar_id: c4710e1d-af95-4cc8-b543-3bb5976a5b57
status: Phase 5 — Complete PASS
title: layout persistence (M5 FINALE)
type: feature
milestone: M5 — The Warp Workspace
references:
  - crates/marley_app/src/settings.rs (PURE: serialize_layout/restore_layout + WorkspaceLayout + persist_layout)
  - crates/marley_app/src/app.rs (SHIM: boot-restore + toggle-persist)
---

## Title
The M5 finale: persist the workspace layout across launches — Marley reopens with the source-control panel
in the state you left it.

## Scope
### In
- PURE `serialize_layout(git_panel_open)` + `restore_layout(s)` + a read/write `workspace.layout` setting +
  `persist_layout`.
- SHIM: boot restores the git-panel state; the ⌘⇧C toggle persists it.

### Out
- Persisting terminal panes / sessions (not serializable — a session can't be restored). A draggable layout.
  New bottom-bar chrome (the #94 footer stays).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — the persistable layout is the git-panel open state (an extensible `git=0/1` blob); malformed → false.
- D2 — `workspace.layout` mirrors the #95 settings round-trip; a new persist fn gets a round-trip test.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `restore_layout(serialize_layout(x))` runs, it shall equal `x`; a malformed blob → false. | unit |
| REQ-002 | WHEN `workspace.layout` is persisted and reloaded, `applied.layout` shall restore it. | unit |
| REQ-003 | gate GREEN, cov/MSI 100 on serialize/restore + the setting; the shim masked. | gate |

## Phase Plan
- **P2** — serialize/restore_layout + WorkspaceLayout + persist_layout; the boot-restore + toggle-persist; test plan.
- **P3** — implement (settings.rs + app.rs).
- **P3.5** — 1 critic: serialize/restore MSI + the round-trip; applied_from/persist; the boot-restore.
- **P4** — the round-trip tests (cov/MSI 100) + gate GREEN.
- **P5** — docs, AAR, architecture doc, archive, close #118 → **close M5 sprint #16**.
