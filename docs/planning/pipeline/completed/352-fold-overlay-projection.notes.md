# Fold projection for the LSP overlay-card geometry (#305 Slice 2) — Notes

- **Forge ticket:** #352 `8f3852b9-de73-48bc-988e-20a3799ebb23` (feature, sprint "M28 — The Registry Payoff" `6558258f` — the train-independent item)
- **AAR:** ee294a3d-26dd-42e6-8014-fdbe0073a045 (opened at /work promotion, 2026-08-04; owner 90f02e73-89f8-4461-82d0-fe212c0dbb01)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-352-fold-overlay-projection.md
- **Pipeline spec:** 352-fold-overlay-projection.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request / provenance:** EXISTING ticket (filed 2026-07-18 at #305 Phase-2 promotion — the
  adversarial blast-radius Explore found the `editor_geom.first` fan-out; the #305 SCOPE DECISION
  split it out as Slice 2, a DOCUMENTED known-limit: cards anchored by
  `(buffer_row - geom.first) * cell_h` mis-position with a fold above the caret on-screen). Pulled
  into sprint M28 "The Registry Payoff" (`6558258f-a0d2-45fb-b601-cd5329a1cd77`) as the
  train-independent item — no #388/#394 ContentId-registry coupling, the parallel-safe lane.
- **Classification / tier:** feature (bug-flavored, forge type kept as filed), ONE slice, small —
  mechanical threading once the helper exists; the named risk is SITE-SET COMPLETENESS, and Phase 1
  already caught the stale inventory drifting (below).
- **Forge recall (§18.3):** `knowledge-search` ("fold projection slot buffer row", "overlay card
  geometry anchor") returns ranked node ids at this layer; `knowledge-context` (which resolves +
  surfacing-logs them) needs the minted AAR — deferred to /work promotion. Rules confirmed live via
  the standing record: **`BF-lsp-hover-extracted-helper-new-mutation-surface-001`** (313/312/322
  notes — DIRECTLY on point: extracting the inline card idiom into a named helper mints a fresh
  mutation surface → direct units, REQ-006), **`BF-claude-skip-detach-pump-fleet-live-001`**
  (skip-detach re-check — the edits land inside existing `mutants::skip` render shims),
  **`PR-claude-trace-the-real-cargo-mutants-list`** (`--list -f` on the ACTUAL touched files),
  **`PR-claude-language-specific-pure-primitive-needs-caller-side-gate-001`** (checked: the helper
  stays language-blind; the shipped `fold_projection()` accessor already carries the Rust gate,
  app.rs:11633-11643 — no new gate needed). `docs-search` with source_filter
  `docs/zed_architecture/*` returned empty (the glob does not cross directories); direct grep found
  the editor.md display-map coverage cited in §20.
