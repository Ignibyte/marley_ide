# Focus-box border occluded by the drag-divider — Notes

- **Forge ticket:** #228 (2d14452f-4beb-4bab-b37a-36e7ddeb179c)
- **AAR:** 02e8710c-4b71-4e08-a047-e8e76fcde61f
- **Local ticket doc:** docs/planning/tickets/open/TICKET-228-focus-border-over-divider.md
- **Pipeline spec:** focus-border-over-divider.spec.md

## Phase 1 — Plan
- **Request:** fix the focus-box border being occluded by the drag-divider on the shared pane edge (chad
  live feedback #4). First ticket of the /goal /work 228-237 auto-approved M13 train.
- **Classification / tier:** work pipeline · bug · M13 · ONE tiny render-order slice (app.rs only).
- **Forge recall (§18.3):** relevant prior rules — `PR-claude-gpui-no-box-sizing-border-inflates` (#220 — a
  border grows an auto-sized cell → use a fill for a fixed-cell state; here the border is already 4 absolute
  fill bars, not a box-sizing border, so that trap doesn't apply); the M12.1 #191 focus-border work + #130
  divider are the deps. Clean-room §20.
- **Discovery (root-caused this turn):**
  - `workspace::focus_border_rects(content, thickness)` (workspace.rs:51, PURE, tested :690) returns 4 edge
    bars `[left, top, right, bottom]`; right = `x+w-thickness`, left = `x`. UNCHANGED by this fix.
  - The focus border is rendered PER-PANE inside the pane loop (app.rs ~:5179-5201): `if is_focused { let
    content = { x:r.x, y:r.y+PANE_TITLE_H, w:r.w, h:(r.h-PANE_TITLE_H).max(0) }; for edge in
    focus_border_rects(content, 2.0) { root = root.child(absolute accent div) } }`. The title bar is drawn
    AFTER the border (~:5213), at `r.y`, `PANE_TITLE_H` tall — but the border's top is at `r.y+PANE_TITLE_H`
    (below the title bar) so they don't overlap.
  - The #130 dividers are drawn AFTER the whole pane loop (~:5258-5303): for each boundary,
    `div().absolute().left(left_rect.x + left_rect.w - 3.0).top(0).w(6).h(content_h).occlude().bg(border)
    .hover(accent).on_mouse_down(...)`. Being drawn later, the 6px divider (edge±3) paints OVER the focus
    border's shared-edge bar (a focused left pane's right bar `x+w-2..x+w`, a focused right pane's left bar
    `x..x+2`) → the bug.
- **Decisions:** D1 fix by render ORDER (border after the dividers, topmost pane-chrome); D2 hoist the focused
  content rect into `Option<Rect>`, draw once after the divider loop; D3 `focus_border_rects` unchanged (no new
  pure fn — paint-order fix); D4 border divs stay non-`.occlude()` so the divider drag survives.
- **Env note:** driven capture may be env-blocked (machine was LOCKED earlier; chad's been active → re-check at
  validate). Fallback = the z-order mechanism + code review; re-verify when unlocked. gate-15 headless.

## Phase 2 — Design

### Architecture / approach
A pure render-ORDER move in the app.rs pane-render (the masked gpui shim). Confirmed structure by reading:
- `let rect_list = …` (app.rs:4467) → the pane loop `for (pane_id, r) in &rect_list {` (:4518);
  `let is_focused = pane_id == focused;` (:4523) — **`focused` is a single pane id, so exactly one pane is
  focused per render** → an `Option<Rect>` is correct (not a Vec).
- The focus border is drawn INSIDE the loop (:5179-5201): `if is_focused { let content = Rect { x:r.x,
  y:r.y+PANE_TITLE_H, w:r.w, h:(r.h-PANE_TITLE_H).max(0.0) }; for edge in focus_border_rects(content, 2.0) {
  root = root.child(div().absolute().left(px(edge.x)).top(px(edge.y)).w(px(edge.w)).h(px(edge.h)).bg(colors.accent)) } }`.
  The `if is_focused` block does ONLY the border (nothing else to preserve). The title bar (:5213-5251) is a
  separate per-pane child, drawn after the border but frames from `r.y` — the border's top is at
  `r.y+PANE_TITLE_H` (below it), so no interaction.
- The pane loop closes ~:5252; the #130 divider loop (:5258-5303) draws the `.occlude()` 6px dividers AFTER;
  the palette/overlay block starts ~:5305.

**The fix (D1/D2):** move the border draw to AFTER the divider loop so it is the topmost pane-chrome:
1. Before the pane loop (~:4517), declare `let mut focused_border: Option<Rect> = None;`.
2. In the loop, replace the `if is_focused { … draw … }` (:5179-5201) with
   `if is_focused { focused_border = Some(Rect { x:r.x, y:r.y+PANE_TITLE_H, w:r.w, h:(r.h-PANE_TITLE_H).max(0.0) }); }`
   (compute the SAME content rect, store it — no draw).
3. After the divider loop, BEFORE `if self.palette_open` (~:5304), drain:
   `if let Some(content) = focused_border { for edge in focus_border_rects(content, 2.0) { root = root.child(<the
   byte-identical accent div>) } }`.

`Rect`, `colors`, `root`, `focus_border_rects` are all in scope at the drain site (the divider block already uses
`colors`/`root`). The border divs stay non-`.occlude()` (D4) → the mouse still passes through to the occluding
divider underneath → the drag survives. Drawing BEFORE the palette block keeps the border UNDER the overlays (D:
overlays still occlude it, unchanged).

### File manifest
| File | Kind | Change |
|---|---|---|
| `crates/marley_app/src/app.rs` | SHIM (render) | Declare `focused_border: Option<Rect>` before the pane loop; store the focused content rect in the loop (was: draw); drain + draw the border after the divider loop (before the palette overlay). A pure move — the border div block is byte-identical. |

No pure-crate changes (`workspace::focus_border_rects` unchanged), no test-file changes.

### Regression Test Plan
| REQ | Test | Note |
|---|---|---|
| REQ-001 | Driven capture: focus a split pane → the 2px accent border visible on ALL 4 edges incl the divider-side edge | The bug's headline; if the machine is LOCKED → env-blocked → the z-order mechanism (border now drawn after the divider → paints on top) + code review, re-verify when unlocked |
| REQ-002 | Code review: the border divs are NOT `.occlude()` (divider is) → drag preserved; + a driven divider-drag if unlocked | Mechanism + review |
| REQ-003 | `workspace::focus_border_rects` unit tests stay green (UNCHANGED); a driven lone-focused-pane capture shows the 4-edge frame unchanged | Regression |

**No new unit tests** — the only pure surface (`focus_border_rects`) is unchanged and already cov/MSI 100
(workspace.rs:690); the fix is a render-order move in `marley_app/src/app.rs`, which is in the gates.sh coverage
`--ignore-filename-regex` (the render shim) + is `mutants::skip`, so there is NO cov/MSI delta and the gate stays
green [diff] from the existing suite. This is a **gate-is-shim** render fix — validated by the driven capture (or
the env-blocked mechanism + code review). **Uncoverable path:** the live GUI paint order — verified by pixels
(driven) or the z-order argument (env-blocked), never by a unit test.

### Risks / decisions
- **R1 — single-focus assumption:** `is_focused = pane_id == focused` → exactly one focused pane → `Option<Rect>`
  is right. (If the focus model ever went multi-select, this would need a Vec — it does not today.)
- **R2 — scope:** `Rect`/`colors`/`root`/`focus_border_rects` all live past the divider loop (the divider block
  uses `colors`/`root`); the drain compiles there. `cargo check` confirms at implement.
- **R3 — layering:** drain BEFORE the `if self.palette_open` block so the palette/overlays still occlude the
  border (unchanged z-order vs the overlays; only the border-vs-divider order flips).
- **R4 — validation:** env-blocked if the machine is still locked → mechanism + code review; the pixels are the
  real proof, re-verify when unlocked.
- Decisions D1-D4 per the spec (render-order, hoist to Option<Rect>, focus_border_rects unchanged, non-occlude).

## Phase 3 — Implement
**Built (manifest as designed) — `crates/marley_app/src/app.rs`, a 3-edit render-order move:**
1. Declared `let mut focused_border: Option<Rect> = None;` immediately before the pane loop
   `for (pane_id, r) in &rect_list {` (~:4518).
2. Replaced the in-loop border DRAW (`if is_focused { let content = Rect{…}; for edge in
   focus_border_rects(content,2.0){ root.child(accent div) } }`) with a STORE: `if is_focused {
   focused_border = Some(Rect { x:r.x, y:r.y+PANE_TITLE_H, w:r.w, h:(r.h-PANE_TITLE_H).max(0.0) }); }`
   (same content-rect computation; the `if is_focused` block did only the border, so nothing else lost).
3. DRAIN + draw after the #130 divider loop, immediately before the `if self.palette_open {` overlay block:
   `if let Some(content) = focused_border { for edge in focus_border_rects(content, 2.0) { root = root.child(
   div().absolute().left(px(edge.x)).top(px(edge.y)).w(px(edge.w)).h(px(edge.h)).bg(colors.accent)) } }` —
   the border div block is byte-identical to the old one, just relocated.

**Deviations from design:** none. `Rect`, `colors`, `root`, `focus_border_rects` were all in scope at the drain
site (compiled first try). No pure-crate/test changes; `workspace::focus_border_rects` untouched.

**Checks:** `cargo fmt` clean; `cargo check -p marley --all-targets` ✓ (the `block v0.1.6` note is the
pre-existing gpui-transitive warning); `cargo clippy -p marley --all-targets -- -D warnings` exit 0.

## Phase 3.5 — Inspect
1 focused general-purpose critic (proportionate to a ~28-line render-order move) + self-review. **NO FINDINGS
— the fix is a correct, minimal, byte-faithful z-order move.** No code changes at inspect.

Lenses verified concretely (critic read the code + enumerated the render):
- **Z-order fix correct** — the border drain is strictly AFTER the #130 divider loop → the 4 accent bars now
  paint over the `.occlude()` divider on the shared edge → bug fixed.
- **Overlays still occlude the border (the focused lens)** — the critic enumerated EVERY `root.child()` between
  the pane-loop start and the drain: only pane body, title bar, divider, drag handlers — NO modal renders before
  the drain. All 15 overlays (palette, save-workflow, agent launcher, file/history finders, forge, fleet, git
  panel, diff, find bar, status footer, top bar/tabs, top search, completion popup, context menu) render AFTER
  the drain → they still paint on top of / occlude the focus border. NO regression (the fix flips only the
  border-vs-divider order, not border-vs-modal).
- **Store/drain equivalence** — the stored content rect + the drained border-div block are byte-identical to the
  removed in-loop draw; the old `if is_focused` block did ONLY the border (nothing dropped).
- **Single-focus** — `is_focused = pane_id == focused` + unique pane ids → at most one `Some` stored →
  `Option<Rect>` correct (not a Vec); `focused` off-grid → stays `None`, draws nothing (unchanged).
- **Drag preserved** — the border divs have no `.occlude()` + no listeners → zero hitbox; the divider's occluding
  hitbox stays topmost in hit-testing regardless of paint order → the #130 drag still fires.
- **Title-bar/edges** — border frames content below the title bar; each border is within its own pane's content
  rect; the focused pane's own title bar now sits under the border but they share only the boundary line (zero
  visual change). Top/bottom/non-divider edges geometrically unchanged (same `focus_border_rects` call).
