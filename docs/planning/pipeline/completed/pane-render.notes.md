# typed-pane render dispatch (M6 seq-2) — Notes

- **Forge ticket:** #121 `108e9743-4c3b-4518-a0d6-b0d86c13b3ca` · **AAR:** `a37488ef-1119-4808-b6e2-223d478f14a6`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-121-pane-render.md

## Phase 1 — Plan
- **Request:** forge #121 (M6 seq-2, first VISIBLE) — dispatch the pane body on kind; boot [terminal | files].
- **Pre-flight:** render loop app.rs ~2347; title bar per-kind already (~2739); terminal body `if let
  Some(term)=terminal(pane_id)` ~2442 (else = empty today); content renders inline (tree ~2052, code ~3143).
- **Decisions:** D1 dispatch on kind() → minimal-but-real bodies reusing pure fns; D2 temp boot default marked for seq-3.
- **AAR id:** `a37488ef-1119-4808-b6e2-223d478f14a6`.

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
- **app.rs render loop (~2347):** the per-pane content — currently `if let Some(term)=self.workspace.terminal(pane_id) { <terminal render> }` (~2442). Add an `else` matching `self.workspace.state(pane_id).map(|s|s.kind())`: FileTree→compact tree (visible_rows()+file_icon+entry_is_dimmed), CodeView→cv.lines if code_view() Some (minimal gutter+lines), Git→"Source Control"+change_summary. Bodies sit below the title bar (PANE_TITLE_H offset applies to all panes).
- **new():** TEMP boot default — after the workspace + FileTree open_pane, refocus PaneId(0); marked `#121 TEMP → seq-3`.
- **Test plan:** none (shim-only masked); gate runs existing suite; a LIVE capture is the proof.
- **Risks:** the pane content div structure must nest the body correctly (title bar + body); the temp default must not break input routing (refocus terminal).

## Phase 3 — Implement
- **Built:** app.rs else-dispatch on the non-terminal pane body (FileTree→the icon+dimmed project tree; CodeView→the pane code_view() lines with gutter; Git→"Source Control" minimal); the pane body wrapped h_full/justify_start so it top-anchors below the title bar. new() TEMP boot default: `let mut workspace = Workspace::new(session); workspace.open_pane(H, After, PaneContent::FileTree); workspace.focus(PaneId(0))` (marked #121 TEMP → seq-3). PaneContent imported.
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (a masked render dispatch; the capture is the real proof).
- **Lenses — no findings:** the dispatch covers all 4 kinds (Terminal via the if-let, FileTree/CodeView/Git via the else-match, `_ => {}` for the None/exhaustive fallback); NO panic — state() + code_view() are Option-guarded; the FileTree body reuses the proven #113 glyph/color/indent from the dock; the temp boot default is clearly `#121 TEMP → seq-3` + refocuses the terminal (input still routes to the terminal, not the Files pane). The pane body top-anchors (h_full/justify_start) below the existing title bar. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** none (shim-only; the render/handlers are masked). Gate runs the existing suite.
- **Self-test:** LIVE static capture (panes121.png) — boot renders the terminal pane (cyan focus border, prompt+cursor) TILED beside a real Files pane showing the project tree (disclosure ▾ + file-type icons), not an empty frame; status "focus: terminal" (the refocus worked). REQ-001+002 PASS. (Minor: the sidebar labels the 2nd pane "terminal 2" — it counts pane_ids; refined in seq-4+.)
- **Gate:** GREEN [diff] 15/15 (shim-only; existing suite passes).

## Phase 5 — Complete
- CHANGELOG; forge #121 → done. **M6 2/8 — first VISIBLE step.** The per-pane render dispatches on kind — a Files pane renders the tree in the grid, live-proven [terminal | Files] beside each other. Temp boot default → seq-3 replaces it. cov/MSI 100 (shim-only).
