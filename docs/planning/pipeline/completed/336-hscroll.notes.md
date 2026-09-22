# Horizontal scroll (#336) — Notes

- **Forge ticket:** #336 c07703c9-e388-471e-b2f1-e005a8178310
- **AAR:** 2c3e5411-190b-4131-acff-2b625dbe46fb
- **Local ticket doc:** ../../tickets/open/TICKET-336-hscroll.md
- **Pipeline spec:** 336-hscroll.spec.md

## Phase 1 — Plan

**Request:** the M22 batch's first ticket + the FIRST RUN of the pre-authored-spec method (promote, don't
author). Phase 1 therefore had two jobs: promote `queued/m22-hscroll.spec.md` → `active/336-hscroll.spec.md`
(done; pipeline_id `a0b0c5ba…`, AAR `2c3e5411…`, local ticket doc written), and **re-verify every seam the
pre-authored spec cites**. Classification: work pipeline, bug (a defect — the tail of a long line is
unreachable), one shippable slice.

**Forge recall:** `knowledge-context` surfaced 13 nodes. The load-bearing one is
**`AD-claude-editor-offset-column-model-001`** (#254 click-to-caret): its own recorded RISK is *"the shim's
pixel→(row,col): the code-area origin (gutter width) + scroll must be subtracted correctly, or the caret
lands off the click."* That is precisely this ticket's hazard, pre-recorded — `scroll_x` joins an existing
subtraction, and the AD's column model must not fork (the same constraint
`AD-claude-two-boundary-maps-for-phantom-text-001` imposed on #331).

### THE VERIFICATION LEDGER — the method's load-bearing step

`git diff --stat a9eb589..HEAD` = **docs only**. **No Rust changed since the spec was written**, so every
drift below is **authorship error, not code movement.** That distinction matters: the method is not just
protecting against a moving codebase, it is catching the spec author (me) citing seams they grepped but did
not read. Recorded as a Phase 1 finding rather than quietly patched.

| # | Claim | Verdict |
|---|---|---|
| 1 | `CodeViewState.scroll` is a row index, no x field | **VERIFIED** (code_view.rs:540) — but see F3 |
| 2 | `clamp_scroll_px(px, total_rows, cell_h)` vertical-only | **VERIFIED** (code_view.rs:277; ONE prod caller, app.rs:10534 `sync_editor_scroll`) |
| 3 | Row render `.whitespace_nowrap()` + the inspect-F3 comment | **VERIFIED** (app.rs:4469; comment verbatim) |
| 4 | 3 wheel handlers; one is the editor's; dx unread | **F1 — WRONG. No editor handler exists.** |
| 5 | `scroll_editor_to_row` is the shared vertical mechanism | **VERIFIED** (app.rs:10418; callers 6146/9481/10434/10512) |
| 6 | Per-file scroll memory (#273) | **VERIFIED** — but the store is NOT where claim 1 implies (F3) |
| 7 | The #198 `scrollbar_thumb` math is reusable | **VERIFIED — genuinely axis-agnostic** (F4 caveats) |
| 8 | Row structure `[gutter \| code region]` | **VERIFIED — and better than assumed (a gift)** |
| 9 | D-SHIFT: riders ride for free | **F2 — TRUE for caret/bands/squiggles, FALSE for the sticky header** |
| 10 | Click→offset is ONE place needing `+ scroll_x` | **F5 — it is 4 places, and there is a double-count hazard** |

**F1 [BLOCKING — REQ-007 had no seam to extend].** The spec's *"the editor's `ScrollWheelEvent` handler gains
the x axis"* rests on a handler **that does not exist**. `on_scroll_wheel` has exactly 3 hits and none is the
editor's: app.rs:12209 = the left rail (`rail_scroll`), :12531 = the Files tree (`files_scroll`), :13153 = the
terminal pane (`scroll_remainder`). The editor scrolls via gpui's own
`uniform_list(...).track_scroll(self.editor_scroll)` (app.rs:4696), and **app.rs:4274 says so outright**: *"the
old editor wheel handler is gone"* — deleted in #266. dx IS confirmed unread (zero `delta.x`/`scroll_x` hits
repo-wide; all three read `.pixel_delta(row_h).y`). → Design must **CREATE** an editor wheel handler and
answer a real feasibility question the spec never posed: **can an `on_scroll_wheel` on the list (or a parent)
observe dx without fighting `uniform_list`'s internal dy consumption, or is the event consumed before it
bubbles?** That is a design fork, not a delta note. Spec corrected; REQ-007 reworded to name the feasibility
gate + its fallback.

**F2 [REQ-006's "one container" was false].** The riders genuinely ride: `code_row` is `.relative()`
(app.rs:4466); the caret bar (:4519-4528, `.left(px(ccol * cell.w))`) and squiggle (:4550-4558) are
`.absolute()` children in CONTENT coords; and the selection band / #272 find bands / #331 hint spans are **not
elements at all** — they are `HighlightStyle` background colors ON the text (`StyledText::with_highlights`,
:4442-4472), so they ride trivially. **But the #330 sticky header does NOT ride:** `sticky_header_overlay()`
(app.rs:4741) attaches to `body`, not the list (:4702-4704), and builds an independent tree with its own git
spacer/gutter/`StyledText` (:4807-4838) — no `code_row`, no shared container. So REQ-006's *"verified
structurally: they are children of the ONE container"* is FALSE for it: it is a **second translate site**,
exactly the "N rider fixes" D-SHIFT claims to avoid. D-SHIFT survives (2 sites, not N), but the claim was
overstated and is now corrected.

**F3 [the D-MEMORY target was mis-cited].** `cv.scroll` is **not the editor's scroll anymore** — app.rs:388
records that it now serves ONLY the #246 read-only split pane. The real store is **`OpenFile.scroll_px: f32`**
(editor_surface.rs:53) — a raw PIXEL offset, ≤ 0, session-only (never persisted). Choke point
`sync_editor_scroll()` (app.rs:10525, called once from :12628); park :10547; restore :10564 —
`set_offset(point(px(0.0), px(incoming_px)))`, **with a hard-coded `px(0.0)` x that is literally the slot
`scroll_x` restore drops into.** D-MEMORY's intent was right; its citation pointed at the wrong struct.

**F4 [the thumb is reusable, with two caveats].** `viewport.rs:98
scrollbar_thumb(content, capacity, start) -> Option<(f32, f32)>` is pure unitless ratio math over three
`usize`s — nothing vertical in the body; a horizontal thumb reads the pair as (left_fraction, width_fraction)
with the args in COLUMNS. But (a) it takes `usize` while D-CLAMP is specified in **px f32** → a px→cols
conversion at the call boundary; and (b) its **sole caller is the TERMINAL pane render** (app.rs:13676), so
the "#198 drag plumbing transfers cheaply" hope has **no editor-side precedent** to copy. Design decides
drag-v1 with that cost visible.

**F5 [the click hazard — the most valuable catch].** Not one site but **four**, none calling `offset_of_col_f`
directly (all bottom out through `offset_for_click`, code_view.rs:225): mouse_down (app.rs:4606-4607),
mouse_move/drag (:4656-4658), the IME `character_index_for_point` (:11010-11011), and the #311 hover-dwell
(:9513, already col-domain via `last_hover_cell` — its producer needs tracing). **The hazard:** all subtract
`x0`, and `x0` is written by a canvas probe at **app.rs:4496-4500 from `bounds.origin.x` — and that canvas is
an `.absolute()` child of `code_row`, the very element D-SHIFT translates.** If gpui's translate moves the
canvas's bounds, `x0` already carries `−scroll_x`, `rel` is already content-domain, and REQ-003's
`rel_x + scroll_x` **double-counts**; if the translate is paint-only, `+ scroll_x` is correct. There are
**zero** uses of `.translate`/`TransformationMatrix` in the repo — no precedent settles it. Design must
resolve this empirically (a probe), and the likely fix is to hoist the x0 canvas OUT of `code_row` onto the
row div so `x0` stays screen-domain while the text/caret/squiggles shift. **This is the #331 lesson again, one
layer down: a coordinate change is only safe when you know which domain each reader is in.**

**F6 [a gift — claim 8 is stronger than assumed].** `code_row` (app.rs:4601, built :4465-4473) **already wraps
ONLY the code text**, is **already `.relative()`**, and already sits as a sibling AFTER the git lane
(:4578-4590, `w(px(3.0))`) and the gutter (:4591-4600). D-SHIFT does not need to introduce a container — it
adds a translate + `overflow_hidden` to one that exists. The gutter/git-lane never-shift requirement
(REQ-005) is satisfied by the existing structure for free.

### Decisions (Phase 1 — the pre-authored decisions SURVIVE; citations corrected)
D-SHIFT / D-WIDTH / D-CLAMP / D-ZERO-IDENTICAL / D-MEMORY all stand — no finding invalidates the *approach*.
What the verification changed is the spec's factual claims (F1 REQ-007's seam, F2 REQ-006's "one" container,
F3 D-MEMORY's target, F4 the thumb's units/precedent, F5 the click hazard's size). Two items are now explicit
**Design forks** rather than assumed-solved: the wheel-dx feasibility (F1) and the x0 domain (F5).

**LIVE drives are OFF-LIMITS this batch** — chad is back at the machine (verified: two captures 32 min apart
showed the frontmost app change into a live Codex session), and synthetic input lands on the FRONTMOST window.
Plan for units + headless + mechanism; a pixel drive is deferred, not skipped, and the notes will say so.

**Method verdict (first run):** the promote-don't-author method **paid for itself immediately.** It cost one
Explore pass and caught a REQ resting on a deleted seam (F1) plus a double-count hazard (F5) — both of which
would have surfaced at Implement or, worse, at Inspect. The lesson to carry: **a grep hit is not a seam.** I
counted 3 `ScrollWheelEvent` handlers and assumed one was the editor's; the verifier READ them and found none
is. That is the inverse of `PR-claude-grep-for-absence-must-prove-the-command-ran` — a grep for PRESENCE must
prove the hit is the RIGHT one.

status: Phase 1 — Plan PASS; ready for Phase 2 — Design

## Phase 2 — Design

Both Phase 1 forks are **settled by reading gpui-0.2.2's source** (a permissive Apache-2.0 dependency — reading
it is adoption, not the §20 copyleft wall). The investigation changed the design twice, so the reasoning is
recorded in full rather than just the verdict.

### FORK 1 (D-X0) — SETTLED: **the click math needs NO change.** REQ-003 becomes a PIN, not an edit.
`window.rs:3296` in `layout_bounds()`: `bounds.origin += self.element_offset();` — and a `canvas` is handed
its bounds straight through (`canvas.rs:71`) from the generic `Drawable::prepaint` (`element.rs:418`), which
calls `layout_bounds`. A scrolling ancestor pushes its offset first (`div.rs:1413
with_element_offset(scroll_offset, …)`). So Marley's x0 probe (app.rs:4493-4511, a canvas child of `code_row`)
reports an origin **already displaced by scroll_x**. Since `col = (click.x − x0) / cell.w` subtracts a
window-space `x0` from a window-space `click.x`, **the scroll term cancels — the formula yields content
columns for free.** The spec's `rel_x + scroll_x` would have DOUBLE-COUNTED.
**The symmetry proof:** the row (y) never goes through pixel math — `uniform_list` hands the row index to the
closure — which is exactly why no click site adds `scroll_y` today. Horizontal now matches vertical.
→ **All 4 click sites (app.rs:4606 / 4656 / 11010 / 9513) are untouched**, and REQ-003 ships as a NAMED
invariant + a pinning test, so a future refactor that hoists the probe out of `code_row` fails loudly.

### FORK 2 (the container) — the investigation killed my own proposal and found a third door.
**Option (b) — per-row `overflow_x_scroll` + a shared handle + `min_w` — is BROKEN**, three ways:
1. **The width-equalizer doesn't exist as I specified it.** `content_size` is the union of **children's**
   bounds and nothing else (`div.rs:1371-1394`). `min_w` on the CONTAINER feeds `bounds.size` — the
   **subtrahend** in `scroll_max = padded_content_size − bounds.size` (`div.rs:1747`) — so it makes scroll_max
   *smaller*, not equal. Right instinct, wrong knob, pointed the wrong way.
2. **Unequal content_size self-destructs a shared handle.** The clamp (`div.rs:1754`) runs once per row,
   cumulatively, on the shared `Rc<RefCell<Point<Pixels>>>` (`div.rs:1602`). A **blank line** has no children
   → `content_size = bounds.size` (`div.rs:1372`) → `scroll_max.width = 0` → **h-scroll pinned at 0 forever**,
   plus intra-frame shear (rows above/below the short row render at different x). Not "last row wins" —
   **shortest row wins.** Every code file has blank lines.
3. **A vertical wheel would scroll rows sideways.** With only `overflow.x == Scroll`, `div.rs:2428` sets
   `delta_x = delta.y` (because `restrict_scroll_to_axis` defaults false, `style.rs:741`, and `overflow.y !=
   Scroll`). gpui's own doc (`style.rs:165-175`) names this exact scenario — "a vertical list that contains
   horizontally-scrollable elements … the scroll will be hijacked" — and **no builder sets the flag**.

**The third door — `uniform_list.with_horizontal_sizing_behavior(Unconstrained)` (`uniform_list.rs:628-641`)
— is real and elegant, and I am NOT taking it.** It sets `overflow.x = Scroll` (:638), computes
`content_width = max(viewport, longest_item)` (:338-347), already applies `scroll_offset.x` to `item_origin`
(:470-478), gives correct wheel semantics free (both axes Scroll ⇒ the :2428 hijack branch is disabled), and
`UniformListScrollHandle` already wraps a full 2-axis `ScrollHandle` Marley reaches through today
(app.rs:10443/10540/10564). **But it shifts the WHOLE ITEM — the row, gutter included — which violates
REQ-005.** Honoring REQ-005 under it would mean lifting the gutter + git lane OUT of the row into a separate
fixed column synced to the list's y — a restructure of the exact render path that #328 (git lane), #310
(squiggles), #330 (sticky header) and #331 (inlay hints) all ride. That is not a defect fix; that is a new
ticket.

### DECISION — **option (c): Marley-owned `scroll_x`, a manual per-row shift of `code_row` only.**
- **REQ-005 is free** — F6's gift: `code_row` (app.rs:4601) already wraps ONLY the code text, already
  `.relative()`, already a sibling AFTER the git lane (:4578) and gutter (:4591). The shift lands on it; the
  gutter cannot move because it is not inside the shifted element.
- **REQ-003 is free** — a `.ml(px(-scroll_x))` is a LAYOUT shift, so bounds still move and Fork 1's
  cancellation holds identically.
- **The pure seam survives and is wanted anyway** — `follow_caret_x` has no substrate equivalent
  (`ScrollHandle::scroll_to_item`'s X branch, `div.rs:3186-3193`, reads `child_bounds`, which `uniform_list`
  never populates — it is dead for our case). `clamp_scroll_x` + `content_cols` stay Marley's, at cov/MSI 100.
- **Smallest blast radius** on a render path four shipped features ride.
**Recorded as a follow-up, not lost:** if the gutter ever leaves the row, switch to `Unconstrained` and DELETE
the hand-rolled clamp + wheel + content-width — the substrate does all three. That is the roadmap-B2 trade
(adopt gpui, delete hand-rolled code) deferred on REQ-005 grounds, with the exact reason on the record.

### The remaining open question (P3 answers it FIRST, cheaply)
**The wheel.** The editor has no handler (deleted in #266, app.rs:4274). Plan: an `on_scroll_wheel` on the
editor body reading `delta.x`. gpui's div handler does **not** `stop_propagation` (`div.rs:2417-2450`), so a
body-level handler should observe the same event the list handles for dy — they touch different state and
should not conflict. **If dx proves unobservable, REQ-007's spec'd fallback applies: ship wheel-less, record
it, and let follow + thumb-drag carry REQ-001. Do not fight the list.**

### §20 — CONFIRMED, unchanged
VS Code = OBSERVED behavior (clip-not-wrap default; an h-scrollbar; caret-follow margins). No copyleft source
read. **gpui (Apache-2.0) source WAS read — that is adoption of a permissive dependency, explicitly outside
the §20 wall** (the wall is Warp AGPL / Zed GPL). The mechanism is Marley's own layout math over the shipped
`LineLayout` column domain, which `AD-claude-editor-offset-column-model-001` and
`AD-claude-two-boundary-maps-for-phantom-text-001` govern — and Fork 1's finding means this ticket does not
fork that domain at all.

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/h_scroll.rs` | **NEW** pure seam: `clamp_scroll_x(x, content_px, viewport_px)`, `follow_caret_x(scroll_x, caret_col, viewport_px, cell_w) -> f32` (margin ≈ 4 cols), `content_px(widths, cell_w) -> f32` (max over visible ∪ caret row). cov/MSI 100. `git add -N` before the gate. |
| `crates/marley_app/src/app.rs` | `scroll_x: f32` view state + the per-row `code_row` shift (`.ml(px(-scroll_x))` + `overflow_hidden`); the #330 sticky-header second shift site (:4741/:4807-4838); the body `on_scroll_wheel` dx handler; caret-follow call beside `follow_editor_caret` (:10434); the h-thumb (px→cols for viewport.rs:98); park/restore beside `sync_editor_scroll` (:10525/:10564). All shims `#[cfg_attr(test, mutants::skip)]`. |
| `crates/marley_app/src/editor_surface.rs` | `OpenFile.scroll_x: f32` beside `scroll_px: f32` (:53) — session-only, matching the existing posture. |
| `crates/marley_app/src/headless_drive.rs` | The REQ drives (below). |

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| 001 | `clamp_scroll_x` truth table — 0 at rest; max = content−viewport+slack; a tail-revealing scroll; narrower-than-viewport pins 0 | pure (cov/MSI 100) |
| 002 | `follow_caret_x` table — caret left of margin → minimal left adjust; right of margin → minimal right; inside → UNCHANGED (the identity row kills the always-adjust mutants) | pure |
| 003 | **PIN (no code change):** `click_at_scroll_lands_on_char_under_pointer_headless` — set scroll_x>0, click, assert the caret char. NAMES the invariant: `x0` is probed INSIDE the shifted element, so `rel` is content-domain; adding scroll_x would double-count | headless |
| 004 | `content_px` — max over visible ∪ caret row; the caret row is included even when off-screen (else follow could clamp away from its own caret) | pure |
| 005 | Gutter/git-lane never shift — structural: the shift is on `code_row`, a SIBLING of both (review + a headless assert that gutter x is scroll-invariant) | review + headless |
| 006 | Caret bar / squiggle / selection+find+hint highlights ride (they are `code_row` children or highlights ON the text) **AND** the sticky header shifts by the same scroll_x (the second site) | headless |
| 007 | Wheel dx → scroll_x, dy unchanged — **or** the recorded fallback if dx is unobservable | headless |
| 008 | The h-thumb via `scrollbar_thumb` (viewport.rs:98) — hidden when content fits; the px→cols conversion pinned | pure (reuse) |
| 009 | `scroll_x == 0` renders byte-identically to the pre-ticket path | property/headless |
| 010 | scroll_x restores with the per-file scroll memory on file switch | headless |
| — | **LIVE pixel drive: OFF-LIMITS** — chad is at the machine and synthetic input lands on the FRONTMOST window (his live Codex session). Deferred, not skipped; units + headless + mechanism carry it, and Phase 4 will say so explicitly. | uncoverable-live |

### Risks
1. **The wheel (open).** Mitigated by the spec's own fallback. Answer it FIRST in P3 — it is the only unknown.
2. **The sticky header is a second shift site** — a forgotten shift there = a header that detaches on scroll.
   REQ-006 covers it; inspect gets it as an explicit lens.
3. **`content_px` per frame over visible rows** — an O(visible) scan, not O(file). The caret row must be in
   the max or follow can clamp away from its own caret (REQ-004's second row pins it).
4. **The `Unconstrained` road not taken** — recorded above so a future gutter restructure knows to collapse
   this ticket's hand-rolled clamp/wheel/width into the substrate.

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement (IN PROGRESS — NOT PASS; see "Remaining" below)

`cargo check --workspace` clean, zero warnings, `cargo fmt --all` applied, `git add -N
crates/marley_app/src/h_scroll.rs` done (the #329 trap: a new untracked file is invisible to the
`--diff` mutation gate).

### Built
- **`crates/marley_app/src/h_scroll.rs` (NEW, pure, no skips)** — `clamp_scroll_x`, `follow_caret_x`,
  `content_px`, `caret_px`, plus a private `sane()` (a non-finite → fallback guard: gpui hands out measured
  px, and one NaN from a degenerate layout would poison `scroll_x` permanently, since every later comparison
  against NaN is false and the clamp would silently stop clamping). `SLACK_CELLS = 2`, `MARGIN_CELLS = 4`,
  both named consts with the reasoning. The module doc states the sign convention up front: **`scroll_x` is
  POSITIVE-RIGHT**, deliberately opposite to gpui's own offsets (which go negative as content scrolls) —
  this module never touches a gpui handle.
- **`editor_surface.rs`** — `OpenFile.scroll_x: f32` + `active_scroll_x()` + `park_scroll_x()`, mirroring
  `scroll_px`'s posture exactly (session-only). The field doc names the sign asymmetry with its neighbour so
  the next reader does not "fix" it.
- **`app.rs`** — `scroll_x: f32` view state; `editor_content_w: Cell<f32>`; `EditorFrameGeom.code_w`;
  `h_scroll_clamp` + `follow_editor_caret_x` (shims, `mutants::skip`) + 3 test hooks; the per-frame
  content-width computation (O(visible), caret row always included); the wheel; the shift + clipper.

### Deviations from design (all forced by live code, none discretionary)
1. **The shift needed a CLIPPER — the design's "add a margin to `code_row`" would have broken REQ-005.**
   `code_row` is a flex SIBLING of the gutter and sizes to its own text (`whitespace_nowrap` → unbounded
   natural width), so a negative margin on it pulls the text LEFT ACROSS THE GUTTER. Actual shape:
   `row → [git lane][gutter][clipper(flex_1, min_w_0, overflow_hidden) → code_row(flex_shrink_0,
   ml(-scroll_x))]`. The clipper is the gutter's sibling and never moves; the shift lives one level in.
   `flex_shrink_0` is load-bearing — without it flex squeezes `code_row` back to the clipper's width and
   there is nothing to scroll. **Phase 1's F6 "the container already exists" was half right:** `code_row` is
   the right element to SHIFT, but it could not also be the element that CLIPS.
2. **`EditorFrameGeom` gained `code_w`, from a SECOND probe.** The clamp needs a viewport width and the
   existing x0 canvas is `w(0).h(0)` — it reports an origin, not a size — and it rides the shift by design.
   So a `w_full().h_full()` canvas on the CLIPPER (stable, screen-domain) reports `bounds.size.width`. Two
   probes with deliberately different domains, each documented at its site. The x0 geom write preserves
   `code_w` (`code_w: x0c.get().code_w`) so the two writers do not clobber each other.
3. **`editor_content_w` is a view `Cell`, not a geom field.** The wheel and follow run OUTSIDE the render and
   have no layouts to measure, so the render parks the frame's width for them.

### THE NAMED INVARIANT (the ticket's real payload) — shipped as a comment + a pinning test at P4
At the `on_mouse_down` site, in full: `x0` is probed INSIDE `code_row`, the element the scroll shifts, and
gpui bakes an ancestor's scroll into a child's reported bounds (`Window::layout_bounds` does
`bounds.origin += element_offset()`), so `x0` already carries −scroll_x, both terms are window-space, and
**the scroll CANCELS — `rel` is content-domain for free.** Adding `+ scroll_x` would double-count. The
comment says "do not fix this by adding + scroll_x" and names the guard test.

### Completed after the first pass (the 5 remaining items)
4. **The sticky header's second shift site (F2)** — same shape as a code row (stable clipper wrapping the
   shifted, `flex_shrink_0` text), so its gutter stays put while its code tracks the scroll. Without it a
   scrolled view leaves the pinned header at column 0 while the code beneath it has moved.
5. **`scroll_x` became a `Cell<f32>`, and the follow rides the VERTICAL one.** `follow_editor_caret(&self)`
   now calls `follow_editor_caret_x()` — so all FIVE existing call sites (app.rs:6258/6371/9575/11764/12138)
   get the horizontal follow for free. The Cell is what made that possible: the vertical twin is `&self`
   (gpui's handle is Rc-backed), and forcing `&mut` to hold a plain f32 would have meant touching five sites
   to gain nothing. **The deeper reason to co-locate them:** a caret move that needs a row scroll needs a
   column scroll for the same reason, and the two silently diverging is exactly this ticket's bug class.
6. **Memory park/restore — both axes, one switch.** `sync_editor_scroll` now parks `scroll_x` beside the
   vertical offset and restores it. **The incoming x is deliberately NOT re-clamped there:** the sync runs
   before the frame that measures the new file, so `code_w`/content still describe the OUTGOING file and a
   clamp against them could discard a valid value. The wheel and follow both clamp against live geometry, so
   a too-wide restored scroll self-corrects on first interaction rather than being lost on switch.
7. **The wheel** — written; verification is Phase 4's headless drive (the principle holds: gpui's div scroll
   listener never calls `stop_propagation`, div.rs:2417-2450, so the same event reaches both handlers and
   they touch disjoint state). If the drive shows dx never arrives, REQ-007's spec'd fallback stands.

### DEVIATION — REQ-008 (the h-thumb) is DEFERRED, per the spec's own condition
The spec made drag-v1 conditional: *"only if that cost is small once seen."* Seen, and it is not small. The
thumb must sit over the CODE column, but an `.absolute()` child of the `relative()` body is positioned
**body-relative**, while the only x this frame publishes (`geom.x0`) is a **window** coordinate that
deliberately rides the scroll. Aligning it needs a third, body-relative probe of the code column's origin —
new geometry, not a call-site tweak. `scrollbar_thumb` is confirmed axis-agnostic and ready; only the
placement is missing. The wheel + caret follow (the affordances the spec called load-bearing) carry REQ-001
without it. Recorded at the site + as a follow-up; **not** shipped mis-aligned, and not bolted on
under-designed at the end of a phase.

**Verification:** `cargo check --workspace` clean, ZERO warnings; `cargo fmt --all`; `cargo nextest run -p
marley --lib` → **599 passed, 0 failed** (no regression from the render restructure — the clipper sits
between the gutter and the text without disturbing either).

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect

2 parallel critics (coordinate math; state + render integration). Both verified empirically — sweeps over
hostile input, a real gpui layout built in a scratch worktree, measured frame costs. Both left the tree clean.

**The headline: BOTH critics independently found the same CRITICAL — I forked the column domain.** One
reached it from the math, the other from the render. That is the #331 lesson recurring one layer down, and it
is worth stating plainly: `AD-claude-editor-offset-column-model-001` says `code_view` owns the column domain,
I wrote `.display.chars().count()` in a shim, and **naming an invariant did not stop me from violating it.**

| # | Sev | Finding | Verdict | Fix |
|---|-----|---------|---------|-----|
| F1 | **CRITICAL** | **Width measured in CHARS, caret in CELLS — a different DOMAIN.** `content_px` was fed `chars().count()` while `caret_px` was fed `col_of_offset` (real cells). Measured: `cjk_pure` 9 vs 18 · `cjk` 21 vs 31 · `emoji` 14 vs 17 · `combining` 6 vs 3 (over-counts). ASCII and tabs agree EXACTLY — which is why it survives casual testing and then strands users. Critic 1's end-to-end trace: 100 CJK chars = 200 cells → `content_px` says 800px, `caret_px` says 1600px → clamp caps scroll_x at 416 → **caret stranded 784px off-screen, permanently** (every later follow recomputes the same wrong max). **This is THIS TICKET'S OWN DEFECT, reintroduced for CJK** — and it defeated `content_px`'s own doc, which claims the caret row is always included so follow can't fight clamp. Second half: the width block measured `line_layout` (no inlays) while the row renders `line_layout_with_inlays` — 14 vs 19 cols with one hint, so a trailing #331 hint was unscrollable. | **REAL ×2** (independent) | `LineLayout::display_cols()` (walks `display` via `char_width`), doc'd "use this, NEVER `chars().count()`" with the ASCII-coincidence trap named; deliberately NOT `col_starts.last()` (that stops before an EOL phantom). Width block now uses the SAME inlay-aware layout the render draws (`einlay` hoisted above it). Critic 1 verified the fix: `display_cols() ≥ caret_px`'s column **by construction**, so content always covers the caret. Also killed the LOW-4 double-build. |
| F2 | **HIGH** | **Three jumps bypass the horizontal follow — including the flagship cases.** I hooked the h-follow into `follow_editor_caret`, but ⌘D (:6327), the **#312 go-to-definition landing** (`consume_pending_center`, :9662) and the **#272 find-next jump** (:10772) call `scroll_editor_to_row` DIRECTLY. Jump to a definition on a long line: the row arrives, the column does not, the target stays off-screen right. My own comment said "one of the two silently not running is precisely the bug this ticket exists to fix" — **it named the invariant and hooked the wrong primitive.** | **REAL** | Moved the h-follow onto **`scroll_editor_to_row`** — the shared primitive — so a vertical scroll can never happen without its horizontal twin, and every future caller inherits it. Removed the now-duplicate call from `follow_editor_caret`. Recorded imperfection: the follow reads the PRIMARY caret, so ⌘D (whose vertical targets the ADDED cursor) follows the primary's column; the identity case usually makes it a no-op, and a column-aware `scroll_editor_to(row, col)` is the follow-up. |
| F3 | **HIGH** | **The wheel was on the SHARED body — the #246 read-only pane silently scrolled the editor tab.** The handler sat before the `if let Some(ed) = editor` split, so the `editor: None` arm (app.rs:13996, the shipped split-right pane) got it too. That arm applies no scroll_x and writes no `editor_geom`, so a swipe there clamped against the EDITOR TAB's stale geometry, mutated its `scroll_x`, notified, and moved nothing. Then `sync_editor_scroll`'s `scroll_owner == Some(nonce)` early-return KEPT the corruption — the nonce never changed. | **REAL** | Attached inside the editable arm only, with the reason at the site. |
| F4 | **HIGH** | **Gutter clicks map to a phantom column at scroll_x > 0.** The mouse handlers sit on the ROW div, which spans the git lane + gutter. `.max(0.0)` pinned a gutter click to column 0 — correct *by accident*, because `x0` sat at the code column's left edge. Now `x0` slides left UNDER the gutter (by design — that is what makes the click scroll-free), so the gutter maps to POSITIVE columns. Measured in a real layout (50px gutter, cell_w 8): scroll_x 200 → a gutter click at x=10 resolved to **column 20**; scroll_x 640 → **column 75**. A column the user never pointed at. | **REAL** | `h_scroll::clamp_click_to_code(rel, scroll_x, code_w)` — `rel` is content-domain, so the visible band is exactly `[scroll_x, scroll_x + code_w]`; clamping into it maps a gutter click to the leftmost VISIBLE column (what "before the text" means when scrolled). Applied at all THREE sites (mouse_down, drag, IME `character_index_for_point`). **Re-homed into `h_scroll.rs`, not app.rs** — it is pure logic, and app.rs is coverage-EXCLUDED but mutation-INCLUDED, so a pure fn there can be mutated and never covered. app.rs #336 mutants: 6 → **0**. |
| F5 | LOW | The "do not fix this" invariant comment **cited the wrong gpui mechanism** — I credited `bounds.origin += element_offset()` (real, but that is gpui's SCROLL-offset stack, which a margin never touches). The actual carrier is taffy folding the margin into `Layout::location` + the parent-origin recursion. On a landmine marker this matters: a reader who checks the citation, finds it is about scroll offsets, and concludes the invariant is bogus would "fix" the very bug it prevents. | **REAL** (I caught this myself before the critic reported; both agree) | Re-attributed to taffy, with an explicit "do NOT credit this to `element_offset()`" and why. |
| F6 | LOW | `h_scroll.rs` doc overclaims totality (`caret_px(usize::MAX, -1e30)` = −inf; `sane()` guards non-finite INPUTS, not overflow from finite ones) and mis-describes the narrow-viewport case as pinning "near the left edge" when it measurably CENTERS. | **REAL, doc-only** | Deferred to Phase 4 with the tests that pin the behavior — the code is right, the prose is wrong. |
| F7 | LOW | The wheel scales the **x** delta by line **height** (`pixel_delta(px(cell_h))`); gpui applies one scalar to both axes, so a discrete mouse's horizontal tilt moves ~1.5× too fast. Trackpads send `Pixels` and bypass it. | **REAL, cosmetic** | Follow-up. |

**Rejected as unfounded (recorded — several are load-bearing and cost real effort to establish):** the predicted
`line_text` panic does NOT exist (buffer.rs:137 guards `row >= len_lines` → `""`); `follow_caret_x`'s bounds can
NEVER cross (`margin = (4*cell_w).min(view/2)` ⟹ `left ≤ right` always; swept every (view, cell_w) — the
degenerate case meets and both branches agree on `caret − view/2`, so it is continuous and centers); identity +
minimal-adjust hold exactly (caret 1px past the margin → scroll exactly 1.0, and re-running is a fixed point —
no oscillation); the ASCII clamp/follow round trip is sound at every viewport 16→1200px; **the sign convention
is right** (gpui's own div does `scroll_offset.x += delta_x` clamped to `[-scroll_max, 0]` — its x is the exact
negation of Marley's, and macOS `scrollingDeltaX` is negative on swipe-left → scroll_x grows → reveals the
tail); **the click invariant holds and a `.ml()` margin DOES move a child's reported bounds** (critic 1 built
the exact structure in a real gpui layout: `x0` = 50 / −150 / −590 at scroll_x 0 / 200 / 640, and col 25 → col
25 at every scroll); **NO layout regression — it is a strict IMPROVEMENT** (the row is `w_full` so the
clipper's `flex_1` absorbs the remainder and free space stays positive; the OLD structure was worse — a long
nowrap `code_row` floored at min-content put shrink pressure on the 3px git lane and gutter — and `min_w_0` is
genuinely load-bearing); `overflow_hidden` verified to clip via `with_content_mask`, and masks are paint-time
scissors so the x0 canvas still records bounds + registers IME when shifted off-mask; no right-edge caret
clipping (MARGIN 4 + SLACK 2 keep it ≥2 cells inside); `code_w` staleness is 1-frame and self-correcting
(first-frame 0 correctly pins 0); the lifecycle is correct (the sync is RENDER-driven, so no switch bypasses
it; gating `park_scroll_x` on `park_scroll` is right — a closed file's memory dies with it); `carets.first()`
vs `active_caret()` is NOT a mismatch (`primary()` IS `selections[0]`); both pure fns are total over 6561
hostile 4-tuples each; the **detach trap** was checked against a baseline worktree at HEAD — 66 vs 66,
line-stripped sets byte-identical.

**Verification after fixes:** `cargo check --workspace` clean, ZERO warnings; `cargo nextest run -p marley
--lib` → **599 passed**; `cargo fmt --all`; §20 grep EMPTY; **app.rs #336 mutants = 0** (every shim skipped),
h_scroll.rs = 56 (the tested surface).

**Validate owes (carried forward — critic 1 did the mutation analysis so P4 doesn't have to guess):**
- **F1 CANNOT be pinned by a unit.** `content_px` is *correct* — it takes `usize` widths and cannot tell they
  were miscounted. The whole bug lived in the coverage-excluded, `mutants::skip`'d shim, so h_scroll.rs units
  pass 100/100 either way. **It needs a headless RENDER-level test** (a CJK line + a line with a trailing hint).
  This is the ticket's most important test and the pure seam is blind to it.
- **Two effectively-equivalent mutants** (`<`→`<=` at the left bound, `>`→`>=` at the right): swept **640,000
  on-boundary inputs** → ZERO kills, because at `caret == left_bound`, `(cur+margin)−margin == cur` — identical
  to the identity branch. Killable ONLY by a deliberate f32-rounding case: `follow_caret_x(1e-7, 40.0, 1000.0,
  10.0)` → base `1e-7`, mutant `0`. Needs a comment explaining why such a test exists.
- **Three mutants need a `scroll_x > 0` test** (`+`→`*` ×2, `>`→`<`): every `cur == 0` input leaves them alive.
  `follow_caret_x(100.0, 2000.0, 1000.0, 10.0)` (base 1040 vs mutant 100) kills them. A suite that only tests
  from rest is insufficient.
- F6's doc corrections; the REQ-003 click pin; the REQ-009 zero-identity property.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

**15 tests added** (10 h_scroll pure · 2 code_view · 1 editor_surface · 3 headless). Runs: `cargo nextest run
--workspace` → **1474 passed, 5 skipped**; `cargo test --workspace --doc` → 0 tests (none in scope);
`scripts/gates.sh --diff` → **GATE GREEN [diff], 15/15** — mutation **65 caught / 0 missed**, coverage 100%.

| REQ | Test | Result |
|---|---|---|
| 004 | `t336_clamp_scroll_x_bounds` — max 216 (neither 0 nor 1, killing the body mutants); fits/exactly-fits pin 0; 1px overflow earns the slack; negative → 0 | PASS |
| 002 | `t336_follow_caret_x_identity_inside_the_band` — 5 carets, all unchanged (the contract) | PASS |
| 002 | `t336_follow_caret_x_minimal_adjust_both_sides` — 1px past → exactly 1.0, and re-running is a FIXED POINT (no oscillation); left side; past-left → 0 | PASS |
| 002 | `t336_follow_caret_x_from_a_scrolled_view` — the `scroll_x > 0` case inspect proved 3 mutants need | PASS |
| 002 | `t336_follow_caret_x_boundary_rounding_kills_the_equivalent_mutants` — BOTH bounds' f32-rounding cases | PASS |
| 002 | `t336_follow_caret_x_narrow_viewport_centers_and_pins_the_margin_divide` — the only shape exercising `.min(view/2)` | PASS |
| 004 | `t336_content_px_max_includes_the_caret_row` — incl. an OFF-SCREEN caret row; empty iterator | PASS |
| 004 | `t336_caret_px_maps_columns_to_pixels` | PASS |
| F4 | `t336_clamp_click_to_code_band` — gutter → leftmost visible; past-right → rightmost; inside unchanged; pre-first-frame | PASS |
| — | `t336_scroll_producers_are_total_over_hostile_input` — 6561 hostile 4-tuples × 2 fns | PASS |
| F1 | `t336_display_cols_measures_cells_not_chars` — ASCII coincides; **CJK 3 chars → 6 cells**; a combiner OVER-counts (3 → 2); tabs | PASS |
| F1 | `t336_display_cols_includes_a_trailing_phantom` — 15 cells vs `col_starts.last()` = 10 (why it is not that accessor) | PASS |
| 010 | `park_scroll_x_by_nonce_round_trips_and_misses_safely` — fresh/isolation/read-back/unknown-nonce miss + **the two axes are independent** | PASS |
| **F1** | **`hscroll_content_width_measures_cells_not_chars_headless`** — THE test inspect demanded (below) | PASS |
| 003 | `click_at_scroll_lands_on_char_under_pointer_headless` — the named invariant's guard | PASS |
| 002/F2 | `jump_runs_the_horizontal_follow_headless` — pins that a jump brings the COLUMN, not just the row | PASS |

**The tests found two real bugs the critics had not — which is the point of writing them.**
1. **The slack made an exactly-fitting line scrollable.** `max_scroll_x` added the slack BEFORE testing for
   overflow (`content − viewport + slack`), so a line that exactly fills the viewport reported a 16px scroll
   range — into blank space, revealing nothing, with the thumb offering it. Fixed by testing overflow first:
   the slack gives the last glyph air once you are ALREADY scrolling; it is not a reason to start.
2. **A negative `cell_w` PANICKED.** It makes the slack negative → `max_scroll_x` negative → `f32::clamp`
   panics on `min > max`. `sane()` does not catch it — a negative width is perfectly finite. Unreachable from
   a real layout, but the fn advertised totality and did not have it. Floored `cell_w` at 0. **The totality
   sweep is what caught it**; every hand-picked row passed.

**The F1 render test is the ticket's most important, and it needed TWO frames.** The width block reads the
PREVIOUS frame's visible range (the geom is written by a canvas during the row closure), so frame 1 measures
the default `0..=0` and only frame 2 sees the CJK row — the 1-frame staleness the design accepted as
self-correcting, now documented in the test rather than papered over. It asserts ~54 cells where a char count
would report ~34. Inspect proved the pure seam is BLIND here (`content_px` takes `usize` widths and cannot
tell they were miscounted), so without this test F1 regresses silently at 100% coverage AND 100% mutation.

**Three gate reds, all fixed at source:**
1. **gate:14 docs** — a PUBLIC fn's rustdoc intra-doc-linked a PRIVATE item (`scroll_editor_to_row` →
   `Self::follow_editor_caret`). Plain text instead of a link.
2. **gate:4 coverage + gate:5 mutation (8 survivors)** — ONE root: `editor_surface`'s new `active_scroll_x` /
   `park_scroll_x` had no tests while their vertical twins did. Added the mirror table.
3. **gate:5 again (2 survivors)** — `/`→`*` in `.min(view / 2.0)` (every ordinary viewport picks the same
   margin either way — only a NARROW one separates them) and `>`→`>=` at the RIGHT bound (inspect handed me
   the rounding trick for the LEFT bound; the right needed its own mirror). Both now killed deliberately, with
   comments saying why the numbers look arbitrary — a future "simplification" would silently drop MSI.

**LIVE pixel drive: NOT run — deferred, not skipped.** chad is at the machine (verified earlier: two captures
32 min apart showed the frontmost app change into a live Codex session), and synthetic input lands on the
FRONTMOST window, so a drive would type into his session. What pixels would add over the 3 headless drives is
narrow: that the shifted text visually clips at the clipper's edge and the gutter visibly holds still. The
math, the click invariant, the follow and the width measure are all pinned headlessly. ~2 minutes on an idle
box.

**Pre-existing, not in scope:** the #334 load-flaky mutation baseline (it did not fire this run; the box was at
load ~4). `block v0.1.6`'s future-incompat warning (an upstream dep).

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete

**Docs (§21):** CHANGELOG entry under **Fixed** (this is a defect, not a feature) above the M21 #331 entry.
`editor.md` — a full h-scroll section (the defect + why it is not wrap; the clipper/shift split and why
`code_row` could not be both; the click invariant **attributed to taffy, not `element_offset`**; the
two-domain probe pair; the follow on the shared primitive; the column-domain rule; the deferred thumb + the
`Unconstrained` road not taken), plus the "Still missing" paragraph corrected — clipping is normal (VS Code's
default), what was broken was the unreachable tail. `crate-map.md` — `marley_app` gains `h_scroll.rs` + the
`display_cols`/`scroll_x` notes. `roadmap.md` — M22's B-a marks Horizontal scroll ✅ SHIPPED with its
follow-ups.

**Knowledge captured:** AAR `2c3e5411` submitted (completed, effectiveness 5, **4 novel findings, 13 verdicts
written** — the surfacing log had real recall to judge). 1 failure + 2 prevention rules + **1 ADR**:
`AD-claude-hscroll-shift-clip-split-and-two-probe-domains-001` — the three structural rules (shift ≠ clip;
the two probes live in OPPOSITE domains on purpose; scroll_x is positive-right and never reaches a gpui
handle). It is recorded as **binding on #337**, whose font-metrics work makes `cell_w` runtime-variable and
which both probes feed — one metrics seam, not a third source. It also records the `Unconstrained` road not
taken, so nobody re-derives why a first-class gpui feature went unused.

**Follow-ups filed:** **#341** the h-thumb (needs a third, body-relative probe — `scrollbar_thumb` is ready
and axis-agnostic; only placement is missing), **#342** a column-aware `scroll_editor_to(row, col)` (⌘D's
follow tracks the primary while its vertical tracks the added cursor — latent, hidden by the identity case),
**#343** the wheel's x delta scaled by line height (mouse tilt only; trackpads bypass it).

