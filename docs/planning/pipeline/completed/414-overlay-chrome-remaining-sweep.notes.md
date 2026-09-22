# 414 — overlay chrome remaining sweep — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-414-overlay-chrome-remaining-sweep.md
- **Pipeline spec:** 414-overlay-chrome-remaining-sweep.spec.md

## Phase 1 — Plan
- **Request:** batch queue position 2 (after #415): convert the 10 remaining
  verbatim chrome sites + the non-modal variant. Auto-approved batch run.
- **Classification / tier:** chore (DRY refactor + one prescribed behavior
  fix); single work pipeline — the sites are mechanical conversions of a
  contiguous, token-identical core.
- **Recall (§18.3):**
  - AD-claude-318-overlay-chrome-two-recipes-001 — one modal chrome, two
    recipes; new consumers declare recipe+modality; the non-modal variant is
    pre-promised there and at the helper doc (app.rs:1042-1047).
  - PR-claude-block-mouse-except-scroll-for-nonmodal-scroll-overlays-001 —
    occlude blocks scroll; non-modal-over-scroll members take the variant.
  - L-claude-318-extraction-testing-posture-…-001 — computational extractions
    (geometry) take exact-value units; render-shape extractions take
    `mutants::skip` + capture-class proof (no new unkilled surface).
  - L-claude-318-driving-the-live-app-…-001 + fresh evidence at #415 validate
    (0×0 off-screen windows): live captures are blocked this session → D3
    posture; debt rides #417.
  - #318 notes F6/D3/D5: launcher/history/fleet zero-drive gap closed by the
    recipe-B smoke (now #415-tightened); hover keeps occlude (D3); anchored()
    not adopted (D5).
- **Discovery (Explore, very thorough):** all ten sites in app.rs; core
  token-identical + contiguous at every site; deviations prefix/suffix only.
  Map (chain ranges): completion 15199-15213 (caret-anchored, W=420, no max_h,
  `popup_origin` flip, mono, NON-MODAL over the editor's uniform_list; its
  scrolled-out-of-viewport guard is the scroll-dismiss); references 15505-15522
  (`base` closure used at 15534/15556; W=620/MAX_H=360, menu_origin 0.15);
  code_action 15620-15636 (560/320, 0.2); file_symbols 15685-15701 (560/380,
  0.12); symbols 15781-15797 (640/380, 0.12); search 15889-15905 (720/420,
  0.10); problems 16020-16036 (760/420, 0.10) — all six mono + header child;
  naming trio 20994-21008 / 21043-21057 / 21072-21086 byte-identical
  (`0.3·w / h/4 / 0.4·w` + `.p_3()` + text_color, no font override → 16px
  default). 9 true modals + 1 non-modal. Near-miss list recorded in spec Out
  (esp. the TERMINAL completion popup 21999 — name collision, deliberate
  occlude-blocks-wheel contract, different chain). Converted-7 idiom anchors:
  hover 15129, def_picker 15449 (no text_color — deliberate), palette 20949,
  launcher 21122, finder 21158, history 21187, fleet 21207.
- **Prior-art sweep:** gpui 0.2.2 fluent `block_mouse_except_scroll`
  (div.rs:995-1012) + `HitboxBehavior::BlockMouseExceptScroll`
  (window.rs:596-624, pass-through arm :783) — direct adoption, "should be
  preferred" per gpui's own doc. POC OverlayShell owns the seam React-side
  (consolidation already exists there; drifts are #416). anchored() still not
  adopted (#318 D5). No external behavior matched.
- **Decisions:** D1 variant sibling, D2 naming-trio EXTRACT, D3 no-live-session
  verification posture (chain-identity + negative grep + suite + #417 debt),
  D4 references closure survives. See spec.

## Phase 2 — Design

**Architecture.** All inside `marley_app` render layer — no seams, no IO, no
typed-error surface. §20 stays N/A (internal refactor; the one behavior delta
is prescribed by our own PR rule + gpui's own "should be preferred" doc).

- **D-VARIANT — sibling fn.** `overlay_card_chrome_over_scroll(colors)`:
  `div().block_mouse_except_scroll()` + the same 7-call tail, adjacent to
  `overlay_card_chrome`, docs cross-referencing both ways (modal vs
  over-scroll membership; PR-claude-block-mouse-except-scroll cited).
  `#[cfg_attr(test, mutants::skip)]` render-shape. Sibling over shared-base:
  zero churn to the shipped modal fn (7 converted sites untouched), preserved
  call order, simplest diff; the 7-tail existing twice is REQ-001's
  anticipated shape (both are helper BODIES; the negative grep counts exactly
  these two).
- **D-NAMING — extract with the shared suffix IN.** Pure
  `naming_card_geometry(win_w, win_h) -> (f32, f32, f32)` =
  `(w*0.3, h/4, w*0.4)` in context_menu.rs beside `overlay_quarter_geometry`,
  same doc idiom (containment by construction: `left+w = 0.7w ≤ w`;
  content-driven height, nothing to clamp). Assembled
  `naming_overlay_card(colors, win_w, win_h)` in app.rs beside
  `quarter_overlay_card` = `overlay_card_chrome(colors).absolute().left(px(ox))
  .top(px(oy)).w(px(ow)).p_3().text_color(colors.foreground)` — `.p_3()` +
  text_color move IN because all three sites carry them byte-identically;
  per-site `.child(...)` content stays out. `mutants::skip` render-shape.
- **Conversion form (the #318 helper-first reorder idiom, hover/def_picker
  precedent):** prefix positioning moves AFTER the helper call — builder
  calls set fields; order is style-equivalent (the #318-accepted equivalence;
  chain identity = same call multiset).

**File manifest.**
- `crates/marley_app/src/context_menu.rs` — add `naming_card_geometry` +
  `#[cfg(test)]` exact-value/containment tests (mirror
  `overlay_quarter_geometry_fractions_and_containment`, :436).
- `crates/marley_app/src/app.rs` —
  1. add `overlay_card_chrome_over_scroll` (beside :1048) + doc; refresh
     `overlay_card_chrome` doc (10-remaining → sweep complete at #414; the
     "follow-up must give" sentence becomes "the #414 variant is
     `overlay_card_chrome_over_scroll`; hover's occlude stays the recorded
     tension").
  2. add `naming_overlay_card` (beside `quarter_overlay_card` :1068) + doc.
  3. completion popup :15199 → `overlay_card_chrome_over_scroll(colors)
     .absolute().left(px(ax)).top(px(top)).w(px(W)).font_family(…)
     .text_size(…)` (the `let mut card =` binding stays).
  4. references :15505 → helper call inside the existing `base` closure
     (searching/results reuse preserved); code_action :15620; file_symbols
     :15685; symbols :15781; search :15889; problems :16020 — each →
     `overlay_card_chrome(colors).absolute().left(px(ox)).top(px(oy))
     .w(px(W)).max_h(px(MAX_H)).font_family(…).text_size(…)` with their
     constants untouched.
  5. naming trio :20994/:21043/:21072 → `naming_overlay_card(&colors,
     bounds.w, bounds.h)` + existing `.child(...)` chains.

**Regression test plan.**

| REQ | Test / check (RUN at validate) | Assert |
|---|---|---|
| REQ-001 | negative grep (python regex over app.rs): `\.flex\(\)\s*\.flex_col\(\)\s*\.bg\(colors\.surface\)\s*\.rounded\(colors\.corner_radius\)\s*\.overflow_hidden\(\)\s*\.border_1\(\)\s*\.border_color\(colors\.border\)` | exactly **2** matches (the two helper bodies); 12 before the sweep (2 helpers + 10 sites) — run BEFORE (12) and AFTER (2) as the negative smoke |
| REQ-002 | per-site chain-identity ledger at inspect | old vs new call multiset identical per site (positioning constants, fonts, max_h unchanged) |
| REQ-003 | review + rustdoc | completion popup calls the over-scroll variant; the variant doc names it; #417 ride-along recorded |
| REQ-004 | `context_menu::tests` new unit(s) | `naming_card_geometry(1000, 800) == (300, 200, 400)`; containment property (`left+w ≤ w`, `0 ≤ top ≤ h`) at several sizes; mutation kills on the arithmetic |
| REQ-005 | `cargo nextest run --workspace` | full suite green; no test-file edits in the diff |

No NEW draw smokes: each site's behavior tests shipped with its own ticket
(#313/#317/#323/#304/#325/#326/#327/#204/#399/#378) and REQ-002/005 pin the
refactor; the #318/#415 recipe-B smoke continues to draw the five quarter
overlays. Uncoverable: live wheel-over-popup (needs a live session — #417).

**Risks.**
- Prefix-reorder equivalence relies on gpui builder semantics (fields, not
  order) — the #318 conversions shipped this exact equivalence; accepted.
- The completion popup hitbox change is the ONE deliberate behavior delta —
  spec REQ-003; wheel confirmation debt recorded.
- `base` closure in references captures `colors` — helper call moves inside;
  borrow shape unchanged (the closure already reads `colors`).

## Phase 3 — Implement
- **React-first: N/A** (per spec — pure Rust DRY refactor; the POC already owns
  its shared OverlayShell).
- Built to the manifest:
  - `context_menu.rs`: `naming_card_geometry` (0.3·w / h/4 / 0.4·w) beside
    `overlay_quarter_geometry`, same containment-by-construction doc idiom.
  - `app.rs`: `overlay_card_chrome_over_scroll` sibling (block_mouse_except_
    scroll head, same 7-call tail, cross-referencing docs, mutants::skip);
    `naming_overlay_card` (chrome + naming geometry + the trio's shared
    `.p_3()` + text_color, mutants::skip); `overlay_card_chrome` doc refreshed
    (sweep complete; variant named; hover D3 tension recorded in place).
  - Conversions: completion popup → over-scroll variant (with the PR-rule
    comment in place); references (helper INSIDE the preserved `base` closure);
    code_action, file_symbols, symbols, search, problems → helper-first
    anchored form (constants untouched); naming trio → `naming_overlay_card`
    (three sites, per-card children untouched).
- **Deviation from design (recorded):** the test-plan's BEFORE count said 12;
  the real BEFORE was **11** (1 existing helper body + 10 sites — the "12"
  wrongly pre-counted the variant helper that didn't exist yet). AFTER
  measured **2** (modal fn + variant fn bodies) — exactly the REQ-001 target.
- `cargo check --workspace --all-targets` green; `cargo fmt` clean (run
  proactively after the #415 rustfmt-red lesson).

## Phase 3.5 — Inspect

Three parallel critics: chain-identity auditor, correctness/behavior,
simplification+provenance. All verified concretely (gpui 0.2.2 source read for
order-independence and hitbox semantics; `cargo check` + context_menu unit runs).

**Chain-identity ledger (REQ-002 evidence — the auditor's per-site verdicts):**
9/10 sites **IDENTICAL** call multisets (references still a closure invoked
twice with same captures; per-site constants W/MAX_H/fractions verified
unchanged: 620/360·0.15, 560/320·0.2, 560/380·0.12, 640/380·0.12, 720/420·0.10,
760/420·0.10; trio 0.3 / h/4 / 0.4 float-identical). 1/10 (completion popup)
carries **exactly the one declared delta**: occlude → block_mouse_except_scroll.
gpui order-independence PROVEN at source: every style call is a per-field
`Some(v)` write, no field written twice in any chain, hitbox flag lives outside
style — permutation yields the identical element. Delta semantics proven at
window.rs:775-797: ExceptScroll truncates hover but does NOT break hit_test, so
the editor's `track_scroll` uniform_list beneath keeps `should_handle_scroll` —
wheel-over-popup now scrolls the editor, the viewport_offset guard fires, and
scroll-dismiss WORKS (it was dead under occlude — the wheel never reached the
editor). The popup has no internal scroll (rows pre-windowed via popup_window),
so nothing relied on wheel-swallowing.

| # | Sev | Finding | Verdict | Action |
|---|---|---|---|---|
| F1 | LOW ×3 | `naming_card_geometry` ships untested in this diff (pure, not skipped — mutants would survive today) | REAL but PRE-OWNED | Phase 4 REQ-004 writes the fractions+containment unit (the pipeline writes tests at validate); explicitly must-not-drop. |
| F2 | LOW | The anchored-six suffix/`menu_origin`-fraction duplication is correctly deferred (per-site values + `&self` fonts kill a free-fn recipe — the #318 D1 constraint; extraction would be shape-enforcement, not dedup) but the deferral was UNRECORDED | REAL (doc gap) | Recorded here + P5 adds the named option (`anchored_overlay_card` as a `&self` method if a 7th anchored picker lands) to the app_shell.md recipes entry. |
| F3 | LOW | context_menu.rs module banner (the #318 role statement) omitted the new resident | REAL | Fixed — banner now enumerates `naming_card_geometry`. |
| F4 | INFO | Horizontal wheel over the popup now h-scrolls the editor while the popup stays (dismiss guard is vertical-only) | ACCEPTED | Intended-semantics extension of pass-through; identical to wheel-beside-popup. Recorded. |
| F5 | INFO | The Out-list's occlude reconciliation: 4 pre-existing non-card interactive sites (jump-to-bottom 20627, divider bars, footer agent segment, cockpit tabs) weren't enumerated | ACCEPTED | None card-shaped, none touched; noted for completeness. |

Lenses clean: provenance (§20 — gpui Apache-2.0 API adoption, own values moved
verbatim), reuse (no existing card helper duplicated; ui_components still has
no card primitive), placement (context_menu.rs is the pure-geometry home),
borrow forms (bare `colors` in method bodies, `&colors` in render — matches
#318 sites exactly), "zero verbatim sites" verified workspace-wide (looser
receiver-agnostic grep also = 2).

No F-/PR- ledger appends: no shipped bug found — the findings are doc/test-plan
hygiene; the anchored-six named option lands in the P5 AD update.

## Phase 4 — Validate
- **Test written (REQ-004):** `naming_card_geometry_fractions_and_containment`
  (context_menu.rs) — exact values at (1000,800)→(300,200,400) + the
  expression-form pins at (2384,1119) (f32-drift-proof: expected side computes
  the identical ops — the L-422 house idiom) + the containment property over
  degenerate/large sizes, mirroring its quarter sibling.
- **Runs (real output):** `cargo nextest run --workspace` → **2148 passed, 5
  skipped, 0 failed**; doctests 0 (ok); the new unit solo: 1/1 PASS.
- **Negative smoke (REQ-001):** core-chain regex count = **2** (the two helper
  bodies; was 11 before implement). Asserted in-transcript with a hard
  `assert n == 2`.
- **REQ-002/003:** carried by the inspect chain-identity ledger (9/10
  IDENTICAL; the popup's single declared hitbox delta proven at gpui source).
- **Live drive (D3, mandatory attempt):** rebuilt the bundle WITH the #414
  code, launched, probed: **five marley windows, all zero-size** — the same
  no-active-session condition as #415's validate, re-evidenced today for this
  build. Pixel captures + the wheel-over-popup confirmation ride the #417
  battery (Deliberate). Not silently skipped.
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff], 15 passed 0
  failed** (first run — fmt was run proactively at implement), receipt written.
- Pre-existing: the `block v0.1.6` future-incompat note (transitive) — not in
  scope.

## Phase 5 — Complete
- **CHANGELOG:** entry under Unreleased/Changed (sweep finished, sibling
  chrome, Recipe C, the popup's one delta + why, anchored-six option).
- **Architecture docs:** app_shell.md's overlay-card-recipe section rewritten:
  three recipes, the over-scroll sibling + membership, zero-verbatim status,
  the recorded anchored-six named option (inspect F2), #417 ride-along.
- **Parity sync:** N/A ticket (no UI delta; the POC's OverlayShell already
  embodies the consolidation — its drifts are #416, next in queue).
- **Ledger appends:**
  AD-claude-414-overlay-chrome-three-recipes-and-the-over-scroll-sibling-001.
  (No F-/PR-: no shipped bug found; no new L-: the durable content is the AD.)
- **Ticket:** TICKET-414 → tickets/closed/, status closed; backlog clean.
- Archived to docs/planning/pipeline/completed/.