- **Borrow/scope** — `focused_border` is a plain local set inside the `&rect_list` immutable-borrow loop + read
  after; `Rect` is `Copy`; exactly one `focus_border_rects` call remains (no stray second draw). Clean-room §20 —
  the #228 hunks reference only internal tickets.

No `failure-record`/`prevention-rule` — the implementation was correct (no defect found).

## Phase 4 — Validate
**Tests added:** NONE — this is a gate-is-shim render-order move. The only pure surface
(`workspace::focus_border_rects`) is UNCHANGED + already cov/MSI 100; `marley_app/src/app.rs` is in the gates.sh
coverage `--ignore-filename-regex` + is `mutants::skip`, so there is no cov/MSI delta and no unit test to add
(per §7 for a gate-is-shim change). Confirmed against the design's test plan.

**RUN:** `cargo nextest run -p marley` → **312 passed, 2 skipped** (all existing green — the render-order move
broke nothing).

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] 15/15** (cov 100 + MSI 100 unchanged — the app.rs
render is excluded/skip). Receipt written for `/commit`.

**Driven capture (REQ-001/003) — DEFERRED (env-considerate), verified by mechanism:** the machine is UNLOCKED
this time, BUT the lock-check screenshot showed chad **actively working on a SHARED desktop** (banner: "desktop
shared with chadpeppers2@gmail.com") with multiple live agent sessions in Warp, and the Marley instance opened
earlier is no longer running — so a driven split-capture would require LAUNCHING Marley over his active screen,
hijacking his focus (and risking keystroke-misdirect into Marley). For a byte-identical z-order move the inspect
critic already verified exhaustively, that intrusion isn't warranted. Verified instead by the MECHANISM: the
focus border is now drawn AFTER the #130 divider loop → it paints on top → the divider can no longer occlude its
shared-edge bar (REQ-001); the critic enumerated all 15 overlays as rendering AFTER the drain → the border stays
under every modal (no regression); the border divs have no `.occlude()`/listeners → the #130 drag is preserved
(REQ-002); the geometry (`focus_border_rects`) is unchanged → the other edges are unchanged (REQ-003). **Offer a
20-second driven split-capture (`H:t,t` seed → focus → the accent border on the divider edge) the moment chad's
screen is free** — no separate ticket. gate-15 is headless → green regardless.

**Pre-existing failures:** none.

## Phase 5 — Complete
**Docs (§21):** CHANGELOG.md — #228 at the top of `[Unreleased] ### Fixed`. app_shell.md — a #228 note on the
#191 focus-border paragraph (drawn last among pane-chrome, after the #130 divider loop, before the modals).

**Forge capture (§19):**
- `aar-submit` 02e8710c — completed, effectiveness 5. Lessons: (a) a per-pane focus affordance that shares an
  edge with a LATER-drawn sibling (the `.occlude()` #130 divider) is occluded by gpui paint order → the fix is a
  render-ORDER move (draw the border after the overlapping chrome), NOT geometry; (b) hoisting a per-item draw
  out of a loop = a clean `Option<Rect>` (single-focus) drain after the loop, keeping the draw byte-identical;
  (c) the right layer for a focus border is "topmost pane-chrome but under the modals" — verify by enumerating
  every overlay renders after the drain (the critic did, for all 15). Process note: the driven capture was
  env-considerately deferred (chad's active shared desktop, no Marley running) → mechanism-verified; a z-order
  move suits mechanism verification well.
- NEW `prevention-rule` **PR-claude-focus-affordance-draws-after-overlapping-sibling-chrome-001** (31c07755).

**Env note:** a 20-second driven split-capture (`H:t,t` seed → focus → the accent border visible on the divider
edge) should be run when chad's screen is free, to close the visual loop — no separate ticket (mechanism +
critic + the passing suite carry it).

**Close + archive:** forge #228 → done. Local TICKET-228 → closed/. Pipeline doc pair → completed/. Spec status
→ Phase 5 — Complete PASS.
