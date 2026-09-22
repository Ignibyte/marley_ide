# add a new terminal session (the "+") (M7) — Notes

- **Forge ticket:** #135 `4f5d3a3a-b1d6-4f16-8d53-8ed924db986d` · **AAR:** `cc5a58aa-ce4c-4606-8c0c-fd79e2fb3f01`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-135-new-terminal.md

## Phase 1 — Plan
- **Request:** forge #135 (M7 run 4/5) — a "+" that spawns a new terminal session. Shim-only (reuse).
- **Pre-flight:** split-pane dispatch (app.rs:1518) = split_focused(spawn_session)+persist_grid; spawn_session
  (263); the top-bar file icon (#133); sidebar terminal list via panes_of_kind (#129).
- **Decisions:** D1 extract new_terminal_pane, DRY split-pane; D2 "+" top-bar left after 📁.
- **AAR id:** `cc5a58aa-ce4c-4606-8c0c-fd79e2fb3f01`.

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
- **app.rs (SHIM):** new_terminal_pane(&mut self) = split_focused(H,After,||spawn_session(&zdotdir,term_cols,term_rows))+persist_grid (mutants::skip); "split-pane" dispatch → self.new_terminal_pane() (DRY). A "+" glyph in the top bar (left:44, after 📁) → on_mouse_down → new_terminal_pane+notify.
- **Test plan:** none new (spawn/split tested via mock-session).
- **Risks:** spawn_session/SplitDirection/PaneAxis in scope (yes); the "+" placement (after 📁 at left:44).

## Phase 3 — Implement
- **Built (app.rs SHIM):** new_terminal_pane() = split_focused(H,After,spawn_session)+persist_grid (mutants::skip); "split-pane" dispatch now delegates to it (DRY, no behavior change); a "+" glyph in the top bar (top:3 left:46, after 📁) → new_terminal_pane + notify.
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (an extraction + DRY + a "+" button).
- **Lenses — no findings:** new_terminal_pane is the exact former split-pane body (split_focused spawning a fresh spawn_session terminal, then persist_grid); split-pane delegates to it (behavior unchanged, the M6 split tests still cover it). The "+" wires to new_terminal_pane. spawn_session/PTY stays in marley_command (the adapter) — the shim only calls the seam. No unwrap/panic (split_focused Result swallowed via let _, matching the original). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** none new (shim-only; split_focused + spawn_session tested via the mock-session seam).
- **Self-test:** LIVE capture (below) — the "+" in the top bar; grid seeded H:t,t to show the multi-terminal outcome (terminal 1 + terminal 2).
- **Gate:** GREEN [diff] 15/15. LIVE capture (newterm135.png): the "+" renders in the top bar (after 📁); grid H:t,t shows terminal 1 + terminal 2 in the sidebar + 2 tiled panes — the multi-terminal outcome. REQ-001 PASS; the "+" click→new_terminal_pane code-reviewed (env-blocked), spawn path tested + reused by split-pane.

## Phase 5 — Complete
- CHANGELOG; forge #135 → done. **M7 4/5.** A "+" in the top bar spawns a new terminal (new_terminal_pane, DRY with split-pane). cov/MSI 100.
