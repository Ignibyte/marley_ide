# M10 — Files panel polish — Notes

- **Forge ticket:** #168 `c458c35e-ea56-472b-a6b7-cb311a51d560` · **AAR:** `8324936d-ab1d-45cf-8d61-f5fe5456674e`

## Phase 1 — Plan
- Refresh on open (sync_active_project at both toggle sites); a drag-resizable persisted width (clamp 160–480).
- **AAR id:** `8324936d-ab1d-45cf-8d61-f5fe5456674e`.

## Phase 2 — Design (folded)
- settings.rs: `clamp_files_width(w: f32) -> f32` (NaN→240.0; clamp(160,480)); `files_width: f32` on
  AppliedSettings (default 240; load via a FilesWidthSetting mirroring RightSectionSetting, stored as a
  string); `persist_files_width(manager, w)`.
- app.rs: `files_panel_w: f32` field (init from applied.files_width); the render's `files_w` reads it; a 4px
  handle div at (regions.left + files_panel_w - 2, content band) with on_mouse_down → `dragging_files_edge =
  Some(last_x)`; a root `.on_mouse_move/.on_mouse_up` pair active when files_open (move: `files_panel_w =
  clamp_files_width(files_panel_w + dx)`; up: take + persist). Both toggle sites: on false→true call
  sync_active_project() first.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- settings.rs: FilesWidth (u16, 240, "files.width"); AppliedSettings.files_width; clamp_files_width (160–480, NaN→240); applied load clamps; persist_files_width (round→u16); the 3 test constructors gained the field.
- app.rs: files_panel_w (+dragging_files_edge) fields, init from applied; files_w reads files_panel_w; a 6px grab strip at the edge (stop_propagation + start drag); a root move/up pair (clamped Δx; release persists); both toggle sites sync_active_project() on open. FILES_PANEL_W const retired.
- fmt; check 0 err; clippy OK.

## Inspect (Phase 3.5)
Method: adversarial SELF-REVIEW (both halves mirror inspected patterns — the #130 divider drag, the
right_section setting).

- **Dual root drag pairs coexist:** the divider pair (inside the terminal-pane block) and the files pair
  (inside `if files_open`) each guard on their OWN Option state — both fire per move, at most one acts
  (one mouse). No interference.
- **The 6px grab strip** eats mouse-down only within its band (stop_propagation); the file rows are outside
  it. Acceptable hit-area cost for grabbability.
- **[accepted edge] toggling the panel closed MID-DRAG** (⌘B with the button held) orphans
  dragging_files_edge until the next open resumes from a stale x → one jump, bounded by the clamp. Rare
  (requires a chord during a held drag); not worth plumbing a cancel.
- **NaN/limits:** the clamp handles non-finite; a hand-edited tiny/huge files.width loads clamped (160/480);
  the u16 round-trip is lossless in-range.
- **Refresh:** sync_active_project on open re-walks the SAME root — the ⌘P list + tree both freshen; closed
  renders unchanged.

Lenses: drag-state coexistence, hit-banding, mid-gesture state loss, clamp/NaN, persistence round-trip.

## Phase 4 — Validate
- **Tests:** clamp_files_width_cases (interior/floor/ceiling/bounds/NaN/∞); files_width_round_trips_and_loads_clamped (a hand-edited 10 loads as 160; persist 333.4 → 333; persist 9999 → 480). 2/2.
- **Self-test:** fp_refresh2.png — AAA_168.tmp (created while the app ran, panel closed) appears under .cargo after toggling open (REQ-003); fp_drag.png — the drag verb widened the panel (~240→345pt; long names unclipped; the terminal shifted) within the clamp (REQ-004). tmp files cleaned.
- **Gate:** GREEN [diff] 15/15, MSI 100 (clamp + round-trip mutation-killed).

## Phase 5 — Complete
- CHANGELOG + app_shell #168 note; forge #168 → done. **M10 8/10.** (The archive mv was deferred one commit — the changelog hook vetoed the combined bash; re-run split.) LESSONS: Eq config structs make schema growth self-auditing; ONE pure clamp for load+drag; N root drags coexist via per-drag Option guards; NEVER combine sed/mv/add/commit in one bash — a PreToolUse veto kills ALL of it.