### The five lessons worth carrying
1. **The pre-authored-spec method paid for itself on run ONE.** Phase 1 spent its effort re-verifying the
   spec's 10 citations instead of authoring, and caught a REQ resting on a seam DELETED in #266 (the editor
   has no wheel handler — app.rs:4274 says so outright) plus a click double-count, both before Design saw
   them. `git diff a9eb589..HEAD` was docs-only, so **no code had moved — every drift was authorship error**.
   The lesson: **a grep hit is not a seam.** I counted 3 `ScrollWheelEvent` handlers and assumed one was the
   editor's; reading them showed none is. That is the inverse of
   `PR-claude-grep-for-absence-must-prove-the-command-ran-001` — a grep for PRESENCE must prove the hit is the
   RIGHT one.
2. **Reading a PERMISSIVE dependency's source is adoption, not a §20 breach.** gpui is Apache-2.0; the wall is
   Warp AGPL / Zed GPL. Reading it settled both design forks with evidence and **killed my own proposal**:
   option (b) (per-row scroll containers sharing a handle, equalized by `min_w`) is broken because
   `content_size` is children-only, so `min_w` on the container feeds the SUBTRAHEND — right instinct, wrong
   knob, pointed the wrong way. One blank line would have pinned h-scroll at 0 forever.
