---
pipeline_id: d65e4185-2527-49f4-9843-c70515644b0e
ticket: forge#191 (578fad03-7e09-467a-8f56-2c13d43907a7)
aar_id: 5aa2c0f2-07eb-4d87-9db9-892bbd667157
---

# Notes — M12.1 #191 full focus border

## Phase 1 — Plan
Current #131 affordance (app.rs ~4607-4632, inside `if is_focused`): two `.absolute()` accent divs — a left bar
(`left(ax) top(ay) w(2) h(ah)`) and a top bar (`left(ax) top(ay) w(r.w) h(2)`), where
`ax=r.x, ay=r.y+PANE_TITLE_H, ah=(r.h-PANE_TITLE_H).max(0)`. chad wants all 4 sides.

**Approach.** A pure `focus_border_rects(content, thickness) -> [Rect;4]` returns the four edge rects; the render
draws one accent div per rect. 4 thin bars (not a frame div) so the pane interior stays clickable (D1). Right and
bottom bars inset by `thickness` so they sit inside the content rect — the right edge is visible now that #189
keeps the pane inside the window (D2).

Edge rects for `content = {x,y,w,h}`, thickness `t`:
- left   = `{x,       y,       t, h}`
- top    = `{x,       y,       w, t}`
- right  = `{x+w-t,   y,       t, h}`
- bottom = `{x,       y+h-t,   w, t}`

Mutation targets: the `x+w-t` / `y+h-t` subtractions (a `+`/`*` swap or a dropped `-t` moves the right/bottom
bar off the edge) — killed by asserting exact right/bottom origins.

**Files.** workspace.rs (`focus_border_rects` + unit test), app.rs (render: build `content` Rect, draw 4 divs).
`Rect`/`inset_right` already live in workspace.rs; this reuses `Rect`.

**Risks.** None load-bearing. A pane thinner than `2·thickness` → the left/right (or top/bottom) bars overlap;
harmless (both accent). No clamp needed. gpui-free pure geometry; the render is the usual mutants::skip shim.

## Phase 2 — Design
**Architecture.** Pure geometry lands in `workspace.rs` (the gpui-free layout module that already owns `Rect` +
`inset_right` + `pane_rects`); the render shim in `app.rs` consumes it. No new types — reuses `Rect`. No IO, no
panics, no process-spawn. Fits the established split: pure fns are unit+mutation tested; the app-view render is
the coverage-excluded `mutants::skip` shim proven by driven capture.

**File manifest.**
- `crates/marley_app/src/workspace.rs` — add `pub fn focus_border_rects(content: Rect, thickness: f32) ->
  [Rect; 4]` returning `[left, top, right, bottom]` (right = `x+w-thickness`, bottom = `y+h-thickness`); + unit test.
- `crates/marley_app/src/app.rs` — in the `if is_focused` arm (~4607), build `content = Rect { x: r.x,
  y: r.y+PANE_TITLE_H, w: r.w, h: (r.h-PANE_TITLE_H).max(0.0) }`, then `for rect in focus_border_rects(content,
  2.0) { root = root.child(accent div at rect) }` — replaces the two inline left+top bars. Import
  `focus_border_rects`.

**Regression Test Plan.**
| Test | Proves |
|---|---|
| `focus_border_rects_frames_all_four_edges` (workspace.rs) | REQ-001 — the 4 edge rects are exact; right origin = `x+w-t`, bottom origin = `y+h-t`, each `t` thick (kills the inset `-` mutants + a field/edge swap) |
| driven capture (focused pane) | REQ-002 — a focused pane renders all 4 accent edges (top+left+right+bottom) |
| driven capture (2-pane split) | REQ-003 — only the focused pane has the border; the other has none |

No trybuild/integration rows (pure geometry + a render tweak). The render is shim (mutants::skip + cov-excluded)
→ driven captures are its proof; the pure `focus_border_rects` carries the unit+mutation load.

