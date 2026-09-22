# M9 seq-1 — pure Workspace/Project/Tab model — Notes

- **Forge ticket:** #150 `ee02dfec-81e3-4549-83d3-abba18cf2cc1` · **AAR:** `72c5fd9d-4940-458b-8c9d-e474a5ed0b9c`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-150-workspace-model.md

## Phase 1 — Plan
- **Request:** forge #150 (M9 run 1/8) — the foundation: pure Workspace → Project → Tab hierarchy + algebra.
- **Pre-flight:** existing workspace.rs `Workspace<S>{group:PaneGroup, panes, focused, next_id}` = the tiled
  grid → rename to `PaneGrid<S>`; new `tabs.rs` for the top-level hierarchy. git @ d28cbbc (clean, cold build).
- **Decisions:** D1 rename Workspace→PaneGrid + new Workspace top-level; D2 multi-project container; D3 mirror
  M6 R28-R30 invariants (never-empty, active-follows-close, last-item errors); D4 generic over S.
- **AAR id:** `72c5fd9d-4940-458b-8c9d-e474a5ed0b9c`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
### Approach
Pure model layer above the pane grid. The existing `workspace.rs::Workspace<S>` (layout tree + pane registry
+ focus) is renamed `PaneGrid<S>` — it IS a grid of panes; behavior byte-identical. A NEW pure module
`tabs.rs` holds the hierarchy `Workspace<S>` → `Project<S>` → `Tab<S>` → `TabContent<S>`, generic over the
session handle `S`, reusing `PaneGrid<S>` inside `TabContent::Terminal`. NO gpui, NO render, NOT wired into
app.rs's active render yet (seq-2). §14: typed error `TabError`, no panics, single-owner.

### Rename scope (precise — identifier `Workspace`, NOT `WorkspaceGrid`/prose)
- workspace.rs: the `Workspace<S>` def + impl + its `#[cfg(test)]` refs (11) → `PaneGrid<S>`.
- app.rs: line 87 import, line 101 field TYPE (`workspace: PaneGrid<TerminalSession>` — field NAME stays
  `workspace`, so every `self.workspace.*` call site is unchanged), line 495 `PaneGrid::new`.
- lib.rs:75 export `Workspace` → `PaneGrid`.
- tests/integration.rs: 2 type refs.
- NOT touched: settings.rs `WorkspaceGrid` (a different identifier), sessions.rs prose, the "WORKSPACE"
  sidebar string, the `[workspace]` config section.

### New types (tabs.rs)
```
pub struct Workspace<S> { pub name: String, projects: Vec<Project<S>>, active_project: usize }
pub struct Project<S>   { pub name: String, pub root: PathBuf, tabs: Vec<Tab<S>>, active_tab: usize }
pub struct Tab<S>       { pub title: String, pub content: TabContent<S> }
pub enum   TabContent<S>{ Terminal(PaneGrid<S>), Cockpit(RightSection) }   // CodeView added seq-5
pub enum   TabError     { LastProject, LastTab, IndexOutOfRange }
```
Pure helper (the mutation-critical core, shared by both levels):
`fn adjust_active(active, closed, new_len) -> usize` = closed<active ⇒ active-1; closed==active ⇒
active.saturating_sub(1) (predecessor; first→0=old successor); else active; then `.min(new_len-1)`.

Algebra: `Workspace::new(name, first_project)`; `add_project(p)` (active=last); `close_project(idx)` →
`Result<Project<S>,TabError>` (IndexOutOfRange if idx≥len; LastProject if len==1; else remove + adjust_active);
`switch_project(idx)` → `Result<(),TabError>`; `active_project()/_mut()`, `project_count()`. Same shape on
`Project` for tabs (add_tab/close_tab→LastTab/switch_tab/active_tab). `Tab::terminal()/_mut()` →
`Option<&PaneGrid<S>>`; `is_cockpit()`.

### File manifest
- `crates/marley_app/src/tabs.rs` — NEW: the types + algebra + adjust_active + tests.
- `crates/marley_app/src/workspace.rs` — MODIFY: rename `Workspace<S>` → `PaneGrid<S>` (+ its test refs).
- `crates/marley_app/src/lib.rs` — MODIFY: `mod tabs;` + export rename + export the new types (as needed).
- `crates/marley_app/src/app.rs` — MODIFY (mechanical): the 3 type refs.
- `crates/marley_app/tests/integration.rs` — MODIFY: 2 type refs.

### Regression Test Plan
| Test (tabs.rs `#[cfg(test)]`, S=()) | AC |
|---|---|
| add_tab_and_project_activate_new — add → active==last at both levels | REQ-001 |
| close_active_tab_moves_to_predecessor ([a,b,c] active c → close c → active b); same for projects | REQ-002 |
| close_first_active_moves_to_successor ([a,b] active a → close a → active is old b at 0) | REQ-003 |
| close_last_tab_errors (1 tab → LastTab, unchanged); close_last_project_errors (LastProject) | REQ-004 |
| switch_out_of_range_errors (idx≥len → IndexOutOfRange, active unchanged) | REQ-005 |
| terminal_tab_wraps_pane_grid (TabContent::Terminal(PaneGrid::new(())) → Tab::terminal() Some) | REQ-006 |
| adjust_active_cases (the 5 boundary cases: before/after/active-mid/active-first/active-last) | REQ-002/003 mutation |
| the FULL existing suite green after the rename (PaneGrid split/close/resize/focus unchanged) | REQ-007 |

### Risks
- The rename must be identifier-precise (not `WorkspaceGrid`). Use exact-string Edits, then `cargo check`.
- New pub types unused by app.rs until seq-2: cov 100 (tests exercise every method) keeps them non-dead; if
  clippy flags unused, the tests' usage covers it (do NOT add render).
