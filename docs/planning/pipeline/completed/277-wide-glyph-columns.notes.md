# 277-wide-glyph-columns — Notes

- **Forge ticket:** #277 d656abeb-06e0-4038-90bc-7dd023feae68
- **AAR:** b3d750a5-9201-4cee-a020-206f01cae745
- **Local ticket doc:** docs/planning/tickets/open/TICKET-277-wide-glyph-columns.md
- **Pipeline spec:** 277-wide-glyph-columns.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan
- **Request:** /goal batch `/work 272-281`, ticket 4 of 10.
- **Recon (code-verified):** `line_layout` (code_view.rs:76) is the ONE
  producer of `col_starts` (len n_chars+1; v1 all-width-1 with the
  documented `char_width(ch)` hook comment at :35). Column consumers:
  `col_of_offset` (:51, clamped read), `offset_of_col` (:59,
  nearest-boundary later-wins), `offset_for_click` (:104),
  `row_selection_cols` (:113 → columns out) — all col_starts-driven,
  genuinely free under the hook. **NOT free (the ticket's premise is
  wrong here):** `cols_to_bytes` (:163) does `char_indices().nth(col)`
  — a display CHAR-INDEX lookup that only equals a column under
  all-width-1. On "日x", cols [2,3) would map to 4..4 (past-end)
  instead of "x"'s 3..4 — every #266 selection band, #272 mark band,
  and #268 syntax remap (via `raw_span_to_display_bytes` :149, which
  composes col_of_offset → cols_to_bytes) drifts on wide lines.
  Corrected in-spec (REQ-003).
- **unicode-width:** already in Cargo.lock at 0.2.2 (transitive) —
  license pre-vetted by cargo-deny; becomes a direct `marley_app` dep.
- **Shims already column-driven:** caret x = ccol·cell.w
  (app.rs ~4967), click col = x/cell.w → `offset_for_click`
  (app.rs ~4995). No shim change expected; headless pins.
- **Design-phase reads queued:** CodeViewState::new max-cols
  truncation (does it cut by chars or cols?), the StyledText highlight
  consumers of cols_to_bytes in the render, and movement.rs (char
  motions are col-agnostic — verify no col math hides there).
- **Risk flagged for critic:** any OTHER display-char-index/column
  conflation in the render path; unicode-width table edges (emoji
  presentation selectors, U+FE0F width 0 vs the base char).
- **Autonomy note:** /goal run — no human pause.

## Phase 2 — Design

### Consumer map (code-verified)
- The EDITOR render's three highlight feeds all funnel through
  `cols_to_bytes` (app.rs:2940 syntax via `raw_span_to_display_bytes`,
  :2956 selection, :2973 find marks) — ONE fix point, as planned.
- The legacy read-only viewer (`code_lines` → `expand_tabs` +
  `truncate_cols(max_cols)` by CHARS, CodeViewState @ app.rs:2602) has
  NO caret/selection math — a >200-col wide line may overflow its box
  by the char/col difference. Consciously OUT of scope (no alignment
  bug); noted for the record.
