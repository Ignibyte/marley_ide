# M9 seq-6 — right-click → Split + nested pane rows — Notes

- **Forge ticket:** #155 `60e0f233-3ec5-4e2d-bbdb-6189da9f6dc7` · **AAR:** `d826f887-0acb-4b7c-be87-d3ca8b8632d9`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-155-split-rail.md

## Phase 1 — Plan
- **Request:** forge #155 (M9 run 6/8) — right-click a terminal → split (reuse M6 split_focused); nested pane
  rows in the rail.
- **Pre-flight:** split_focused (workspace.rs:394) + ⌘D already split; rail_rows (tabs.rs:377) → W/P/Tab rows;
  PaneGrid::pane_ids()/focused(); PaneId = pub u64. No right-click wired.
- **Decisions:** D1 only a split (>1 pane) tab nests; D2 right-click splits Horizontal/After (⌘D dir); D3 label
  "pane {k+1}", active = tab active AND pane_ids[k]==focused.
- **AAR id:** `d826f887-0acb-4b7c-be87-d3ca8b8632d9`.

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
PURE tabs.rs (nested rail rows) + a masked shim (right-click split + rail Pane arm). No render body-move — reuse
the M6 split + the seq-3 rail render.
- **tabs.rs (PURE, cov/MSI 100):** `RailLevel::Pane`; `RailRow.pane: Option<usize>`; `rail_rows` — after each
  Tab row, `if let Some(grid)=tab.grid()` and `grid.pane_ids().len()>1`, push a Pane row per pane: label
  `format!("pane {}", k+1)`, active = `tab_active && pane_ids[k]==grid.focused()`, project=i, tab=Some(j),
  pane=Some(k). All existing RailRow constructions gain `pane: None`.
- **app.rs (SHIM masked):** `fn split_focused_pane(&mut self)` (mutants::skip) — reuse the agent-split spawn
  (zdotdir/cols/rows hoist → `workspace_mut().split_focused(Horizontal, After, || spawn_session(...))`) +
  persist_grid. The terminal pane div gets `.on_mouse_down(MouseButton::Right, → split_focused_pane + notify)`.
  The rail render gains a `RailLevel::Pane` arm (indent deeper than Tab, e.g. pl(px(40)); on click →
  switch_project(p)+switch_tab(t) then `workspace_mut().focus(grid.pane_ids()[k])`).

### File manifest
- `crates/marley_app/src/tabs.rs` — PURE: RailLevel::Pane, RailRow.pane, rail_rows nesting + tests.
- `crates/marley_app/src/app.rs` — SHIM: split_focused_pane; the pane right-click; the rail Pane arm.

### Regression Test Plan
| Test (tabs.rs, S=()) | AC |
|---|---|
| rail_rows_nested_panes — a terminal tab split to 2 panes → after its Tab row, 2 Pane rows (pane 0/1, focused active, project/tab set); a single-pane tab → NO Pane row | REQ-001/002 |
| DRIVEN: right-click a terminal → 2 tiled panes + 2 nested "pane" rows under the tab | REQ-003 |

### Risks
- rail_rows now reaches into PaneGrid (pane_ids/focused) — generic S; the test builds a 2-pane grid via
  split_focused(|| Ok::<(),()>(())). Keep the >1 guard tested (single-pane → none) to kill the mutant.
- Right-click must not also trigger a left-click handler on the pane; MouseButton::Right is distinct.
- persist_grid after split (the grid changed) so the split survives a reload.

## Phase 3 — Implement
- **tabs.rs PURE:** RailLevel::Pane; RailRow.pane: Option<usize>; rail_rows emits, after each Tab row, a nested Pane row per pane when tab.grid().pane_ids().len()>1 (label "pane {k+1}", active=tab_active && pane_ids[k]==focused, pane=Some(k)); the 3 existing RailRow constructions get pane:None.
- **app.rs SHIM (masked):** split_focused_pane() (reuse the agent-split spawn hoist → workspace_mut().split_focused(Horizontal,After,spawn) + persist_grid); the terminal pane div → on_mouse_down(MouseButton::Right → focus(pane_id) + split_focused_pane); the rail render RailLevel::Pane arm (pl 40px, active-highlight, click → switch_project+switch_tab + focus pane_ids[k]).
- **Verify:** fmt; check --all-targets 0 err; clippy -D warnings OK.

## Phase 4 — Validate
- **Tests:** rail_rows_nested_panes (REQ-001/002: 2-pane split → 2 nested Pane rows after the Tab row, pane 0/1, exactly the focused pane active; single-pane → no Pane rows; a split in a NON-active tab → no active pane). Passes (rail 3/3).
- **Self-test:** added a rightclickat verb to drive.swift; split.png (right-click a terminal → 2 tiled panes [both live "Marley" prompts, right focused/cyan] + the rail shows terminal 1 → pane 1[muted]/pane 2[active] nested).
- **Gate:** first RED gate:5 (tabs.rs:420 ==→!= survived) → added the focused-pane + label asserts. Re-gate GREEN [diff] 15/15, cov/MSI 100. Critic clean (fixed the Pane filter guard). Capture: split.png (right-click → 2 tiled panes + terminal 1→pane 1/pane 2 nested, focused pane active).

## Inspect (Phase 3.5)
Method: 1 background critic (rail nesting + right-click split routing) + my review + the driven capture + the mutation gate.

- **Critic — verified clean on the 3 core areas** (rail_rows correctness, right-click split routing, rail Pane click borrows/safety). Two findings:
  - **[LOW → FIXED] orphaned pane rows when the "Search tabs" filter hides the parent tab.** The Tab arm skips filtered rows but the Pane arm didn't → nested "pane N" rows could dangle under a filtered-out tab. Added `if !self.session_filter.is_empty() { continue; }` to the Pane arm (their generic labels never match a title search anyway).
  - **[NIT → accepted] the right-click Split is on every pane div, not only terminals.** A terminal tab can hold a tiled Git pane; right-clicking it splits a terminal sibling. Harmless (split_focused works on any focused pane; consistent with the Left handler's uniform focus(pane_id)) — left as-is; the comment notes "terminal".
- **[gate:5 mutation — 1 survivor → FIXED] tabs.rs:420 `*pid == focused` → `!=` survived.** My "exactly 1 active" count stayed 1 with the WRONG pane active. Added `assert!(panes[focused_idx].active)` + `assert!(!panes[1-focused_idx].active)` (capturing the focused pane's index before the grid moves) + the "pane 1"/"pane 2" label asserts. Re-gated.

Lenses: rail_rows nesting + active-flag, the >1 guard, right-click focus-then-split routing, per-button dispatch, the rail Pane click borrows/OOB, the filter interaction. No blocking findings remain.

## Phase 5 — Complete
- CHANGELOG + app_shell seq-6 note; forge #155 → done. **M9 6/8.** Right-click → split; nested pane rows in the rail. Added a rightclickat verb to drive.swift. LESSON: for an "iff X" flag, assert the identity not the cardinality (the ==→!= mutant kept count=1); a new nested rail row type needs the same filter guard as its parent.
