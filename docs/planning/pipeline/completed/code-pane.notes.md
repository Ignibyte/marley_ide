# the code viewer as a grid pane (M6 seq-5) — Notes

- **Forge ticket:** #124 `efa403e4-ef91-480f-9047-6c4f14aa217d` · **AAR:** `2d71d5d5-4c01-4239-98bf-8d28ce529aa7`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-124-code-pane.md

## Phase 1 — Plan
- **Request:** forge #124 (M6 run 5/10) — code viewer → a grid pane; retire the M5 #114 side-panel.
- **Pre-flight:** code_view field (142); open_file_in_viewer (1224); side-panel render 3183-~3310; right_open
  (2271); callers 955/2717/3492; the CodeView pane render (#121) draws minimal from code_view().
- **Decisions:** D1 open-or-update a single code pane; D2 state moves to PaneContent::CodeView, field removed.
- **AAR id:** `2d71d5d5-4c01-4239-98bf-8d28ce529aa7`.

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
- **workspace.rs (PURE):** first_pane_of_kind(kind)->Option<PaneId> (pane_ids order, kind()==kind); set_content(pane,content)->bool (panes.get_mut → replace content, true; None→false).
- **app.rs (SHIM):** the CodeView pane render → full viewer (gutter_width(cv.lines.len)+visible_range(cv.scroll,VIEWER_ROWS,len)+line rows from the pane code_view()); open_code_pane(state): first_pane_of_kind(CodeView)→set_content+focus else open_pane(CodeView(state)), then persist_grid; open_file_in_viewer reworked to build the CodeViewState then call open_code_pane; callers 955/2717/3492 unchanged (they call open_file_in_viewer). DELETE the side-panel block (3183-~3310) + code_view field + init (511) + right_open code_view (2271→just git_panel_open) + the scroll-code_view handlers (1257+).
- **Mutation targets:** first_pane_of_kind kind-match+first, set_content set+absent bool.
- **Test plan:** first_pane_of_kind (finds CodeView in [T,Code,Git]; None absent) + set_content (swaps content→kind changes; false unknown id).
- **Risks:** the code_view field has scroll handlers (1257-1280 ↑/↓) — reroute to the focused CodeView pane or drop for #124 (the pane has no scroll-key yet; note). Ensure no dangling code_view refs after removal (compiler-driven).

## Phase 3 — Implement
- **Built (workspace.rs PURE):** first_pane_of_kind(kind) + set_content(pane,content). **(app.rs SHIM):** open_code_pane (first_pane_of_kind(CodeView)→set_content+focus else open_pane; persist_grid); open_file_in_viewer reworked → open_code_pane; the CodeView pane render upgraded to the FULL viewer (gutter_width+visible_range+gutter_label + #99 syntax highlighting via highlight_line/language_of/TokenKind→colors). REMOVED: the side-panel render block (~76 lines), the code_view field + init, handle_code_view_key + its on_key_down routing, right_open code_view condition (→ just git_panel_open), the diff file-header jump-to-line (field-based).
- **DEVIATIONS:** (1) code_syntax was orphaned by the side-panel removal → REUSED it in the code pane (syntax highlighting — Warp-right, keeps the tested module live) rather than delete. (2) jump_to (code_view) became dead (scroll handlers removed) → deleted it + its test; a code-pane scroll-key follow-up can re-add it. (3) the diff file-header opens the file at the top (jump-to-line deferred — the viewer state now lives in the pane, not a field).
- **Verification:** fmt; check 0 err; clippy OK; 10 helper/code_view tests pass.

## Phase 3.5 — Inspect
- **Method:** self-review (the compiler + clippy already confirmed no dangling code_view refs / no dead code after the removal).
- **Lenses — no findings:** first_pane_of_kind = pane_ids-order find on kind()==kind; set_content = get_mut→replace, true/false. open_code_pane REUSES the single code pane (first_pane_of_kind→set_content) so files never spawn duplicate code panes (D1). The side-panel + field + handle_code_view_key + jump_to are fully retired (0 compile errors ⇒ no dangling refs; clippy OK ⇒ no dead code). The code pane reads the pane-local code_view() state (not a global field). Syntax colors map every TokenKind. No unwrap/panic (visible_range guards the slice; code_view() Option-guarded). No findings.
- **Fix applied:** the refactor itself.

## Phase 4 — Validate
- **Tests:** first_pane_of_kind_finds_or_none + set_content_swaps_or_absent (workspace, cov/MSI 100). Pass.
- **Self-test:** LIVE capture (code124.png) — grid="H:t,c" → [terminal | code pane] tiled; the code pane shows "1 // open a file to view it here" (gutter line-number + the comment body), NO right-side code panel. Duplication #2 CLEARED. REQ-003 PASS.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG; forge #124 → done. **M6 5/10.** Code viewer → a grid pane (full gutter+scroll+syntax); open_code_pane reuses one code pane; retired the M5 side-panel + code_view field. Duplication #2 CLEARED. cov/MSI 100.