- **Discovery — THE VERIFIED SITE SWEEP (2026-08-04, HEAD `940286d`; the ticket's 2026-07-18 line
  numbers had all drifted):**
  - **Drift table (stale → current):** hover ~10583 → `hover_card_overlay` app.rs:13001 (window
    check :13021, Y :13031); completion ~10657 → `completion_popup_overlay` :13076 (:13097/:13105);
    signature ~10741 → `signature_overlay` :13171 (:13182/:13189); rename ~10805 →
    `rename_draft_overlay` :13234 (:13246/:13253); hover-dwell inverse ~10492 →
    `on_editor_mouse_move` :12932 (row calc :12940); content-width probe ~4818 → in `code_view_body`
    :5852.
  - **TWO sites the original sweep MISSED (both #267/M16 — they EXISTED on 2026-07-18):** the IME
    caret rect `bounds_for_range` :14901 (check :14917, Y :14925 — the OS candidate window is a
    row-anchored anchor rect) and the IME inverse `character_index_for_point` :14932 (row calc
    :14944, clamped `min(geom.last-1)` in slot domain). This validates the ticket's own stated risk
    — the sweep undercounts; Phase 2 re-sweeps regardless.
  - **ONE site the #305 inspect NAMED but the ticket text omits:** the inlay-hints fetch window in
    `refresh_inlay_hints` :4466 (`want_first` :4502, `want_last` :4505-4508, served-check :4511,
    slots sent as the LSP request's buffer-row range :4544-4546; `INLAY_PAGE = 50` :733 — a fold
    hiding >50 rows starves on-screen rows of hints). 305-code-folding.notes.md E-3 [3] counts
    "inlay-prefetch" in the documented #352 known-limit. Membership recommendation: IN
    (D-OPEN-SITE-SET).
  - **Verified NON-sites (the "check whether they anchor the same way" sweep):** #312
    `def_picker_overlay` :13325, #317 `references_overlay` :13390, #323 `code_action_overlay`
    :13495, `goto_overlay` :13289, `file_symbols_overlay` :13558, `symbols_overlay` :13658,
    `search_overlay` :13764, `problems_overlay` :13893 — ALL centered/window-anchored
    (`menu_origin(win_w…, win_h…)`, no row math). Already-projected #305 crossings confirmed
    standing: sticky :6461-6463 (`buffer_row(geom.first)`), `scroll_editor_to` :14082-14086
    (`slot_of`), the rim :5877-5885; click/drag rows are rim-supplied (`h_scroll.rs` invariant — no
    interior y→row conversion).
  - **Net inventory: NINE sites in four classes** (5 forward row→Y incl. IME rect; 2 inverse
    pixel→row; 2 viewport windows; 0 elsewhere) — carried as the spec's D-OPEN-SITE-SET starting
    set. The dwell inverse is worse than "card mis-positions": `last_hover_cell`'s wrong row feeds
    `tick_hover_dwell` :12908-12922 → the LSP hover REQUEST itself targets the wrong buffer row.
  - **The seam to reuse:** `FoldProjection` crates/syntax/src/fold.rs:92-156 (`new` :102,
    `visible_count` :126, `buffer_row` :132, `folded_headers` :147, `slot_of` :156 — half-open
    internal runs after the #305 equivalent-mutant fix; `slot_of(hidden)` snaps to the header slot,
    the D-OPEN-OFFSCREEN-ANCHOR recommendation for free). Accessor `fold_projection()`
    app.rs:11633-11643 — identity + NO parse when nothing is folded (the D4 byte-identity
    substrate) + the Rust language gate.
  - **Drive substrate:** for_test seams exist — `drive_hover_for_test` :9623, `hover_card_for_test`
    :9642, `signature_card_for_test` :10023, `editor_geom_cell_w_for_test` :14209; #355's slice-2
    drive pattern (headless_drive.rs:8677-8684) covers the focused-pane sharing of
    `active_editor()`+`editor_geom`. CAUTION for P2's test design: a `simulate_keystrokes` render
    resets `editor_geom` to 0 (headless_drive.rs:5854) — the per-card drives must assert at the
    helper/for_test level or seed geom deliberately.
- **POC check (React-first):** NO fold engine in marley-web — the only "fold" hits in
  artifacts/marley-ide/src are find-bar CASE folding (`findFold`, App.tsx:56-114,
  EditorView.tsx:97-181). The overlay cards ARE mirrored (EditorView.tsx hover :317-319, rename
  draft :220) and their unfolded geometry is untouched → **React-first: N/A** (no-fold
  byte-identity REQ-002 doubles as the parity proof).
- **§20 / prior art:** VS Code/Zed anchor overlays correctly below a fold (OBSERVED — the ticket's
  recorded observation, extending #305's sweep); docs/zed_architecture/crates/editor.md:80/:92/:104
  maps Zed's display-map chain (research, not source — the wall holds). Published: the LSP spec's
  buffer-line positions explain WHY anchor rows arrive unprojected; nothing published on
  fold-overlay anchoring itself (expected). Permissive deps: the seam is IN-HOUSE
  (`FoldProjection`); gpui renders primitives only — no external owner.
- **Decisions:** locked D1 (ONE shared helper; #318's chrome recipe NOT absorbed), D2 (`geom.first`
  stays a slot; project the buffer row before subtracting), D3 (reuse `slot_of`/`buffer_row` — no
  second projection), D4 (no-fold byte-identity, identity path stays parse-free), D5
  (geometry-only diff). D-OPENs for Phase 2: HELPER-HOME (recommend beside `FoldProjection`,
  scalar args — marley_syntax stays gpui-free), SITE-SET (the 9-site starting inventory; re-sweep
  with the greps + `cargo mutants --list -f`; inlay-window membership recommended IN),
  OFFSCREEN-ANCHOR (recommend the header-slot snap `slot_of` already gives; alternative: suppress).
- **What Phase 2 MUST re-sweep (the standing risk):** the idiom greps (`geom.first` / `geom.last` /
  `geom.cell_h` row-math) over app.rs at promotion-time HEAD — this file drifts fast (6 of 6 stale
  line numbers moved by ~2400 lines in 17 days, and two sites were missed the first time); walk all
  13 `*_overlay` fns + both `EntityInputHandler` impls; confirm no new row-anchored surface shipped
  between now and promotion.
- **Forge ids:** ticket #352 `8f3852b9-de73-48bc-988e-20a3799ebb23`; sprint
  `6558258f-a0d2-45fb-b601-cd5329a1cd77` (M28); pipeline `2fe61fe9-de6e-4e51-97b3-27ab2a17489d`;
  AAR `ee294a3d-26dd-42e6-8014-fdbe0073a045`. Depends-on: #305 (SHIPPED — completed M19); no other deps.
- **PROMOTED 2026-08-04 (/work 352, auto-approved through commit):** queued pair → active/; forge
  claimed (owner `90f02e73`), status in-progress, AAR opened; bulletins: none; pre-flight all green
  (cargo 1.96.0, mutants 27.1.0, llvm-cov 0.8.7, hooks wired, marley-web OK). Phase 1 PASS.

## Phase 2 — Design (2026-08-04, auto-approved run)

### D-OPEN resolutions (all three settled)
- **D-OPEN-HELPER-HOME → beside `FoldProjection` in crates/syntax/src/fold.rs**, as ONE method:
  ```rust
  /// #352: slot offset of `buffer_row` within the viewport slot range [first_slot, last_slot)
  /// (EditorFrameGeom.first/last — end-exclusive). Multiply by cell_h for the pixel Y. None when
  /// outside the viewport. A row hidden inside a collapsed fold anchors at its fold HEADER's slot
  /// (slot_of's snap — the D-OPEN-OFFSCREEN-ANCHOR resolution).
  pub fn viewport_offset(&self, buffer_row: usize, first_slot: usize, last_slot: usize) -> Option<usize>
  ```
  Body = `slot_of(buffer_row)` + window check + `slot - first_slot`. **usize-only** (no f32 in
  marley_syntax — the `* cell_h` multiply stays at each masked call site), delegating to the shipped
  `slot_of` (D3 — no second projection). Identity ≡ today's `row < first || row >= last` check +
  `row - first` subtraction, exactly (REQ-002's floor).
  **The INVERSE needs NO new API** — the shipped `buffer_row(slot)` IS it; the two inverse sites wrap
  their slot arithmetic in it (clamping in SLOT domain first where the site clamps today).
- **D-OPEN-SITE-SET → the 9-site inventory CONFIRMED at HEAD f3d1bd2** (grep re-run this phase — output
  matches Phase 1's list exactly; docs-only commits since the sweep, zero drift). Inlay window IN.
  Non-sites re-confirmed: sticky :6461-6463 + `scroll_editor_to` :14084 already project.
- **D-OPEN-OFFSCREEN-ANCHOR → the header-slot snap** (free via `slot_of`); pinned by a helper unit row
  (hidden row → Some(header_slot - first)) — a card whose anchor is folded away renders at the header.

### Architecture / §20 / React-first
Pure helper in the gpui-free marley_syntax crate (cov/MSI-100 home); nine masked app.rs shim edits,
each 1-3 lines, chrome/origin-flips untouched (D1/D5). §20 CONFIRMED as spec'd: observed VS Code/Zed
anchor-below-fold behavior + zed editor.md's project-before-positioning convention (research only; no
copyleft source). React-first **N/A re-confirmed** — no fold engine in the POC (`findFold` is find-bar
case folding); no marley-web half in the manifest; no parity pair (REQ-002 no-fold byte-identity is
the parity proof).

### File manifest
| File | Change |
|---|---|
| `crates/syntax/src/fold.rs` | add `FoldProjection::viewport_offset` (+ doc + `#[cfg(test)]` truth table — the new REQ-006 mutation surface) |
| `crates/marley_app/src/app.rs` | thread the 9 sites (table below); no other change |
| `crates/marley_app/src/headless_drive.rs` | (Phase 4) the new drives per the test plan |

### The per-site conversion table (all line refs verified at HEAD this phase)
Each overlay site gains `let proj = self.fold_projection();` (identity + no-parse when nothing folded).
1. **hover** :13021/:13031 — `let dy = proj.viewport_offset(card.anchor_row, geom.first, geom.last)?;`
   replaces the window check; `ay = geom.y0 + dy as f32 * geom.cell_h + geom.cell_h` (below-anchor +cell_h kept).
2. **completion** :13097/:13105 — same shape, `row` = caret row; `ay = geom.y0 + dy as f32 * geom.cell_h`.
3. **signature** :13182/:13189 — keep the `cell_w/cell_h <= 0.0` guard; then viewport_offset on
   `open.anchor_row` (the fixed anchor).
4. **rename** :13246/:13253 — same as completion (caret row).
5. **IME rect** `bounds_for_range` :14917/:14925 — same; origin y uses `dy`.
6. **dwell inverse** :12940 — `let slot = geom.first + ((y - geom.y0).max(0.0) / geom.cell_h) as usize;
   let row = self.fold_projection().buffer_row(slot);` (see beyond-EOF note in Risks).
7. **IME point inverse** :14944 — clamp in slot domain as today, then convert:
   `let row = proj.buffer_row((geom.first + (dy / geom.cell_h) as usize).min(geom.last.saturating_sub(1)));`
8. **probe** :5852 — reuse the EXISTING `proj` binding at :5813 (same render scope):
   `(geom.first..=geom.last.max(geom.first)).map(|slot| row_cols(proj.buffer_row(slot)))`;
   `row_cols(caret_row)` unchanged (already a buffer row).
9. **inlay window** :4502-:4511 — convert the viewport ends FIRST, then page in buffer domain:
   `let first_row = proj.buffer_row(geom.first); let end_row = proj.buffer_row(geom.last);`
   `want_first = first_row.saturating_sub(INLAY_PAGE); want_last = end_row.saturating_add(INLAY_PAGE).min(len_lines-1);`
   served-check: `rows.start <= first_row && end_row <= rows.end`. Identity: `buffer_row(x) = min(x, total-1)`
   makes every arm equal today's values (the pre-existing generous exclusive-end + clamp reproduce exactly).

### Regression Test Plan
| REQ | Test | Where / observable |
|---|---|---|
| REQ-001 | `viewport_offset` truth table: identity in-window (`Some(row-first)`), identity out-of-window both edges (None at `first-1`, at `last`), fold-above shifts (hidden [1,4): row 5 → slot 2), boundary `slot==first`→Some(0) / `==last-1`→Some / `==last`→None, saturation past EOF | fold.rs `#[cfg(test)]` |
| REQ-001/004 | hidden-anchor row: buffer_row inside a collapsed fold → `Some(header_slot - first)` (the snap, pinned) | fold.rs table |
| REQ-002 | no-fold identity drives: dwell request row, inlay `InlayKey.first_row/last_row`, probe `editor_content_w` all equal today's formula values (in-crate drives read the private state directly) | headless_drive.rs |
| REQ-003 | fold-above dwell drive: fold rows 1..=3, mouse at slot 2's pixel Y → the hover REQUEST fires for buffer row 5 (not 2) — asserted at the request/caret seam (`drive_hover_for_test` :9623 idiom) | headless_drive.rs |
| REQ-005 | probe drive: a long line hidden inside a fold within the viewport → `editor_content_w` excludes it (visible rows only); inlay drive: with a fold, `InlayKey.first_row/last_row` span the PROJECTED buffer range ± INLAY_PAGE | headless_drive.rs (in-crate private reads) |
| REQ-006 | `cargo mutants --list -f crates/syntax/src/fold.rs` → viewport_offset mutants enumerated + all killed by the table (gate:5 --diff receipt) | validate |
| REQ-007 | live capture: fold above caret + hover card open → card at the correct visual row (bundle-app + drive.swift + screencapture, READ the PNG) | validate |
| — regression | #305 fold suites (fold.rs tables + drives), #311 hover / #313 completion / #320 signature / #322 rename suites, full `cargo nextest run --workspace` | validate |
| — parity | N/A — React-first N/A (no POC fold engine); REQ-002 byte-identity carries parity by construction | — |

**Card-Y verification calibration (design refinement of the spec's REQ-001 verify wording):** the four
card overlays + IME rect are masked render shims whose computed pixel Y is not observable headlessly
without adding per-card test plumbing; per the masked-shim discipline the Y MATH floor is carried by
the helper truth table (the exact expression each site now delegates to), the fold-active BEHAVIOR by
the dwell/probe/inlay drives (whose observables ARE readable in-crate), and the pixel truth by the
REQ-007 live capture. No per-card `_for_test` Y accessors are added — keeping the shims thin beats
duplicating the helper's own math into test plumbing.

**Uncoverable (named):** the IME candidate window itself is OS-drawn — mechanism-only proof (helper
units + the shared code path); no OS-window pixel assert exists.

### Risks / decisions record
- **fold_projection() per-overlay call cost:** with folds active each call parses `fold_regions`;
  up to 4 extra calls/frame when every card is open. ACCEPTED — matches the existing per-frame call
  pattern (:5813 render + :6463 sticky + :14084 scroll already run per frame), folds-active only, and
  the no-fold fast path is parse-free (identity). A projection memo is future work, not this ticket.
- **Beyond-EOF dwell convergence:** today the un-clamped dwell row can exceed the buffer (Buffer's
  line APIs clamp downstream); `buffer_row(slot)` clamps at the projection instead. The stored
  `last_hover_cell` tuple differs for beyond-EOF positions but the OBSERVABLE (the hover request's
  caret) is identical — the no-fold drive asserts the request seam, not the internal tuple.
- **Drive geometry seeding:** `simulate_keystrokes` renders reset `editor_geom` to 0
  (headless_drive.rs:5854) — every new drive seeds `editor_geom.set(..)` AFTER its last keystroke,
  before asserting (in-crate access).
- **Float identity:** the touched expressions keep the same op order (usize sub → as f32 → mul), so
  no-fold identity is exact, not approximate.

## Phase 3 — Implement (2026-08-04)
- **React-first: N/A** — no UI delta (the POC has no fold engine; the spec's `## React-first (parity)`
  section declares it). No marley-web work; the no-fold byte-identity ACs carry parity by construction.
- **Built, per the design manifest:**
  - `crates/syntax/src/fold.rs` — `FoldProjection::viewport_offset(buffer_row, first_slot, last_slot)
    -> Option<usize>` after `slot_of`, delegating to it (D3); usize-only; doc carries the identity +
    header-snap contract. (+20 lines)
  - `crates/marley_app/src/app.rs` — all NINE sites threaded exactly per the conversion table:
    hover / completion / signature / rename / IME-rect forward anchors now
    `viewport_offset(...)` + `dy as f32 * cell_h` (hover keeps its `+ cell_h` below-anchor offset;
    signature/rename keep their split `cell_w/cell_h` guards); dwell + IME-point inverses compute the
    SLOT then `buffer_row(slot)` (IME clamps in slot domain first, as before); the width probe maps
    each iterated slot through the EXISTING `proj` binding (:5813); the inlay window projects the
    viewport ends and compares the served-check in buffer domain.
- **Deviation from the design (with reason):** the design's inlay row-9 sketch used
  `end_row = proj.buffer_row(geom.last)` and claimed identity — WRONG for a fully-visible short file
  (`geom.last == total` clamps to `total-1`, flipping the served-check and silently "fixing" the
  pre-existing short-file re-request behavior — a byte-identity violation). Implemented instead as the
  exclusive-end-through-the-last-visible-slot form: `end_row = if last == 0 { 0 } else {
  buffer_row(last - 1) + 1 }` — exact identity for every input (verified against the storage shape at
  app.rs:4633: `rows = key.first_row..key.last_row`, an inclusive value in an exclusive position —
  pre-existing; untouched). The short-file re-request pathology is recorded below for inspect, NOT
  fixed here (out of scope; behavior-preserving ticket).
- `cargo check --workspace` clean (8.76s); `cargo fmt --all` applied. No tests written (Phase 4's job);
  no gate weakened.
- **Flag for inspect:** (a) the pre-existing short-file inlay re-request pathology (served-check can
  never pass when `geom.last == total` — today's behavior, preserved bit-for-bit; candidate follow-up
  ticket); (b) the beyond-EOF dwell convergence note from the design risks (observable request
  identical; internal tuple clamps earlier now).

## Phase 3.5 — Inspect (2026-08-04)
Two adversarial critics over the diff (correctness lens; completeness/integrity lens), independently
converging on the top finding. All four confirmed findings FIXED at source; re-check green
(`cargo check` clean, `cargo fmt`, marley_syntax 81/81).

| # | Sev | Finding | Verdict | Fix |
|---|---|---|---|---|
| F1 | HIGH | **The geometry recorder itself was a missed mixing site**: both mount gates compared `row == first` — a BUFFER row against a SLOT (app.rs:6098 canvas that records `EditorFrameGeom`, :6243 the clipper `code_w` probe). With a collapsed fold fully above the viewport, `buffer_row(slot) > slot` for every slot → neither canvas mounts → `editor_geom` FREEZES stale → every consumer (all nine #352 sites, click→caret, h-scroll clamps) reads frozen geometry in exactly the fold-above-viewport scenario. Pre-existing since #305 (which moved `row` into the buffer domain and left the gates); found by BOTH critics; the Phase-1/2 sweeps missed it because they grepped `geom.first` ARITHMETIC — this is an EQUALITY gate on a shadowing local. | REAL (verified by own read: `let first = range.start` :5896 slot; `let row = proj.buffer_row(slot)` :5899 buffer; no rebinding) | `if slot == first` at both sites + domain comments |
| F2 | HIGH | **Fold-active steady-state parse burn, introduced by this diff**: `fold_projection()`'s parse arm is `buf.text()` (O(n) String) + a full tree-sitter parse; the diff put it on the 16ms pump (`refresh_inlay_hints`, app.rs:1840 ticks unconditionally) and on every mouse-move — ~60 full-file parses/second while IDLE with any fold active. Pre-diff both paths were integer compares. Identity path (no folds) was and stays parse-free. | REAL (pump cadence PUMP_INTERVAL_MS=16 verified) | `fold_proj_cache: RefCell<Option<(nonce, BufferVersion, Vec<Anchor>, FoldProjection)>>` memo on the parse arm only — reload re-mints nonce, an edit bumps version, a fold verb changes the anchor set; any change recomputes ONCE, every other call is a cheap clone. All 14 call sites benefit; identity arms never touch the cache. |
| F3 | MED | **The Phase-3 "dwell convergence" claim was FALSE**: below EOF the old raw row fed `line_text("")` → `offset_for_click("", col) = 0` → caret at COLUMN 0 of the last line; the new `buffer_row` clamp landed on the last line's REAL text → caret ≈ col — a different hover-request `position.character` (and the per-slot dedup collapsed, changing card-dismiss behavior below EOF). No panic either way (Buffer's line APIs clamp — the critic verified buffer.rs:141-163). | REAL (my Phase-3 note corrected) | The dwell keeps the RAW slot when `slot >= proj.visible_count()` (byte-identical below-EOF behavior in every arm — identity AND folds) and projects only in-range slots. |
| F4 | LOW | **`viewport_offset` doc overclaimed + stale-anchor saturation**: `slot_of` saturates a beyond-EOF row to the last row's slot → `Some(...)` where the old check returned `None`. Critic attacked every forward site for persistent reachability: NOT reachable beyond ≤1-frame transients (completion/rename/IME rows are live-caret `line_col`; signature's anchor tracks the caret via `dismiss_stale_signature`; hover dies on the next 16ms poll). Fixed anyway for letter-exactness. | REAL (transient-only) | `if buffer_row >= self.total_rows { return None }` guard + doc corrected ("hides rather than saturates") |
| F5 | LOW | Pre-existing short-file inlay churn: with the file bottom rendered (`geom.last == total`), the served-check can never pass (`rows` stores an INCLUSIVE `want_last` in an exclusive position — app.rs:4644) → one identical re-request per LSP round-trip, forever. Both critics walked the algebra; the diff PRESERVES it bit-for-bit (the `buffer_row(last-1)+1` form was chosen exactly for this). | REAL, pre-existing, OUT of this behavior-neutral ticket | Deferred — follow-up ticket to file at Complete (store the exclusive end `want_last + 1`). |
| F6 | INFO | `hover poll` compares `geom.first` to `card.first_row` — both SLOTS (recorded at :11022), no mixing; but the field NAME now misleads. | Accepted — rename candidate only | none (cosmetic; noted for a rider) |

**Survived attack (critic-verified PASS):** no-fold byte-identity at all five Y-anchors (algebra walked incl. partial viewport / short file / `geom.last==0` / single-line / empty-file unreachability); inlay identity incl. the `geom.last==0` underflow guard; boundary semantics (`slot==first`→0, `==last-1` visible, `==last` hidden; hover `+cell_h`; probe max-domain equivalence; IME slot-domain clamp order); borrows (probe closure scopes end before the 'static list closure); site-set classification (every `geom.first/last`/`cell_h`/`editor_geom` hit in the workspace classified — the pty-grid `cell_h` sites are a different widget, no fold domain); no new mixing (all `viewport_offset` args traced to buffer rows, all `buffer_row` args to slots); scope/provenance (no attribute/persistence/test/gate changes; §20 clean — helper composed from in-house `slot_of` only).

**Ledger note on process:** the async critic wait was bridged by parking the dependent phase tasks
(deleted + recreated) — the enforce-phase-tasks contract held; all inspect work completed this phase.

## Phase 4 — Validate (2026-08-04)
### Tests added
- **fold.rs `viewport_offset` truth tables (2 tests, 12 assertion rows):**
  `viewport_offset_identity_boundaries_and_eof_guard` (identity in-window/out-both-edges, EOF-row
  live, past-EOF → the inspect-F4 guard not saturation, empty-buffer None) and
  `viewport_offset_projects_folds_and_snaps_hidden_to_header` (fold-above shift, scrolled first,
  header row itself, hidden-row → header-slot snap, hidden-row + header-off-viewport → None).
  REQ-001/002/004/006.
- **headless drive `dwell_cell_projects_folds_and_keeps_raw_below_eof_headless`:** the dwell inverse
  driven directly through three new `#[cfg(test)]` seams (`seed_editor_geom_for_test` /
  `mouse_move_editor_for_test` / `last_hover_cell_for_test`) — identity in-range (slot==row),
  identity below-EOF (RAW slot kept), folded in-range (slot 2 → buffer row 5), folded below-EOF
  (raw slot, byte-identical). REQ-002/003.
- **Stated, not silently skipped:** the four card overlays + IME rect + width probe + inlay window
  are masked render shims — the headless platform's draw is a no-op (no canvas fills
  `editor_geom`), so their MATH is pinned by the fold.rs tables and the BEHAVIOR by the live
  capture below (the design's calibration note).

### Suite run (actual)
`cargo nextest run --workspace` → **1987 tests run: 1987 passed, 5 skipped** (7.8s; includes the
3 new tests + all #305/#311/#313/#320/#322 regression suites green unchanged).

### Live capture (REQ-007) — CARD-BELOW-FOLD PROVEN ON PIXELS
Isolated fixture run (the user's `~/.marley/config/settings.toml` backed up → replaced with a
fixture config → **restored byte-identical after** — diff-verified): a scratch cargo crate
(`f352fix`, 21-line main.rs: alpha/beta/gamma/main fns), real rust-analyzer via `[[lsp.servers]]`
(absolute path — GUI PATH lacks ~/.cargo/bin), the #163 shell blob booting the fixture as the
restored root. Driven via scripts/selftest/drive.swift (palette, finder, Files-dock click, ⌘K).
- `cap/14_hover_nofold.png` — IDENTITY baseline: `lsp: ready`; caret in `ga|mma` line 13; the
  hover card (`f352fix / fn gamma() -> u32`) anchors one cell below line 13; inlay `: u32` hints
  live.
- `cap/15_hover_folded.png` — **THE FIX**: palette Fold All → every fn folded (`⋯ N lines`
  markers; gutter 1→6→7→12→13→16→17→21); the caret's gamma header (buffer row 13, 1-based) renders
  at SLOT 4; **the hover card anchors directly below the gamma header's VISUAL row** with 8 buffer
  rows hidden above — pre-fix it rendered ~8 rows lower (unprojected `(row−first)·cell_h`), in the
  dead space under `fn main`.
  Captures at scratchpad `cap/` (session-local; the notes record what was seen — §15 the
  transcript carries the READ of both PNGs).
- Incidental live confirmations: the fold-all + finder + palette + Files-dock flows all healthy on
  the fixed build; the ⋯-marker inlay channel renders with the probe threading live.
- The F1 (geom-recorder) scenario-specific arm (fold above a SCROLLED viewport) is not reachable
  in a 21-line fixture — carried by the fold.rs tables + the slot==first fix's construction; the
  unfolded/folded captures prove the recorder keeps geometry fresh through fold/unfold cycles.
- Live-drive detour worth recording: ⌘P (finder) in TERMINAL focus inserts the picked path at the
  prompt (it does NOT open an editor) — the editor open came via the Files dock tree. The FileRef
  "Open in Editor" menu row also did not open an editor from the terminal context in this build —
  noted as a possible follow-up observation, NOT touched by this ticket.

### Gate
- **First `--diff` run: GATE RED at gate:12 no-suppressions** — the inspect-F2 memo field carried
  `#[allow(clippy::type_complexity)]`, a §0-banned suppression (and the run had piped through
  `tail`, swallowing the non-zero exit — the printed verdict, not the pipe status, is the truth).
  Fixed at source: a named `FoldProjCache` type alias beside the existing `InlayCache` precedent
  (app.rs:737); the allow deleted.
- **Re-run: `GATE GREEN [diff]` — 15 passed, 0 failed** (fmt, clippy -D warnings, nextest+doctests,
  audit, deny, machete, gitleaks, shellcheck, no-suppressions, SAST, docs, coverage ≥100% lines,
  mutation MSI ≥100% on the diff, miri, visual/AX). Receipt written
  (.git/ignibyte-gate-receipt) — the /commit binding.
- Pre-existing exclusions: none touched; the `block v0.1.6` future-incompat note is an upstream
  dependency advisory, unchanged by this ticket.

## Phase 5 — Complete (2026-08-04)
- **Docs (§21):** CHANGELOG entry added (the #352 entry atop Unreleased/Changed); editor.md's #305
  block updated — the known-limit parenthetical now points at the new **#352 bullet** documenting the
  nine-site conversion + the recorder-gate fix + the FoldProjCache memo; the archived
  305-code-folding spec/notes known-limit lines annotated "SHIPPED 2026-08-04 (M28)".
- **Parity sync:** React-first N/A (no POC fold engine) — nothing to back-port; MARLEY-PARITY.md
  unchanged (no zone/port-map movement).
- **Knowledge captured (forge):** AAR `ee294a3d` submitted (outcome completed, effectiveness 5;
  4 materialized codes): failures `BF-claude-geom-recorder-buffer-row-eq-slot-gate-001` (the
  inspect HIGH) + `BF-claude-fold-projection-parse-on-pump-tick-001` (the introduced-then-caught
  pump cost); rules `PR-claude-domain-sweep-includes-equality-gates-001` (domain sweeps must grep
  comparison/equality sites, not just arithmetic) +
  `PR-claude-gate-verdict-owns-the-exit-code-never-pipe-001` (the tail-swallowed GATE RED).
- **Follow-up filed:** forge **#401** — the pre-existing short-file inlay served-check churn
  (inspect F5, preserved bit-for-bit here; one-line exclusive-end fix + a short-file unit).
- **Ticket closed** (#352 forge + local doc → tickets/closed/); pipeline pair archived to
  completed/. Delivered via /commit (gate receipt bound).