3. **Naming an invariant did not stop me violating it.** `AD-claude-editor-offset-column-model-001` already
   declared `code_view` the owner of the column domain, and I still wrote `.display.chars().count()` — in a
   coverage-excluded, `mutants::skip`'d shim where no pure test could see it. Both critics found it
   independently, from opposite ends. The generalized rule is now
   `PR-claude-a-width-in-chars-is-not-a-width-in-cells-001`, and its sharpest clause is: **a pure seam being
   green says nothing about whether its inputs are in the right units.**
4. **I hooked the right invariant to the wrong primitive.** The h-follow went on `follow_editor_caret` with a
   comment saying "one of the two silently not running is precisely the bug this ticket exists to fix" —
   while three gestures called the underlying `scroll_editor_to_row` directly and bypassed it, including
   go-to-def and find-next, the flagship cases. `PR-claude-hook-the-shared-primitive-not-one-of-its-callers-001`:
   grep the PRIMITIVE, not the wrapper; if any caller is not the wrapper you hooked, you hooked too high.
5. **The tests found two bugs the critics missed, and the SWEEP is why.** Slack-before-overflow made an
   exactly-fitting line scrollable into blank space; a negative `cell_w` made `f32::clamp` panic on
   `min > max` (`sane()` does not catch it — a negative width is finite). Every hand-picked row passed; the
   6561-tuple hostile sweep caught both. Also: the mutation gate forced two *deliberately* strange tests
   (an f32-rounding case at each bound, and a narrow-viewport case) that are the only things separating
   otherwise-equivalent mutants — both carry comments saying why, because a future "simplification" would
   silently drop MSI.

**Outstanding:** the LIVE pixel drive, deferred not skipped — chad is at the machine and synthetic input lands
on the frontmost window. What pixels would add over the 3 headless drives is narrow (that the shifted text
visibly clips at the clipper's edge and the gutter visibly holds still); the math, the click invariant, the
follow and the width measure are all pinned headlessly. ~2 minutes on an idle box.

status: Phase 5 — Complete PASS
