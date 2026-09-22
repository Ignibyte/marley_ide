# pane title bars — Notes

- **Forge ticket:** #108 `b8cd2f66-f520-40b6-9324-4017b40011a0` · **AAR:** `b5a055d0-d3de-4df6-83cd-301ea1826534`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-108-pane-title-bars.md

## Phase 1 — Plan
- **Request:** forge #108 (M5 2/12) — a title bar atop each pane.
- **Pre-flight:** the per-pane render (app.rs ~2012) is a bottom-anchored flex_col; close = workspace.close.
  Approach: a separate absolute title-bar strip + shrink the content pane's top/h (D1) — no child restructure.
- **Decisions:** D1 strip + shrunk content; D2 × reuses workspace.close.
- **AAR id:** `b5a055d0-d3de-4df6-83cd-301ea1826534`.

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
- **workspace.rs (PURE):** `pub fn pane_title(kind: PaneKind, name: &str) -> String` (Terminal→name-or-"terminal"; FileTree→"Files"; CodeView→name-or-"code"; Git→"Source Control"); `pub fn pane_icon(kind: PaneKind) -> &'static str` (Terminal ▸ / FileTree 📁 / CodeView 📄 / Git ⎇ — 4 distinct).
- **app.rs SHIM (masked):** `const PANE_TITLE_H: f32 = 24.0` (bare literal — no mutable arithmetic). In the per-pane loop: (1) the content pane div → `.top(px(r.y + PANE_TITLE_H)).h(px((r.h - PANE_TITLE_H).max(0.0)))` (shrinks below the bar); (2) a NEW `title_bar = div().absolute().left(px(r.x)).top(px(r.y)).w(px(r.w)).h(px(PANE_TITLE_H)).bg(surface).border_b_1().flex().flex_row().items_center().gap_1()` with `pane_icon(kind)` + `pane_title(kind, &name)` (name = self.project_root.file_name basename) + a spacer + "⋮"(stub) + an "×" child whose on_mouse_down → `let _ = view.workspace.close(pane_id); cx.notify()`; add both the content pane AND the title bar to root (root.child(pane).child(title_bar)). kind read via `self.workspace.state(pane_id).kind` (defaults Terminal).
- **Mutation targets:** pane_title arms (name-vs-fixed + empty fallback), the 4 pane_icon arms.
- **Test plan:** pane_title_by_kind (Terminal name/empty, FileTree, CodeView name, Git); pane_icon_distinct (4 glyphs, all different). cov/MSI 100. The strip + shrink + × masked (live capture).
- **Risks:** the content pane must SHRINK (top+PANE_TITLE_H, h-PANE_TITLE_H clamped ≥0) so the prompt isn't clipped; the × close refuses the last pane (existing algebra); the title bar is drawn AFTER (over) the pane border so it reads as a header.

## Phase 3 — Implement
- **Built:** pane_title + pane_icon (workspace.rs, 4 distinct glyphs); app.rs PANE_TITLE_H=24.0 + the content pane shifted/shrunk below the bar + a title-bar strip (icon + name[project basename] + ⋮ stub + × close via workspace.close). All panes Terminal → all show ▸ + the cwd.
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (two small total match fns + a masked render strip).
- **Lenses — no findings:** pane_title (Terminal/CodeView name-or-fallback, FileTree/Git fixed — all arms tested); pane_icon (4 distinct glyphs); the content pane top+=PANE_TITLE_H and h-=PANE_TITLE_H clamped .max(0.0) so the prompt is NOT clipped + h never negative; the × close reuses the proven workspace.close (refuses the last pane); kind read via state().unwrap_or(Terminal) — no panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** pane_title_by_kind (6 cases) + pane_icon_distinct (4 glyphs, all unique). `cargo nextest` → 2 pass; clippy OK.
- **Self-test:** LIVE static capture (titlebar108.png) — the terminal pane shows a title-bar strip at its top: "▸ ~" (icon + name; "~" = the fallback since the open-launched cwd is "/", no basename) on the left + "⋮ ×" (menu stub + close) on the right, ABOVE the content, prompt "❯ / |" still at the bottom (not clipped). REQ-003 PASS.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG ### Added; forge #108 → done. **M5 2/12.** pane_title + pane_icon (cov/MSI 100) + the title-bar strip (icon+name+⋮+× close). Live-proven (titlebar108.png). First visible Warp chrome.