- `movement.rs` vertical motion (#257) picks the same CHAR column on
  the adjacent row — through a wide line the caret's DISPLAY column
  jumps (the bar still renders correctly via col_of_offset). Known
  pre-existing semantic, not touched; goal-column/display-col vertical
  motion is a recorded follow-up candidate, not a #277 regression.

### Fn diffs (all in crates/marley_app/src/code_view.rs)
- NEW `pub fn char_width(ch: char) -> usize` — the ONE width
  authority: `UnicodeWidthChar::width(ch).unwrap_or(1)` (None =
  control → 1 keeps every char caret-reachable, D2; zero-width →
  Some(0); wide/fullwidth → Some(2)). `\t` is NOT special-cased —
  both callers handle tabs positionally before consulting it (doc'd).
- `line_layout`: the non-tab branch becomes
  `display.push(ch); col += char_width(ch);` — nothing else moves
  (tab branch already advances over the ACCUMULATED col → REQ-005
  composes for free).
- `cols_to_bytes`: the `nth(col)` char-index lookup becomes a
  width-accumulating walk: return the start byte of the first char
  whose span `[acc, acc+w)` CONTAINS col; fall through to
  `display.len()`. Consequences (documented + tested): a col inside a
  wide char maps to that char's start (a sub-glyph band [0,1) on "日"
  is EMPTY — a glyph is never split); zero-width chars have empty
  spans so they are never band boundaries — their pixels ride the
  preceding base cell (a selected combiner tints via its base's cell;
  D4 cosmetics).
- `raw_span_to_display_bytes` — UNCHANGED code, fixed transitively
  (composes col_of_offset → cols_to_bytes).
- `col_of_offset`/`offset_of_col`/`offset_for_click`/
  `row_selection_cols` — UNCHANGED (col_starts-driven; the later-wins
  tie now also guarantees a click never lands between a base and its
  zero-width combiner: equal boundaries, later index wins).

### Manifest
| file | change |
|---|---|
| crates/marley_app/Cargo.toml | + unicode-width 0.2 (lock already has 0.2.2) |
| crates/marley_app/src/code_view.rs | char_width + the two fn diffs + tests |
| crates/marley_app/src/headless_drive.rs | one wide-fixture caret/selection flow |

### §20 confirmation
UAX#11 East-Asian-Width via unicode-width — a public Unicode spec
table, not an app to observe; the design consumes the crate's standard
width fn and wires it into Marley's own #250 seam. No copyleft source
involved. Holds as planned.

### Regression test plan
| REQ | Test |
|---|---|
| REQ-001 | `line_layout_wide_zero_width_and_control`: "a日b😀c" → col_starts [0,1,3,4,6]+7; "e"+U+0301+"x" → [0,1,1]+2; a control char (U+0007) counts 1; an ASCII+tab line reproduces the exact pre-#277 vectors |
| REQ-002 | `wide_round_trips_and_click_ties`: col_of_offset/offset_of_col round-trip the wide family; offset_of_col(1) on "日x" → 1 (right-half → after); on "e◌́x" → 2 (never mid-cluster); offset_for_click composes |
| REQ-003 | `cols_to_bytes_is_column_aware`: "日x" (2,3) → 3..4; (0,2) → 0..3; the sub-glyph band (0,1) → 0..0 (glyph never split); `raw_span_to_display_bytes` maps a raw "x" span on "日x" to display 3..4 and across a tab expansion on "日\tx" |
| REQ-004 | `row_selection_cols_wide_band`: selecting 日 alone in "a日b" → cols (1,3) |
| REQ-005 | in REQ-001's vectors: line_layout("日\tx", 4) → display "日  x", col_starts [0,2,4]+5 |
| REQ-006 | the ENTIRE existing suite is the ASCII pin (every pre-#277 layout/cols test unchanged); plus the roster test names stay green |
| headless | `editor_wide_glyph_caret_headless`: open a "日日x" fixture, place the caret by char, assert the app-side caret display column (the ccol the render consumes) = 4 at char 2; select 日日 and assert the selection cols (0,4) through the live view state |
| uncoverable | the FONT's actual 2×cell advance for CJK (pixels) — driven capture if the machine is unlocked, else ENV-BLOCKED + the residual noted (the ticket pre-authorizes pinning "what IS true") |

### Risks / decisions
- R1 — the sub-glyph-band-empty rule (D3 refinement) is the "never
  split a glyph" stance; boundary-derived callers never produce
  interior cols, so it is edge-only behavior.
- R2 — unicode-width 0.2.2 emoji widths: single-scalar emoji = 2 ✓;
  VS16/ZWJ sequences sum components (out of scope, follow-up noted in
  the spec).
- R3 — `char_width` lives in code_view (marley_app), NOT marley_editor
  — the pure editor crate stays column-agnostic (movement is
  char-based); columns are a RENDER concern.
- R4 — equivalent-mutant hygiene: the walk uses one `col < acc + w`
  comparison (both-side boundary tests kill `<`→`<=`/`==`); no
  redundant guards; `.unwrap_or`/`.max` method calls are unmutated.

## Phase 3 — Implement
- **Cargo.toml:** `unicode-width = "0.2"` (the lock already resolved
  0.2.2; no lock churn).
- **code_view.rs:** `pub fn char_width(ch)` =
  `UnicodeWidthChar::width(ch).unwrap_or(1)` (doc'd: `\t` handled
  positionally by BOTH callers; None=control→1; the ONE authority);
  `line_layout`'s non-tab branch advances `col += char_width(ch)`
  (tab branch untouched — stop math over the accumulated col);
  `cols_to_bytes` rewritten to the span-contains walk (`col < acc + w`
  → the char's start byte; fall-through `display.len()`).
- **Designed semantic-change port (not a regression):** the
  pre-existing `cols_to_bytes_multibyte_and_clamps` pinned the OLD
  char-index semantics on the exact wide fixture ("aé日😀b") — its
  vectors re-derived under width cells (spans a[0,1) é[1,2) 日[2,4)
  😀[4,6) b[6,7), total 7): (1,3)→1..3 (col 3 inside 日 → its start),
  (0,5)→0..6 (a band never splits 😀), EOL now col 7, inverted
  (4,1)→6..1. All other 26 code_view tests passed UNCHANGED (they
  were the ASCII/tab/narrow-é pins — REQ-006's evidence).
- No deviations from the design manifest; headless flow lands in
  Phase 4. `cargo check` + clippy `-D warnings` + fmt clean; full
  suite 974/974.

## Phase 3.5 — Inspect
### Critic findings ledger (2 critics: conflation sweep ×10 hunts w/
probe crate `colprobe`; width-table edges w/ probe `width_probe`
pinned to unicode-width =0.2.2 — every planned width CONFIRMED)
| # | Finding | Verdict | Fix |
|---|---|---|---|
| C-F2 | [MED] Wide-glyph click hotspot at 25% not 50%: all THREE click sites `round(x/cell_w)` to an integer col BEFORE the nearest-boundary scan — on a 2-cell glyph a click at 0.5 cells rounds to col 1 = exact tie → LATER → after-glyph; probed flip at 25%. | REAL | `offset_of_col_f(f32)` (same scan, un-quantized distances, same later-wins tie); `offset_for_click` takes f32; the 3 app sites pass the unrounded `rel / cell_w`. The integer `offset_of_col` wrapper went DEAD in the lib target → DELETED (no dead delegates), tests ported to the float fn. Flip point probed back at the true midpoint. |
| C-F1 | [MED] The `LineLayout` STRUCT doc still asserted the pre-#277 one-column contract on the authority type; a test comment claimed col_starts "strictly increasing" (now non-decreasing under zero-width ties); a stale "char-cols" test title. | REAL (doc contract) | All three rewritten to the width-cells contract (non-decreasing + the later-wins tie = never-mid-cluster). |
| W-F1 | [LOW] cols_to_bytes doc over-claimed "a sub-glyph band is empty" — probe: only a band STRICTLY inside a glyph is empty; (1,2) on "日x" → 0..3 (touching either boundary includes the WHOLE glyph — interior cols round DOWN). | REAL (doc + kill vector) | Doc reworded to the precise invariant; (1,2)→0..3 added to the Phase-4 kill list (kills the walk's `<`→`<=`). |
| W-F2 | [LOW] Orphan LEADING zero-width char ("◌́x"): bytes 0..2 unreachable by any band; caret-before-it keyboard-only (later-wins tie). Defective decomposed input; the cluster-snap is arguably right. | NOTED (stance-pin) | No code change; Phase 4 pins the stance so a refactor can't silently flip it. |
| W-F3 / C-F3 | [LOW] Sequence widths diverge BOTH directions: ZWJ families OVER-count (model 6 vs ~2 rendered), VS16/keycap UNDER-count (model 1 vs ~2; flags coincide at 2); and the caret/click cell grid vs gpui's font layout are two authorities wherever font advance ≠ width×cell_w (documented model premise). 0.2.2's STR-level width is already sequence-correct → a grapheme-cluster walk is the eventual fix. | NOTED (follow-up) | Spec Out-section widened to name both directions + the str-level fix path. |
| — | Width-table corrections worth recording: VS16 U+FE0F = Some(0) NOT None (the notes risk resolved — unwrap_or(1) never fires for it); soft hyphen U+00AD = 0 in 0.2.2 (was 1 in 0.1.x — not a width-1 fixture); ambiguous class (±/→/★/─) defaults NARROW = 1 via `width()` (the code consistently uses non-CJK width). | intel | Folded into the Phase-4 fixture choices. |
| — | Cleared: caret x is cells end-to-end (col_of_offset → ccol·cell.w, app.rs 6352→3085); no display-slice-by-col-as-char anywhere; boxes are w_full+nowrap (never char-count-sized); CODE_MAX_COLS/truncate_cols feeds ONLY the read-only split-pane arm (no caret/mouse math); no horizontal caret-follow exists; efind passes raw char offsets; code_view is crate-private (no cross-crate consumer); the prompt caret is flow-layout (no col math); raw_span_to_display_bytes tiles exactly under widths; gutter/visible_range/clamp_scroll_px col-agnostic; overflow-safe (usize::MAX col falls through; acc bounded). BONUS: #276's bare-caret Tab pad became width-CORRECT automatically (col after 日 = 2 → pads 2 to stop 4). | — | — |

- Post-fix verification: check + clippy `-D warnings` + fmt clean;
  full suite 974/974 (the ported float-domain tests keep the exact
  same vectors — integer cols cast to f32).

## Phase 4 — Validate
- **Units (6, `t277_*` in code_view.rs tests):** char_width table
  pins (incl. VS16=0, BEL None→1 — the `1` constant mutant dies on
  日=2 AND U+0301=0); line_layout wide/zero-width/tab-stop vectors
  ("a日b😀c" [0,1,3,4,6,7]; "e◌́x" [0,1,1,2]; "日\tx" → "日  x"
  [0,2,4,5]; "e◌́\t x" [0,1,1,4,5]; display never padded); wide
  round-trips + the FLOAT hotspots (0.9→0 / 1.0→1 tie-later / 1.1→1
  on 日; 1.6 kills `-`→`+` and `/`; never-mid-cluster 1.0→2 on
  "e◌́x"); cols_to_bytes width columns ((2,3)→3..4, (0,1)→0..0
  strictly-inside-empty, (1,2)→0..3 the W-F1 boundary-includes-glyph
  killer, combiner rides included/excluded base, the ORPHAN-combiner
  stance pin (0,1)→2..3 on "◌́x", usize::MAX clamp);
  raw_span_to_display_bytes wide + across-tab (x → 5..6 on "日  x");
  row_selection_cols wide band (1,3).
- **Existing-vector port (Phase 3, re-stated):**
  cols_to_bytes_multibyte_and_clamps re-derived under width cells —
  the ONLY pre-existing test that pinned the char-index coincidence.
  The other 26 code_view tests + the whole suite are REQ-006's
  ASCII-identity evidence, unchanged.
- **Headless flow:** `editor_wide_glyph_caret_headless` — REAL
  Right/shift-Right keystrokes over a "日日x" fixture; caret display
  cols 0→2→4→5 (each 日 advances TWO cells through the live
  buffer+caret at the pinned default tab width); shift-selection of
  日日 = the (0,4) four-cell band via the live view state.
- **Mutation (real runs):** `cargo mutants -f code_view.rs` → 158
  tested: 82 caught + 45 unviable + 31 timeout-detected, **0 missed**
  (exit 0); gate:5 MSI 100 confirms. (The timeouts are the marley
  test binary's per-mutant cost at the auto 20s ceiling — detected,
  not survivors.)
- **Suite:** 981/981 (974 + 6 units + 1 flow) + doctests green;
  clippy `-D warnings` + fmt clean.
- **Driven capture: ENV-BLOCKED (machine asleep/locked)** — the 8×8
  RGB downsample of a fresh full-screen probe is max 0 (pure black;
  alpha excluded). Same protocol as #272/#276: no unlock attempt;
  behavior carried by the headless flow (the real key ladder) +
  MSI-100 units. The FONT-ADVANCE residual (does the mono font render
  CJK at exactly 2·cell.w?) is precisely the check the ticket
  pre-authorized deferring — pin on unlock (batched with #272/#276
  re-verifies; ~1 min total, no ticket).
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] — 15/15
  PASS** (fmt · clippy · tests · audit · deny · machete · gitleaks ·
  shellcheck · no-suppressions · source-bans · docs · coverage 100 ·
  MSI 100 · miri · visual/AX). No pre-existing failures; nothing
  excluded.

## Phase 5 — Complete
- CHANGELOG under "### Fixed" (bug-typed); editor.md code_view section
  rewritten to the width-cells contract (float click, the cols_to_bytes
  rewrite, residuals, the untouched legacy viewer).
- AAR b3d750a5 submitted (completed; materialized:
  BF-claude-click-col-quantized-before-nearest-scan,
  PR-claude-authority-type-doc-must-move-with-semantics-001).
- Forge #277 closed (done); local ticket → closed/.
- Lessons: (1) verify a ticket's "downstream follows for free" claim
  against the ACTUAL code at plan time — cols_to_bytes was char-index
  based and every band funnels through it; (2) quantize-then-scan
  discards the half-cell the tie rule needs — keep click mapping float
  until the final compare; (3) probe the REAL width table (VS16=Some(0)
  not None; soft hyphen 0 in 0.2.2) before writing kill vectors.
