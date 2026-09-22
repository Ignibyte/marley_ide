# Code folding (the first visible↔buffer row projection) — Notes

- **Forge ticket:** #305 13b140d5-ca11-44a6-80c3-29bc6dbac0f3
- **AAR:** 9bd998a9-ccfc-48f8-953d-2428ca72a3db
- **Local ticket doc:** docs/planning/tickets/open/TICKET-305-code-folding.md
- **Pipeline spec:** 305-code-folding.spec.md

<!-- Working scratch. Each phase appends its entry. -->

## Phase 1 — Plan
- **Request:** code folding — the pure fold_regions + FoldProjection row-projection, anchor-keyed state, auto-reveal, the gutter chevron + ⋯-inlay tail, ⌥⌘[/] chords, Rust-only. Promoted from the pre-authored queued spec (the Fable method); FIFTH of `/work 300,302,303,304,305,314,315,316,317,259`. The HIGHEST-RISK ticket in the batch (the first buffer-row↔visible-row projection).
- **Classification / tier:** work pipeline — but flagged for a POSSIBLE split if the blast radius / project-once seam doesn't hold (assessed below).
- **Promotion done:** `git mv` queued→active; pipeline_id `2a6b5293-2778-43e6-ab6d-deb975146f79`; aar `9bd998a9-ccfc-48f8-953d-2428ca72a3db`; TICKET-305 created.
- **Seam re-verification (spec on Fable; #300/#302/#303/#304 landed since — app.rs drifted 4×).** An Explore agent is attacking the blast-radius claim + the anchor.rs first-consumer risk + all seams; the "project once at the boundary" premise is the make-or-break. _Findings table below._

### Phase 1 findings (seam re-verification)

**F1 — anchor.rs CONFIRMED shipped + unused in production (folds are the first consumer), WITH a nuance the spec glossed.**
`crates/editor/src/anchor.rs` ships `Bias` (:18) + `Anchor` (:28); the API is on `Buffer` — `anchor_at(offset, bias) -> Anchor` (buffer.rs:223) + `resolve_anchor(&Anchor) -> CharOffset` (buffer.rs:236), both stamped with the buffer version + tested (buffer.rs:1207+). Grep for consumers: ONLY buffer.rs (the methods + their tests) — **NO production consumer** (app.rs clean). D-ANCHOR-KEYED's "folds are the FIRST" holds; the first-consumer risk is real.
  - **THE NUANCE (a real Phase-1 finding for the design):** `resolve_anchor` returns a **`CharOffset` (clamped), NOT `Option`** (buffer.rs:1214/1218 tests: a past-EOF anchor resolves to the clamped end, never None). So the spec's "a region deleted by an edit resolves to nothing and the fold evaporates" is NOT free from `resolve_anchor` — it always yields a position. The design must detect a collapsed/deleted fold DIFFERENTLY: after resolving the header anchor per (nonce, version), RE-DERIVE the fold region (re-run `fold_regions`) and keep the fold only if a foldable region still starts at/contains the resolved header; else drop it. The anchor carries the POSITION through edits; the region VALIDITY is re-checked against the fresh parse. (This is the anchor-first-consumer subtlety the spec named but under-specified.)

**F2 — ⌥⌘[ / ⌥⌘] CONFIRMED FREE (distinct from the existing bracket chords).** keymap.rs binds ⌘] `(T,F,F,F,"]")` (:211), ⌘[ `(T,F,F,F,"[")` (:242), ⌘⇧] `(T,F,F,T,"]")` (:247), ⌘⇧[ `(T,F,F,T,"[")` (:251) — but NO **alt**+cmd variant. So ⌥⌘[ `chord(true,false,true,false,"[")` + ⌥⌘] `chord(true,false,true,false,"]")` (cmd+ALT+bracket) are FREE. Roster is NOW 78/33 (post-#304) → target **80/35** (2 Editor-scoped rows). Tuple order `(cmd, ctrl, alt, shift, key)` confirmed.

**F3 — `fold_regions` is NEW; the projection precedent CONFIRMED.** No `fold_regions` exists (grep clean). `crates/marley_app/src/nav.rs` has the #184 **command-block** folding: `FoldState` (:33) + `fold_visible_rows(output_line_counts, folds) -> Vec<RowKind>` (:67) + `fold_boundary_rows` (:107) — the shipped FLAT projection precedent (a full row list → a visible `Vec<RowKind>` with map-back), used by `workspace.rs` (`folds: FoldState` field :310). So the FoldProjection shape is proven in-repo (the TERMINAL block-fold), and #305's editor FoldProjection mirrors it. NOTE: nav.rs's `FoldState` is the TERMINAL block-fold state — #305's editor fold state is a NEW, DISTINCT structure (anchor-keyed, per-editor-file), not a reuse of nav's (which is command-block-index-keyed). The PROJECTION PATTERN is the reuse; the state is new.

**F4 — the editor uses `uniform_list` (#266) — the rim where the projection slots in.** Confirmed the editor renders on `uniform_list` (app.rs comments :4751, :14166, :5376; the scroll handle :431).

**F5 — THE BLAST-RADIUS VERDICT (verified inline; the make-or-break): the "project once at the boundary" premise HOLDS — #305 is ONE shippable slice.** The editor `uniform_list("editor-lines", total, |range| …)` (app.rs:**4840**) uses the callback's `row` index DIRECTLY as a buffer row at every interior site — `buf.line_start(row)`/`buf.line_text(row)` (:4858-4860), `einlay.get(&row)` (#331 inlays :4874), `syntax_cache…lines.get(row)` (:4885), and the caret/click/squiggle math all consume the resulting per-row `text`+`layout`. `total` = `len_lines()` today (visible==buffer). **The projection slots in cleanly:** set `total = proj.visible_count()` and make the callback's FIRST line `let row = proj.buffer_row(slot)` — then EVERY interior site is UNTOUCHED (they keep buffer-row semantics, reading the projected `row`). The spec's "~17 sites" are exactly those UNTOUCHED interior sites — the fear ("17 sites break") is inverted into the ARGUMENT FOR project-once: they don't break because they never see a slot.
  - **The CROSSING sites (what DOES convert) — ~5-6, all bounded:** (1) the render rim (`buffer_row(slot)`, inbound); (2) `scroll_editor_to_row(row)` → `scroll_to_item(proj.slot_of(row))` (app.rs:**11526→11528** — scroll_to_item scrolls by SLOT, confirming Claim 8, outbound); (3) the frame-geom `editor_geom` (first/last are recorded as SLOTS from uniform_list's `range` — :4845-4846) → convert slot→buffer_row before sticky-headers' `first_visible_row` consumes it (outbound); (4) caret-follow (caret's buffer row → slot to scroll, outbound); (5) **the click→offset + (6) drag** — these compute a row from the pixel Y independently of the render callback, so pixel→slot→`buffer_row` is ALSO a crossing (a refinement the spec's clean "1 rim + 3 outbound" understated — it's the rim + click/drag + 3 outbound ≈ 5-6). Still bounded + convertible; NOT a 17-site rewrite. **SCOPE JUDGMENT: ONE slice** (the projection is contained to a handful of boundary crossings; the interior is inert). An Explore agent is corroborating the exact crossing count; the inline analysis above is authoritative for the one-slice decision.

- **§20 + prior art CONFIRMED** — folding is OBSERVED VS Code/Zed (chevrons, ⋯ tail, click-toggle, auto-reveal, folds-survive-edits-above); tree-sitter is published-API reuse; the projection + anchors are Marley-original; **Zed's FoldMap is architecture-CONCEPT only (source unread — §20 wall holds)**, and Marley's per-row `uniform_list` admits a far smaller row-projection than a display-map layer. The prior-art sweep PAID (F1-F5): nav.rs's FoldState + FileTree::visible_rows are the in-repo projection precedents, anchor.rs is the shipped-unused rebase capability, the #331 inlay channel is the ⋯-tail mechanism, gpui's `uniform_list` takes an arbitrary count.
- **AC / decisions:** all 10 REQ hold; the 6 locked decisions hold. **Deltas to carry into Design:** (i) F1 — `resolve_anchor` returns a clamped CharOffset not Option, so "fold evaporates on delete" = re-derive the region against a fresh `fold_regions` parse, not an anchor-None (D-ANCHOR-KEYED refined); (ii) F5 — the click/drag pixel→row is ALSO a crossing (the projection touches ~5-6 boundary sites: rim + click + drag + scroll + geom + caret-follow, not the spec's "1 rim + 3 outbound"). The adjacent ungated-parse gap = the ALREADY-FILED **#351** (referenced, not re-filed).

## Phase 2 — Design

### 1. Architecture / approach
Two pure seams (cov/MSI 100) + an app-side fold-state/projection layer, over one `uniform_list` rim conversion
+ a bounded set of boundary crossings (F5). §20: folding is OBSERVED VS Code/Zed; Zed's FoldMap is
architecture-concept-only (source unread — the wall holds); Marley's per-row `uniform_list` admits a small
row-projection instead of a display-map layer. No new deps.

**(a) THE PURE MODEL — NEW `crates/syntax/src/fold.rs`** (a module beside `symbols.rs`, re-exported from lib.rs):
- `pub struct FoldRegion { pub header_row: usize, pub end_row: usize }` — a foldable region (its header line +
  its last line; folding hides `header_row+1..=end_row`).
- `pub fn fold_regions(src: &str) -> Vec<FoldRegion>` — the FOLD_KINDS nodes (`function_item`, `impl_item`,
  `mod_item`, `trait_item`, `struct_item`, `enum_item`, `match_expression`) that span >1 ROW, in document
  order, nesting kept. An ITERATIVE `TreeCursor` walk (the `all_headers` shape, lib.rs:599 — recursion
  overflows the UI stack on a deep file; the 2000-level regression pins it). Uses the `.into_iter()`-over-Option
  parse idiom (#304 — no dead region, no `for_loops_over_fallibles`). Total (empty/garbage → empty Vec).
- **`pub struct FoldProjection` — THE mutation surface** (pure row arithmetic; lives in fold.rs, NOT app-side,
  because it's pure interval math testable in isolation): `FoldProjection::new(total_rows: usize, folded:
  &[(usize, usize)])` (folded = the ACTIVE `(header_row, end_row)` pairs; each hides `header+1..=end`) →
  `visible_count() -> usize`, `buffer_row(slot) -> usize` (the slot-th visible buffer row), `slot_of(buffer_row)
  -> usize` (a visible row's slot; a HIDDEN row → the slot of its enclosing fold's header — never off the end).
  A prefix-sum over the sorted, merged folded intervals. Invariant: `buffer_row(slot_of(r)) == r` for any
  VISIBLE r; `slot_of` is monotone. NESTED/OVERLAPPING folds merge (a fold inside a folded region contributes
  nothing extra). Empty folded set → identity (`visible_count()==total_rows`, `buffer_row(s)==s`).

**(b) THE APP-SIDE (app.rs shim):**
- **Fold STATE — `EditorFoldState` per file (nonce-keyed), anchor-keyed folds** (D-ANCHOR-KEYED, anchor.rs's
  FIRST production consumer): each active fold stored as its header's `Buffer::anchor_at(header_line_start,
  Bias::Left)`. Resolved per `(nonce, version)` (the #330/#304 memo shape): resolve each anchor →
  header CharOffset → header row; re-run `fold_regions` (memoized per (nonce,version)); **keep a fold ONLY if a
  region still starts at that resolved header row (the F1 re-derive — `resolve_anchor` returns a clamped offset,
  NOT None, so region VALIDITY is re-checked against the fresh parse; a deleted region → the fold evaporates).**
  The surviving `(header_row, end_row)` pairs feed `FoldProjection::new`. Typing ABOVE a fold shifts its anchor
  down → same region (REQ-005).
- **THE RIM + THE 6 CROSSINGS (D-PROJECT-AT-THE-BOUNDARY — the interior ~17 sites UNTOUCHED):**
  | # | site | conversion | dir |
  |---|---|---|---|
  | 1 | the `uniform_list("editor-lines", …)` rim (app.rs:4840) | `total = proj.visible_count()`; callback 1st line `let row = proj.buffer_row(slot)` | in |
  | 2 | `scroll_editor_to_row(row)` (app.rs:11526→`scroll_to_item`) | `scroll_to_item(proj.slot_of(row))` | out |
  | 3 | sticky-headers' `editor_geom.first` (a SLOT) | `proj.buffer_row(first)` before `sticky_rows` consumes it | out |
  | 4 | `follow_editor_caret` (app.rs:11538 — the shared primitive) | the caret's buffer row → `slot_of` for the scroll | out |
  | 5 | the editor click `on_mouse_down` (app.rs:5229/5463) — pixel→row | pixel → SLOT → `proj.buffer_row(slot)` | in |
  | 6 | drag (same pixel→row path) | same | in |
- **AUTO-REVEAL (D-AUTO-REVEAL-ON-CARET) — hook the SHARED primitive, not each caller** (the #336 lesson): at
  the TOP of `follow_editor_caret` (app.rs:11538 — reached by find-next/F8/go-to-def/go-to-line/symbol-jump AND
  ordinary caret moves), if the caret's buffer row is HIDDEN by an active fold, remove that fold (reveal) before
  the scroll. One hook covers every navigation (REQ-006).
- **The gutter chevron** — a NEW per-row gutter click target (▾ open / ▸ folded) on foldable header rows ONLY,
  with `stop_propagation` (the gutter has NO handler today → a bare click there hits the row-wide `on_mouse_down`
  = a col-0 caret click; the regression to pin — REQ-007). Toggles the fold at that header.
- **The "⋯ N lines" tail** — via the SHIPPED #331 inlay channel (D-MARKER-IS-AN-INLAY): a folded header row gets
  an EOL phantom inlay `⋯ N lines` (N = `end_row - header_row`). No new render path.
- **Chords + palette:** ⌥⌘[ → `"fold-at-caret"` (fold the INNERMOST `fold_regions` region containing the caret),
  ⌥⌘] → `"unfold-at-caret"`; Editor-scoped. **Fold All / Unfold All are PALETTE verbs** (`"fold-all"`/`"unfold-all"`
  — D-PALETTE-ONLY, no multi-stroke keymap machinery).
- **The language gate** (D-CALLER-GATES-LANGUAGE, the #340/#304 shape, AD-...-001): `fold_regions` runs only when
  `language_of(path) == Rust`; a non-Rust file → no folds, no chevrons (REQ-010). §14 total throughout.

### 2. File manifest
| File | Change |
|---|---|
| `crates/syntax/src/fold.rs` | NEW — `FoldRegion`, `fold_regions`, `FoldProjection` + the pure tables |
| `crates/syntax/src/lib.rs` | `mod fold; pub use fold::{fold_regions, FoldRegion, FoldProjection};` |
| `crates/marley_app/src/app.rs` | `EditorFoldState` + the (nonce,version) memo + the anchor resolve/re-derive; the 6 crossings (rim/scroll/geom/caret-follow/click/drag); auto-reveal in `follow_editor_caret`; the chevron click target + ⋯-inlay; the `fold-at-caret`/`unfold-at-caret`/`fold-all`/`unfold-all` dispatch; test hooks |
| `crates/marley_app/src/keymap.rs` | ⌥⌘[ / ⌥⌘] rows; roster 78→80 / scoped 33→35; individual asserts + a resolution unit |
| `crates/marley_app/src/palette.rs` (or the cockpit-cmd table) | Fold All / Unfold All verbs (every_cockpit_command_resolves) |
| `crates/marley_app/src/headless_drive.rs` | the drives (fold/unfold, anchors, auto-reveal, chevron-no-caret, the 3 outbound crossings, non-Rust) |

### 3. Regression Test Plan (≥1 row per REQ)
| # | Test | Kind | Pins |
|---|---|---|---|
| REQ-001 | `fold_regions_extracts_kinds_multiline_nested` | pure | the 7 kinds, >1-row only (a 1-line fn NOT foldable), nesting kept, document order; empty/non-Rust → [] |
| REQ-002 | `fold_projection_bijection` | pure | **THE mutation surface** — visible_count; buffer_row(slot) skips hidden; slot_of(hidden)=header slot; `buffer_row(slot_of(r))==r` for visible r; nested/overlapping merge; empty folded = identity |
| REQ-003 | `fold_hides_rows_and_restores_headless` | headless | fold → visible_count drops by (end-header); the ⋯ inlay present; unfold → exact restore |
| REQ-004 | `fold_unfold_innermost_at_caret_headless` | headless | ⌥⌘[ folds the innermost region at the caret; ⌥⌘] unfolds |
| REQ-005 | `fold_survives_text_typed_above_headless` | headless | type a line above a fold → the fold stays on ITS region (anchor carries; the F1 re-derive) |
| REQ-006 | `navigate_into_hidden_row_auto_reveals_headless` | headless | a go-to-line into a hidden row unfolds it (the shared-primitive hook) |
| REQ-007 | `gutter_chevron_click_toggles_without_moving_caret_headless` | headless | chevron click folds + the caret does NOT move (stop_propagation) |
| REQ-008 | `scroll_to_row_past_folds_targets_slot_headless` + `sticky_consumes_buffer_rows_headless` | headless | the outbound crossings — scroll_editor_to_row(buffer row) → the right slot; sticky_rows gets a converted buffer row |
| REQ-009 | `fold_all_unfold_all_resolve` | unit | the palette verbs resolve (every_cockpit_command_resolves) |
| REQ-010 | `non_rust_no_folds_headless` | headless | a .md file → fold_regions not run, no chevrons (the gate) |

Coverage/MSI 100 on fold.rs (fold_regions + FoldProjection — the pure surface). app.rs fold state/rim/crossings
are shims (mutants::skip); the drives prove the wiring. **LIVE chevron/⋯ PIXEL deferred-not-skipped (chad at
machine)** — the projection/fold-state/auto-reveal/scroll-slot STATE is ALL headless; only the gutter-chevron +
⋯-tail rendering is unverified-by-pixel.

### 4. Risks / decisions
- **The anchor FIRST-CONSUMER (F1, D-ANCHOR-KEYED)** — expect anchor bugs to surface HERE; they are anchor bugs
  (fix at source in anchor.rs/buffer.rs), not fold bugs. The re-derive-region-validity rule (resolve_anchor is
  clamped-not-Option) is the load-bearing correctness edge — inspect focus.
- **The 3 OUTBOUND crossings (scroll/geom/sticky)** — a green interior render CANNOT prove them; the headless
  STATE drives (REQ-008) must pin them explicitly (the P3.5 focus).
- **The click/drag pixel→row crossing (F5)** — the spec's "1 rim + 3 outbound" understated these; they're
  inbound crossings that must convert pixel→slot→buffer_row (else a click lands on the wrong buffer row past a
  fold).
- **The chevron-vs-row-click propagation** — `stop_propagation` on the chevron before the row-wide handler
  (REQ-007), the gutter's FIRST mouse handler.
- **Auto-reveal completeness** — hooking `follow_editor_caret` (the shared primitive) covers every navigation;
  a caller that places the caret WITHOUT going through it would silently break — verify the placement paths all
  funnel through it (the #336 discipline).
- **FoldProjection home = crates/syntax** (pure interval math with the fold model), not app-side — testable in
  isolation, the mutation surface.

**Explore corroboration (RECONCILED — a high-value finding that SCOPES the ticket):**
- **The INTERIOR premise is CONFIRMED even more strongly than F5.** `h_scroll.rs:1-4` pins the invariant that
  the vertical axis belongs to `uniform_list` and **NO click/interior site ever converts a y-pixel to a row** —
  so the single rim `let row = proj.buffer_row(slot)` mechanically corrects ALL ~17 interior sites (gutter,
  caret-x, click→col-only, syntax, squiggle, git, find, bracket, selection). My F5 "click is a crossing" was
  WRONG in one respect: the click computes only the COLUMN from pixels; the row is the slot handed by
  uniform_list — so the click is fixed by the rim too (not a separate crossing). Good — fewer inbound crossings.
- **BUT the OUTBOUND crossings are UNDERCOUNTED (the real finding).** The `editor_geom.first` (a SLOT) seam
  fans out into **~7-9 sites** that mix a buffer row with the slot-indexed `geom.first` — the scroll primitive
  (+ ~6 buffer-row callers of `scroll_to_item`: go-to-line, go-to-symbol, find, caret-follow), the content-width
  probe (app.rs:4818), `sticky_rows` (5397), AND **FIVE LSP overlay cards** each copy-pasting
  `(buffer_row - geom.first) * cell_h` with NO shared helper: hover (10583), completion (10657), signature
  (10741), rename (10805), hover-dwell inverse (10492). These cards MIS-POSITION when a fold sits above the
  caret on-screen and are NOT fixed by the rim projection.

**SCOPE DECISION — SPLIT (the Explore recommendation, adopted):**
- **#305 = SLICE 1 (this ticket) — the core fold engine, exactly the design above:** `fold_regions` +
  `FoldProjection` + the rim conversion + `scroll_to_item(slot_of)` + sticky's `geom.first`→buffer_row + the
  gutter chevron + the ⋯ inlay + auto-reveal + anchors + chords. This is CORRECT for the core interaction
  (fold/unfold, render, click-to-caret, scroll, sticky headers, navigation). Every mechanism exists; the
  interior premise holds.
- **The 5 LSP overlay cards (hover/completion/signature/rename/hover-dwell) are SCOPED OUT of #305** — a
  DOCUMENTED KNOWN-LIMIT: with a fold above the caret on-screen, those cards anchor at the wrong Y (they add
  `(buffer_row - geom.first) * cell_h`, un-projected). A bounded VISUAL imperfection, not a correctness break of
  editing/navigation. **Filed as a NEW follow-up ticket (Slice 2): thread a shared `visible_row(buffer_row)`
  projection through the frame-geometry/overlay layer.** Referenced in the #305 spec's Out + Phase 5 docs.
- The design's crossing table is AMENDED: the ~5-6 I named are Slice 1; the 5 overlay cards + the content-width
  probe are Slice 2. Slice 1's crossings (rim, scroll+its buffer-row callers, sticky geom.first, caret-follow)
  are the ones Phase 4's headless STATE drives must pin.

**Minor drifts folded in (Explore):** `scroll_editor_to_row` 11029→**11526**; `all_headers`/`collect_headers`
599→**602/616**; anchor API is `buffer.rs:223/236` (anchor.rs holds `Anchor`/`Bias`/`rebase_offset`); the scoped
count is a keymap.rs comment, not a standalone assert (so the roster guard is the `chords.len()==78`→80 assert +
the individual `.contains` — no `scoped==35` assert to bump, matching #304's pattern). `match_expression` is the
one net-new FOLD_KIND (the other 6 already used by is_header_kind/symbols); confirm it's a real grammar kind at
implement.

## Phase 3 — Implement

Built the full Slice-1 manifest; `cargo check --workspace` clean (no warnings), `cargo fmt`, `git add -N fold.rs`.

**What was built:**
- **`crates/syntax/src/fold.rs`** (NEW, pure) — `FoldRegion{header_row,end_row}`; `fold_regions(src)` (the FOLD_KINDS
  table via the ITERATIVE `collect_headers`-shape `TreeCursor` walk + the `.into_iter()`-over-Option parse idiom;
  multi-line only; document order); `FoldProjection::new(total, folded)` + `visible_count`/`buffer_row`/`slot_of`/
  `folded_headers` — the prefix-sum bijection over merged hidden intervals, total (saturating). The mutation
  surface.
- **`crates/syntax/src/lib.rs`** — `mod fold;` + `pub use fold::{fold_regions, FoldProjection, FoldRegion};`.
- **`crates/marley_app/src/app.rs`** — field `editor_folds: HashMap<PathBuf, Vec<Anchor>>` (per-file, anchor-keyed)
  + init. Methods (`mutants::skip`): `fold_projection` (IDENTITY + NO parse when nothing folded — the common,
  byte-identical path; else parse + resolve anchors + F1 re-derive validity + project), `region_at_caret`,
  `fold_at_caret`/`unfold_at_caret` (⌥⌘[/]), `remove_folds_at_row` (shared, `hidden_only` param), `reveal_caret_row`
  (auto-reveal), `fold_all`/`unfold_all` (palette), `toggle_fold_at_row` + `foldable_header_rows` (the gutter).
  Wired: the RIM (`uniform_list` count → `proj.visible_count()`, callback 1st line `let row = proj.buffer_row(slot)`);
  the ⋯-inlay tails (via the #331 channel); the crossings — `scroll_editor_to_row`→`slot_of`, sticky `geom.first`→
  `buffer_row`, auto-reveal in `follow_editor_caret` (now `&mut self`); the gutter-click fold-toggle in the row
  handler (a click left of `x0` on a foldable header toggles + returns before the caret logic — REQ-007, one
  handler, no stop_propagation needed); the 4 dispatch verbs; the language gate.
- **`crates/marley_app/src/keymap.rs`** — ⌥⌘[ → fold-at-caret, ⌥⌘] → unfold-at-caret (Editor-scoped); roster
  **78→80 / scoped 33→35**; individual `.contains` asserts; a `fold_chords_resolve_on_the_editor` unit.
- **`crates/marley_app/src/palette.rs` + `app.rs cockpit_commands`** — `CommandId(23)`→"fold-all", `CommandId(24)`→
  "unfold-all"; "Fold All"/"Unfold All" palette entries.

**Deviations from design (with reason):**
- **The gutter chevron is a click-region on the row handler, not a separate element** — a click left of the code
  origin `x0` on a foldable header toggles the fold, returning before the caret logic. One handler decides
  gutter-vs-text, so no `stop_propagation` juggling in a 'static closure (cleaner + the same REQ-007 behavior).
- **The visible chevron GLYPH (▾/▸) is deferred** (the design's "chevron pixel deferred") — the ⋯-tail shows folded
  state + the gutter-click toggles; a follow-up adds the glyph. The functional toggle IS wired (REQ-007 testable).
- **`fold_projection` recomputes on demand (no memo)** — but PARSES ONLY when folds are active (identity + no parse
  the rest of the time), so the common path is byte-identical to pre-#305. A per-(nonce,version) memo is a perf
  follow-up (or #349); Slice-1 correctness doesn't need it.
- **`follow_editor_caret` became `&mut self`** (for auto-reveal) — every caller compiled (all are `&mut self`
  contexts), so the shared-primitive hook (D-AUTO-REVEAL-ON-CARET) landed with no call-site churn.
- **The 5 LSP overlay cards + the content-width probe were NOT touched** — scoped to #352 (Slice 2 — SHIPPED
  2026-08-04, M28: the real set was NINE sites incl. the two #267 IME fns + the inlay window), the documented
  known-limit.

**Compile/test as-you-go:** `cargo check --workspace` clean, no warnings; `cargo nextest -p marley_syntax` → 51
passed (fold.rs compiles); `cargo nextest -p marley --lib -E 'test(/chord|roster|keymap|cockpit|palette|fold_chords|command_resolves/)'`
→ 43 passed (⌥⌘[/] resolve, roster 80/35 honest, Fold/Unfold All resolve). NO test expansion beyond compile +
the roster/keymap asserts — Phase 4 owns fold.rs's tables + the drives.

## Phase 3.5 — Inspect

My own independent verification (a throwaway integration test over the FoldProjection bijection + fold_regions,
since removed; `mutants --list`; clean-room grep; rustdoc) + 2 parallel general-purpose critics (fold.rs
correctness; the wiring/crossings/anchor/auto-reveal/gutter-click). **My verification: the pure heart is CORRECT.**
Critic reports reconciled below on return.

**E-1 — the pure model (my throwaway) — ALL PASSED:**
- **The bijection** (THE mutation surface): `FoldProjection::new(5,&[(1,3)])` → visible_count 3, buffer_row [0,1,4],
  slot_of(hidden 2,3)→header slot 1; **the invariant `buffer_row(slot_of(r))==r`** holds for every visible r
  (identity, nested (1,6)+(2,4)→merged, two-disjoint (1,3)+(4,6)→row-4-stays-visible). Identity when empty; a
  one-row fold hides nothing; out-of-range saturates (no panic); empty buffer no panic; a header-0 fold → no
  underflow (slot_of hidden→0).
- **fold_regions:** a one-line `fn` is NOT folded (multi-line only); struct/enum/trait/mod/match all fold; nested
  (mod + its inner fn) both present in DOCUMENT order; `match_expression` folds; empty/garbage/truncated → no panic.

**Mutation surface (verified):** fold.rs = **51 mutants** (the pure target — the bijection + fold_regions +
merge/prefix-sum math; `git add -N`'d, `A` in git status); app.rs fold shims = 0 (mutants::skip). Phase 4 must
kill the 51 (a thorough bijection + fold_regions table). **Hygiene:** no zed/warp/**FoldMap** in added lines (the
§20 wall holds — Zed's FoldMap concept-only, source unread); rustdoc `-D warnings` CLEAN on marley_syntax (no
#303 private-link trap).

**Wiring (self-verified by construction — reconciling with Critic 2 on return):** the rim (`visible_count` +
`buffer_row(slot)`); IDENTITY + no-parse when nothing folded (the early return in `fold_projection`); the F1
anchor re-derive (resolve → header row → keep only if a region still starts there); the crossings
(scroll→slot_of, sticky geom.first→buffer_row); the gutter-click toggle (returns before the caret logic).

**W-1 (my own finding — a REAL bug, HIGH, to FIX): auto-reveal is INCOMPLETE — the navigation jumps BYPASS it.**
Auto-reveal was hooked in `follow_editor_caret` (app.rs:11746), but the jumps — go-to-symbol (#304, app.rs:9549),
go-to-line (#302), go-to-definition (#312), find-next (#272), F8 (#290) — place the caret then call
`scroll_editor_to_row(row)` DIRECTLY (app.rs:3548/3582/6889/9549/10753/11865/12092), NOT `follow_editor_caret`.
So a jump INTO a folded body would strand the caret in a hidden row without unfolding (REQ-006 broken). **The
follow-the-shared-primitive discipline picked the WRONG primitive:** `scroll_editor_to_row` is the truly-shared
one (every jump AND `follow_editor_caret` call it). **FIX:** move the auto-reveal into `scroll_editor_to_row`
(&mut self) — reveal any fold hiding the target `row` (hidden_only) BEFORE computing `slot_of(row)` — and drop
the `reveal_caret_row` call from `follow_editor_caret` (now covered, since it scrolls to the caret row). This is
the #336 lesson recurring: hook the LOWEST shared primitive; a partial hook silently strands the feature on the
paths that bypass it. (Applied at inspect-fix after the critics land, to avoid clobbering their throwaways.)

**W-1 fix (designed, precise — the 5 bypass sites read + confirmed):** the bypass sites that place a
caret/cursor/selection then scroll DIRECTLY are `goto_commit` (3548, caret at target), `jump_to_file_symbol`
(9549, caret at target), `⌘D add-next` (6889, the ADDED cursor's row — not the primary), `consume_pending_center`
(10753, the #312 go-to-def deferred-scroll target = the caret), `find-next` (12092, the match selection). The ONE
scroll that must NOT reveal: `goto_preview` (3582) — its caret stays at the ORIGIN and it fires per-keystroke, so
revealing there would unfold unrelated regions as you type a line number. **FIX = a new shim
`reveal_and_scroll_to_row(&mut self, row)` = `remove_folds_at_row(path, row, /*hidden_only*/ true)` +
`scroll_editor_to_row(row)`** (the general row-reveal primitive `remove_folds_at_row` already exists — 9659 —
and `reveal_caret_row` is just it applied to the caret row). Wire it at the 5 jump sites; leave `goto_preview`
and `follow_editor_caret`'s internal scroll (11865, already reveals via `reveal_caret_row`) unchanged. All 5
sites are `&mut self` with `row` a plain `usize` (the `active_editor()` borrow already ended) → no borrow
conflict. A new headless drive (jump into a folded body → the fold reveals, the caret is on a VISIBLE slot)
pins REQ-006 for the jump paths at Phase 4.

**E-2 — Critic 1 (fold.rs bijection correctness) COMPLETE — NO defects; CORROBORATES my throwaway.** 9 throwaway
tests (deleted, tree restored to the 165-line original, `A` intent-to-add intact — no `git checkout`): the
bijection + reverse round-trip `slot_of(buffer_row(s))==s`; nested `[(1,6),(2,4)]`→merged `[(2,6)]`; the
merge-ADJACENCY guard `[(1,3),(4,6)]`→`[(2,3),(5,6)]` NOT merged (row 4 the 2nd header stays visible — asserted
the private `hidden` field directly); identity; the one-row `(2,2)` filter; out-of-range saturation (no panic);
`new(0,…)` no panic; header-0 `(0,3)`→no `hs-1` underflow; the 7 kinds each fold multi-line, `match_expression`
is the REAL grammar kind (empirically), 7 regions in preorder, one-line `fn` not folded, `""`/garbage/truncated
no panic; the multi-line filter double-guarded (fold.rs:61 AND :100). **Critic 1's non-defect note (for Phase 4):**
`buffer_row(slot)` for an OUT-OF-RANGE slot (≥ visible_count) can return a *hidden* row when the last rows are
folded (`new(5,&[(2,4)]).buffer_row(3)`→4, a hidden row) — documented saturation, callers only pass valid slots
`0..visible_count`, no panic. Not a break; the truth table will note the valid-slot domain.

**E-3 — Critic 2 (wiring/crossings/anchor) COMPLETE — CORROBORATES W-1 as [HIGH]; [1][2][3][5][7] CONFIRMED
sound.** It independently found the SAME bypass and added precision: go-to-def (F12) / ⌘T / ⌃- / diagnostics-jump
ALL funnel through `open_and_place_caret` → park `pending_center_row` → `consume_pending_center` (10753), so ONE
reveal there covers that whole family (I'd traced this too). Confirmed sound, each with a concrete check: **[1]**
the rim — count `visible_count()` (4866), callback 1st line `buffer_row(slot)` (4872), every interior site reads
buffer-row `row` (line_start/line_text/einlay/syntax_cache), `slot` only at the rim; IDENTITY early-return (9564,
no `fold_regions` call, byte-identical to pre-#305, no per-frame parse). **[2]** the F1 re-derive correct AND
panic-safe (`resolve_anchor` clamps ≤ len_chars → `line_col` accepts [0,len_chars] → no OOB; a deleted region has
no `header_row` match → `filter_map` drops the fold; typing-above shifts the anchor → resolves to the new row →
the fresh parse finds the region → fold stays). **[3]** both outbound crossings convert (`scroll_editor_to_row`→
`slot_of`; sticky `buffer_row(geom.first)` before `sticky_rows` at 5435) — the 5 LSP cards + width-probe +
inlay-prefetch stay un-projected = the documented #352 known-limit, NOT silent. **[5]** the gutter click returns
before the caret logic (5258), window-absolute coords consistent, a non-foldable row falls through. **[7]** keymap
23/23 (roster 80/scoped 35 honest, `fold_chords_resolve`, cmd 23/24), fold.rs 51 mutants, app.rs fold shims 0,
no zed/warp/FoldMap, `.into_iter()` idiom.

**W-1 RECONCILED + FIXED.** Both critics + my read agree it is real & HIGH. **My trace caught a 4th bypass Critic
2 MISSED: `select_efind_current` (12092) — the FIND navigation (⌘G), which REQ-006 names explicitly ("find").**
⌘D `add-next-occurrence` (6889) scrolls to the ADDED cursor but is a multi-cursor gesture, NOT in REQ-006's
find/F8/goto/def/symbol list (and already carries a documented "known imperfection" about following the added
cursor) → OUT of scope, LEFT (revealing there = scope creep). **FIX (applied):** a new shim
`reveal_and_scroll_to_row(&mut self, row)` = `remove_folds_at_row(path, row, /*hidden_only*/ true)` +
`scroll_editor_to_row(row)` (app.rs:11849), wired at the 4 in-scope bypasses — `goto_commit` (3548),
`jump_to_file_symbol` (9549), `consume_pending_center` (10753, the parked F12/⌘T/⌃-/diag family), and
`select_efind_current` (12092, find). `goto_preview` (3582, caret-at-origin, per-keystroke) + `follow_editor_caret`
(11865, already reveals via `reveal_caret_row`) + ⌘D LEFT unchanged. Chose the explicit helper over Critic 2's
"replace with `follow_editor_caret`" because the find/⌘-family scroll targets a row that is NOT always the primary
caret's row (a multi-line find match; the parked def target) — revealing the EXACT scrolled row is robust; the
motion primitive `follow_editor_caret` reads the primary caret. `cargo check -p marley` CLEAN; the mutant surface
is UNCHANGED (the new shim + all fold shims = 0 in `mutants --list`, no `mutants::skip` detach; fold.rs still 51);
hygiene clean. Phase 4 adds a REQ-006 drive that jumps (go-to-line + find) INTO a folded body and asserts the fold
reveals + the caret lands on a VISIBLE slot — the exact scenario the design's verb-choice would have shipped green
past (Critic 2's "if the author reaches for an arrow motion the test passes and the bypass ships silently").

**W-2 [LOW] — PARKED as a follow-up (not a Slice-1 blocker; both critics agree):** `editor_folds` (app.rs:249) is
never cleared on file/tab close — a bounded anchor leak (one `Vec<Anchor>` per distinct-file-ever-folded) + a
stale-fold-on-reopen edge (reopening a path resolves the OLD anchors against the fresh buffer; the F1 re-derive
DROPS anchors not on a live region header, so a re-fold only if an old anchor happens to resolve onto a valid
header). No unbounded growth (`fold_at_caret`/`toggle`/`fold_all` all guard against dup pushes; the projection
merges). Mitigated by F1; file a forge follow-up at Phase 5 (clear `editor_folds` on close + on the path key change).

**Lenses covered:** fold.rs bijection/merge/extraction correctness (Critic 1 + my throwaway — no defects); the
wiring — rim/identity, F1 anchor re-derive, the 3 outbound crossings, auto-reveal completeness, the gutter click,
roster/mutants/clean-room (Critic 2 — 1 HIGH found & fixed, 1 LOW parked). **Verdict: Phase 3.5 PASS** — W-1 fixed
at source (no suppression), W-2 parked with rationale, the pure surface + wiring otherwise confirmed sound.

## Phase 4 — Validate

**Tests added (19 total): 13 fold.rs unit tables + 6 headless drives.**

**(A) `crates/syntax/src/fold.rs` `mod tests` — the PURE truth tables (THE mutation surface, all 47 mutants):**
- REQ-001 `fold_regions`: `fold_regions_all_seven_kinds_document_order` (the inspect-verified flat fixture →
  exactly `[(0,4)fn,(1,3)match,(5,7)impl,(8,10)mod,(11,13)trait,(14,16)struct,(17,19)enum]` — all 7 kinds,
  preorder, nesting kept); `one_line_not_folded`; `nested_fn_in_impl_both_impl_first`; `non_fold_kinds_excluded`;
  `totality_no_panic` (empty/garbage/truncated).
- REQ-002 `FoldProjection` bijection: `basic_bijection` (the full `new(5,&[(1,3)])` value table + `buffer_row∘slot_of`
  and the reverse round-trip); `nested_folds_merge`; `adjacent_folds_do_not_merge`; `identity_when_empty`;
  `one_row_fold_hides_nothing`; `out_of_range_saturates`; `zero_total_no_panic`; `header_row_zero_no_underflow`.

**(B) `crates/marley_app/src/headless_drive.rs` — 6 drives (all GREEN):**
- REQ-004/003 `fold_at_caret_hides_body_marker_and_unfold_restores` — ⌥⌘[ hides fn a's 3 body rows (visible 8→5),
  the `⋯` marker is `(header 1, 3 lines)`, ⌥⌘] restores exactly.
- REQ-010 `fold_at_caret_noop_on_non_rust` — a `.md` file has 0 foldable headers, ⌥⌘[ is a no-op (the caller gate).
- REQ-005 `fold_survives_edit_above_and_evaporates_on_region_delete` — typing a newline above carries the fold
  (header 1→2 via the anchor); deleting the header line evaporates it (the F1 re-derive → `folded_headers` empty).
- **REQ-006 `goto_line_into_folded_body_auto_reveals` + `find_match_inside_fold_auto_reveals` — the W-1 fix, BOTH
  jump paths: a go-to-line AND a find INTO a hidden body reveal the fold + land the caret on a VISIBLE row.**
- REQ-007 `gutter_toggle_folds_without_moving_the_caret` — `toggle_fold_at_row` folds/unfolds, caret unchanged.
- REQ-009 (Fold All / Unfold All palette resolve) — the Phase-3 `every_cockpit_command_resolves` unit (CommandId
  23/24). REQ-008 (scroll→slot / sticky→buffer_row crossings): the MATH is the fold.rs bijection tables; the
  `scroll_to_item` HANDLE + the sticky render are masked shims (pixel deferred — LIVE drives off-limits).

**Phase-4 fixes at source (§0):**
- **The EQUIVALENT-MUTANT fix (the load-bearing one).** The first mutation run left 2 fold.rs mutants missed; one —
  `slot_of` `he < br` → `he <= br` — was a TRUE EQUIVALENT mutant: with inclusive `[start..=end]` hidden runs, the
  boundary sits on the LAST hidden row, where "snap to header" and "count-the-interval-past" compute the identical
  value (`hs-1-prior`) → no test can distinguish them. Fixed by **switching `FoldProjection`'s internal runs to
  HALF-OPEN `[start, end)`**, which moves the boundary onto the first VISIBLE row after a fold, where snap-vs-count
  genuinely differ → `slot_of(4)==2` kills it. The public API + every value is unchanged (all 13 tests still green).
- The other missed mutant (`slot_of` `end - start` `-`→`/`, and later `(start-1) - hidden_before` `-`→`+`) needed
  `slot_of` asserts with a LARGE run (`3/2==1` and `hidden_before==0` hid the small/first-fold cases): added
  `slot_of(7/8/9)` on the merged `[2,7)` run + `slot_of(5/6)` on the SECOND fold of the adjacency case.
  **Re-run: fold.rs 47 mutants → 45 caught + 1 timeout(=caught) + 1 unviable, 0 MISSED (MSI 100).**
- **The drive-hang lesson (harness-safe order).** The first drive run HUNG with no output — a failed assert BEFORE
  `reap_sessions` panics on the PTY drop chain instead of reporting (the module NOTE). Killed the hung test-binary
  process GROUP (`marley_app-…`, not `marley-…`; safe — nextest, not an in-flight `cargo mutants`), then
  restructured EVERY drive to **collect all observations → `reap_sessions` → assert**. The re-run then REPORTED the
  real failure: the find drive needed an Enter (typing only refreshes matches; `select_efind_current` fires on the
  ENTER match-navigation at app.rs:2175) — added `enter`, GREEN.

- **Gate: `scripts/gates.sh --diff` → GATE GREEN [diff], 15 passed / 0 failed.** gate:3 nextest (incl. the 6
  new drives) + doctests green; **gate:4 coverage ≥ 100% lines** (fold.rs fully covered by the 13 tables; app.rs
  coverage-excluded); **gate:5 mutation MSI ≥ 100%** (fold.rs 47 mutants 0-missed; the fold app.rs shims carry
  `mutants::skip`; keymap/palette #305 additions killed by the roster/`fold_chords`/`every_cockpit_command_resolves`
  units); gate:6 miri, gate:14 docs (rustdoc -D warnings clean — no private intra-doc-link), gate:15 visual all
  green. Receipt `gate_state_hash 369e18ee8985b165e3946b052316c6ab4b477702` == `.git/ignibyte-gate-receipt`
  (commit-valid). **No pre-existing failures; nothing excluded.**
- **LIVE-drive deferral (chad at the machine — units+headless+mechanism):** the gutter chevron ▾/▸ glyph, the `⋯`
  EOL pixel, and the `scroll_to_item` HANDLE scroll are the ONLY deferred surfaces (deferred-not-skipped) — the
  fold/projection/anchor/auto-reveal/scroll STATE is headless-proven by the 6 drives + the fold.rs bijection
  tables; the crossing math is the fold.rs unit surface.

**Phase 4 — Validate PASS.**

## Phase 5 — Complete
- Docs updated; AAR capture (lessons / failures / prevention rules / ADs); archive.