- `adjust_active` boundary mutants (`<` vs `<=`, saturating_sub, `.min`) — the 5-case test kills them.

## Phase 3 — Implement
- **Rename:** Workspace<S> → PaneGrid<S> (workspace.rs def/impl/tests, app.rs 3 type refs [field name `workspace` kept], lib.rs export, integration.rs 2 refs). Behavior byte-identical. WorkspaceGrid/prose untouched.
- **NEW tabs.rs (PURE):** Workspace<S>/Project<S>/Tab<S>/TabContent<S>{Terminal(PaneGrid)|Cockpit(RightSection)}/TabError + the algebra (new/add/close/switch at both levels, accessors) + the private adjust_active(active,closed,new_len) helper. `active` field name (not active_tab) to avoid the field/method clash. lib.rs exports the new types (→ public API, no dead_code).
- **Verification:** cold cargo check 0 err; clippy clean.

## Inspect (Phase 3.5)
Method: 2 background critics (algebra correctness; rename-safety/reuse) + my own concrete verification of each
lens. Reviewed the full `tabs.rs` + the `git diff` of the rename.

- **[correctness] adjust_active — REJECTED (no finding).** Verified all 4 cases: closed<active→active-1 (safe,
  active≥1 there); closed==active→saturating_sub (predecessor, or 0=successor when first); closed>active→
  unchanged; then `.min(new_len-1)`. `new_len-1` cannot underflow — `close_*` returns `LastX` when `len==1`
  BEFORE `Vec::remove`, so post-remove `len≥1`. Traced [A,B,C]/[A,B] examples for active mid/first/before/after.
- **[correctness] guard order — REJECTED (no finding).** `IndexOutOfRange` (idx≥len) → `LastTab/LastProject`
  (len==1) → `remove`. Out-of-range-on-len-1 yields IndexOutOfRange (a distinct, correct error); the last
  element is never removed.
- **[correctness] active-index validity — REJECTED (no finding).** `new`(0), `add_*`(len-1), `close_*`
  (adjust_active clamps to new_len-1), `switch_*`(idx only if idx<len) all keep `active ∈ 0..len`, so the
  `active_tab()/active_project()` indexes never panic. No unwrap/expect anywhere.
- **[rename] precision — REJECTED (no finding).** grep confirms the only remaining `Workspace` is the NEW
  top-level type (tabs.rs + its lib.rs export); `WorkspaceGrid` config, the "WORKSPACE" string, `[workspace]`
  TOML, and prose are untouched. `git diff app.rs` is type-only (field name `workspace` intact) → behavior
  byte-identical. No accidental `PaneGrid` in prose/config.
- **[reuse] adjust_active vs M6 close_pane — REJECTED.** Different domains (usize index arithmetic vs a
  PaneId tree with a depth-first order); sharing would couple unrelated layers. Duplication is justified + both
  are independently tested.
- **[dead-code] REJECTED.** The new pub types are re-exported via `lib.rs pub use tabs::{...}` → real API for
  seq-2; cold `cargo check` + clippy -D warnings both clean, no dead_code.

Lenses covered: correctness/edge-cases/panics, rename-safety, reuse, dead-code. **No confirmed findings.**

### Critic results (background agents)
- **Rename-safety/reuse critic — NO FINDINGS.** Confirmed: grep-clean rename (only the new type + its export
  remain; WorkspaceGrid/[workspace]/"WORKSPACE"=group.name.to_uppercase()/prose untouched); app.rs diff
  type-only (field `workspace` intact, ~90 call sites unchanged); the new Workspace::new(name, project)
  signature differs from the old ::new(session) so any missed site would be a type error (all green);
  adjust_active-vs-close duplication justified (Vec-index shift vs stable-PaneId tree — different domains);
  no dead_code (lib.rs re-exports all 5 types).
  - **[low, informational, deferred]** File/type naming tension: workspace.rs now holds `PaneGrid`. No action
    this seq (renaming the file would add churn + risk the behavior-identity constraint). Consider
    `workspace.rs` → `pane_grid.rs` in a later M9 seq. (Logged; not blocking.)
- Correctness critic + my own concrete verification: no confirmed findings (see the ledger above).

## Phase 4 — Validate
- **Tests (tabs.rs, 19 total for the module surface):** add_activates_the_new_item (REQ-001);
  close_active_moves_to_predecessor (REQ-002); close_first_active_moves_to_successor (REQ-003);
  close_last_is_refused (REQ-004); out_of_range_is_refused (REQ-005); terminal_tab_wraps_a_pane_grid
  (REQ-006); accessors_and_mut_paths; adjust_active_boundaries (6 cases — kills </== / saturating_sub / .min
  mutants). All pass. The existing 37 pane-grid tests pass after the rename (REQ-007).
- **UI:** N/A — pure model, no render/gpui surface (seq-2 renders it).
- **Gate:** first run RED (2): mutation `active_project_index -> 1` survived (only a ==1 assertion) + rustdoc gate:14 (pub docs linked the private `adjust_active`). FIXED: added a ==0 assertion; de-linked `adjust_active` (3 docs). Re-gate GREEN [diff] 15/15, cov/MSI 100. Both critics NO FINDINGS (their low notes: file-rename defer, clamp-mutant already covered by adjust_active(2,5,2)).

## Phase 5 — Complete
- CHANGELOG + app_shell.md M9 section; forge #150 → done. **M9 1/8.** Pure Workspace/Project/Tab model + algebra (cov/MSI 100); Workspace<S>→PaneGrid<S> rename (behavior-identical). 2 prevention rules recorded (accessor-const mutant; private intra-doc-link). NEXT: seq-2 #151 full-screen render.
