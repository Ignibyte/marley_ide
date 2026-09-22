# PaneContent — typed pane content (M6 FOUNDATION) — Notes

- **Forge ticket:** #120 `ac00c9c1-3da7-4053-92a6-911fbdc340bb` · **AAR:** `4719c07e-3767-42f0-b74b-46dc231a0c8d`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-120-pane-content.md

## Phase 1 — Plan
- **Request:** forge #120 (M6 seq-1, FOUNDATION) — refactor to typed pane content so panes can be session-less.
- **Scope (measured):** app.rs field accesses to migrate — 36 `.session`, 26 `.caret`, 20 `.buffer`,
  12 `.viewport`, 8 `.pty_size`, 7 `.selection`, 6 `.history`, 2 `.scroll_remainder` (~117) across ~42
  state-getter sites. `CodeViewState` already exists (code_view.rs:146) — reuse it in the CodeView variant.
- **Decisions:** D1 PaneContent enum, algebra unchanged; D2 new=Terminal, open_pane inserts session-less;
  D3 shim narrows once per getter (`if let Some(term)=state.terminal_mut()`), no panic path.
- **Warp reference (studied, 3 screenshots):** panes have centered title bars + a left/top cyan focus edge;
  the file explorer is a toolbar + disclosure tree; the git pane has a repo:branch header + "Uncommitted
  changes" + Commit + a "± No open changes" empty state; a bottom status bar. (These land seq-2..8; seq-1 is
  the invisible model.)
- **AAR id:** `4719c07e-3767-42f0-b74b-46dc231a0c8d`.

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
- **workspace.rs (PURE):**
  - `pub struct TerminalPane<S> { session, buffer, caret, history, pty_size, viewport, scroll_remainder, selection }` (the 8 fields moved verbatim).
  - `pub enum PaneContent<S> { Terminal(TerminalPane<S>), FileTree, CodeView(CodeViewState), Git }` (import CodeViewState from crate::code_view).
  - `pub struct PaneState<S> { pub content: PaneContent<S> }`; `pub fn new(session: S) -> Self` = Terminal(TerminalPane{ session, buffer: Buffer::new()... same inits as before }).
  - `pub fn kind(&self) -> PaneKind` (match content → the 4 PaneKind); `pub fn terminal(&self) -> Option<&TerminalPane<S>>` + `terminal_mut` (Some only for Terminal).
  - `pub fn open_pane(&mut self, content: PaneContent<S>) -> Option<PaneId>` on Workspace<S>: allocate a PaneId, insert PaneState{content}, split the focused leaf (reuse the split-insertion helper split_focused uses, minus the session spawn). Returns the new id.
  - The PaneGroup algebra (split/close/focus/neighbor) UNCHANGED. Any code reading state.kind stays working via kind().
- **app.rs (SHIM, masked):** at each of ~42 state-getter sites, narrow `if let Some(term)=state.terminal_mut(){…}` (or `.terminal()`), then the ~117 inner `state.<field>` → `term.<field>`. The pane render/input loop guards terminal-only work behind the narrow. `.kind` reads → `.kind()`. No open_pane calls yet.
- **Mutation targets:** kind() per-variant, terminal()/_mut() Some-only-Terminal, open_pane insertion+content, content preserved across split/close.
- **Test plan:** pane_new_is_terminal, open_pane_filetree_sessionless, open_pane_codeview_preserves_state, split_close_preserve_content, kind_derives_per_variant + the existing split/close/focus/neighbor suite unchanged.
- **Risks:** the ~117-access migration must not break the terminal — compiler-driven + a LIVE terminal capture in P4. open_pane must reuse the EXACT split insertion (ratio/axis) so geometry matches split_focused.

