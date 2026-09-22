---
pipeline_id: 38f2708f-600b-4ac2-a45f-6bd8e6612ef4
ticket: forge#122 (43b3a888-195a-4af0-bb61-b97b180678a1) · local docs/planning/tickets/open/TICKET-122-grid-layout.md
aar_id: f4f5806d-af25-44c5-9396-0475125e7d7b
status: Phase 5 — Complete PASS
title: persist + boot the pane grid (M6 seq-3)
type: feature
milestone: M6 — The Warp Layout
references:
  - crates/marley_app/src/grid_layout.rs (NEW PURE: GridLayout, serialize_grid, restore_grid)
  - crates/marley_app/src/settings.rs (workspace.grid setting + persist_grid)
  - crates/marley_app/src/app.rs (SHIM: boot rebuild + persist; retires #121 TEMP)
  - crates/marley_app/src/lib.rs (mod grid_layout)
---

## Title
Marley boots into a saved pane arrangement, and remembers the one you leave. Lets us open straight into a
`[terminal | files | code | git]` Warp layout — and retires the #121 temporary boot hack.

## Scope
### In
- PURE `serialize_grid`/`restore_grid` over a flat row of pane kinds (axis + kinds) + a `workspace.grid`
  setting + `persist_grid`.
- SHIM: boot rebuilds the workspace from the restored grid; persists on split/close/open-pane.

### Out
- Exact nested-split fidelity + per-pane ratios (a flat row; ratios arrive with #130 drag-resize). Restoring
  a non-Terminal *first* pane (it restores as Terminal — the boot pane).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — the grid is modelled as a flat row (`GridLayout { axis, kinds }`); serialize walks the tree's leaves
  in DFS order → `"H:t,f,c"` (or `"t"` for one pane); malformed/empty → a single Terminal.
- D2 — `workspace.grid` mirrors the #118 settings round-trip; the boot rebuild is Workspace::new (Terminal 0)
  then open each of `kinds[1..]`, refocus pane 0.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `restore_grid(serialize_grid(g))` runs, the axis + kind sequence shall equal g's leaves. | unit |
| REQ-002 | WHEN the blob is empty or malformed, `restore_grid` shall yield a single Terminal. | unit |
| REQ-003 | WHEN `workspace.grid` is persisted + reloaded, `applied.grid` shall restore it. | unit |
| REQ-004 (visual) | WHEN booted with `grid="H:t,f,c,g"`, four panes (terminal/files/code/git) shall render in a row. | live capture |
| REQ-005 | gate GREEN, cov/MSI 100 on grid_layout + the setting; the shim masked. | gate |

## Phase Plan
- **P2** — GridLayout + serialize/restore_grid + the setting + the boot rebuild; test plan.
- **P3** — implement (grid_layout.rs + settings.rs + lib.rs + app.rs).
- **P3.5** — 1 critic: serialize/restore MSI, malformed default, the boot rebuild.
- **P4** — round-trip tests (cov/MSI 100) + a LIVE capture (4 panes in a row) + gate GREEN.
- **P5** — docs, AAR, archive, close #122.
