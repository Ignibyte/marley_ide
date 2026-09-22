# deterministic file/pane placement (M8) — Notes

- **Forge ticket:** #139 `9736b3a8-268f-4b84-9f20-e9e1c99b306e` · **AAR:** `034dbc10-3be5-4675-a3cc-afb38aa84fa0`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-139-pane-placement.md

## Phase 1 — Plan
- **Request:** forge #139 (M8 run 3/8) — non-terminal panes open rightmost, not split-of-focus.
- **Pre-flight:** insert_focused_split (workspace.rs:343) splits self.focused; open_pane (374); group.panes()
  is DFS-leaf order → rightmost = last. open_kind_pane (#128) / open_code_pane (#124) are the non-terminal openers.
- **Decisions:** D1 rightmost_pane = panes().last() (no expect); D2 share insert_split_at.
- **AAR id:** `034dbc10-3be5-4675-a3cc-afb38aa84fa0`.

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
- **workspace.rs (PURE):** extract insert_split_at(target,axis,dir,state) from insert_focused_split (which delegates with self.focused); rightmost_pane()=*group.panes().last().unwrap_or(&focused); open_pane_rightmost(axis,dir,content)=insert_split_at(rightmost_pane(),...).
- **app.rs (SHIM):** open_kind_pane (OpenNew) + open_code_pane (OpenNew) → open_pane_rightmost(Horizontal,After,content) (was open_pane).
- **Test plan:** rightmost_pane_is_dfs_last ([T]→T,[T,F]→F,[T,F,C]→C); open_pane_rightmost_lands_last (focus non-rightmost, open → panes().last()==new, order [T,F,C]).
- **Risks:** insert_split_at refactor keeps focused-split behavior; PaneState{content} construction; the openers open Horizontal/After.

## Phase 3 — Implement
- **Built (workspace.rs PURE):** insert_split_at(target,...) extracted; insert_focused_split delegates (self.focused); rightmost_pane()=*group.panes().last().unwrap_or(&focused); open_pane_rightmost(axis,dir,content). **(app.rs SHIM):** open_code_pane + open_kind_pane OpenNew branches → open_pane_rightmost (were open_pane).
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (an extraction + a placement fn + opener routing).
- **Lenses — no findings:** insert_split_at is the exact former insert_focused_split body with target parameterized; insert_focused_split delegates with self.focused (focused-split / ⌘D / terminals unchanged). rightmost_pane = panes().last() (DFS-last leaf) with an unwrap_or(&focused) fallback (no expect/panic; R30 keeps non-empty). open_pane_rightmost splits the rightmost leaf After → the new pane lands last. The non-terminal openers (code/kind) route OpenNew through it; the FocusExisting reuse path is unchanged. No unwrap/panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** rightmost_open_lands_last_regardless_of_focus ([T]→rightmost T; open FileTree→[T,F]; focus T then open Git→[T,F,G] not [T,G,F]). Pass.
- **Self-test:** LIVE capture (below) — grid H:t,c,g (terminal + code + git, rightmost order); the focus-independence is proven by the unit test (driving a click with a set focus is env-blocked).
- **Gate:** GREEN [diff] 15/15, cov/MSI 100. Capture (placement139.png): terminal | code | Source Control in order (non-terminal right of the terminal). REQ-003 PASS; focus-independence via the unit test.

## Phase 5 — Complete
- CHANGELOG; forge #139 → done. **M8 3/8 — chad's 3 layout items all shipped.** Non-terminal panes open RIGHTMOST (rightmost_pane + open_pane_rightmost, pure). cov/MSI 100.