## Phase 3 — Implement
- **Built (workspace.rs PURE):** TerminalPane<S> (8 terminal fields) + PaneContent<S> enum + PaneState<S>{content} + kind()/terminal()/terminal_mut()/new(); open_pane(axis,dir,content)->PaneId (session-less split, mirrors split_focused minus the spawn); convenience getters focused_terminal()/focused_terminal_mut()/terminal(id)/terminal_mut(id) to collapse the shim churn. The split/close/focus/neighbor algebra UNCHANGED.
- **Migrated (app.rs SHIM):** ~120 terminal-field accesses across ~42 getter sites — focused_state_mut()→focused_terminal_mut() (17), state(focused())→focused_terminal(), state(id)/state_mut(id)→terminal(id)/terminal_mut(id), the 2 states_mut() pump/resize loops narrowed per-item (`let Some(term)=state.terminal_mut() else {continue}`), the .kind field read→kind(), and 3 helper fns (content_rows/content_row_texts/pane_grid_pos) retyped &PaneState→&TerminalPane. Integration tests migrated to the accessors.
- **DEVIATIONS:** (1) open_pane returns PaneId not Option (no spawn can fail — mirrors split_focused infallibility). (2) added 4 convenience getters (not in the plan) to minimize the masked-shim churn + risk — each a trivial delegator, tested in P4. (3) reworked the M5 pane_kind_survives test to open_pane(Git) since kind is now derived (can no longer set .kind on a terminal pane).
- **Verification:** fmt; check 0 err (lib + all-targets); clippy OK; 41 workspace/pane tests pass (algebra intact).

## Inspect (Phase 3.5)
3 critics over the ~265-line refactor diff (correctness · algebra/integrity · simplification). **No real bugs** — the migration is structurally clean.

- **[MED] open_pane/split_focused duplicated the insert idiom** (simplification critic) — REAL, FIXED: extracted a private `insert_focused_split(axis,dir,state)->PaneId` both call (id-bump + group.split + debug_assert + insert + focus). Removes the risk of the two copies (+ the R30 invariant comment) drifting.
- **[LOW] stale PaneKind doc** — REAL, FIXED: it claimed "every pane is Terminal today / kind rides PaneState / gain content M5 seq-7/8/9"; now documents kind() is derived from PaneContent.
- **[LOW] R30 invariant test omits open_pane** (algebra critic) — VALID: registry_matches_tree covers split/close only; open_pane is covered implicitly (pane_kind_survives splits an opened pane). DEFERRED to P4 — add an explicit open_pane step to the invariant test.
- **Confirmed CLEAN (correctness critic):** NO panic risk — zero `.terminal[_mut]().unwrap()/.expect()` in app.rs runtime (all ~40 sites are if-let/let-else/map/match guarded); all-pane work (focus, title bar, ×, rect layout, mouse handlers) stays OUTSIDE the terminal narrows — only pump/resize/prompt/selection are guarded; no semantic drift (terminal()==state() at runtime today since every pane is a terminal).
- **Noted for downstream (not #120):** non-terminal panes currently render an empty framed pane + swallow keys (by design → seq-2 adds content/handlers); the code viewer will have two homes (RootView.code_view Option + PaneContent::CodeView) until seq-5 retires the Option.

## Phase 4 — Validate
- **Tests added:** pane_new_is_terminal, open_pane_filetree_is_sessionless (incl. R30 assert_registry_matches_tree — the algebra critic hardening), open_pane_codeview_preserves_state, kind_derives_per_variant + the reworked pane_kind_survives_split_and_close (open_pane(Git)). `cargo nextest` → 5 pass; 40 workspace/pane tests green.
- **Self-test:** LIVE terminal capture (term120.png) — the terminal pane still renders (cyan focus border, prompt + cursor at bottom, full chrome) after the ~120-access migration. REQ-005 PASS.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100. Fixes en route: (1) added Workspace::terminal(id)/terminal_mut(id) both-arm tests (killed the terminal_mut→None mutant); (2) gate:14 rustdoc — the new pub PaneState/kind() docs linked non-re-exported PaneContent/PaneKind + bare method names → plain backticks (PR-...-no-public-to-private-intra-doc-link); (3) gate:4 — the codeview test had an unreachable `_ => panic!` arm → added a code_view() accessor (seq-5 seam) so the assert is branch-free + both arms covered.

## Phase 5 — Complete
- CHANGELOG + app_shell.md M6 section; forge #120 → done. **M6 1/8 — the foundation.** PaneContent<S> typed panes (Terminal(TerminalPane)|FileTree|CodeView|Git) + open_pane + the narrow accessors; ~120 app.rs accesses migrated; terminal live-proven intact; 3 critics clean; cov/MSI 100. NO visual change (foundation) — seq-2 (render dispatch) is the first visible step.
