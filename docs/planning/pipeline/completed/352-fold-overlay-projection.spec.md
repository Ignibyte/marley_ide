---
pipeline_id: 2fe61fe9-de6e-4e51-97b3-27ab2a17489d
ticket: forge#352 (8f3852b9-de73-48bc-988e-20a3799ebb23) · local docs/planning/tickets/open/TICKET-352-fold-overlay-projection.md
aar_id: pending-promotion
status: Phase 5 — Complete PASS (auto run, 2026-08-04)
title: Fold projection for the LSP overlay-card geometry — thread visible_row through the frame-geom layer (#305 Slice 2)
type: feature
milestone: M28 — train-independent carry (original area M19/#305)
references:
  - docs/planning/pipeline/completed/305-code-folding.spec.md
  - docs/planning/pipeline/completed/305-code-folding.notes.md
  - crates/syntax/src/fold.rs
  - crates/marley_app/src/app.rs
  - crates/marley_app/src/headless_drive.rs
  - docs/zed_architecture/crates/editor.md
---

## Title
#305's one documented known-limit, cashed in. The fold engine shipped THE buffer↔visible row
projection (`FoldProjection::slot_of`/`buffer_row`, crates/syntax/src/fold.rs:92-156) and converted
the rim + scroll + sticky crossings — but the frame-geometry fan-out still mixes BUFFER rows with the
slot-recorded `editor_geom.first/last` (`EditorFrameGeom`, app.rs:742-754; `first/last` are
uniform_list's range = SLOTS). With a fold above the caret on-screen, a card whose `anchor_row` is a
buffer row subtracts a slot — it renders at the wrong Y (a bounded VISUAL imperfection, not a
correctness break; #305 shipped this as a named cut, 305-code-folding.notes.md "SCOPE DECISION —
SPLIT"). The ticket's 2026-07-18 site list is STALE; the 2026-08-04 re-sweep (HEAD `940286d`)
verifies **nine sites, four classes** (the inventory below — D-OPEN-SITE-SET). Today the five forward
sites copy-paste `(row - geom.first) as f32 * geom.cell_h` with NO shared helper. The work: extract
ONE shared projection helper (reusing the SHIPPED `slot_of`/`buffer_row` — no second projection
implementation), then thread it through the verified sites so each computes
`(proj.slot_of(buffer_row) - geom.first) * cell_h` — `geom.first` stays a slot; the buffer row is
projected before subtracting. Mechanical once the helper exists; the RISK is finding ALL the sites —
the sweep is Phase 2's first job.

**The verified site inventory (2026-08-04, all read at HEAD; stale ticket line → current):**
- **(i) Forward row→Y anchors (5)** — the copy-paste idiom (window-check + Y both mix):
  1. `hover_card_overlay` app.rs:13001 — check :13021 (`card.anchor_row < geom.first || >= geom.last`),
     Y :13031 (`anchor_row.saturating_sub(geom.first) * cell_h + cell_h`). Was ~10583.
  2. `completion_popup_overlay` app.rs:13076 — check :13097, Y :13105 (caret row). Was ~10657.
  3. `signature_overlay` app.rs:13171 — check :13182, Y :13189 (`open.anchor_row`, the fixed call
     anchor). Was ~10741.
  4. `rename_draft_overlay` app.rs:13234 — check :13246, Y :13253 (caret row — the #322 card). Was ~10805.
  5. **IME caret rect `bounds_for_range` app.rs:14901 — check :14917, Y :14925 (#267/M16 — predates
     the 2026-07-18 sweep and was MISSED by it).** The OS candidate window is a sixth row-anchored
     "card", OS-drawn.
- **(ii) Inverse pixel→row (2):**
  6. Hover-dwell `on_editor_mouse_move` app.rs:12940 — `row = geom.first + (y - y0)/cell_h` (a SLOT)
     stored in `last_hover_cell` and consumed as a BUFFER row by `tick_hover_dwell` :12908-12922
     (`line_text(row)`/`line_start(row)` → the LSP hover REQUEST fires for the WRONG row, then the
     card anchors wrong too). Was ~10492.
  7. **IME `character_index_for_point` app.rs:14944 (#267 — also MISSED).**
- **(iii) Viewport-window ranges (2):**
  8. Content-width probe in `code_view_body` app.rs:5852 — `(geom.first..=geom.last).map(row_cols)`
     feeds SLOTS to `line_text(r)` (buffer-row API): the h-scroll content width measures the WRONG
     rows past a fold. Was ~4818.
  9. Inlay-hints fetch window in `refresh_inlay_hints` app.rs:4502 (`want_first`), :4505-4508
     (`want_last`), :4511 (served-check), :4544-4546 (slots sent as the LSP request's buffer-row
     range); `INLAY_PAGE = 50` :733 absorbs only folds hiding ≤50 rows. **Named un-projected by the
     #305 inspect (notes E-3 [3]: "the 5 LSP cards + width-probe + inlay-prefetch stay un-projected =
     the documented #352 known-limit") but ABSENT from the ticket's 7-site text.**
- **(iv) Confirmed NON-sites:** `def_picker_overlay` :13325 (#312), `references_overlay` :13390
  (#317), `code_action_overlay` :13495 (#323), `goto_overlay` :13289, `file_symbols_overlay` :13558,
  `symbols_overlay` :13658, `search_overlay` :13764, `problems_overlay` :13893 — all
  centered/window-anchored (`menu_origin` over win_w/win_h; no row math). Sticky :6461-6463 and
  `scroll_editor_to` :14082-14086 already project (#305). Click/drag rows are rim-supplied
  (`h_scroll.rs` invariant — no interior y→row conversion).

## Scope
### In
- **The shared projection helper (pure — home = D-OPEN-HELPER-HOME):** a forward seam — buffer row →
  slot-relative anchor (visibility + `(slot_of(row) - first) * cell_h`) — and its inverse — pixel Y →
  `buffer_row(first + dy/cell_h)` — both DELEGATING to the shipped `FoldProjection::slot_of`/
  `buffer_row`. The five-site copy-paste idiom dies; the per-card chrome (widths, `menu_origin`/
  `popup_origin`/`signature_card_origin` flips, the hover card's below-anchor `+ cell_h`) stays
  per-card — this helper owns ONLY the row↔Y projection math.
- **Thread the verified sites** (classes i-iii above; the final set = D-OPEN-SITE-SET after the
  Phase-2 re-sweep): each forward site window-checks and positions by the PROJECTED slot; each
  inverse converts slot→buffer row before storing/consuming; the two window ranges convert both ends
  (the probe reads `buffer_row(slot)` per iterated slot; the inlay window requests
  `buffer_row(first)-PAGE ..= buffer_row(last)+PAGE` and the served-check compares in one domain).
- **Per-card fold/no-fold headless drives + identity pins;** the live capture (REQ-007).
- **Regression:** the #305 fold suites (13 fold.rs tables + 6 drives), #311 hover / #313 completion /
  #320 signature / #322 rename suites — green unchanged.

### Out (explicitly deferred)
- **The #318 shared overlay-card RECIPE extraction** — the style/chrome copy (surface div, borders,
  clamps) is the SIBLING refactor #313 explicitly parked for #318 (313-lsp-completions.notes.md:375-377);
  this ticket extracts the GEOMETRY seam only and must not absorb it.
- **Fold behavior changes** — no new fold kinds, persistence, chords, or reveal rules; #305's engine
  is consumed, not modified.
- **Sticky-header interactions beyond what #305 shipped** (:6461-6463 stands as-is).
- **The #333/#351 lexer / ungated-parse gaps** (separately filed; #351 referenced by #305 P1).
- **The #305 W-2 `editor_folds` leak follow-up** (parked at #305 inspect — not this ticket).

## Reference (§20)
**VS Code / Zed — OBSERVED behavior:** both anchor hover/completion/signature popovers at the
VISUALLY correct row below/above a fold (the ticket records the observation; it extends #305's
observed folding sweep — 305-code-folding.spec.md §20). The behavior map
docs/zed_architecture/crates/editor.md corroborates at the research level: Zed routes ALL geometry
through the display-map chain (`inlay_map → fold_map → tab_map → wrap_map`, editor.md:80;
`display_map/fold_map.rs` "hide folded regions behind a placeholder → FoldPoint", :92; movement and
positioning operate "in display space … respecting wrap/folds", :104) — i.e. downstream consumers
never see a buffer row unprojected. Marley matches the BEHAVIOR with its far smaller per-site
projection (the #305 stance: no display-map layer; Zed's FoldMap remains architecture-CONCEPT only,
source unread — the §20 wall holds; no Warp/Zed source consulted).

### Prior art
1. **Behavior maps — checked.** docs/zed_architecture/crates/editor.md:80/:92/:104 (above) — the
   "project before positioning" convention, research not source. docs/warp_architecture/ has no
   editor-fold surface (terminal-side; nothing to carry).
2. **Published material — one relevant fact, else none.** The LSP spec addresses positions in
   BUFFER lines (published protocol) — which is exactly WHY every card's `anchor_row` arrives as a
   buffer row and must be projected at render; no published material covers overlay-anchoring above
   folds specifically (none expected — it is an internal geometry concern).
3. **OUR PERMISSIVE DEPS + IN-HOUSE — the seam already exists.** The adoption leg is IN-HOUSE:
   `FoldProjection` (crates/syntax/src/fold.rs — cov/MSI-100, the half-open-interval prefix-sum with
   `slot_of`'s documented hidden-row→header-slot snap) ALREADY computes both directions; this ticket
   reuses it, never re-derives it. gpui (Apache-2.0) provides only render primitives
   (`div().absolute().top(px(..))`, `uniform_list`) — it owns no row-projection seam, confirmed.
   ropey/regex/tree-sitter own nothing here. Checked — no external owner.

## React-first (parity)
**N/A — no UI delta:** the POC has no fold engine (verified 2026-08-04: the only "fold" hits in
marley-web/artifacts/marley-ide/src are find-bar CASE folding — `findFold`, App.tsx:56-114,
EditorView.tsx:97-181 — a commandSimulator-era scaffolding gap), so the fold-above-caret scenario
cannot be built or verified there; the fix is Rust-side geometry correction verified via the headless
drives + live capture. The overlay cards themselves are already mirrored in the POC (EditorView.tsx
hover card :317-319 anchored one cell below the symbol; rename draft :220) and their UNFOLDED
geometry is untouched — no-fold byte-identity is an AC (REQ-002), so parity is preserved by
construction.

## Locked-In Decisions
- **D1 — extract ONE shared helper; the copy-paste dies.** All forward sites (the 4 LSP cards + the
  IME rect) position through one seam; the two inverses go through its inverse; the two window
  ranges through the same projection calls. The #318 overlay-card RECIPE (style chrome) is the
  sibling refactor and is NOT absorbed — this helper owns row↔Y projection math only; per-card
  offsets and origin-flip seams (`popup_origin`, `signature_card_origin`, `menu_origin`, the hover
  `+ cell_h`) stay where they are.
- **D2 — `geom.first`/`geom.last` STAY slots.** The #305 recording (uniform_list's range,
  app.rs:742-754) is untouched; conversion happens at each consumer: the card's BUFFER row is
  projected to a slot before subtracting — `(proj.slot_of(buffer_row) - geom.first) * cell_h` —
  never the reverse (re-basing geom would re-open every already-correct #305 crossing).
- **D3 — the helper REUSES `FoldProjection::slot_of`/`buffer_row` — no second projection
  implementation.** No parallel interval math anywhere; the helper is a thin geometry adapter over
  the shipped, mutation-proven bijection (fold.rs's 47-mutant surface stays the single source of
  row truth).
- **D4 — no-fold behavior byte-identical.** With the identity projection (`fold_projection()`'s
  no-parse early return, app.rs:11633-11643) every touched site computes exactly today's values —
  positions, window checks, request params. Byte-identity is pinned (REQ-002), and the identity
  path stays parse-free.
- **D5 — geometry-only.** No fold-behavior, card-content, or card-chrome change; the diff at each
  site is the projection delegation and nothing else.

**D-OPEN (Phase 2 decides, with evidence):**
- **D-OPEN-HELPER-HOME** — where the helper lives. **Recommendation: beside `FoldProjection` in
  crates/syntax/src/fold.rs** (methods or free fns taking SCALARS — first/last slots, y0/cell_h as
  plain usize/f32; `EditorFrameGeom` is app-private and marley_syntax must stay gpui-free) — the
  cov/MSI-100 home, tested with the projection it adapts. Alternative: a small app-side pure module
  beside the existing origin seams (`editor_complete::popup_origin` et al). Phase 2 confirms
  against call-site ergonomics (9 sites, two directions).
- **D-OPEN-SITE-SET** — the FINAL site inventory. The nine-site inventory above (verified
  2026-08-04) is the STARTING set; Phase 2 re-sweeps at design time: grep `geom.first` /
  `geom.last` / `geom.cell_h` row-math over app.rs + every `*_overlay` fn + the input-handler
  impls, plus `cargo mutants --list -f` on the candidate files to expose helper-adjacent surface.
  The one membership judgment call: the inlay fetch window (site 9 — a request-RANGE, not a card;
  its symptom is missing hints past a >50-row fold). **Recommendation: IN — same class, same
  two-line mechanical fix,** and the #305 inspect already counted it in this ticket's known-limit.
- **D-OPEN-OFFSCREEN-ANCHOR** — a card whose anchor row is INSIDE a collapsed fold (e.g. the
  signature card's fixed anchor after a palette Fold All). **Recommendation: anchor at the fold
  header's slot** — `slot_of`'s documented hidden-row→header snap gives this for free, matching the
  #305 discipline (hidden rows present as their header; auto-reveal already keeps caret-anchored
  cards out of this state in the common paths). Alternative: suppress the card (`None`) while the
  anchor is hidden. Decide with the drive scenario; pin whichever renders (REQ-004).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a fold above the caret's on-screen row is active, each row-anchored overlay (hover, completion, signature, rename, IME caret rect) shall compute its anchor Y from the PROJECTED slot — `(slot_of(anchor_row) - geom.first) * cell_h` — and its visibility window check shall compare slots with slots, placing the card at the visually correct row. | helper truth tables (fold-above cases); ONE headless drive PER CARD (fold above → the card's computed anchor equals the slot-projected value, read via the for_test seams — `hover_card_for_test` :9642, `signature_card_for_test` :10023, new accessors as designed) |
| REQ-002 | WHEN no fold is active, every touched site shall produce byte-identical positions, window checks, and request parameters to today (the identity projection; no parse on the identity path). | identity rows of the helper tables; no-fold drives per card asserting today's exact values; #305 + #311/#313/#320/#322 suites green unchanged |
| REQ-003 | WHEN the pointer dwells at a pixel Y (hover) or the OS asks `character_index_for_point` (IME) with a fold above that Y in the viewport, the computed row shall be the BUFFER row of the slot under the pointer — the hover request fires for the row the user actually sees. | inverse-helper units; a dwell drive with a fold (the hover REQUEST's row asserted via `drive_hover_for_test` :9623 seams) |
| REQ-004 | WHEN a card's anchor row lies INSIDE a collapsed fold, the card shall render at the position D-OPEN-OFFSCREEN-ANCHOR resolves (recommended: the fold header's slot — `slot_of`'s snap), never at a phantom unprojected offset. | helper unit (hidden-row input) + a headless drive folding over an open card's anchor |
| REQ-005 | WHILE folds hide rows inside the viewport, the content-width probe shall measure the BUFFER rows actually on screen (per-slot `buffer_row`), and the inlay-hint fetch window shall request/serve-check the projected buffer-row range (± INLAY_PAGE). | units over the projected window math; a probe drive (fold → width reflects visible rows); the inlay request params asserted via the host for_test seam |
| REQ-006 | The extracted helper shall be a NEW pure seam with direct units at cov/MSI 100 — every new named helper is a fresh mutation surface (`BF-lsp-hover-extracted-helper-new-mutation-surface-001`). | gate:4/5 on the helper's module; `cargo mutants --list -f` receipt showing the helper's mutants all killed |
| REQ-007 | WHEN a fold is active above the caret and a hover (or completion) card is open, a live capture of the running app shall show the card positioned at the correct visual row below the fold. | driven/headless render-executing capture receipt at Validate; if the machine state blocks driving, the recorded env-blocked protocol applies (documented explicitly; units + drives + mechanism carry — the #204/#205 precedent) |

## Floors (constitution)
The helper is a PURE seam at **cov/MSI 100** (the new mutation surface — REQ-006; direct truth
tables, both directions, identity + fold + hidden-row + boundary rows). The nine call sites are
MASKED render/input shims (app.rs is coverage-excluded; the overlay fns and input handlers carry
`mutants::skip`) — the drives are the proof (the #307 pattern). Run `cargo mutants --list -f` on the
ACTUAL touched files after placement (`PR-claude-trace-the-real-cargo-mutants-list`), and re-verify
neighboring `#[mutants::skip]` bindings — the edits land inside existing skip-marked shims, the
skip-detach trap's home turf (`BF-claude-skip-detach-pump-fleet-live-001`). The helper stays
language-blind: the shipped `fold_projection()` accessor already carries the Rust gate
(app.rs:11633-11643; `PR-claude-language-specific-pure-primitive-needs-caller-side-gate-001`
satisfied by the existing caller — no new gate).

## Phase Plan
- **P2 Design** — **the re-sweep IS the design's first job** (the ticket's named risk): grep
  `geom.first`/`geom.last`/`geom.cell_h` row-math over app.rs, walk all 13 `*_overlay` fns + the
  `EntityInputHandler` impls, `cargo mutants --list -f` on the candidate files; settle
  D-OPEN-HELPER-HOME / D-OPEN-SITE-SET / D-OPEN-OFFSCREEN-ANCHOR; exact helper signature(s) + the
  per-site conversion table (the #305 crossing-table shape); per-REQ test plan incl. the drive
  observables (note: `simulate_keystrokes` renders reset `editor_geom` to 0 — headless_drive.rs:5854
  — so the drives assert at the helper/for_test level or seed geom, designed here).
- **P3 Implement** — the helper + its truth tables FIRST (the pure seam), then thread the sites
  MECHANICALLY (one-line delegations per site; chrome untouched — D1/D5).
- **P3.5 Inspect** — adversarial site-set completeness against the DIFF (the stated risk: does any
  `geom.first`-mixes-buffer-row site survive un-threaded?); off-by-one at fold boundaries (the hover
  card's below-anchor `+ cell_h`, the probe's inclusive `..=`, both inverse clamps at
  `geom.last.saturating_sub(1)` — clamp in SLOT domain then convert); the probe's width math (cells
  vs slots); the IME rect is OS-observable-only (mechanism proof, no pixel claim); skip-detach
  re-check; §20 provenance on added lines.
- **P4 Validate** — write + RUN the per-REQ tests (helper tables; per-card fold + no-fold drives;
  the dwell/probe/inlay drives); `cargo mutants --list -f` on the actual touched files; gate green
  (`--diff`), cov/MSI 100 on the helper; #305 + #311/#313/#320/#322 regressions green; the REQ-007
  capture (or the documented env-blocked fallback).
- **P5 Complete** — CHANGELOG; close the #305 known-limit note (spec Out + notes SCOPE DECISION now
  point at a shipped fix); editor-docs projection section gains the geom-fanout paragraph; AAR
  capture; archive; close #352.
