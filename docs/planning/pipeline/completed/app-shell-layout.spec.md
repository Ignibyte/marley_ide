---
pipeline_id: 333e220c-8fa8-442e-9a93-e14e18148520
ticket: forge#20 (ed1f5164-ab0d-4ffd-906c-ff76b01bf95c) · local docs/planning/tickets/open/TICKET-020-app-shell-layout.md
aar_id: aba53b1c-44e6-4b14-af2c-11de0836b138
sprint: M1.B — The Cockpit (cbc92bf0) seq 4/5
status: Phase 5 — Complete PASS
title: app_shell layout — 3-region docks + the PaneGroup split/close/neighbor algebra
type: feature
milestone: M1
references:
  - ../../../specs/SPEC-app-shell.spec.md
  - ../../../marley_architecture/app_shell.md
---

## Title

TICKET-020 — the [`SPEC-app-shell`](../../../specs/SPEC-app-shell.spec.md) workspace layout (R4–R13) in
`marley_app`: the PURE `PaneGroup` tree algebra (single / split / close / panes / neighbor / ratios) +
`DockSide`/`DockState` + the 3-region `RootView` render + the cmd-d/cmd-w → split/close wiring. M1.B "The
Cockpit" seq 4/5 — the biggest tested surface (the recursive pane tree).

## Scope

### In
- **`layout.rs` (PURE)** — `PaneId(u64)`, `PaneAxis{Horizontal,Vertical}`, `SplitDirection{Before,After}`,
  `Direction{Up,Down,Left,Right}`, `PaneError{PaneNotFound,LastPane}`, `enum PaneGroup{ Leaf(PaneId),
  Split{axis:PaneAxis, children:Vec<PaneGroup>, ratios:Vec<f32>} }`:
  - `single(pane)` → `Leaf`; `panes()` → depth-first leaves (R7).
  - `split(target,new,axis,dir)` → replace the `target` leaf with a `Split{axis, children:[new,target] if
    Before else [target,new], ratios:equal(2)}`; `PaneNotFound` if `target` absent (R8/R11/R13).
  - `close(pane)` → remove; collapse a 2-child `Split` to its survivor (R9); `LastPane` if it is the only
    pane (R10); `PaneNotFound` if absent (R13); reset the affected node's ratios to equal (R11).
  - `neighbor(from,dir)` → the R12 boundary-adjacency rule (path-based; see notes).
  - `DockSide{Left,Right}` + `DockState{Open,Closed}` + `DockState::toggled()` (R6 invert).
- **`app.rs` (SHIM)** — `RootView` gains `pane_group: PaneGroup` + `docks:[DockState;2]`; `dock(side)` /
  `toggle_dock(side)` (delegates to `toggled()` + notifies, R6) / `pane_group()`; the 3-region render
  (left dock | center pane-group | right dock; a Closed dock gets zero width so the center reflows,
  R4/R5); cmd-d → split the focused pane / cmd-w → close it (the #18 keymap actions).
- §21 — CHANGELOG + app_shell.md.

### Out / deferred
- Focus tracking / which pane is "active" for split/close (M1.B minimal: split/close a designated pane;
  full focus-follows in a later ticket). Per-pane terminal sessions (the center hosts the M1.A terminal;
  multi-terminal is later).

## Acceptance Criteria (EARS — adopt SPEC-app-shell R4–R13)
- **AC-R7** — `single(p).panes() == [p]`. Verify: `single_pane_has_one_leaf`.
- **AC-R8** — `split` replaces the target leaf with a `Split` of the given axis; `new` before/after the
  original per `Before`/`After`. Verify: `split_before_and_after_order_and_axis`.
- **AC-R9** — closing one of two split children collapses to the survivor; no 1-child splits remain.
  Verify: `close_collapses_two_child_split_to_survivor`.
- **AC-R10** — `close` on the only pane → `Err(LastPane)`, tree unchanged. Verify: `close_last_pane_errs`.
- **AC-R11** — `split`/`close` reset the node's ratios to equal fractions summing 1.0, len == child count.
  Verify: `ratios_renormalize_equal_after_split_and_close`.
- **AC-R12** — `neighbor(from,dir)` returns the boundary-rule adjacent pane (incl nested 2×2) and `None`
  at the edge. Verify: `neighbor_boundary_rule_flat_and_nested`.
- **AC-R13** — `split`/`close` on an absent `PaneId` → `Err(PaneNotFound)`, tree unchanged. Verify:
  `absent_pane_errs`.
- **AC-R6** — `DockState::toggled()` inverts Open↔Closed. Verify: `dock_state_toggles`.
- **AC-gate** — `layout.rs` (the algebra + dock toggle) cov 100 / MSI 100; the app.rs render + dock-region
  + keymap wiring ACCEPTED-UNTESTABLE (already excluded); FULL `scripts/gates.sh` → `GATE GREEN [diff]` +
  a headed both-docks/split assertion.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
1. **The pure/shim seam** — the whole `PaneGroup` algebra + `DockState::toggled` are PURE (no gpui/IO) →
   cov 100 / MSI 100. The 3-region gpui render + the zero-width-closed reflow + the cmd-d/cmd-w wiring are
   SHIM in app.rs (already `mutants::skip` + rust_cov-excluded — no gates.sh change).
2. **Split always makes a 2-child Split at the target leaf** (R8 — "replace that leaf with a Split"); it
   does NOT merge into a same-axis parent. Invariant: every `Split` has ≥2 children (single→Leaf,
   split→2, close collapses at 1) — so internal `children[0]`/`[len-1]` indexing is invariant-safe (not
   input-reachable; no §14 panic-path concern).
3. **neighbor is path-based** (R12): compute the root→`from` path (child indices), walk it upward to the
   first ancestor `Split` with `axis == dir`'s axis AND a sibling on the movement side, then descend that
   sibling taking the nearest-boundary child (last for Left/Up, first for Right/Down) to a leaf. Detail in
   notes.
4. **Frozen contract** — the `PaneGroup` (+ its methods) + `DockSide`/`DockState` signatures are frozen
   for M2 panels.

## Phase Plan
- **P2 Design** — the algebra bodies (split/close recursion, neighbor path-walk, ratios) + the mutation
  map (the spec's targets) + the flat-2x1 + nested-2x2 fixtures + the app.rs shim shape.
- **P3 Implement** — layout.rs + the app.rs render/dock/wiring (write-then-check the gpui render).
- **P3.5 Inspect** — `cargo mutants --list` (only layout.rs) + the neighbor coverage (both directions,
  both movement sides, the edge-None, the nested descend) + the collapse/LastPane/PaneNotFound.
- **P4 Validate** — the 8 pure tests + the #[ignore] headed docks/split + gate.
- **P5 Complete** — §21; close #20; → #21 settings (the finale).
