# editor — hand-lexer fallback phantom fix — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-333-hand-lexer-inlay-phantom.md
- **Pipeline spec:** 333-hand-lexer-inlay-phantom.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** `/work next ticket auto approved` → top of BACKLOG Queue =
  TICKET-333 (bug, editor). Auto-approved run: autonomous through commit.
- **Classification / tier:** work pipeline, single shippable slice — a
  two-call-site render fix + tests. No split needed.
- **Recall (§18.3):**
  - `AD-claude-two-boundary-maps-for-phantom-text-001` (#331) PRESCRIBES this
    fix: give the fallback the primary path's shape — lex RAW, map through
    `raw_span_to_display_bytes` (phantom-safe via `col_of_span_end`).
  - #331 notes F8 traced the honest reachable set: after an edit both caches
    invalidate together; #331's language gate keeps non-Rust hint-free; the
    permanent window is a Rust file with an exotic line separator (bare
    `\r`/FF/NEL/LS/PS → `lines.get(row)` None forever). Cosmetic bleed only.
  - The tab-expansion trap (ticket + F8): display-lex + span-shift is NOT an
    insertion offset — a phantom changes tab-stop expansion. Rejected (D2).
  - Floor languages: TOML deliberately stays on the hand-lexer floor
    (AD: outer-wins sweep vs broad container captures); Markdown/Plain return
    no spans; grammar languages ride the floor transiently during async parse.
    Regression bar = D4.
- **Discovery (Explore):**
  - Main fallback: `crates/marley_app/src/app.rs:6486-6508` — option-chain
    `syntax_cache → lines.get(row) → map(primary remap) → unwrap_or_else(||
    highlight_ranges(&layout.display, lang))`. Raw `text` already in scope
    (`:6460-6474`); `layout = line_layout_with_inlays(&text, tab_width,
    row_inlays)` (`:6480`); primary arm maps via
    `raw_span_to_display_bytes(&text, &layout, r)` (`:6497`).
  - Sticky-header fallback: `app.rs:7055-7073`, same shape; its layout is
    `line_layout(&text, tab_width)` — NO inlays, so no phantom bug there (D5:
    design decides unify-vs-leave).
  - `highlight_ranges(line, lang) → Vec<(Range<usize>, TokenKind)>`
    (`code_syntax.rs:440`): byte ranges into WHATEVER string it is handed;
    ascending, disjoint, Plain dropped. Handing it raw `text` yields raw-byte
    spans ready for the mapper.
  - `raw_span_to_display_bytes(line, layout, span)` (`code_view.rs:304-317`):
    raw line-local bytes → display bytes; end via `col_of_span_end(...).max(c0)`
    — code-side, phantom-safe; clamps, never panics.
  - `grammar_lang` floor arms (`code_syntax.rs:40-58`): Toml → None
    (permanent); Markdown|Plain → None (no spans at all); Rust/Json/Shell/
    Python/JS/TS/TSX → Some (floor only transient).
  - Existing tests: `highlight_ranges` unit (`code_syntax.rs:681,:695` — raw
    lines only, nothing exercises the display-lex path);
    `raw_span_to_display_bytes` well-tested incl. phantom cases
    (`code_view.rs:1198,:1223,:1435,:1667,:1693,:1710`).
  - Hover-card lexing (`app.rs:15567`) lexes raw text, no layout — out of
    scope.
- **Decisions:** D1–D5 locked in the spec. Prior-art sweep: Zed behavior map
  confirms highlight-over-buffer-text with inlays as a coordinate transform;
  LSP 3.17 inlayHint = presentation-only; no dep owns the seam — the in-repo
  #331 mapper is the owner (recorded in spec `### Prior art`).

## Phase 2 — Design

### Architecture / approach
- **The logic is a PURE function; the render sites become one-line calls.**
  `crates/marley_app/src/app.rs` is coverage-EXCLUDED and its render fns are
  `mutants::skip` (the gpui shim — gates.sh `--ignore-filename-regex`), so any
  logic left in the fallback closure would be invisible to gate:4/5. New fn in
  `code_syntax.rs` (fully covered module, no skip):

  ```rust
  /// The fallback arm of the render's syntax feed (#333): hand-lex the RAW
  /// line, then map each span into display bytes — the SAME shape as the
  /// primary tree-sitter arm, so inlay phantom text (present only in
  /// `layout.display`, never in `line`) can never seed lexer state that
  /// bleeds into real code (#331 F8).
  pub fn highlight_display_ranges(
      line: &str,
      layout: &crate::code_view::LineLayout,
      lang: Language,
  ) -> Vec<(core::ops::Range<usize>, TokenKind)> {
      highlight_ranges(line, lang)
          .into_iter()
          .map(|(r, k)| (crate::code_view::raw_span_to_display_bytes(line, layout, r), k))
          .collect()
  }
  ```
- Both call sites' fallback arms become
  `.unwrap_or_else(|| crate::code_syntax::highlight_display_ranges(&text, &layout, lang))`.
- **D5 SETTLED — UNIFY the header site.** Its layout (`line_layout`, no
  inlays) differs from raw only by tab expansion, and the hand lexer treats
  `\t` exactly like a space (rule 5 "any other char is plain": neither can
  start/alter a comment, string, number, or keyword token; Plain is dropped).
  With no phantoms the mapper IS the tab-expansion transform, so raw-lex+map
  is BYTE-IDENTICAL to the old display-lex — proven by identity tests, not
  just argued. Unifying removes the LAST display-string lex in the codebase.
- **Domain/contract checks:** downstream (`styled_slices_with_marks` after the
  hints-first prepend) consumes ascending disjoint display-byte ranges. The
  mapper is monotone (`col_of_offset`/`col_of_span_end` are monotone;
  `.max(c0)` can't invert), so disjoint-ascending survives — same property the
  primary arm has relied on since #268/#331. New fallback spans NEVER cover
  phantom bytes (`col_of_span_end` construction), strictly stronger than
  before; the Hint prepend still paints phantom cells (REQ-003, unchanged).
- **§20 confirmed:** matches the Zed-behavior reference (highlighting computed
  from buffer text; inlays a coordinate transform, never lexer input) from our
  own behavior map; no copyleft source read. React-first stays N/A — no UI
  delta (restores intended colors on a rare fallback; nothing to mirror in the
  POC).
- §14: no new errors/IO/unsafe; pure total fn (clamping mapper, never
  panics); single-owner types unchanged (TokenKind stays in code_syntax,
  LineLayout in code_view).

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/code_syntax.rs` | ADD `highlight_display_ranges` (pure, ~10 lines + doc) + `#[cfg(test)]` t333 test matrix (below) |
| `crates/marley_app/src/app.rs` | Two fallback arms (~:6506-6508 main row, ~:7073 sticky header) call the new fn; comment at ~:6517-6519 updated (the "lexes `display`, hints and all" caveat dies) |

No other files (CHANGELOG + arch docs at Phase 5).

### Regression Test Plan (all pure `#[cfg(test)]` unit tests in code_syntax.rs)
| # | Test | Proves |
|---|---|---|
| T1 | `t333_fallback_phantom_no_bleed` — Rust `let x = q;` + mid-line phantom `: "weird` (server-controlled text; the ticket's own repro) and a `// note` phantom variant: new shape emits NO Str/Comment span; every emitted span's display bytes avoid `layout.hint_spans()`; per-token covered display text equals the no-phantom lex of the same raw line. | REQ-001 fix; REQ-003 (no fallback span covers phantom cells → Hint prepend owns them) |
| T2 | `t333_fallback_phantom_negative_control` — same construction through the OLD shape (`highlight_ranges(&layout.display, lang)`): asserts it DID emit a Str span bleeding across ` = q;`. | REQ-001 catchability (the test fails on the old code) |
| T3 | `t333_fallback_floor_identity` — TOML line (permanent floor) + Rust line (transient floor) + Markdown/Plain (empty), tab-free, no inlays: new == old span-for-span. | REQ-002 / D4 |
| T4 | `t333_fallback_tab_identity` — TOML with `\t` before `=` and `\t` inside the quoted string, plus a CJK-in-string case, tab_width 4: new == old span-for-span (the mapper is exactly tab expansion). | REQ-002 / D4 tabs+multibyte |
| T4b | `t333_fallback_zero_width_absorption_pinned` — a combining mark immediately after a keyword: the mapped span ABSORBS the mark's bytes (old display-lex left them in dropped Plain). Pins the REQ-002 exception as deliberate. | REQ-002 exception (inspect find) |
| T5 | `t333_fallback_spans_disjoint_ascending` — with phantoms present the output stays ascending + disjoint (the `styled_slices_with_marks` feed contract). | consumer contract |
| T6 | Existing #331 suites (`t331_*` mapping/ordering tests) stay green, unmodified. | REQ-003 |
| T7 | `scripts/gates.sh --diff` green at validate (fmt/clippy/tests/coverage/mutation on touched lines). | REQ-004 |

- No `trybuild` rows: no new type contract (plain `&str`/`&LineLayout`/`Language`
  params; no newtype seam introduced).
- Uncoverable: the two app.rs call-site lines sit in the coverage-excluded gpui
  shim (already on the §0 documented exclude); their logic is the tested pure fn.
- REQ-003's render-assembly half (the literal hints-first prepend) lives in
  excluded app.rs and is unchanged by this ticket; its guarantee is carried by
  T1's no-phantom-coverage assertion + the untouched #331 ordering tests.

### Risks / decisions
- R1 (low): disjointness under mapping — monotone maps, primary-arm precedent,
  T5 asserts it.
- R2 (low): floor regression — T3/T4 identity closes it.
- R3 (nil): header site — no phantoms exist there; identity proven.
- R4 (nil): perf — per-span char_indices walk on visible fallback rows only,
  the same cost the primary arm already pays per cached span.

## Phase 3 — Implement
- **React-first: N/A** — no UI delta (per spec section); nothing built in the POC.
- Built exactly to the manifest, no deviations:
  - `code_syntax.rs`: added `highlight_display_ranges(line, layout, lang)` —
    the pure fallback arm (raw-lex → per-span `raw_span_to_display_bytes`),
    doc-commented with the #331 F8 provenance + the hint-free byte-identity
    property + the disjoint-ascending feed contract.
  - `app.rs` main row (~:6506): fallback arm now calls the new fn; the
    #268 syntax-source comment gained the #333 line; the hints-first comment
    no longer claims the fallback "lexes `display`, hints and all" (it now
    notes both arms lex raw and ordering is pure belt-and-braces).
  - `app.rs` sticky header (~:7073): same one-line rewire (D5 unify).
- `grep highlight_ranges(&layout.display` → zero call sites remain.
- `cargo check --workspace` green (pre-existing `block v0.1.6`
  future-incompat note only, transitive dep, untouched).

## Phase 3.5 — Inspect
Three parallel critics (correctness/domain-integrity; reachability/AC;
simplification/reuse/provenance), each fed the subsystem's prior failure
classes (#331 F1 end-swallow, #336 chars-vs-cells, #340 byte/char localization,
#315 floor regression, TICKET-022 false-premise). Two ran live probes with
replicas of the real fns (unicode-width pinned to the lockfile's =0.2.2).

| # | Sev | Finding | Verdict | Fix |
|---|---|---|---|---|
| I1 | MED | The raw→display span remap existed in THREE copies after the diff (both primary arms inline in the excluded app.rs shim + the new fn) — rule-of-three met; D1's "identical shape" was enforced only by parallel text; the excluded-shim copies are logic outside the 100% denominator by policy. | **REAL** (simplification critic; scope-weighed: D5's own unify logic + coverage architecture argue for it) | Hoisted `code_view::spans_to_display_bytes(line, layout, &[(Range, TokenKind)])` beside the mapper; `highlight_display_ranges` = one-line composition over it; BOTH primary arms rewired to one-line calls (~24 lines out of the shim; three copies → one). `cargo check` + `fmt --check` green. |
| I2 | LOW | Doc/AC overstatement: "byte-identical on every hint-free row" is FALSE for a zero-width char (combining mark) immediately after a token — `cols_to_bytes` deliberately never makes an empty cell window a boundary, so the end map ABSORBS the mark's bytes into the token span (old display-lex left them in dropped Plain). Probed concretely by TWO critics independently (`"if\u{301} x"`: old `0..2`, new `0..4`); the ONLY divergence class across a 25-line × 3-tab-width corpus. | **REAL (doc + AC pin, not a code bug)** — behavior is primary-arm-consistent since #268 and keeps the grapheme cluster one styled run; NOT the #315 floor-regression class. | Doc comment softened with the exact exception; spec REQ-002 amended with the pinned deliberate difference; test plan gains T4b asserting the NEW behavior. |
| I3 | LOW | New fn ships without its own unit test (whole-fn → `vec![]` and drop-the-map mutants currently unkilled). | **EXPECTED** — tests are Phase 4 by pipeline design; gate:4/5 will demand them (code_syntax.rs/code_view.rs are covered + unskipped — verified against gates.sh:229). | None here; validate owes T1–T5 + T4b. |
| I4 | LOW | Dependency-direction observation: code_syntax → code_view is a new reverse edge (code_view → code_syntax existed since #266). Legal intra-crate mutual ref with three existing precedent pairs; the I1 helper split puts the remap in code_view, keeping the composition thin. | NOTED, no defect | — |

Verified-clean lenses (evidence in critic transcripts): domain composition raw-bytes→display-bytes correct incl. multibyte/tabs/EOL/empty (probe corpus, 0 disjoint-ascending violations); EOL-anchored phantom excluded by `col_ends` construction (probe: exact abutment); mid-token phantom REACHABLE (server anchors are arbitrary) but harmless — hints-first + first-containing-range resolution makes phantom cells Hint even inside an enclosing span (probe control: hints-LAST would leak — ordering is load-bearing and present); adjacent mapped spans touch, never overlap (col_ends[e-1] ≤ col_starts[e], byte_at monotone); both call sites bind `text`/`layout` from the same iteration (no stale-display scenario); primary arms behavior-unchanged (I1 rewire is shape-only); all other lexer consumers clean (hover card raw-domain; #246 split-pane has no inlay channel; ⌘/ reads the untouched lang_spec table); §20 provenance original (composition of in-repo #99/#331 fns, shape prescribed by the in-repo AD).

**Reachability upgraded (ledger-worthy):** the exotic-separator mechanism is
CONFIRMED in code — the memo splits rows on `\n` only
(`lines_from_spans`, crates/syntax/src/lib.rs:138-141, self-documented) while
the ropey buffer also breaks on bare `\r`/VT/FF/NEL/LS/PS
(crates/editor/src/buffer.rs:136-139) → rows past the first exotic break
mis-align and rows ≥ memo length miss FOREVER. And a SECOND phantom producer
needs no LSP race at all: the #305 fold-ellipsis marker (`"  ⋯ N lines"`)
rides the same `einlay` channel version-independently, so a folded header on
an async-window tail row coexists with the fallback deterministically. Also:
the ticket's `'a`-lifetime example was never a real trigger (Rust's hand-lex
quote set is `"` only) — `"` and `//` are the true seeds.

## Phase 4 — Validate
- **Tests added** (code_syntax.rs `#[cfg(test)]`, per the Phase-2 plan):
  - T1 `t333_fallback_phantom_no_bleed` — `let x = 1; // c` + mid-line phantoms
    `: "weird` and `// note`: exact mapped spans (shift parametrized by phantom
    width), covered display text asserted, no span intersects `hint_spans()`.
  - T2 `t333_fallback_phantom_negative_control_old_shape_bled` — the pre-#333
    display-lex on the same construction yields `(7..23, Str)` — the bleed
    covering the real `1`; proves T1's construction catches the old bug.
  - T3 `t333_fallback_floor_identity_no_phantoms` — TOML/Rust exact spans +
    new==old identity; Markdown/Plain empty.
  - T4 `t333_fallback_tab_identity` — tabs before/inside a TOML string +
    CJK-in-string: new==old span-for-span, expanded text covered.
  - T4b `t333_fallback_zero_width_absorption_pinned` — `if\u{301} x`: old
    `(0..2)`, new `(0..4)` — the REQ-002 exception pinned as deliberate.
  - T5 `t333_fallback_spans_disjoint_ascending_with_phantoms` — mid-line +
    EOL phantoms: ascending, disjoint, char-boundary, no phantom overlap, EOL
    hint abuts the last span exactly.
- **Runs (real, transcript):** `cargo nextest run --workspace` →
  **2071/2071 passed, 5 skipped** (pre-existing headed/#[ignore] lane); all
  t331 suites green unmodified (T6). `cargo test --workspace --doc` → ok.
  (One authoring slip caught by the suite itself: T1's first cut hardcoded the
  8-char shift for the 7-char `// note` phantom — fixed to `phantom.len()`.)
- **Live-app drive (render path touched → mandatory):** bundled
  (`bundle-app.sh debug`), direct-exec'd from the repo root (the `open` launch
  hit the documented M29 #405 TCC stall — process alive, zero AX windows —
  twice; the README's direct-exec fallback worked first try). Drove
  ⌘P → `Cargo.toml` → ⌘↵ (System Events key code 36 — `drive.swift`'s `cmd:`
  verb is single-char only): Cargo.toml opened as an editor tab and the
  PERMANENT-FLOOR fallback (`highlight_display_ranges` live) rendered it —
  captures `scratchpad/t333_pre.png` + `t333_toml.png`, READ: all four string
  literals green, keys plain, layout/caret intact. Pixel-sampled (magick
  histograms): Str region glyphs `#60BF67` (green), key region `#E0E1E3`
  (plain fg) — objective, not eyeballed. Sticky-header site not separately
  driven (needs a scrolled fold-scope; same helper, identity-proven by
  T3/T4) — noted, not silently skipped. React parity pair: N/A per spec
  (no UI delta).
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]**, all 15 gates
  PASS; coverage ≥100% lines; mutation 2 caught / 0 missed → MSI 100.0%;
  receipt written. Two red cycles first, both fixed at source: (1) gate:1 —
  the t333 test block was written after the last fmt pass (caught by the
  Stop-hook fmt check; `cargo fmt --all`); (2) gate:2 — `clippy::type_complexity`
  on T3's `cases` array annotation (dropped the annotation; assert_eq
  back-propagates the type). No pre-existing failures encountered; the
  `block v0.1.6` future-incompat note is a transitive-dep advisory outside
  this change (visible in cargo output, not a gate).

## Phase 5 — Complete
- **CHANGELOG:** Fixed entry added (Unreleased → Fixed, TICKET-333) — the
  bleed mechanism, the confirmed reachability (memo `\n`-split vs ropey exotic
  breaks; fold-marker as the race-free second producer), the shared-remap fix,
  the pinned zero-width difference, the test matrix + live capture + gate line.
- **Architecture docs:** `docs/marley_architecture/editor.md` — the
  "Known narrow limit (#333)" paragraph replaced with the fixed record (raw-lex
  + the one shared remap; any future span producer maps through
  `spans_to_display_bytes`).
- **Parity sync:** N/A — no UI delta (spec `## React-first`), nothing to
  back-port.
- **Ledger appends (§19):**
  - `PR-claude-identity-claims-over-display-maps-probe-zero-width-boundaries-001`
    (appended at inspect).
  - `L-claude-fallback-arms-share-the-primary-arms-shape-structurally-001`.
  - `AD-claude-one-raw-to-display-remap-for-all-span-producers-001`.
  - No F- block: no shipped-behavior bug was found by the critics (the two
    reds were in-pipeline doc/format/lint faults, fixed at source).
- **Ticket:** TICKET-333 → `tickets/closed/`, status line updated; BACKLOG
  swept (row left at promotion — none stale).
- **Archive:** spec status set Phase 5 PASS; pair moved to
  `docs/planning/pipeline/completed/`.
