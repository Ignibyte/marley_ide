---
pipeline_id: 8220d02e-771b-48cf-98ca-7327d3b685b8
ticket: forge#295 (8b6a333d-83b0-4d07-8164-fa01c5bd5c38) · local docs/planning/tickets/open/TICKET-295-cross-tab-diagnostics.md
aar_id: e78f23f9-f8b1-461e-83d7-9b6871fdb3da
status: Phase 5 — Complete PASS
title: Diagnostics gutter aggregates the last failed block across ALL terminal grids
type: bug
milestone: M22
references:
  - crates/marley_app/src/app.rs
  - crates/marley_app/src/tabs.rs
  - crates/marley_app/src/links.rs
---

## Title
`open_file_diagnostic_rows` (app.rs:8405, the #289 diagnostics-gutter source, also feeding #290 F8-nav and
#291 trace frames) iterates `self.workspace().states()`, and `workspace()` = `active_project().tab_grid(
terminal_grid_index())` = the ACTIVE tab's grid if terminal, else the FIRST terminal tab's grid. So with the
editor tab active (the normal case for viewing diagnostics) it scans only the FIRST terminal tab — a failed
command block in ANY other terminal tab never lights the gutter. Fix: iterate EVERY terminal grid in the active
project (a new pure `Project::terminal_grids()`), keeping the per-grid failed-block body + the `merged_rows`
union unchanged.

## Scope
### In
- `crates/marley_app/src/tabs.rs` (pure, cov/MSI 100): `Project::terminal_grids(&self) -> impl Iterator<Item =
  &PaneGrid<S>>` = `self.tabs.iter().filter_map(|t| t.grid())` — the per-project sibling of the existing
  all-projects `Workspace::grids()` (tabs.rs:437). Yields every terminal tab's grid in tab order; skips
  editor/cockpit tabs.
- `crates/marley_app/src/app.rs` (shim, `#[cfg_attr(test, mutants::skip)]`): `open_file_diagnostic_rows`
  replaces `for (_id, state) in self.workspace().states()` with
  `for grid in self.shell.active_project().terminal_grids() { for (_id, state) in grid.states() { …unchanged… } }`.
  The last-failed-block body (`exit_status_kind` → `diagnostics_for_file` + `trace_diagnostic_rows`) and the
  `merged_rows(lsp, terminal_rows)` union are byte-identical.

### Out (explicitly deferred)
- The pure row computation (`diagnostics_for_file`, `trace_diagnostic_rows`, `merged_rows`) — unchanged (#289/
  #291/#310).
- The LSP lane, `open_file_diag_spans`, F8 nav — untouched.
- Filtering by project: scoped to the ACTIVE project only (matches today's `workspace()` = active project).

## Reference (§20)
N/A — Marley-specific. The terminal↔editor **diagnostics fusion** (a failed command Block's `file:line` refs
lighting the editor gutter) is Marley's own wedge — Zed has no command-block model and no analog (roadmap
Phase C). No reference-app behavior to match; no copyleft source consulted.

### Prior art
1. **OUR own code (highest-yield):** `Workspace::grids()` / `grids_mut()` (tabs.rs:437/445) ALREADY iterate every
   terminal grid across every project via `tabs.iter().filter_map(|t| t.grid())`. `Project::terminal_grids()` is
   the per-project sibling — the SAME `filter_map(grid)` idiom, one scope narrower. And `merged_rows` (marley_lsp,
   tested #310) already owns the union+sort+dedup. So the fix reuses both; only the enumeration SCOPE widens
   (one grid → all grids in the project). No new aggregation logic.
2. **Behavior maps / published:** n/a (Marley-specific fusion).

## Locked-In Decisions
- **D1 — `Project::terminal_grids()`, not a widened `workspace()`.** `workspace()` intentionally returns the ONE
  active-or-first grid (many callers depend on that single-grid meaning); the diagnostics shim is the caller that
  wants ALL grids, so it gets its own enumerator. Mirrors the existing `grids()` split (per-project vs all).
- **D2 — Active project only.** Today's `workspace()` is already active-project-scoped; `terminal_grids()` keeps
  that (a background project's terminals don't leak into the active editor's gutter). The all-projects `grids()`
  is deliberately NOT reused.
- **D3 — Union unchanged.** Each grid contributes its last-failed-block rows; `merged_rows` (already tested)
  dedups/sorts. Iterating a grid that is ALSO the active one is safe (each grid appears once; merged_rows dedups).
  Order-independent per the #289 invariant.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `Project::terminal_grids()` shall yield every terminal tab's grid in the project (tab order) and no grid for an editor/cockpit tab. | tabs.rs unit: a project with [terminal, editor, terminal] tabs → exactly the 2 terminal grids, in order |
| REQ-002 | `Project::terminal_grids()` on a project with NO terminal tabs shall yield nothing (empty). | tabs.rs unit: an editor-only project → empty iterator |
| REQ-003 | WHEN the active project has ≥2 terminal tabs and the editor is active, `open_file_diagnostic_rows` shall consult the last failed block of EVERY terminal grid (not just the active-or-first). | headless: a multi-terminal-tab workspace + editor open → the shim runs over all grids without panic; the positive path (a ref in tab-B's failed block lights the gutter) is carried by REQ-001 (all grids enumerated) ∘ the byte-identical per-grid body already proven by #289/#291 + the #291 live drive |

## Testing boundary (honest)
`terminal_grids()` is pure → cov/MSI 100 in tabs.rs. `open_file_diagnostic_rows` is `mutants::skip` +
app.rs-coverage-excluded (the gate regex). A synthetic **failed block** cannot be seeded from an app test
(`BlockList::open_running` is `pub(crate)` in terminal_blocks; blocks come from real PTY output + shell
integration), so the POSITIVE end-to-end is proven by composition (REQ-001 all-grids ∘ the unchanged, already-
tested per-grid body) rather than a seeded-failure headless test. The headless multi-tab test proves the widened
loop executes safely.

## Phase Plan
- **P2 Design** — the `terminal_grids()` signature + the exact shim rewrite; the test plan (REQ-001/002 tabs.rs
  units; REQ-003 headless multi-tab safety + the composition argument).
- **P3 Implement** — add `terminal_grids()`; rewrite the shim loop.
- **P3.5 Inspect** — adversarial: active-project-only scope, no double-count, order-independence, the borrow of
  `active_project()` across the loop, no regression to the single-terminal case (#291's proven path).
- **P4 Validate** — tabs.rs units cov/MSI 100; the headless multi-tab test; `scripts/gates.sh --diff` green.
- **P5 Complete** — archive, AAR, close #295.
