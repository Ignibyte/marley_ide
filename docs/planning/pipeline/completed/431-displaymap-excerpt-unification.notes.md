# 431-displaymap-excerpt-unification — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-431-displaymap-excerpt-unification.md
- **Pipeline spec:** 431-displaymap-excerpt-unification.spec.md

## Phase 1 — Plan

- **Request:** TICKET-431 — DisplayMap excerpt unification, first of the M33 tail
  (shelf: `docs/planning/design-notes/m33-tail-and-wedge-shelf.md`; the B-c chain's
  recorded "later prize", pinned in display_map.rs:8-9's own module doc). Systems:
  `marley_app` display_map + multibuffer + the app.rs mb rim. Drafted at HEAD 52a4c4d.
- **Classification / tier:** feature (refactor-shaped), M–L, ONE shippable slice by
  default — both halves are byte-identical so nothing forces a split. IF Phase 2 finds
  the diff too large, the split line is exactly: (a) excerpt stage lands in the facade +
  the app.rs rim re-routes (model intact, stage delegates), then (b) the model shrinks
  (slot math re-homes, delegation deleted). Never split anywhere else — D2's one-home
  rule holds only at a slice boundary, not inside one.
- **Recall (knowledge codes actually read, with what each contributes):**
  - `AD-claude-425-display-map-facade-and-typed-row-spaces-001` — ONE facade owns every
    row-space conversion; #427's excerpt base was explicitly recorded as the deferred
    unification. This ticket cashes that record.
  - `AD-claude-426-soft-wrap-is-a-facade-layer-on-cell-arithmetic-001` — the
    layer-insertion pattern (Arc'd memoized index; crossing sites untouched when a layer
    lands); "the #427+ excerpt base slots UNDER folds in the same facade" is written
    there — the stage-ordering prior for Phase 2.
  - `AD-claude-427-multibuffer-own-row-model-snapshot-origin-001` — the deliberate
    own-row-model decision this ticket RETIRES; its snapshot/origin semantics must
    survive the move.
  - `AD-claude-430-diagnostics-refresh-gates-on-a-publish-epoch-not-row-counts-001` —
    Note is a REAL SLOT (uniform_list demands uniform heights; in-slot stacking was a
    design dead end) — the excerpt stage must mint Note slots identically.
  - `AD-claude-half-open-intervals-kill-the-equivalent-boundary-mutant-001` — any new
    prefix-sum/snap projection stores half-open runs from the start (MSI-100 floor).
  - `L-claude-425-type-the-seam-and-the-compiler-finishes-your-site-sweep-001` — the
    manifest below only needs to be good enough to START; flipping the shared fields to
    `DisplayRow` makes the compiler the exhaustive enumerator.
  - `L-claude-430-a-a-closure-can-hide-a-dead-arm-from-coverage-001` — the forward-only
    normalize invariant (a non-empty slot list always ends with a Line) and its
    closure/coverage trap: when moving `refresh_cum`'s normalize, keep the invariant
    comment + the plain-loop form.
  - `BF-claude-geom-recorder-buffer-row-eq-slot-gate-001` (F-#352) — the row-vs-slot
    equality class the typed rim kills; the mb rim is today's untyped second surface.
  - `BF-claude-fold-projection-parse-on-pump-tick-001` — price construction on its
    path's cadence: a `DisplayMap` may be built at every crossing, the INDEX must not be
    (WrapIndex's Arc discipline; the excerpt stage follows it — built at refresh_cum
    time, shared after).
  - `PR-137` — after the move, grep the ORIGINAL pattern with ANY receiver; and the
    uniform_list sizer + 'static row closure must resolve the model the same way.
  - `PR-1370` — equivalence tables need discriminating fixtures (≥3 out-of-order, every
    boundary class), not minimal ones.
  - `PR-1691` — slot math becomes ONE function/home, never a delegating copy left beside
    the moved original (the D2 clause).
- **Completed-pipeline recall:** `completed/425-display-map-foundation.{spec,notes}.md` —
  the byte-identical-by-proof recipe this spec clones (equivalence tables + suite
  unchanged + grep audit + trybuild + live smoke), and the quality bar.
  `completed/427-multibuffer-core.notes.md` — the mb row grammar's design provenance
  (prefix-sum locate "the WrapIndex shape", selection in the model, the render rim).
  `completed/430-problems-multibuffer.*` (via ledger + code) — Note slots + the
  normalize. #426's notes via its AD — the layer-insertion shape.
- **Discovery (what a designer must verify, file:line at HEAD 52a4c4d):**
  - **The facade** — `crates/marley_app/src/display_map.rs`: `DisplayMap { folds, wrap:
    Option<Arc<WrapIndex>> }` :66-69; `WrapIndex { lines, cum }` :34-41 (the prefix-sum
    precedent); methods `visible_count` :97, `locate` :107 (TOTAL, saturating, returns
    `(BufferRow, seg)`), `buffer_row` :128, `slot_of` :137, `slot_at` :147,
    `segs_of_row` :171, `folded_headers` :182, `viewport_offset_at` :196. `pub(crate)`;
    the editor instance is built per crossing by `RootView::display_map()`
    app.rs:15866-15874 over the memo. NOTE the contract MISMATCH the design must not
    paper over: facade `locate` is total; mb `locate` is `Option` (`None` only when
    empty) — the excerpt stage keeps the mb contract (REQ-001 pins it).
  - **The mb model** — `crates/marley_app/src/multibuffer.rs`: `Row::{Header(fi),
    Note(fi,mi), Line(fi,li)}` :84-93; private `slots: Vec<Row>` :153; `total_rows`
    :162; `locate` :168 (saturates to last slot; `None` iff empty); `line_at` :178;
    `move_selection` :190 (skips non-Line, clamps, `wrapping_sub` underflow arm);
    `refresh_cum` :504 (the slot MINT: Header, then per line Note-for-each-noted-match-
    with-content, then Line; re-clamp + FORWARD-ONLY normalize :537-546); `slot_of_row`
    :551. `selected: usize` :113 (raw slot — the typing prize). Untouched by design:
    `build`/`build_with_meta` :244/:260, `rebuild_lines` :419, anchors/journal/batches
    fields :118-148, `footer_summary` :212.
  - **The app.rs rim (the REQ-003 audit set):** `multibuffer_body` :6406 — sizer
    `total + 1` :6436 (footer is the trailing slot), footer compare `slot ==
    model.total_rows()` :6452, caret `slot_of_row` :6449, the `locate` match :6469
    (Note :6474 / Header :6515 / Line :6569 arms); empty-materialize guards
    `total_rows() == 0` :12853 (search) and :13055 (problems); `handle_multibuffer_key`
    :13166 — `move_selection` :13182 + `selected` → `scroll_to_item` :13186-13190;
    `active_mb_id` :13223; `mb_place_caret` :13537 — `locate` :13542; frame sync
    `refresh_cum` :14147; `multibuffer_jump_selected` :14158. The editor-instance
    `map.locate` calls at :7218/:17578/:19853 are the OTHER stack — untouched.
  - **Other consumers:** content.rs :66/:212/:225 (`Content::Multibuffer(Box<model>)` +
    accessors — the model's home stays); editor_problems.rs :115/:149 (build-side
    producer, no slot math — out of the rim set); headless_drive.rs (9 public-API hits —
    the REQ-002 "tests move with code" watch list); multibuffer.rs's own `mod tests`
    (locate/mover/refresh tests — the re-home set if the model shrinks).
  - **Typing ground:** `marley_text_offsets` owns `BufferRow`/`DisplayRow` + the standing
    trybuild suite (`tests/ui/mixed_rows_fail.rs` + `.stderr`) — REQ-004's home if a new
    mix becomes representable. Row's `fi`/`li`/`mi` are model INDICES, not row spaces —
    they stay `usize`.
- **Decisions:** D1 byte-identical by proof (#425 recipe; editor instance bit-for-bit);
  D2 public mb behavior stays, consumers move to the facade, the model may shrink with
  ONE home for slot math; D3 no behavior riders. Full text in the spec.
- **Prior-art verdicts (§20, three legs):**
  1. **Zed behavior** (subsystems/03-editor-multibuffer.md §3+§6, research only, no Zed
     source): excerpts sit at the stack's BASE (the multibuffer IS the text model; the
     DisplayMap on top is excerpt-count-indifferent); headers are BLOCK rows a layer
     inserts — projection-minted non-document slots, i.e. Marley's Header/Note at bigger
     shape; the singleton unification is the endgame and is explicitly Out here. Adopt
     the one-authority + inserted-rows BEHAVIOR, not SumTree/sync machinery.
  2. **Published:** injected non-document rows in a view-model line map are standard
     editor architecture (VS Code view-model lines, CodeMirror 6 block widgets);
     prefix-sum + binary-search is public CS. Nothing to import — in-tree already
     implements the pattern twice; the ticket deletes the "twice".
  3. **Permissive deps:** gpui 0.2.2 `uniform_list`/`scroll_to_item` rim is `usize`
     (the #425 registry read; unchanged) and gpui owns no slot-projection helper; ropey
     owns line ground truth only; no sum-tree crate in the permissive set. Verdict: the
     seam is Marley-original — `WrapIndex.cum`, `FoldProjection`, and the mb `slots`
     mint are the art being unified.

## Phase 2 — Design

Designed at HEAD 53ab523 (the mint commit — planning-only over 52a4c4d, so every
Phase-1 file:line stands; re-verified by reading display_map.rs + multibuffer.rs whole
and every app.rs rim range live).

### Architecture (the stage shape — the P2 forks, decided)

- **D-SHAPE — third arm on the ONE facade struct, stages disjoint this ticket.**
  `DisplayMap` gains `excerpt: Option<Arc<ExcerptIndex>>`. Every existing constructor
  (`new`/`with_wrap`/`identity`) sets `None` — the editor instance is bit-for-bit
  (REQ-005; the derived `PartialEq` extends by an always-`None`-on-editor-paths field).
  New constructor `DisplayMap::excerpts(Arc<ExcerptIndex>)` builds the mb instance:
  `{ folds: FoldProjection::new(0, &[]), wrap: None, excerpt: Some(idx) }`. Editor
  methods never consult `excerpt`; the `excerpt_*` doors never consult folds/wrap —
  DISJOINT stages behind one type. The identity fold slot + `wrap: None` in the mb
  instance are not warts: they are exactly where the wedge features (folds/wrap INSIDE
  excerpts) will compose, facade-internally, instead of an app.rs rewrite. A stage
  ENUM was rejected: it forces the editor arms through a new match (bit-for-bit risk)
  and buys nothing until stages actually compose.
- **D-INDEX — `ExcerptIndex` in display_map.rs is THE home for slot math** (Arc'd —
  BF-claude-fold-projection-parse-on-pump-tick: built at `refresh_cum` time only,
  shared per crossing after):
  `ExcerptIndex { slots: Vec<Row>, line_rows: Vec<Vec<usize>> }` — `slots` is the
  #427/#430 mint moved VERBATIM (Header; per line: Note for each in-meta note index;
  Line); `line_rows[fi]` snapshots each file's excerpt-line rows at mint so
  `slot_of_row` is self-contained (a data snapshot, not a second derivation —
  PR-1691 clean; the WrapIndex `lines` precedent). Pure methods, all moved verbatim:
  `total()`, `locate(DisplayRow) -> Option<Row>` (`None` iff empty; past-end saturates
  `min(total-1)` — the last slot is always a Line by the normalize invariant),
  `slot_of_row(fi: usize, row: usize) -> Option<DisplayRow>`,
  `move_selection(DisplayRow, down: bool) -> DisplayRow` (PURE form of the mutating
  loop — identical clamp/skip/`wrapping_sub`-underflow semantics; answers the input
  unchanged when no Line is found), `normalize_selection(DisplayRow) -> DisplayRow`
  (clamp + FORWARD-only scan, plain-loop form + invariant comment kept —
  L-claude-430's closure/coverage trap).
  `DisplayMap` rim doors delegate to the arm: `excerpt_total` (0 absent),
  `excerpt_locate` (`None` absent), `excerpt_slot_of_row` (`None` absent),
  `excerpt_move_selection` (input unchanged when absent). The doors are the facade's
  one public face (a facade door is not a PR-1691 "delegating copy" — the math has
  exactly one body, in the stage).
- **D-MODEL — the shrink.** `MultibufferModel.slots: Vec<Row>` becomes private
  `excerpt: Arc<ExcerptIndex>`. DELETED from the model (one home; delegating copies
  banned by D2): `total_rows`, `locate`, `move_selection`, `slot_of_row`. KEPT:
  `refresh_cum` — the one rebuild door, now `self.excerpt =
  Arc::new(ExcerptIndex::new(&self.files)); self.selected =
  self.excerpt.normalize_selection(self.selected)` (mint BEFORE normalize, as today);
  `line_at(DisplayRow)` — a files-BORROW adapter (payload access over the index's
  locate, not slot math — the jump's shape survives); `footer_summary` and all
  build/rebuild/anchor/journal/batch surface — untouched. NEW:
  `display_map(&self) -> DisplayMap` = `DisplayMap::excerpts(self.excerpt.clone())` —
  the mb twin of `RootView::display_map()` (a map per crossing, the INDEX never).
  `Row` STAYS in multibuffer.rs: `fi`/`li`/`mi` are model indices — "DisplayRow in,
  model indices out" (REQ-004's own carve-out).
- **D-TYPE — the typed rim.** `selected: usize` → `pub selected: DisplayRow`
  (offset_newtype: `from`/`as_usize`/`zero`/Ord/arith all minted). Conversions happen
  ONCE at the gpui boundary: `for raw in range { let slot = DisplayRow::from(raw); …}`,
  `scroll_to_item(sel.as_usize(), …)`, sizer/footer-compare stay raw `usize` COUNTS
  (the facade's standing count-vs-coordinate discipline). `mb_place_caret(mb,
  slot: DisplayRow)`. Flipping the field makes the compiler enumerate every straggler
  (L-claude-425) — the REQ-003 sweep is compiler-enforced because the model methods
  are GONE, with the grep audit as the recorded receipt.
- **D-CYCLE — accepted in-crate module cycle**, documented in both module docs:
  display_map.rs imports `multibuffer::{FileExcerpts, Row}` (the stage consumes the
  model's group vocabulary); multibuffer.rs imports `display_map::ExcerptIndex` (the
  model memoizes the stage). Same-crate, legal, and honest about the seam.
- **§20 confirmed:** matches the Zed-deconstruction BEHAVIOR only (one authority owns
  every slot↔row conversion; header/band rows are projection-inserted slots; layers
  compose behind one facade; excerpt-count-indifference is the endgame, deferred). No
  Zed source read or to be read. React-first stays N/A — no UI delta.
- **One slice, confirmed** (the Phase-1 split line stays unused): the stage + rim +
  shrink form one compile unit; splitting would ship the delegating-copy state D2 bans.

### File manifest (crates/*/src only — React N/A)

- `crates/marley_app/src/display_map.rs` — +`ExcerptIndex` (struct + mint + the five
  pure methods, moved verbatim); +`excerpt` arm on `DisplayMap` (+`excerpts()`
  constructor + four `excerpt_*` doors); module doc: the deferred-prize line RETIRES,
  the stage + cycle documented. New stage tests (oracle property + tables).
- `crates/marley_app/src/multibuffer.rs` — `slots` field → `excerpt: Arc<ExcerptIndex>`;
  `refresh_cum` re-minted as above; `locate`/`total_rows`/`move_selection`/
  `slot_of_row` DELETED; `line_at` re-seated over the index, `selected: DisplayRow`;
  `display_map()` door; module doc updated. Existing slot-math tests adapted in place
  to the facade doors (assertion VALUES verbatim).
- `crates/marley_app/src/app.rs` — the rim re-route, all compiler-led:
  `multibuffer_body` (:6417 total → `map.excerpt_total()`; :6449 caret
  `map.excerpt_slot_of_row` with `(DisplayRow, usize)` caret_pos; :6469 locate match →
  `map.excerpt_locate(slot)`; :6591 selected compare typed; :6636 click passes typed
  slot); empty guards :12853/:13055 → `display_map().excerpt_total() == 0`; mover
  :13182-:13188 → `m.selected = m.display_map().excerpt_move_selection(m.selected,
  down)` + `as_usize` at scroll; `mb_place_caret` :13537 typed param + facade locate;
  :14147 `refresh_cum` unchanged; :14164 `line_at(m.selected)` unchanged in shape.
  Editor-stack `map.locate` (:7218/:17578/:19853) untouched.
- `crates/marley_app/src/headless_drive.rs` — 7 drive-test sites (:15214, :16037-:16247)
  adapt call shape to the facade doors; assertion values verbatim (REQ-002 watch list).
- `crates/marley_app/src/editor_problems.rs` — :317 one test-side `total_rows()` adapt.
- NOT touched: content.rs (accessors stay), `marley_text_offsets` (existing
  `mixed_rows_fail` pin covers the axis — no new mix becomes representable; REQ-004
  else-arm, to be re-verified honestly at inspect).

### Regression test plan (≥1 row per REQ)

| REQ | Test | Where / how |
|---|---|---|
| REQ-001 | `excerpt_stage_equals_the_427_oracle` — property: `ExcerptIndex::new(files).slots == oracle_slots(files)` where `oracle_slots` is the #427/#430 refresh_cum mint carried VERBATIM as the test oracle (the direct model is deleted, so the old derivation lives on as the oracle — the #425 equivalence recipe adapted to a consuming refactor) | display_map.rs tests; fixtures via `build`/`build_with_meta`: multi-file, Note-bearing + noteless, EMPTY, adjacent bands, ≥3 out-of-order windows (PR-1370), post-refresh reshapes (truncate/clear then refresh) |
| REQ-001 | locate/move/normalize/slot_of_row TABLE tests — the re-homed #427/#430 verbatim values (Header/Note/Line at every boundary slot, None-iff-empty, past-end saturation probed ≥2 past range, mover skip/clamp/underflow, forward-only normalize, reclamp) | display_map.rs stage tests + multibuffer.rs adapted tests |
| REQ-002 | full `cargo nextest run --workspace` green, zero behavioral edits; every existing-test hunk classified move-only at inspect | validate + inspect diff audit |
| REQ-003 | model slot methods DELETED (compiler-enforced absence) + grep audit `\.locate(\|slot_of_row(\|\.total_rows()\|move_selection(` — classify the editor-stack `map.locate` sites and `command_bar::move_selection` (a name-collision free function) as other-stack | inspect + re-run at validate |
| REQ-004 | `selected: DisplayRow` + typed doors compile; NO new trybuild case — the existing `mixed_rows_fail` pin covers BufferRow↔DisplayRow, and no new signature takes two row spaces (recorded honestly; inspect re-checks) | marley_text_offsets standing suite in the REQ-002 run |
| REQ-005 | display_map.rs editor tests byte-untouched; editor render/geometry drives pass unchanged | the REQ-002 run + inspect diff audit (zero editor-path assertion edits) |
| REQ-006 | live smoke: launch, materialize search mb + problems mb, drive ↑/↓/Enter/click-caret/jump vs the #427/#430 captures; `scripts/gates.sh --diff` green (cov 100 / MSI 100 on the new pure surface) | validate |

Uncoverable: none new — `multibuffer_body` stays the standing `mutants::skip` render
shim; every new line is pure in display_map.rs/multibuffer.rs.

### Risks (reversible but load-bearing)

- **R1 — the pure mover.** `move_selection`'s pure form must keep the exact
  "no Line found → input unchanged" arm and the `wrapping_sub` underflow clamp; the
  re-homed mover/boundary tables pin both.
- **R2 — model equality.** `Arc<ExcerptIndex>` derives content `PartialEq` (ptr fast
  path, then deref) — model equality semantics preserved for `replace_mb_model` etc.
- **R3 — mint-before-normalize order** in `refresh_cum` preserved (normalize consults
  the fresh slots), or the reclamp reads stale state.
- **R4 — the mb map's identity folds slot** documented as the composition landing zone,
  not dead weight (module doc), so a later cleanup doesn't "simplify" the shape away.
- **R5 — the module cycle** is deliberate and documented; a future crate split would
  need `Row`/`FileExcerpts` to move INTO the facade's module first.

## Phase 3 — Implement

React-first: N/A (no UI delta — the spec's parity section; nothing to build in the POC).

Built exactly to the manifest:

- **display_map.rs** — `ExcerptIndex { slots, line_rows }` with the mint moved verbatim
  (the #430 in-meta guard + comment intact) and the five pure methods; `locate` keeps the
  mb contract (None iff empty, past-end `min(total-1)` saturation); `move_selection` is
  the pure form of the #427 mutating loop — clamp-into-surface, skip non-Line,
  `wrapping_sub` underflow → clamped, and the no-Line-found arm answers the INPUT
  unchanged (the original's semantics: `selected` was only assigned on a found Line);
  `normalize_selection` carries the #430 forward-only comment + plain-loop form
  (L-claude-430). `DisplayMap` gained the `excerpt` arm (all three editor constructors
  set `None`), `excerpts()`, and the four `excerpt_*` doors. Module doc: the deferred
  prize retired, the disjoint-stages + composition-landing-zone + cycle notes added.
- **multibuffer.rs** — `slots` → private `excerpt: Arc<ExcerptIndex>`;
  `selected: DisplayRow`; `total_rows`/`locate`/`move_selection`/`slot_of_row` DELETED
  (one home); `refresh_cum` = re-mint Arc + `normalize_selection` (mint-before-normalize
  kept, R3); `line_at(DisplayRow)` re-seated over the index (borrow adapter);
  `display_map()` door added. build inits `DisplayRow::zero()` + an empty index.
- **app.rs** — the rim re-route as designed: `multibuffer_body` builds ONE map per body
  + ONE per frame closure (Arc clone; index untouched — BF discipline), caret_pos typed
  `(DisplayRow, usize)`, footer compare vs `map.excerpt_total()`, the ONE rim conversion
  `let slot = DisplayRow::from(slot)` above the locate match, selected compare types
  through; empty guards ×2 via the door; the mover assigns
  `excerpt_move_selection` + `as_usize` at `scroll_to_item`; `mb_place_caret` takes
  `DisplayRow` + locates through the door; jump's `line_at(m.selected)` unchanged in
  shape. Editor-stack `map.locate` sites untouched.
- **headless_drive.rs / editor_problems.rs** — test adapts, assertion VALUES verbatim.

Deviations from the Phase-1 manifest (not from design — the design predicted the
mechanism): the compiler sweep (selected/`mb_place_caret` type flips) surfaced **11
additional headless_drive sites** the Phase-1 grep hadn't listed — 9 literal
`mb_place_caret(mb, N)` calls and 2 `m.selected`/`total_rows` asserts — exactly the
L-claude-425 effect the design counted on; all adapted mechanically. Test-side door
shims `tot/loc/sor/mv` added in multibuffer.rs `mod tests` so the pinned assertion
values stay verbatim (the inspect move-only audit should read them as the adaptation
layer, not new behavior).

Checks: `cargo check --workspace --tests` green; `cargo clippy --workspace
--all-targets -D warnings` green; `cargo fmt --all` applied. (The `block v0.1.6`
future-incompat cargo note is pre-existing upstream/transitive — untouched by this
change.)

## Inspect (Phase 3.5)

Four independent critics over the full diff — correctness, security/secrets/
provenance, data/state integrity, simplification — plus the lead's own audits
(REQ-003 grep, REQ-002 value audit, REQ-004 honesty check, D2 one-home). Unanimous
top-line: **byte-identical, no high findings**. The ledger:

1. **[medium · simplification] `normalize_selection` re-implemented `move_selection`'s
   forward Line-scan** 20 lines apart in the stage — the re-home co-located a
   duplication the old code kept in separate methods; the ticket's own governing rule
   (PR-1691) flags it. VERDICT: real → **FIXED**: normalize now delegates its non-Line
   arm to `move_selection(clamped, down=true)` (provable equivalence: inclusive scan
   from a known-non-Line slot ≡ scan-from-next; both keep the clamp on exhaustion;
   empty short-circuits to zero). 37/37 scoped tests green post-fix. Recorded as a
   DESIGN DEVIATION: Phase 2 said "plain-loop form kept" — superseded by this find;
   the forward-only invariant COMMENT stays, and the delegate is itself the plain loop,
   so the L-claude-430 closure/dead-arm trap stays satisfied.
2. **[low · correctness] `locate` doc overclaimed** "last slot is always a Line" — a
   rebuild-emptied trailing group leaves a Header (behavior identical pre/post; the
   keep-clamp arm covers it). VERDICT: real, doc-only → **FIXED** (softened + the
   state-integrity critic's misattribution nit folded in: the invariant is the MINT
   shape, not normalize).
3. **[low · simplification] `move_selection` doc ghost** "`dir` (+1/-1)" vs the real
   `down: bool` — pre-existing drift the move carried. VERDICT: real, doc-only →
   **FIXED**.
4. **[low · correctness] `slot_of_row`'s lookup moved to the mint-time `line_rows`
   snapshot** — a LATENT divergence hazard if a future mutation skips `refresh_cum`
   (today unobservable: the one production mutation refreshes in the same borrow; the
   new form is MORE internally consistent than the old live/stale cross-match).
   VERDICT: accepted-with-doc → **FIXED** by adding the PAIRING INVARIANT to the
   `excerpt` field doc (any files/lines/match_meta mutation → `refresh_cum` in the
   same borrow).
5. **[low] dead `Arc` allocation at `build_with_meta` init** (placeholder index
   overwritten by the immediate `refresh_cum`). VERDICT: **rejected for change** — the
   shape preserves exactly ONE mint+normalize composition site (`refresh_cum`);
   restructuring would fork it into a second site to save one tiny allocation per user
   gesture. Both critics independently called keeping it defensible.
6. **[info · state-integrity] `mb_place_caret` stores the unclamped slot** after a
   saturating locate — PRE-EXISTING, byte-identical to HEAD, unreachable from
   production inputs (render click slots come from the list range; every downstream
   consumer clamps/saturates). Out per D3 no-riders; flagged as a hardening candidate
   for #432's design (it touches `mb_place_caret` anyway).
7. **[info] REQ-004 recorded honestly**: `excerpt_slot_of_row(fi: usize, row: usize)` —
   `row` is the mb model's file-row index (`ExcerptLine.row`, usize since #427 — the
   spec's model-indices carve-out). No NEW two-row-space signature became
   representable; the standing `mixed_rows_fail` trybuild pin covers the
   BufferRow↔DisplayRow axis; **no new trybuild case needed**.
8. **[info] `refresh_cum` name fossil** (no cumulative sums remain) — D2 pins the
   model's public API; recorded for the wedge batch, not renamed here. Likewise
   `line_at`'s door-bypass via `self.excerpt.locate` is the designed borrow adapter
   (D-MODEL), noted, kept.
9. **Provenance: CLEAN** — line-for-line hunk correspondence with the deleted model
   code; new names follow in-tree grammar (`WrapIndex` family); zero Zed mechanism
   vocabulary in added lines; the only Zed channel is the behavioral deconstruction
   doc. Secrets/unsafe/transmute/spawn/new-allows/test-deletion: all CLEAN. Editor
   stack (REQ-005): editor methods never read `excerpt`; zero editor-test edits.
10. **REQ-002 value audit (both the correctness critic's full walk and the lead's
    edit-time check): every adapted assertion VALUE verbatim**; the one structural
    rewrite (`slots.iter().all` → door-probe loop) is value-equivalent. **REQ-003
    grep: PASS** — classifications: editor-stack `map.locate` ×3 (other stack),
    `command_bar::move_selection` (name-collision free function), `line_at`'s
    internal index read (designed), test shims/doors (correct form).

**Ledger appends:** no `F-` (no behavior bug — the medium find is behavior-correct
duplication); no new `PR-` (the class IS PR-1691 — this is an instance caught by the
standing rule, recorded here).

## Phase 4 — Validate

**New tests (display_map.rs, both green):**
- `excerpt_stage_equals_the_427_oracle` (REQ-001) — `oracle_slots` is the #427/#430
  refresh_cum mint carried VERBATIM as the in-test oracle (the direct model is deleted;
  the old derivation lives on as the equivalence reference — the #425 recipe adapted to
  a consuming refactor). Sweeps slots/total/locate/slot_of_row/move/normalize against
  oracle-derived answers over: a multi-file search build with ≥3 out-of-order windows
  (PR-1370), a problems build with adjacent bands + a note-less file + a past-EOF match
  whose meta drops, the EMPTY model, and post-refresh reshapes (truncated group; a
  TRAILING group emptied — the inspect edge). Every slot probed to total+2; movers both
  directions from every start incl. past-end.
- `excerpt_stage_keep_clamp_and_online_arms` (REQ-001) — the delegation arms the sweep
  can't isolate: an all-non-Line surface ([Header] only) keep-clamps mover + normalize
  (input-unchanged pinned), the on-Line normalize short-circuit, and the four doors'
  absent-arm defaults on an editor (stage-less) map.

**Suite runs (real, in transcript):** scoped `display_map::` 8/8; full
`cargo nextest run --workspace` **2244/2244 PASS** (7 pre-existing skips) +
`cargo test --workspace --doc` clean. REQ-002 (zero behavioral edits), REQ-004 (the
standing `mixed_rows_fail` pin ran in-suite; no new mix — inspect item 7), REQ-005
(editor display_map tests byte-untouched, green) all covered by the run.

**Live smoke (REQ-006, search surface — real pixels, bundled app):** captures in the
session scratchpad (`431-live-*.png`), all READ:
- `431-live-overlay6.png` — ⌘⇧F overlay streaming "excerpt_locate · 9 matches in
  5 files" grouped per file (the LIVE uncommitted tree — line numbers match the new
  code).
- `431-live-mb5.png` — ⌘⏎ materialized **Search: excerpt_locate** as the stitched
  multibuffer: group bands with per-file counts, TRUE line numbers (509–513 /
  247–251 / 865–869 / 16061+), match washes, first-Line pre-selection — the whole
  surface rendering through `DisplayMap::excerpts` → `excerpt_total`/`excerpt_locate`.
- `431-live-arrows.png` — ↓↓ moved the selection (511's selected wash dropped to
  plain match wash) — the `excerpt_move_selection` door driving live state.
- `431-live-jump.png` — Enter jumped to `multibuffer.rs` at line 511 with the caret
  on the match column, the mb tab still open (the #427 contract; `line_at` over the
  memoized index). The jump landed inside the very code that built it.
Drive-harness note for future validators: `drive.swift cmd:enter` resolves the FIRST
CHAR ('e' → ⌘E), not ⌘⏎ — send ⌘⏎ via `osascript key code 36 using {command down}`.
Also: the app launched from /Volumes/Offload trips a one-time removable-volume TCC
dialog that can wedge the first launch's window creation — click Allow, relaunch.

**Problems surface (REQ-006, recorded honestly):** no LIVE problems drive — this
workspace has no diagnostics source (status bar `lsp: failed`; rust-analyzer not
attached) and the #430 scratch-HOME + fixture + rust-analyzer rig was ad-hoc and is
gone. The receipt instead: (1) the rim is ONE code path for both surfaces
(`multibuffer_body`/movers/jump serve Search and Problems alike — the live search
drive exercised exactly the re-routed doors); (2) the problems-specific arm (Note
slots) is pinned by the oracle fixtures and by the three REAL-RootView headless drives
(`problems_materialize_edit_and_jump_headless`,
`problems_mb_refresh_quiescent_headless`,
`problems_materialize_empty_fails_closed_headless`) — all green in the 2244; (3) the
diff changes no render markup, and #430's live captures pin the pre-change pixels.

**Gate:** `scripts/gates.sh --diff` → first run RED on gate:1 alone (rustfmt drift in
the freshly written Phase-4 tests — fmt hadn't been re-run after authoring them; all
14 other gates PASSED including coverage ≥100% and MSI ≥100% on the diff). Fixed with
`cargo fmt --all` (verified `--check` clean), full re-run: **GATE GREEN [diff] —
15/15 passed**, receipt written (`.git/ignibyte-gate-receipt`, 2026-08-15 09:28).
Validator gotcha recorded: piping the gate through `tail` masks its exit code — read
the printed summary, never the pipe status.

## Phase 5 — Complete

- **§21 (a) CHANGELOG**: entry added under Unreleased → Changed (the unification story:
  one projection stack, the deleted parallel model, the oracle proof, gate 15/15).
- **§21 (b) Architecture docs**: `docs/marley_architecture/editor.md` — the multibuffer
  section gains the **#431** paragraph (the prize cashed; the deferred line re-pointed
  at #432/#433 + the wedge + the recorded singleton endgame), and the #425/#426
  display-map foundation paragraph now names `ExcerptIndex` beside `WrapIndex` (one
  stack, two instances).
- **§21 (c) Parity sync**: N/A — no UI delta shipped; `marley-web`'s `MultibufferView`
  stays the standing #427–#430 reference; MARLEY-PARITY rows unchanged.
- **Knowledge (§19) appended**:
  `L-claude-431-carry-the-deleted-model-as-the-test-oracle-001`,
  `L-claude-431-a-verbatim-move-can-carry-a-duplication-into-plain-sight-001`,
  `AD-claude-431-excerpt-stage-is-a-facade-arm-disjoint-until-composed-001`.
  Inspect appended no `F-`/`PR-` (no behavior bug; the class is standing PR-1691 —
  per the inspect ledger).
- Ticket closed → `tickets/closed/`; BACKLOG row left at promotion (swept then; no
  stale row). Pipeline pair archived to `completed/`.
