# the sessions sidebar (persistent left) — Notes

- **Forge ticket:** #109 `4015210d-112e-451b-8a0b-ae2035922d1b` · **AAR:** `ad701083-ef81-4427-86b3-d571ac75b3a2`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-109-sessions-sidebar.md

## Phase 1 — Plan
- **Request:** forge #109 (M5 3/12) — a persistent left sessions sidebar.
- **Pre-flight:** left dock = file_tree (app.rs ~101); panes = workspace.pane_ids(); agents map (~113) marks
  agent panes; focused = workspace.focused(). Fleet #68 = a ⌘⇧E overlay. NEW sessions.rs holds the pure surface.
- **Decisions:** D1 new sessions.rs; D2 session_rows maps (icon/active/order); D3 Files stays reachable below.
- **AAR id:** `ad701083-ef81-4427-86b3-d571ac75b3a2`.

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
- **sessions.rs (NEW PURE):** Session{id:PaneId,title,subtitle,is_agent} + SessionRow{id,icon:&'static str,title,subtitle,active} + `session_rows(sessions,focused)`: map each → SessionRow{ id, icon: if is_agent {"✳"} else {"▸"}, title/subtitle cloned, active: id==focused }; order preserved.
- **lib.rs:** `mod sessions;` (alphabetical, near right_dock/status_bar).
- **app.rs SHIM:** build `let sessions: Vec<Session> = self.workspace.pane_ids().iter().enumerate().map(|(i,&id)| { let is_agent = self.agents.contains_key(&id); Session{ id, title: format!("{} {}", if is_agent {"agent"} else {"terminal"}, i+1), subtitle: "main".into(), is_agent } }).collect();` then `session_rows(&sessions, self.workspace.focused())`; render a "Sessions" header + the rows in the LEFT dock ABOVE the Files tree; a row on_mouse_down → `let _ = view.workspace.focus(id); cx.notify()`; active row → accent bg.
- **Mutation targets:** the is_agent→icon branch, active id==focused, order.
- **Test plan:** session_rows_maps (2 sessions agent+terminal → ✳/▸ + order); session_rows_active (focused → active true, other false); session_rows_empty ([]). cov/MSI 100. The sidebar render + click masked (live capture).
- **Risks:** Files must stay reachable (below the sessions) — no access gap before seq-7; the click focus uses the row id (Copy); pane_ids() order is stable (sorted by id).

## Phase 3 — Implement
- **Built:** sessions.rs (Session/SessionRow + session_rows, with inline tests); mod sessions; app.rs left dock now renders a "SESSIONS" list (from the workspace panes: is_agent via the agents map, a numbered title, "main" subtitle) above a "FILES" header + the file tree — a session row click focuses the pane, the active row gets a surface bg.
- **Verification:** fmt; check 0 err; clippy OK; 3 session_rows tests pass.
- **Note for P4:** the title/subtitle clone lines in session_rows need explicit test asserts (the shim only renders icon+title) to kill their MSI mutants — strengthen session_rows_maps in validate.

## Phase 3.5 — Inspect
- **Method:** self-review (a small pure mapper + a masked left-dock list reusing the focus + file-row patterns).
- **Lenses — no findings:** session_rows maps id/title/subtitle/active + the is_agent→icon; active = id==focused; order preserved (iter().map keeps it) — all tested (title/subtitle asserts to be strengthened P4); the sidebar click reuses the proven workspace.focus (a stale id is a no-op); Files STAYS reachable (below the FILES header) — no access gap before seq-7. No unwrap/panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** session_rows_maps_icon_and_order (icon/order + title/subtitle passthrough) + session_rows_active_marks_focused + session_rows_empty. `cargo nextest` → 3 pass.
- **Self-test:** LIVE static capture (sidebar109.png) — the left dock shows a "SESSIONS" header + "▸ terminal 1" (the active pane, highlighted brighter than the muted file rows) above a "FILES" header + the file tree. REQ-003 PASS. One row = boot has one pane (split adds more; synthetic ⌘D env-blocked).
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (the title/subtitle asserts killed the clone mutants).

## Phase 5 — Complete
- CHANGELOG ### Added; forge #109 → done. **M5 3/12.** NEW sessions.rs (session_rows, cov/MSI 100) + the persistent left SESSIONS list (click→focus, active-highlight) above FILES. Live-proven (sidebar109.png).
