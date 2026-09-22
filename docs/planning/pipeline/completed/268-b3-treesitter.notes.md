# 268-b3-treesitter — Notes

- **Forge ticket:** #268 68014b92-f5b9-4cf4-9267-18980b286418
- **AAR:** 9b7dc2f7-06f2-4fa9-b42d-dbff893bd770
- **Local ticket doc:** docs/planning/tickets/open/TICKET-268-b3-treesitter.md
- **Pipeline spec:** 268-b3-treesitter.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan
- **Request:** /goal batch, ticket 10 of 10 (the LAST). B3 slice 1.
- **Gate viability recon (checked BEFORE the spec):**
  - deny.toml allowlist has MIT/Apache — tree-sitter (MIT) +
    tree-sitter-rust (MIT) admissible; `cargo add --dry-run` resolves
    0.26.11 / 0.24.2 cleanly.
  - gate:6 miri is conditional: a crate is miri-checked iff its OWN src
    contains `unsafe` and no exemption is declared. `marley_syntax`
    will contain no own unsafe → skipped by rule (the FFI lives in the
    dependency, exactly like gpui for marley_app).
  - Consumers today: `highlight_ranges` at app.rs:2744 (the #266 editor
    branch — the swap point) and `highlight_line` at app.rs:2951 (the
    #246 read-only pane — UNTOUCHED this slice).
  - `language_of`: rs/toml/json/sh/md → only `Rust` moves to the parse.
- **Slice rationale:** the forge ticket names incremental + off-thread;
  those ride the NEXT slice — v1 anchors correctness with a synchronous
  whole-file parse memoized by `(path, buffer.version())` (files we
  dogfood are small; tree-sitter parses MBs in ms). Recorded in the spec
  Out-list + a ticket comment at close.
- **Autonomy note:** /goal run — no human pause.

## Phase 2 — Design

### API ground truth (probe crate, real runs — scratchpad/ts-probe)
- tree-sitter 0.26: `Parser::new()` + `set_language(&LANGUAGE.into())` +
  `parse(src, None)`; `Query::new(&lang, tree_sitter_rust::
  HIGHLIGHTS_QUERY)`; `QueryCursor::matches` yields a
  **StreamingIterator** (`while let Some(m) = it.next()`).
- The grammar's 21 capture names (printed, closed set for 0.24.2):
  type, type.builtin, property, constant, constructor, function,
  function.method, function.macro, comment, comment.documentation,
  punctuation.bracket, punctuation.delimiter, variable.parameter, label,
  keyword, variable.builtin, string, constant.builtin, escape,
  attribute, operator.
- Probe-pinned mapping facts: numbers (42, 1.5e3, 0xFF) AND
  true/false → `constant.builtin`; char literals → `string`; `///`
  comments double-capture (`comment` + `comment.documentation`, SAME
  range); `escape` (\n) captures NESTED inside its `string`; a doc
  comment's span INCLUDES the trailing `\n`; block comments span lines;
  a string containing `//` is one `string` capture (no comment).

### marley_syntax (NEW crate, crates/syntax — no own `unsafe`)
- `enum TokenKind { Keyword, Str, Number, Comment, Plain }` (a mirror —
  the crate can't depend on marley_app; the app maps at the seam).
- `kind_of_capture(name) -> TokenKind`: keyword→Keyword; string→Str;
  comment|comment.documentation→Comment; constant|constant.builtin→
  Number; `_`→Plain. The 21-name truth table is a test.
- `highlight_lines(src) -> Vec<Vec<(Range<usize>, TokenKind)>>`:
  1. parse + query; collect `(start, end, kind)` DROPPING Plain-mapped
     captures FIRST (an unknown capture nested in a string must not
     punch a hole);
  2. sort by (start, end) then SWEEP to disjoint (`s = s.max(at)`;
     keep if `s < e`; `at = e`) — the duplicate doc-comment capture
     clips to empty (dropped) and a nested escape clips away entirely
     (the string paints whole);
  3. split at line starts; per line `[ls, le)` (le EXCLUDES `\n` — a
     span's trailing-\n tail clips off); intersect + rebase to
     line-local bytes.
- The compiled `Query` lives in a `OnceLock` (compiling the scm is the
  expensive part; per-keystroke refresh must not re-compile); a fresh
  `Parser` per call (cheap, and Parser is !Sync).
- deps: tree-sitter 0.26, tree-sitter-rust 0.24 (both MIT).

### App integration
- `code_syntax::kind_from_syntax(marley_syntax::TokenKind) -> TokenKind`
  — the 5-arm seam map (pure, tested).
- `code_view::raw_span_to_display_bytes(line, layout, span) ->
  Range<usize>` — raw line bytes → char indices → display cols
  (`col_of_offset`) → display bytes (`cols_to_bytes`): composes three
  EXISTING tested pieces; handles tabs inside/before tokens + multibyte.
- RootView: `syntax_cache: Option<(PathBuf, BufferVersion,
  Rc<Vec<Vec<(Range<usize>, TokenKind)>>>)>` (code_syntax kinds,
  pre-mapped at refresh; Rc so the 'static row closure clones cheaply).
  A pure `needs_syntax_refresh(cache_key, path, version) -> bool` +
  `refresh_syntax_cache(&mut self)` called in the render path BEFORE
  `EditorDraw` borrows the buffer; Rust files only.
- The row closure (app.rs:2744 region): if the entity's cache covers
  this row → remap each raw span via `raw_span_to_display_bytes` and
  feed `styled_slices` as today; else (non-Rust / no cache) → the
  existing `highlight_ranges(display, lang)` fallback. The #246 pane
  (app.rs:2951) untouched.

### File manifest
| file | change |
|---|---|
| Cargo.toml | workspace member crates/syntax |
| crates/syntax/{Cargo.toml, src/lib.rs} | NEW marley_syntax (wrapper + map + sweep + clip + tests) |
| crates/marley_app/Cargo.toml | dep marley_syntax |
| crates/marley_app/src/code_syntax.rs | + kind_from_syntax |
| crates/marley_app/src/code_view.rs | + raw_span_to_display_bytes + tests |
| crates/marley_app/src/app.rs | syntax_cache field/init + needs_syntax_refresh + refresh + row-closure swap |

### Regression test plan
| REQ | Test |
|---|---|
| REQ-001 | the 21-capture truth table through kind_of_capture (kills every arm) + kind_from_syntax 5-arm map |
| REQ-002 | highlight_lines fixtures: the probe fn source (keywords/string-with-`//`/types-Plain); 2-line block comment (line 2 = one Comment span 0..len); doc-comment trailing-\n clip; escape-in-string (whole Str, no hole); duplicate-capture dedupe (ONE comment span); multibyte ident lines (char-boundary byte spans); empty src; trailing line without \n; numbers/bool → Number |
| REQ-004 | needs_syntax_refresh truth table (same/diff path, same/diff version — kills the == mutants); headless: open the 120-line `long.rs` fixture → cache Some, 121 line-vecs, line 0 = Comment (the fixture's `// line N` rows) |
| REQ-003 | the block-comment continuation line — headless span assert (line 2 Comment from the parse; the hand lexer CANNOT know) + driven capture of movement.rs (its `//!` header + block comments painted) |
| REQ-005 | existing hand-lexer tests unchanged; driven #246 pane capture (byte-path untouched) |
| raw remap | raw_span_to_display_bytes rows: no-tab identity; tab BEFORE a token (display shift); tab INSIDE a token (expansion widens); multibyte é/😀 |
| REQ-006 | gates.sh --diff (deny MIT, machete, rustdoc) |
- Uncoverable: the render-closure branch + refresh call sites live in the
  documented app.rs exclude — asserted by the headless cache asserts +
  driven captures.

### Risks / decisions
- R1 Query compile per refresh → OnceLock (compile once per process).
- R2 v1 parses the WHOLE file synchronously per version change
  (keystroke): ms-scale at dogfood sizes; the off-thread/incremental
  slice is the recorded follow-up (the ticket's own text).
- R3 Grammar-version drift renames captures → the truth-table test
  fails loudly on a bump (deliberate pin).
- R4 marley_syntax's parse-calling fn: body-replace mutants killed by
  the fixture tests (Vec has Default — viable and killable).

## Phase 3 — Implement
- **Built to manifest:** NEW `crates/syntax` (marley_syntax: the mirror
  `TokenKind`, `kind_of_capture` exact-arm map, `highlight_query()`
  OnceLock [compile-once], `highlight_lines` = collect-with-Plain-dropped
  → sort → sweep-to-disjoint → per-line clip [le excludes \n; a span
  crossing lines is NOT advanced past at the break so its remainder
  reaches the next line]); marley_app dep; `code_syntax::
  kind_from_syntax` (5-arm seam); `code_view::raw_span_to_display_bytes`
  (raw bytes → char count → col_of_offset → cols_to_bytes — composes
  three existing tested pieces); app.rs — `SyntaxLines` alias (clippy
  type_complexity fixed at source), `syntax_cache` field/init, pure
  `needs_syntax_refresh(cached_key, path, version)`,
  `refresh_syntax_cache` (Rust-only, clears on non-Rust, whole-file
  parse + kind pre-map into the Rc) called at the TOP of the code-view
  render arm (before EditorDraw's shared borrows — the arm now checks
  `is_some()` then re-reads), and the row closure prefers the memo
  (per-row `lines.get(row)` + raw→display remap) with the #266
  `highlight_ranges` hand-lexer fallback for non-Rust/no-cache; the
  #246 pane path untouched.
- **Deviations:** none of substance; the render-arm head became
  is_some()+expect (checked two lines above — the borrow order needed
  `&mut self` refresh before the shared `cv` borrow).
- **Verification:** fmt; check clean; clippy -D warnings clean; full
  `cargo nextest run --workspace` **936/936**.

## Phase 3.5 — Inspect
- **Critic run:** 1 deep adversarial critic (probe crate mounting the REAL
  sources; **11 hand-built single-operator mutant copies of lib.rs diffed
  over 419 inputs** — 4 corpus files incl. the 416KB app.rs + 400
  structured fuzz cases; gpui frame-coherence traced in the vendored
  source; licensing verified in the vendored crates). Parallel
  self-review pre-traced the equivalent-mutant sites and had the reshape
  designed before the report landed — the critic CONFIRMED both
  equivalence claims empirically (0 diffs/419 each) and the reshape's
  killability, with two provisos (both applied).
- **Findings ledger:**

| # | sev | finding | verdict | action |
|---|---|---|---|---|
| F1 | HIGH (gate-5) | 5 EQUIVALENT mutants in the sweep/clip loops (93:14 `<`→`<=`, 113:23 `<`→`>` [skip loop = pure optimization], 117:53 `<`→`<=`, 124:18 `>`→`==`/`>=` [the crossing-span break is semantically DEAD given sweep disjointness]) — each probe-proven output-identical over 419 inputs | REAL | FIXED by RESHAPE: `sweep_disjoint` extracted as its own seam (an admitted empty span is now OBSERVABLE — `sweep([(3,3)])==[]` kills the `<=`); the scanning clip loop replaced by per-span `clip_to_lines` (first/last lines via `partition_point`, `saturating_sub`, min/max, ONE `cs < ce` guard killed by the empty-line-in-comment fixture). Post-reshape `--list`: every viable mutant has a fixture; the skip/break sites no longer exist |
| F2 | MED | the cache key `(path, version)` ≠ buffer identity — `BufferVersion` restarts at 0 per fresh Buffer, so REOPENING a file the agent rewrote on disk (Marley's core loop!) rendered STALE spans until the first keystroke (S2); same-path-two-projects ditto (S1) | REAL | FIXED: `OpenFile.nonce` (process-monotonic AtomicU64 mint) + `active_nonce()`; the memo re-keyed `(nonce, version)`; `needs_syntax_refresh` takes the nonce |
| F3 | MED | the "single-digit ms" perf claim fails at dogfood size — MEASURED: app.rs (8,089 lines) = **25.9ms**/refresh (14.6 parse + 9.5 query + 1.8 sweep/clip); 319 lines = 1.1ms; 721 = 2.4ms. Per-keystroke sync at 8k lines blows the 16.7ms frame | REAL | v1 ACCEPTED with the numbers disclosed (CHANGELOG + the follow-up note): fine to ~2.5k lines; the incremental/off-thread slice (already the ticket's recorded follow-up) is REQUIRED for self-hosting, and `set_byte_range` only halves the query side — `InputEdit` via #269 deltas is the real fix |
| F4 | LOW | line-model divergence: ropey also breaks on bare `\r`/VT/FF/NEL/LS (legal Rust whitespace); highlight_lines splits on `\n` only → exotic files misalign rows after the break (rows past the cache fall back to the hand lexer; no panic — probed) | REAL, accepted | documented in `highlight_lines`' doc (the same divergence class buffer.rs already records for the #246 pane) |
| F5 | LOW/WATCH | gate-4: the `unreachable!` closure in `highlight_query` = a dead uncovered region | REAL | FIXED: `.expect(...)` (single covered line; compile-time-constant grammar pair, documented not-an-input-path) |
| F6 | NIT | per-row `Vec` clone in the row closure; a misleading "do not advance past it" comment | REAL | FIXED: map over the borrow (`iter()` + `r.clone()` per span, not per row-vec); the comment died with the reshape |

- **Verified clean (critic, probe-executed):** all 9 clip-lens cases exact
  (multi-line middles full-width; end-at-EOL; start-at-line-start;
  adjacent; empty-line-in-comment → EMPTY row; no-trailing-\n; ""→[[]];
  crossing-then-later spans; multibyte); corpus invariants over 8,090
  rows/4,782 spans (ascending, disjoint, in-bounds, char-boundary,
  non-empty, no Plain leak); wrong-kind-tail UNREACHABLE (5,164-span
  census: overlaps are exclusively identical doc-comment duplicates — 0
  same-start-different-end, 0 partial, 0 cross-kind); line-count parity
  with ropey on \n-only files (""/"a"/"a\n"/"a\n\n" = 1/1/2/3 both
  sides); cache/frame coherence PROVEN in gpui source (notify → dirty →
  draw_roots re-renders the root; uniform_list's render_items runs
  same-frame prepaint; the refresh precedes list build → no frame where
  rows and cache disagree); REQ-003 wins confirmed ("a // not a comment"
  → Str only; `formatter`/`for_x` → not keyword); REQ-005 fallback
  intact (`Some([])` ≠ fallback — an empty Rust row does NOT flash the
  hand lexer); `raw_span_to_display_bytes` exact on tabs/multibyte/OOR;
  mutants::skip integrity (no detach; app.rs 17 listed = 12 pre + 5
  pure); licensing (tree-sitter 0.26.11 MIT, tree-sitter-rust 0.24.2 MIT
  [LICENSE verified], tree-sitter-language MIT, HIGHLIGHTS_QUERY ships
  inside the MIT grammar; one locked version each; no GPL text).
- **Phase-4 kill list (post-reshape census: 50 listed; the tuple/Range
  Default bodies + Query Default are UNVIABLE [no TokenKind/Query
  Default]):** T1 21-name truth table (4 arm-deletes); T2 ""/"x" →
  [[]] (+ the empty/plain guard); T3 `"let mut x = 42;"` exact row;
  T4 `"// tail\nlet x = 1;\n"` exact rows (kills the line_starts `+1`
  and `ns - 1` arithmetic); T5 4-line block comment exact rows
  (interior-row rebase `- ls` swaps + `take(last+1)` arithmetic);
  T6 `"/* a\n\nb */\n"` row 1 == [] EXACTLY (kills `cs < ce`→`<=`);
  T7/T8 multi-line string + multibyte; sweep-seam rows: dedupe, nested,
  EMPTY-SPAN-dropped (`sweep([(3,3)])==[]` — the F1 kill), unsorted-in;
  needs_syntax_refresh 4-row truth table (5 viable, 0 equivalent);
  raw_span "\ta" 4..5 + multibyte (3 viable); kind_from_syntax 5-row
  zero-mutant guard.
- **Post-fix verify:** fmt; check; clippy -D warnings; **936/936**;
  `--list` re-run — the equivalent class is structurally GONE.

## Phase 4 — Validate
- **Kill-list tests (the critic's post-reshape census, all landed):**
  marley_syntax 11 tests — T1 the 21-name truth table ASSERTED AGAINST THE
  REAL QUERY (set equality with `parse::query_capture_names()` — a grammar
  bump fails loudly); the sweep seam (empties dropped/dedupe/nesting/
  unsorted — the F1 observable); T2 empty/plain per-line empties incl.
  ropey line-count parity; T3 exact single-line row; T4 line-comment+code
  rows (the `+1`/`ns-1` arithmetic); T5 4-line block comment (interior
  rebase); T6 empty-line-in-comment row == [] EXACTLY (the `cs < ce`
  killer); T7 string-with-`//` + multi-line string closing at col 0 (the
  partition boundary); T8 multibyte + escape-in-string; doc-comment
  dedupe; numbers/bools → Number. App: `needs_syntax_refresh` 4-row truth
  table (nonce-keyed); `raw_span_to_display_bytes` tab/multibyte/OOR rows;
  `kind_from_syntax` 5-row zero-mutant guard; `editor_surface` nonce
  uniqueness incl. the REOPEN case (the F2 regression pin).
- **Headless (the #264 lane, 9/9):** NEW
  `syntax_cache_populates_and_rekeys_on_edit_headless` — a 120-line REAL
  Rust fixture opens through the render path; the memo populates; line 0
  == `[(0..9, Comment)]` SEMANTICALLY; a keystroke keeps the nonce but
  moves the version key (REQ-004 live).
- **Driven captures (bundled fresh binary, PNGs read):**
  - `268-1-semantic.png` — movement.rs through the PARSE: doc comments
    muted, keywords accented, `"word"`/`'_'` strings green, numbers
    amber; crucially line 13's `` `pub(crate)` `` INSIDE a doc comment
    stays comment-muted (the hand lexer would keyword-paint it) and
    line 16's string+`||` renders as string-then-code (REQ-003 pixels).
  - `268-2-string-divergence.png` — typed live into line 6:
    `let s = "a // not comment";` → `let` keyword-cyan, the WHOLE string
    green INCLUDING the `//` (the semantic win the ticket names), `;`
    plain, dirty ● on. Typed through the #267 IME path — B2+B3 composing.
  - `268-3-246-pane.png` — the #246 read-only split pane renders
    movement.rs rows 1-40 through its untouched hand-lexer path next to a
    live terminal (REQ-005; the scratch line was ⌘Z'd out first — line 6
    back to empty).
- **Full suite:** 952/952 → 953 after the headless add (the final gate
  run's nextest).
- **Gate:** run 1 RED ×3 — source-bans (two prose `unsafe` mentions in
  the new crate's comments — the gate greps every file under crates/;
  reworded to "entirely safe Rust"), miri (same literal-word trigger:
  the WORD 'unsafe' in a comment made the gate think the crate needs
  miri; the reword clears it — the crate has zero real unsafe), coverage
  (llvm-cov attributed phantom zero-count regions to the parse-chain
  lines). Coverage fixed at SOURCE by the documented shim-split
  precedent: the FFI-adjacent half moved to `syntax/src/parse.rs` and
  joined the gate's ACCEPTED-UNTESTABLE exclude list with its reason
  (behavior pinned by lib.rs's exact fixtures + the NO-excludes mutation
  gate — parse.rs's 10 listed mutants all remain killable and green);
  the pure layer stays in the 100% denominator. Re-run: **GATE GREEN
  [diff] 15/15** (receipt written).

## Phase 5 — Complete
- (pending)
