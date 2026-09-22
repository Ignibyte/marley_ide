# workspace groups in the sidebar — Notes

- **Forge ticket:** #110 `ee9a310e-a2bf-437e-aac5-625c2d8f955c` · **AAR:** `87fd8c72-820d-492b-85b5-e9ceee19174d`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-110-workspace-groups.md

## Phase 1 — Plan
- **Request:** forge #110 (M5 4/12) — group the sessions sidebar by workspace.
- **Pre-flight:** extends #109 sessions.rs (Session/SessionRow/session_rows). Add Session.workspace +
  group_sessions (reuse session_rows per group). One project → one group for now (multi = M3).
- **Decisions:** D1 first-seen group order + reuse session_rows.
- **AAR id:** `87fd8c72-820d-492b-85b5-e9ceee19174d`.

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
- **sessions.rs (PURE):** Session gains `pub workspace: String`; `pub struct SessionGroup{ pub name: String, pub rows: Vec<SessionRow> }`; `pub fn group_sessions(sessions, focused) -> Vec<SessionGroup>`: for s in sessions { if let Some(g)=groups.iter_mut().find(|g| g.name==s.workspace) { g.rows.extend(session_rows(std::slice::from_ref(s),focused)) } else { groups.push(SessionGroup{ name: s.workspace.clone(), rows: session_rows(from_ref(s),focused) }) } }. Reuses session_rows (stays live).
- **app.rs SHIM:** Session build adds `workspace: self.project_root.file_name().and_then(to_str).unwrap_or("workspace").to_string()`; the left dock replaces the flat SESSIONS header+loop with `for group in group_sessions(&sessions, self.workspace.focused()) { files=files.child(header(group.name)); for row in &group.rows { <the #109 row div, click-focus, active-bg> } }`.
- **Mutation targets:** the group find (name==workspace), first-seen push-vs-extend, within-group order, the reused active flag.
- **Test plan:** group_sessions_first_seen (workspaces [A,B,A] → [A(s0,s2), B(s1)]); group_sessions_one (1 workspace → 1 group); group_sessions_empty ([]); group_sessions_active (focused row active within its group). Update the #109 sess() helper for the workspace field. cov/MSI 100.
- **Risks:** session_rows must stay reused (not duplicated); the #109 sess() helper + the app Session build need the new field; one project → one group for now.

## Phase 3 — Implement
- **Built:** Session.workspace + SessionGroup + group_sessions (reuses session_rows per group; first-seen order) in sessions.rs; the #109 sess() helper → sess_in(id,agent,ws); app.rs computes ws_name (project basename) + the Session build carries it + the left dock now renders group_sessions as a header-per-group (name uppercased) + its rows (replacing the flat SESSIONS list).
- **Verification:** fmt; check 0 err; clippy OK (session_rows stays live via group_sessions).

## Phase 3.5 — Inspect
- **Method:** self-review (a small grouping fn reusing #109 session_rows + a masked grouped render).
- **Lenses — no findings:** group_sessions walks sessions in order, find-or-push by name (first-seen group order), extends the matching group with session_rows(from_ref(s)) → within-group order preserved + the row mapping (icon/active) reused unchanged; empty → []; the focused row stays active within its group. No unwrap/panic. The render groups correctly; Files still reachable. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** group_sessions_first_seen_order + group_sessions_one_and_empty + group_sessions_marks_active (+ the #109 session_rows tests still pass). `cargo nextest` → pass.
- **Self-test:** LIVE static capture (groups110.png) — the left dock heads the session list with a "WORKSPACE" group header (the project basename uppercased; "WORKSPACE" = the open-launched cwd "/" has no basename) + "▸ terminal 1" nested under it, above FILES + the tree. REQ-003 PASS — sessions now group under a named workspace header.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (first try).

## Phase 5 — Complete
- CHANGELOG ### Added; forge #110 → done. **M5 4/12.** Session.workspace + SessionGroup + group_sessions (reuses session_rows; cov/MSI 100) + grouped sidebar headers. Live-proven (groups110.png).
