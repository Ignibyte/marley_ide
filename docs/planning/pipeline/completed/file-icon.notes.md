# file-explorer icon in the top bar (M7) — Notes

- **Forge ticket:** #133 `ee402b99-849d-4993-8a90-a0e6ac168a86` · **AAR:** `71e24b56-db2c-4107-b51e-0cab2d38748f`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-133-file-icon.md

## Phase 1 — Plan
- **Request:** forge #133 (M7 run 2/5) — a top-bar file-explorer icon → opens the FileTree pane. Shim-only.
- **Pre-flight:** pane_icon(FileTree)="📁" (workspace.rs:190, tested); open_files_pane (#128, tested open_or_focus);
  the top-bar bg row at app.rs:3408.
- **Decisions:** D1 reuse pane_icon + open_files_pane (no new pure); D2 icon top-left of the bar.
- **AAR id:** `71e24b56-db2c-4107-b51e-0cab2d38748f`.

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
- **app.rs (SHIM):** after the top-bar bg row, add a clickable 📁 (pane_icon(FileTree)) at top:4 left:12; on_mouse_down → self.open_files_pane() + cx.notify. Shim-only (reuse).
- **Test plan:** none new (open path tested via #128 open_or_focus).
- **Risks:** pane_icon + PaneKind + MouseButton imported (yes). Placement not overlapping the sidebar toggle / search.

## Phase 3 — Implement
- **Built (app.rs SHIM):** a clickable 📁 (pane_icon(FileTree)) at top:5 left:12 in the top bar; on_mouse_down → self.open_files_pane() + cx.notify. Shim-only (reuse).
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (one clickable icon reusing tested paths).
- **Lenses — no findings:** the 📁 glyph = pane_icon(FileTree) (tested); the click → open_files_pane → open_or_focus (tested #128) → focus-or-open a single FileTree pane + persist_grid. No unwrap/panic. Placement top-left of the bar (before the centered search; not over the sidebar toggle). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** none new (shim-only; the open path is cov/MSI 100 via #128).
- **Self-test:** LIVE capture (fileicon133.png) — the 📁 file-explorer icon renders at the top-left of the top bar. REQ-001 PASS. (Capture also shows a FileTree pane from the persisted grid = what the icon opens.) Click→open_files_pane code-reviewed (env-blocked); open path tested via #128.
- **Gate:** GREEN [diff] 15/15.

## Phase 5 — Complete
- CHANGELOG; forge #133 → done. **M7 2/5.** A 📁 file-explorer icon in the top bar → open_files_pane. Shim-only.
