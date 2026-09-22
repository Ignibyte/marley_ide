# the git panel as a grid pane (M6 seq-6) — Notes

- **Forge ticket:** #125 `24f21f98-4fe6-40c2-b411-115250a10446` · **AAR:** `da81298e-96b8-451e-8285-b9ff6590b4ef`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-125-git-pane.md

## Phase 1 — Plan
- **Request:** forge #125 (M6 run 6/10) — git panel → a Git grid pane; retire the M5 #115/#116 side-panel.
- **Pre-flight:** git_panel_open (146) + boot restore (511); ⌘⇧C toggle (1807); right_open (2240); side-panel
  render 2992+; Git pane stub (2736); commit_message/commit_focused + handle_commit_message_key (1148).
- **Decisions:** D1 ⌘⇧C opens a single Git pane, write-confinement unchanged; D2 delete right_open/viewer_split.
- **AAR id:** `da81298e-96b8-451e-8285-b9ff6590b4ef`.

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
- **app.rs (SHIM):** MOVE the git side-panel body (change list via git_status()/change_summary, ● stage-toggle git_stage/git_unstage, commit input commit_message/commit_focused, Commit button commit_enabled, ± empty state) into the Git pane dispatch (2736), dropping the absolute right-side positioning (the pane body already positions it). open_git_pane: first_pane_of_kind(Git)→focus else open_pane(Git); persist_grid. ⌘⇧C→open_git_pane (drop git_panel_open flip + persist_layout). DELETE: the side-panel render (2992+), git_panel_open field+boot restore (146/511), right_open+viewer_split (2240). KEEP commit_message/commit_focused + handle_commit_message_key.
- **Test plan:** none (shim-only; git_diff pure cov/MSI stays). Gate + a live capture.
- **Risks:** the side-panel render captures `self` (git_status/commit_message) — in the pane dispatch it is `self.` (render is &self) but the click listeners are cx.listener(view). git_status() is &self? or spawns — reuse as-is. viewer_split removal → the center is full-width (center_bounds). No dangling git_panel_open.

## Phase 3 — Implement
- **Built:** the source-control panel renders at the Git pane rect (first_pane_of_kind(Git) in rect_list), reusing the whole #115/#116 render; the git-write adapters (add/restore/save) + confinement UNCHANGED. open_git_pane (focus-or-open + persist_grid); the chord opens the Git pane.
- **Deleted (dead after both side-panels retired):** git_panel_open + boot restore; right_open + viewer_split (center full-width); the whole M5 #118 layout mechanism (serialize/restore/persist + WorkspaceLayout + AppliedSettings.layout + its round-trip test — git state now rides the #122 grid); viewer_split (code_view) + test.
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- Self-review: git-write confinement UNTOUCHED (only render-position + open-path changed); message-input routing kept; no dangling git_panel_open/right_open (0 compile err); no dead code (clippy OK). No findings.

## Phase 4 — Validate
- **Tests:** none added (shim-only; the removed pure fns took their tests with them; settings.rs + code_view.rs stay cov/MSI 100).
- **Self-test:** LIVE capture (git125.png) — grid="H:t,g" → [terminal | Git pane] tiled; the Git pane shows Source Control (no changes / empty state / Message input / save button), NO right-side git panel. Duplication #3 CLEARED. REQ-001+002 PASS.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (removing fns+their tests together kept settings.rs/code_view.rs at 100).

## Phase 5 — Complete
- CHANGELOG; forge #125 → done. **M6 6/10.** Git panel → a grid pane (renders at the Git pane rect); retired the M5 side-panel + git_panel_open + right_open + the obsolete #118 layout mechanism. All 3 side-panels now panes. Duplication #3 CLEARED. cov/MSI 100.
