# the file explorer as a real pane (M6 seq-4) — Notes

- **Forge ticket:** #123 `e79c0d80-4794-48cf-bcb5-d9675a74d8b4` · **AAR:** `eb4f9bc5-5703-4c83-8594-f4834c2ce95d`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-123-files-pane.md

## Phase 1 — Plan
- **Request:** forge #123 (M6 run 4/10) — retire the dock file tree; the FileTree pane is the sole tree.
- **Pre-flight:** dock FILES section app.rs 2108-~2185 (toggle 2148, file-click 2154+); FileTree pane ~2759 (static).
- **Decisions:** D1 sidebar sessions-only; D2 pane rows get toggle + ⌘-click viewer (plain click no-op → #128).
- **AAR id:** `eb4f9bc5-5703-4c83-8594-f4834c2ce95d`.

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
- **app.rs (SHIM):** (a) delete the dock FILES section — the "FILES" header (~2108) + the `for (index,row) in self.file_tree.visible_rows()` loop (~2116) through its close; the left dock then ends after the sessions. (b) the FileTree pane render (#121, ~2759): give each row the folder-toggle (dir → view.file_tree.toggle(idx)) + file rows ⌘-click → open_file_in_viewer(viewer_open_path(path,false)); a plain file click is a no-op (→ #128). Rows carry the enumerate index for toggle/path_at.
- **Test plan:** none (shim-only masked); gate + a live capture.
- **Risks:** find the EXACT loop close to delete the whole FILES block (no orphan braces); the pane rows already iterate visible_rows in #121 — add on_mouse_down there with the index.

## Phase 3 — Implement
- **Built:** removed the dock FILES section (87 lines — header + the visible_rows loop) → the left sidebar is sessions-only; enriched the #121 FileTree pane render with per-row on_mouse_down (dir → file_tree.toggle(index); file ⌘-click → open_file_in_viewer via viewer_open_path; plain click no-op → #128). Rows now enumerate for the index.
- **Verification:** fmt; check 0 err (lib+all-targets); clippy OK (no unused — file_icon/entry_is_dimmed/viewer_open_path still used by the pane).

## Phase 3.5 — Inspect
- **Method:** self-review of a masked dock-removal + pane-handler move.
- **Lenses — no findings:** the dock FILES block deleted cleanly (compiler confirms no orphan braces; the sessions `files` still feeds dock_panel); no dangling refs (file_icon/entry_is_dimmed/viewer_open_path/open_file_in_viewer still used by the pane). The pane rows enumerate → toggle(index)/path_at(index) address the RIGHT row (same iteration order as the old dock). No panic (path_at Option-guarded). The file tree now lives in ONE place (the pane). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** none (shim-only; render/handlers masked). Gate runs the existing suite.
- **Self-test:** LIVE capture (files123.png) — the left sidebar is SESSIONS-ONLY (WORKSPACE terminal 1/2, NO "FILES" section) + the Files pane shows the tree. Duplication #1 (two file trees) CLEARED. REQ-001+002 PASS.
- **Gate:** GREEN [diff] 15/15 (shim-only).

## Phase 5 — Complete
- CHANGELOG; forge #123 → done. **M6 4/10.** Retired the dock file tree (sidebar sessions-only) + moved toggle/file-click into the Files pane. Duplication #1 CLEARED (live-proven). Shim-only, gate GREEN.
