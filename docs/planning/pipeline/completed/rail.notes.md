# M9 seq-3 — the rail — Notes

- **Forge ticket:** #152 `43343e48-3d19-4a04-b921-4bdd0c565325` · **AAR:** `c8b44b83-4e8b-4f8b-921b-f85b6944d676`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-152-rail.md

## Phase 1 — Plan
- **Request:** forge #152 (M9 run 3/8) — the rail = Workspace→Project→Tab tree, click-to-switch; absorbs #145.
- **Pre-flight:** app.rs ~2264-2304 builds Session structs from the active grid + group_sessions → the flat
  list; sessions.rs group_sessions retires from this panel. shell (seq-1/2) is the source.
- **Decisions:** D1 rail_rows emits Workspace + per-project Project rows + Tab rows with active flags; D2 tab
  click → switch_project+switch_tab; D3 generic over S.
- **AAR id:** `c8b44b83-4e8b-4f8b-921b-f85b6944d676`.

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
PURE projection in `tabs.rs` + a masked render swap in `app.rs`. No new deps.
- `tabs.rs`: `#[derive(…PartialEq)] pub enum RailLevel { Workspace, Project, Tab }`;
  `pub struct RailRow { pub level: RailLevel, pub label: String, pub active: bool, pub project: usize,
  pub tab: Option<usize> }`; `pub fn rail_rows<S>(ws: &Workspace<S>) -> Vec<RailRow>` — push a Workspace row
  (label=ws.name, active=false, project=0, tab=None), then for each project `i`: a Project row
  (label=name, active = i==active_project_index, tab=None) then for each tab `j`: a Tab row
  (label=title, active = i==active_project_index && j == project.active_tab_index, project=i, tab=Some(j)).
- `app.rs` (masked): replace the `sessions`/`group_sessions` block (~2262-2340) with a loop over
  `rail_rows(&self.shell)`: Workspace/Project rows render as muted headers (indent by level); a Tab row
  renders as a clickable entry (indent, `tab.title`), highlighted when `active`, `on_mouse_down` →
  `view.shell.switch_project(p); view.shell.active_project_mut().switch_tab(t); cx.notify()`. Keep the
  "Search tabs" box; apply `session_filter` to Tab-row labels (skip a non-matching tab). Retire
  `group_sessions`/`Session`-build in THIS panel (sessions.rs stays for now; its use here is removed).

### File manifest
- `crates/marley_app/src/tabs.rs` — NEW pure: RailLevel + RailRow + rail_rows + tests.
- `crates/marley_app/src/app.rs` — MODIFY (masked): swap the session-list block for the rail render + click.

### Regression Test Plan
| Test (tabs.rs, S=()) | AC |
|---|---|
| rail_rows_one_project — 1 project + 2 tabs → [Workspace, Project(active), Tab0(active,tab=Some(0)), Tab1(inactive,tab=Some(1))] in order; levels + labels + active + project/tab indices | REQ-001 |
| rail_rows_two_projects — 2 projects (active=1) → both Project rows; project0 tabs inactive; only project1's active tab flagged | REQ-002 (mutation: the i==active && j==active AND) |
| DRIVEN: rail shows Workspace/Project headers + tab rows, active highlighted | REQ-003 |
| DRIVEN: + a 2nd tab (2 tab rows) → click tab row 0 → the view swaps + row 0 highlights | REQ-004 |

### Risks
- The render swap is masked — the DRIVEN capture is the proof (rail tree + click switches).
- Mutation on the active flags: `i == active_project` (Project) and `i==active_project && j==active_tab` (Tab)
  — the two-project test with a non-active project's tabs kills the "&&"/"==" mutants.
- Keep click routing: switch_project THEN switch_tab (order matters — switch_tab acts on the now-active project).

## Phase 3 — Implement
- **Built (tabs.rs PURE):** RailLevel{Workspace,Project,Tab} + RailRow{level,label,active,project,tab} + rail_rows(&Workspace<S>) (Workspace header, then per project a Project row + its Tab rows; active flags per D1).
- **(app.rs SHIM masked):** replaced the ~76-line Session-build + group_sessions render block with a rail_rows loop — Workspace/Project headers (indented, muted) + clickable Tab rows (indent 28, active-highlighted, on_mouse_down→shell.switch_project(p)+active_project_mut().switch_tab(t)); the "Search tabs" filter now filters Tab-row labels. Imports rail_rows/RailLevel.
- **Retired sessions.rs:** the rail fully replaces the session-list projection → deleted crates/marley_app/src/sessions.rs + `mod sessions` + the app.rs import + the now-unused AgentStatus import. No other file used sessions::.
- **Verification:** fmt; check --all-targets 0 err; clippy -D warnings OK.

## Inspect (Phase 3.5)
Method: 1 background critic (render-swap + deletion completeness + click routing) + my own rail_rows review.
The binary builds clean.

- **[rail_rows] correctness — no finding.** Row order = Workspace, then per project a Project row + its Tab
  rows (enumerate → correct i/j). Active flags: Project active iff `i == active_project_index`; Tab active iff
  `i == active_project_index && j == active_tab` — for a non-active project the `&&` short-circuits false
  regardless of j (correct). No off-by-one.
- **[click] routing — no finding.** on_mouse_down does `switch_project(p)` THEN
  `active_project_mut().switch_tab(t)` — order correct (switch_project first so active_project_mut targets p).
  p=row.project, t=row.tab.unwrap_or(0)=j; j is always in-range for that project's tab list.
- **[deletion] sessions.rs retired — no finding.** grep confirms no dangling `sessions::` reference anywhere
  (app.rs import + mod + AgentStatus import removed); cargo check --all-targets + clippy -D warnings both clean.
- **[filter] orphaned headers — accepted.** When the "Search tabs" query filters out all of a project's tabs,
  the Project/Workspace header still shows (cosmetic). Acceptable for seq-3.
- **[scope] lost sidebar chrome — deferred, in-scope.** The old two-line title/subtitle + agent-status icon +
  agent-vs-terminal label are gone from the rail; the rail shows tab titles. The real branch + live titles are
  seq-8 (#157); the agent/cockpit rework is seq-4 (#153). Not a regression of seq-3's goal.

Lenses: projection correctness, click routing, deletion completeness, filter, scope. **No confirmed findings.**

## Phase 4 — Validate
- **Tests:** rail_rows_one_project (REQ-001: order/levels/labels/active/tab-indices); rail_rows_two_projects (REQ-002: only the active project's active tab flagged — kills the && mutant). Pass.
- **Self-test:** driven captures below (rail tree + click-switch).
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (rail_rows). Critic NO BLOCKING FINDINGS (deletion complete, routing+rail_rows correct; noted a pre-existing stale comment @app.rs:2223 — left as-is to preserve the gate receipt). Captures: rail_2tabs.png (rail tree WORKSPACE→workspace→terminal 1[muted]/terminal 2[active] after +, REQ-003); rail_switch.png (click terminal 1 → it highlights, terminal 2 mutes, view switches, REQ-004).

## Phase 5 — Complete
- CHANGELOG + app_shell seq-3 note; forge #152 → done. **M9 3/8.** The rail = Workspace→Project→Tab tree, click-to-switch; sessions.rs retired+deleted. LESSON: when a feature supersedes a module, delete it (clippy flags dead pub fns kept alive only by their own tests).