**Risks/decisions.** D1/D2/D3 in the spec (4 bars not a frame; inset right/bottom; 2px accent). The `content`
height clamp `(r.h-PANE_TITLE_H).max(0.0)` is pre-existing (#131) — keep it. `[Rect; 4]` (fixed array, not Vec)
— the border is always exactly 4 edges; avoids an allocation and makes the test exhaustive.

## Phase 3 — Implement
Built to the manifest; `cargo check -p marley` clean.
- `workspace.rs` — `pub fn focus_border_rects(content, thickness) -> [Rect; 4]` (left/top/right/bottom; right =
  `x+w-thickness`, bottom = `y+h-thickness`), placed right after `inset_right`.
- `app.rs` — imported `focus_border_rects`; the `if is_focused` arm now builds the `content` Rect and loops
  `focus_border_rects(content, 2.0)` drawing one accent div per edge (replaced the two inline left+top bars).
**Skip-detach check:** none — `focus_border_rects` is a standalone pure `pub fn` between two pure fns
(`inset_right` / `CellSize`), no adjacent `mutants::skip`. **Deviation:** none.

## Inspect (Phase 3.5)
One correctness critic over the diff. **Verdict: CLEAN** on the two key checks.
- **Geometry (REQ-001) — exact.** Hand-traced `focus_border_rects({10,20,100,50}, 2.0)` = `[{10,20,2,50},
  {10,20,100,2}, {108,20,2,50}, {10,68,100,2}]`. Right edge 108+2=110=content-right, bottom 68+2=70=content-
  bottom → border on the inner edge, no overflow (preserves #189's in-window right edge). No off-by-thickness.
- **Mouse-blocking (REQ-002) — no regression.** The 4 bars are bare `.absolute()` accent divs with NO
  `.on_mouse_down`/`.occlude()` → gpui inserts no hitbox, clicks fall through to the pane's own handler (same as
  the #131 edges + the agent/remote badges). Still 4 thin 2px bars, not a filled frame.
- **Unfocused (REQ-003) — correct.** Border draws only inside `if is_focused`; the per-pane `.border_1()` uses
  `colors.border` (not accent) on every pane, so unfocused = base frame only. No accent leak.
- **Edge cases** cosmetic/harmless (thin/short panes; the opaque title bar even paints over a degenerate bottom
  bar). Build clean; `focus_border_rects` not skip'd, no skip to detach.
- **Mutation sizing:** 8 viable mutants (right `x+w-t` ×4, bottom `y+h-t` ×4); the whole-fn `Default::default()`
  mutant is UNVIABLE (Rect has no `Default`). One 4-rect assert with distinct x/y/w/h/thickness kills all 8.

**[LOW → FIXED] Stale comment.** app.rs:4117 still called the affordance a "left+top accent EDGE … not a full
border" — now false. Reworded to "a base frame on every pane; the focused pane's accent is a full 4-edge overlay
border (#191)."

Verdict: **Phase 3.5 PASS** — geometrically exact, no mouse-blocking, focus-gated; one stale comment fixed.

## Phase 4 — Validate
**Test.** `focus_border_rects_frames_all_four_edges` (workspace.rs, REQ-001) — asserts the exact 4 rects for
`{10,20,100,50}, 2.0`. `cargo nextest run -p marley focus_border_rects` → **1 passed**. Mutation
`-F focus_border_rects` → **9 caught, 1 unviable** (the `Default::default()` whole-fn; Rect has no Default) → all
8 viable arithmetic mutants killed.
**Driven (REQ-002/003).** Rebuilt; right-click → Split Right (2-pane split, `clearmods` applied). Capture
`marley-191-border.png`: the FOCUSED (right) pane shows a full 4-side cyan accent border; the UNFOCUSED (left)
pane shows only the grey base border. Corner crop `marley-191-corner.png` confirms the new right + bottom cyan
edges meet inside the pane (the #189 gutter visible to the right of the accent edge → the right edge is
in-window). chad's "only top+left" complaint resolved.
**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15/15 (cov 100%, mutation MSI 100%, visual/AX).
No pre-existing failures in scope. **Phase 4 PASS.**

## Phase 5 — Complete
**Docs.** CHANGELOG `### Changed` entry (TICKET-191). `app_shell.md` — the #131 focus-edge note now records the
#191 full 4-side border + the pure `focus_border_rects`.
**Knowledge.** `aar-submit` (aar `5aa2c0f2…`, completed, effectiveness 5 — clean, one LOW stale-comment fix). No
failure-record (no runtime defect). Applied the `clearmods`-first lesson from #190 proactively — the driven split
captured on the first try.
**Lessons.** Extracting the border geometry to a pure `focus_border_rects` (vs 4 inline divs) bought a
mutation-tested regression lock on the error-prone right/bottom insets — the same pattern as #189's `inset_right`.
The visual pipeline (#189 gutter → #191 border) composes: the border's inset right edge is visible precisely
because #189 keeps the pane in-window. forge wired — captured above + locally (§19).
