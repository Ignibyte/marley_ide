# the sessions sidebar for the pane world (M6) — Notes

- **Forge ticket:** #129 `7102ac4f-ca8e-44e5-a741-c26ef6a0a680` · **AAR:** `1363b465-4f04-4d45-9d56-9d4a174c0be8`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-129-sidebar-sessions.md

## Phase 1 — Plan
- **Request:** forge #129 (M6 run 9/10) — sidebar phantom "terminal N" for session-less panes → terminal-only.
- **Pre-flight:** sidebar build app.rs 2039-2064 (Session per pane_id); panes_of_kind generalizes first_pane_of_kind.
- **Decisions:** D1 panes_of_kind(Terminal); D2 filter then number, focus the terminal id.
- **AAR id:** `1363b465-4f04-4d45-9d56-9d4a174c0be8`.

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
- **workspace.rs (PURE):** panes_of_kind(kind)->Vec<PaneId> = pane_ids filtered to kind()==kind.
- **app.rs (SHIM):** the sidebar sessions list builds from self.workspace.panes_of_kind(PaneKind::Terminal) (was pane_ids) — filter FIRST then enumerate for "terminal 1/2"; is_agent/running/title from the terminal id.
- **Test plan:** panes_of_kind_filters ([T,F,T,G]→2 terminal ids; Git→1; CodeView→[] none).
- **Risks:** the sidebar map uses (i, id) + is_agent(id) — swap pane_ids()→panes_of_kind(Terminal); numbering over the filtered list.

## Phase 3 — Implement
- **Built (workspace.rs PURE):** panes_of_kind(kind)->Vec<PaneId> (pane_ids filtered to kind). **(app.rs SHIM):** the sidebar sessions list builds from panes_of_kind(PaneKind::Terminal) (was pane_ids) — non-terminal panes contribute no row; numbering over the terminals.
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (a tiny filter + a one-line sidebar swap).
- **Lenses — no findings:** panes_of_kind = pane_ids.filter(kind()==kind) — the terminal-only filter; the sidebar builds Sessions from panes_of_kind(Terminal) so files/code/git panes contribute no phantom row; is_agent/running/focus id read from the terminal id (unchanged); numbering enumerates the filtered terminals. No unwrap/panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** panes_of_kind_filters_to_the_kind ([T,F,T,G]→2 terminal ids; Git→1; CodeView→empty). Pass.
- **Self-test:** LIVE capture (sidebar129.png) — grid="H:t,f,g" (terminal + FileTree + Git panes tiled); the sidebar WORKSPACE list shows ONLY "terminal 1" (no phantom "terminal 2/3" for the files/git panes). REQ-003 PASS.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG; forge #129 → done. **M6 9/10.** panes_of_kind (pure); the sidebar lists only terminal panes (no phantom "terminal N"). cov/MSI 100.
