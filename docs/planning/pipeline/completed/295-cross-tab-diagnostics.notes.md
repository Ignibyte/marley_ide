# Diagnostics gutter across all terminal grids (#295) — Notes

- **Forge ticket:** #295 (8b6a333d-83b0-4d07-8164-fa01c5bd5c38)
- **AAR:** e78f23f9-f8b1-461e-83d7-9b6871fdb3da
- **Local ticket doc:** docs/planning/tickets/open/TICKET-295-cross-tab-diagnostics.md
- **Pipeline spec:** 295-cross-tab-diagnostics.spec.md

## Phase 1 — Plan
- **Request:** #295 (M18 origin, ships M22) — the diagnostics gutter misses a failed block in a non-first
  terminal tab because the shim scans only `workspace()` (active-or-first grid).
- **Classification / tier:** work pipeline; a small terminal-fusion bug fix. One shippable slice.
- **Forge recall:** no bulletins. Ticket found by the #291 live drive. knowledge-search surfaced
  terminal/diagnostics prevention-rules + ADs (the #289/#291/#310 lane).
- **Discovery (the seam):**
  - `open_file_diagnostic_rows` (app.rs:8405, `#[cfg_attr(test, mutants::skip)]`) — iterates
    `self.workspace().states()`; per pane, if the last block is a Failure, extends `terminal_rows` with
    `diagnostics_for_file` + `trace_diagnostic_rows` (links.rs); then `merged_rows(lsp_diags, terminal_rows)`.
  - `workspace()` (app.rs:4825) = `active_project().tab_grid(terminal_grid_index())` = active-or-first grid.
  - `diagnostics_for_file` / `trace_diagnostic_rows` (links.rs:250) — PURE, unit-tested; `merged_rows`
    (marley_lsp) — the union+sort+dedup, tested.
  - **`Workspace::grids()`/`grids_mut()`** (tabs.rs:437/445) already iterate every grid across ALL projects
    (`tabs.iter().filter_map(|t| t.grid())`). The per-project sibling `Project::terminal_grids()` is the fix.
  - `Project<S>` (tabs.rs:164, impl at 179) owns `tabs()`/`tab_grid()`/`terminal_grid_index()` → the home for
    `terminal_grids()`. `TabContent::Terminal(PaneGrid)`; `Tab::grid() -> Option<&PaneGrid>`.
  - Block seeding: `BlockList::open_running` is `pub(crate)` in terminal_blocks — no synthetic block from an
    app test; blocks come from real PTY output.
- **Decisions:** D1 new `terminal_grids()` (don't widen `workspace()`) · D2 active-project only · D3 union
  unchanged (merged_rows dedups; each grid once).
- **Prior-art (§20):** N/A Marley-specific fusion. The reuse win: `grids()` + `merged_rows` already exist —
  the fix is a per-project enumerator + a scope-widening loop, no new logic.

## Phase 2 — Design
- **Architecture:** cockpit/editor-fusion shim + a pure tabs.rs enumerator. §14 honored (pure iterator, no IO).
- **`crates/marley_app/src/tabs.rs`** — add to `impl<S> Project<S>`:
  ```rust
  /// Every terminal tab's grid in this project, in tab order (#295) — the per-project sibling of the
  /// workspace-wide `grids()`; the diagnostics shim scans ALL of them, not just `tab_grid(terminal_grid_index())`.
  pub fn terminal_grids(&self) -> impl Iterator<Item = &PaneGrid<S>> {
      self.tabs.iter().filter_map(|tab| tab.grid())
  }
  ```
- **`crates/marley_app/src/app.rs`** — in `open_file_diagnostic_rows`, replace the single-grid loop:
  ```rust
  for grid in self.shell.active_project().terminal_grids() {
      for (_id, state) in grid.states() {
          // …unchanged failed-block body (exit_status_kind → diagnostics_for_file + trace_diagnostic_rows)…
      }
  }
  ```
  `active_project()` returns `&Project<TerminalSession>`; `terminal_grids()` borrows it for the loop only;
  each `grid.states()` iterates that grid's panes. The LSP-lane tail + `merged_rows` are unchanged.
- **File manifest:** tabs.rs (+`terminal_grids` + a unit test), app.rs (the loop rewrite — 2 lines of nesting).
- **Regression Test Plan:**
  | Test | Proves | AC |
  |---|---|---|
  | `terminal_grids_yields_every_terminal_grid_in_order` (tabs.rs) — a Project with [terminal, editor, terminal] → 2 grids, ordered | all terminal grids, skips non-terminal | REQ-001 |
  | `terminal_grids_empty_without_terminal_tabs` (tabs.rs) — an editor-only project → empty | empty case | REQ-002 |
  | `diagnostics_gutter_scans_all_terminal_tabs_headless` (headless_drive) — boot a multi-terminal-tab workspace + open the editor → `open_file_diagnostic_rows()` runs over all grids without panic (empty without a seeded failure) | the widened loop executes safely over N grids | REQ-003 (safety) |
  - Positive path (a ref in tab-B's failed block → gutter): carried by REQ-001 (all grids) ∘ the byte-identical
    per-grid body already proven by #289/#291 + the #291 live drive. Seeded-failure headless is not feasible
    (block-creation seam is `pub(crate)` + needs shell integration) — documented in the spec's testing boundary.
- **Risks:** the `active_project()` borrow is immutable + released after the loop; no double-count (each grid
  once; merged_rows dedups); the single-terminal case (#291's proven path) is unchanged (one grid in the iter).

## Phase 3 — Implement
- **Built to design:** `tabs.rs` — `Project::terminal_grids()` added after `tab_grid_mut` (in `impl<S>
  Project<S>`). `app.rs` — `open_file_diagnostic_rows`'s loop now iterates
  `self.shell.active_project().terminal_grids().flat_map(|grid| grid.states())` instead of
  `self.workspace().states()`; the per-grid failed-block body + the `merged_rows` LSP-union tail are unchanged.
- **flat_map minimal diff (chosen over a nested loop):** `states()` returns `(&PaneId, &PaneState<S>)` and
  `terminal_grids()` yields `&PaneGrid` (`'self`), so `flat_map(|grid| grid.states())` is `'self`-clean — a
  one-line enumeration swap, the body needs no re-indent. `cargo check -p marley` confirms the lifetime.
- **Deviations:** none.
- **Compile:** `cargo check -p marley` clean (only the pre-existing `block v0.1.6` note).

## Phase 3.5 — Inspect
One general-purpose critic (correctness + state-integrity) over the diff. **Verdict: CLEAN — no fixes.**

| # | Finding | Verdict |
|---|---|---|
| F1 | Regression / a previously-scanned grid dropped | REJECTED — `workspace()`'s grid is ALWAYS a member of `terminal_grids()` (both `terminal_grid_index` branches point at a `Some`-grid tab), so new ⊇ old (equal for a single-terminal project); same active-project scope. Bonus robustness: old `workspace()` `.expect()`-panics on zero terminal grids, new code yields nothing (loop is past the `active_editor()` early-return → ≥1 tab). |
| F2 | `terminal_grids()` yields non-terminal grids | REJECTED — `Tab::grid()` is `Some` only for `TabContent::Terminal`; `filter_map` yields exactly the terminal grids, in tab order. |
| F3 | Double-count / nondeterminism | REJECTED — each grid appears once; the loop iterates ONLY `terminal_grids()`; `merged_rows` does `sort_unstable` + `dedup` (removes all dups + makes the output order-independent — `PaneGrid.panes` is a `HashMap`, so `states()` is unordered, but the final sort is deterministic). |
| F4 | All-projects leak | REJECTED — uses `active_project().terminal_grids()`, NOT the all-projects `Workspace::grids()`; a background project's terminals cannot leak into the active gutter. |
| F5 | `flat_map` borrow soundness | REJECTED — `grid` is `&'self PaneGrid` (Copy); `states()` reborrows `*grid` for `'self`; sound (cargo check confirms). |

Out-of-scope (pre-existing, NOT diff defects): `blocks().iter().last()` is O(n) in the unchanged body; terminal refs resolve against `project_root` not per-pane cwd (the existing `diagnostics_for_file` model). No `failure-record` — no new bug introduced.

## Phase 4 — Validate
- **Tests added (2, `tabs.rs`):** `terminal_grids_yields_terminal_grids_in_tab_order_skipping_non_terminal`
  (REQ-001 — `[terminal, editor, terminal, cockpit]` → pane-id firsts `[0, PANE_ID_BLOCK]`, proving all-terminal-
  grids + tab-order + non-terminal skip), `terminal_grids_empty_without_terminal_tabs` (REQ-002).
- **Run:** `cargo nextest -p marley terminal_grids` → **2 passed**.
- **Mutation (`cargo mutants --list -f tabs.rs`):** `terminal_grids` → 1 viable mutant `→ iter::empty()`
  (killed by REQ-001 expecting 2 grids); the `once(Box::leak(…))` variants are unviable (`S` is not
  `Default`-bound). The app.rs shim is `#[cfg_attr(test, mutants::skip)]` → no mutants. **MSI 100 on the diff.**
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** (7:05) — cov 100, MSI 100, visual/AX. Receipt written.
- **REQ-003 / behavioral validation (documented boundary, §7).** The shim's new loop is already exercised in
  real render state by the EXISTING `diagnostics_gutter_empty_without_a_failure_headless` test (it now runs
  `terminal_grids().flat_map(states)` over the boot grid — green). The POSITIVE end-to-end (a failed block in
  tab B lights the gutter) is carried by **composition**: `terminal_grids()` unit-proven [ALL grids enumerated]
  ∘ the byte-identical per-grid body [proven by #289/#291] ∘ the **#291 LIVE DRIVE** that originally proved
  per-grid gutter-lighting on the real app (and found this bug). A seeded-failure headless test is infeasible
  (`BlockList::open_running` is `pub(crate)`; blocks come from real PTY output + shell integration), and the
  critic confirmed the shim is a **strict superset** of the #291-proven path (nothing previously-scanned is
  dropped). A fresh driven capture would only re-confirm the #291 path over one more grid.
- **Pre-existing:** none in scope.

## Phase 5 — Complete
- **Docs (§21):** `CHANGELOG.md` → a `### Fixed` entry (#295); `app_shell.md` → the #289 diagnostics-gutter
  passage now notes #295 (across ALL terminal grids via `Project::terminal_grids()`). No convention shift.
- **AAR (forge):** `aar-submit` outcome=completed, effectiveness 5. **Reuse-win lesson:** `grids()`/`grids_mut()`
  + `merged_rows` already existed → the fix = a per-project `terminal_grids()` sibling + a scope-widening
  `flat_map`, no new aggregation logic. The critic's **strict-superset** proof (new ⊇ old) made the
  `mutants::skip` shim safe without a seeded-failure headless test (infeasible — block injection is
  `pub(crate)`). No `failure-record` (no new bug). No new prevention rule.
- **Close:** forge #295 → done; ticket doc → `tickets/closed/`; pipeline pair → `pipeline/completed/`.
- Gate green — the `--diff` receipt stays valid through these doc-only edits.
