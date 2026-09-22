# Bracket-match highlight (#340) — Notes

- **Forge ticket:** #340 b35af9e6-04c1-47a6-91af-f1614a2581fe
- **AAR:** d6b81d92-45c1-4f2c-bf64-9473de807e98
- **Local ticket doc:** ../../tickets/open/TICKET-340-bracket-match.md
- **Pipeline spec:** 340-bracket-match.spec.md

<!-- Working scratch. Each phase appends its entry. Excluded from gate:14 doc-todos. -->

## Phase 1 — Plan

Promoted `queued/m22-bracket-match.spec.md` → `active/340-bracket-match.spec.md` (pipeline_id
`4c3a1e3e-4d31-455d-ace5-5cf3897add1f`, AAR `d6b81d92…`, local ticket doc written). Classification: work
pipeline, feature, one slice. `active/` was empty. `main` @ `516eda4`. **The FIFTH and LAST ticket of the goal.**

### THE VERIFICATION LEDGER — the streak holds: **5 runs, 5 authorship errors caught**

The public seams all EXIST with essentially the claimed shape. But the **load-bearing cadence claim is FALSE**,
and it reshapes the design — the same class as #337's premise and #339's central decision.

| # | Claim | Verdict |
|---|---|---|
| 1 | `matching_delimiters(&HighlightSession, byte_pos)` mirrors the #329 `enclosing_ranges` / #330 `all_headers` shape | **VERIFIED — a viable IN-CRATE addition** (F2: the mirror is not literal). |
| 2 | `HighlightSession` holds the tree; `enclosing_ranges`/`all_headers` are iterative lib.rs walks | **VERIFIED** — session lib.rs:309-318 (tree PRIVATE); `enclosing_ranges` :461, `all_headers` :505, both real. |
| 3 | "tree = truth, a `(` in a string/comment is not a delimiter node" | **F3 — VERIFIED at the SPAN level (T7/T8), NOT the tree level.** Strong but must be fixture-confirmed. |
| 4 | `styled_slices_with_marks` + `MarkTier` + a mark RETAINS a Plain slice | **VERIFIED** — code_view.rs:368 / :346 / :411. |
| 5 | `raw_span_to_display_bytes` is "the right mapper for a mark" | **F4 — it EXISTS (:304, phantom-safe) but is the CODE-SPAN mapper; today's marks use the cols path.** A choice, not precedent. |
| 6 | **"recompute per caret move on the CACHED tree, never a reparse per frame"** | **F1 — FALSE. There is NO cached tree app-side. The #329/#330 route reparses the whole file per caret move.** |
| 7 | ⌘⇧\ is FREE | **VERIFIED** — no backslash binding of any spelling in keymap.rs; roster is 66. |
| 8 | No NavStack push (intra-expression) | **VERIFIED coherent** — `NavStack` exists (editor_nav.rs:29); the decision holds. |

---

**F1 [MATERIAL — the cadence claim is false, and it reshapes the design].**
The spec's central performance story: "Recompute per caret move on the CACHED tree (the #330 cadence: one
reparse per EDIT, never per caret move or frame)". **There is no cached tree on the app side.** The Explore
sweep confirmed:
- The live incremental `HighlightSession` (which owns the `tree_sitter::Tree`) is created **inside the syntax
  worker thread** (app.rs:11293) and never crosses back — the worker sends only `SyntaxResp{lines}`
  (per-line spans), not the tree. A struct-field scan of `marley_app` found no `HighlightSession`/`Tree` field.
- The caches keyed on `(nonce, version)` (`syntax_cache` app.rs:459, `sticky_headers` :283, the #329 ladder
  key :279) all store **computed results**, never the tree.
- The #329/#330 node-range callers **reparse a throwaway session** — `HighlightSession::new()` +
  `highlight_full(&text)` (a FULL whole-buffer parse) — every time they need a node range (app.rs:3224-3226,
  3296-3298). The doc comments call it "a throwaway HighlightSession".

→ **Bracket-match recomputes on EVERY caret move.** Following the #329 route literally means a **full-file
reparse on every arrow key** — the opposite of "never a reparse". D-TREE-IS-TRUTH still stands (the tree is
the correctness source), but the cadence is a **Design Fork**, not a locked cheap path:
- **Fork A-(a): accept the per-caret-move reparse.** #329's selection-ladder already does exactly this and
  shipped, so it is a known-acceptable cost — but the ladder fires on ⌥↑ (rare), while bracket-match fires on
  every arrow. tree-sitter full-parse is fast (low-ms), so on typical files this may be fine; **Design must
  bound it** (a size gate? debounce? measure the reparse on a large file).
- **Fork A-(b): surface/cache the tree app-side.** Keep an app-side persistent `HighlightSession` reparsed
  incrementally on edit (bounded plumbing), so a caret move reads a cached tree. More work, but it makes the
  spec's original claim true and benefits any future tree consumer.
Design decides with a measurement, not a guess (the #336 lesson: read the substrate, don't assume the cost).

**F2 [the mirror is similar-shaped but NOT a literal copy].** `enclosing_ranges` uses
`named_descendant_for_byte_range` (a NAMED node) then climbs `.parent()`. But bracket delimiters are
**anonymous leaf tokens** — the code itself relies on this (lib.rs:474-477). So `matching_delimiters` must use
the **non-named** `descendant_for_byte_range` + `child(0)`/`child(child_count-1)`, a different walk. It is
still a small in-crate addition, and **PRIOR ART confirms the primitives are the crate's**: tree-sitter owns
`child`/`child_count`/`descendant_for_byte_range`/`parent` (lib.rs:1713-1972) but has NO delimiter/bracket
concept — so `matching_delimiters` is a **thin adapter over tree-sitter's node API**, exactly the #339
D-EMPTY-ADVANCE pattern (build on the crate's primitives, don't hand-roll a tree walk).

**F3 [the correctness claim is span-tested, not tree-tested].** "A `(` inside a string/comment is not a
delimiter node" is the load-bearing correctness claim. The repo proves it at the **highlight-span** level (T7
`strings_are_semantic_not_lexical` keeps `//` inside a string as one `Str` span; T8 the escape-in-string
hole), but no test dumps the **tree** at a byte inside a string to prove the node is `string_content`, not an
anonymous `(`. The bracket-matcher queries the tree, so this is one step removed. **Design MUST confirm with a
real tree-dump fixture** — parse `let s = "(";` and `// (` and assert the node at the inner `(` is
`string_content`/`line_comment`. Very likely true (same grammar mechanism), but "very likely" is not a test.

**F4 [the mark mapper is a design choice].** `raw_span_to_display_bytes` exists and is phantom-safe on the END
(col_of_span_end, the #331 seam), and it is the mapper the syntax spans use (app.rs:4607). But **today's marks
— the find bands — do NOT use it**; they map via `row_selection_cols` → `cols_to_bytes` (the selection path,
app.rs:4659-4666). A bracket delimiter is a raw byte span from the tree (like a syntax span), so mapping it via
`raw_span_to_display_bytes` lands it in the right display-byte domain — **defensible, but a choice, not
precedent.** Design must pick deliberately and say why.

**F5 [MarkTier variant touches 3 exhaustive sites].** Adding a `MarkTier` variant is feasible but must update
the two exhaustive consumers: the tier-priority `max_by_key` (code_view.rs:410) and the render band-priority
`match mark {…}` (app.rs:4690-4694). Small; flagged so it is complete, not partial (the #339 lesson — don't
drop half the wiring).

### Confirmations
- **§20 CONFIRMED** — VS Code / Zed = OBSERVED behavior (adjacency preference, the ⌘⇧\ jump, highlight-not-
  select). tree-sitter = published-API reuse (MIT). The walk + mark plumbing are Marley-original over shipped
  seams. No copyleft source read.
- **The EARS AC (REQ-001..008) STAND** — F1 changes the *implementation* of the recompute REQ (a reparse
  cadence decision), not its truth condition (the pair lights on caret move). REQ-006-ish (the "never reparse"
  performance REQ, if the spec pins one) is the one to re-word honestly.
- **Decisions:** D-TREE-IS-TRUTH, D-PAIR-SET, D-MARK-NOT-TOKEN, D-BEFORE-THEN-AT, D-PRIMARY-ONLY, D-NO-NAVSTACK
  all **stand**. **The cadence sub-claim under D-TREE-IS-TRUTH is REOPENED** (F1) — Fork A.
- **The chord**: ⌘⇧\ is `chord(true, false, false, true, "\\")` — punctuation is spelled as the literal char
  (precedent: `/` is `"/"` for #299's comment toggle, not `"slash"`). The roster assert (66) becomes 67.

### Forks for Design
- **Fork A (F1):** the recompute cadence — accept the #329-style per-caret full reparse (measure it), or build
  an app-side cached tree. A measurement decides, not a guess.
- **Fork B (F3):** confirm the string/comment tree structure with a real fixture before relying on it.
- **Fork C (F4):** the mark's display mapper — `raw_span_to_display_bytes` (code-span domain) vs the cols path.

### Standing context
**LIVE synthetic-input drives OFF-LIMITS** (chad at the machine) → units + headless + mechanism. A bracket
highlight is a **RENDER change**: the #339 lesson is binding — do NOT drop the render half, and do NOT declare
Phase 3 PASS without a headless STATE assert + a deferred-pixel note. **Push remains UN-OK'd** (all commits
LOCAL). Batch lessons + the two newest (**diff Phase 3 vs the Phase 2 manifest**; **a passing test proves
nothing till you watch it fail**) are in [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).

status: Phase 1 — Plan PASS; ready for Phase 2 — Design

## Phase 2 — Design

**All three forks answered by a spike + a measurement (the #336/#339 discipline: read the substrate, don't
guess), and one — Fork B — is a genuine correctness result the repo could not prove.** The spike ran the same
grammar versions the crate pins (tree-sitter 0.26.11 / tree-sitter-rust 0.24.2), then was DELETED.

### FORK B — RESOLVED: D-TREE-IS-TRUTH holds, proven at the TREE level (not just spans)

The spike parsed the exact fixtures and dumped the node at the inner delimiter:

| input | node at the inner `(` | `matching_delimiters` |
|---|---|---|
| `let s = "(";` | `string_content` (named) | **None** |
| `let x = 1; // (` | `line_comment` (named) | **None** |
| `fn f() {}` (the real `(`) | `"("` (anonymous, `is_named=false`) | `Some((4..5, 5..6))` |
| `let v = [1,2];` | `"["` | `Some((8..9, 12..13))` |

So a `(` inside a string/comment is genuinely NOT a delimiter node — it is swallowed by the parent
`string_content`/`line_comment`, and `descendant_for_byte_range` lands *inside* that leaf, whose children are
not a delimiter pair. **The classic false-positive class cannot exist.** This is the whole justification for
Rust-only-via-the-tree, and it is now proven rather than asserted (Phase 1 F3 flagged it as span-tested only).

### FORK C sub-answer — RESOLVED: the anonymous node's kind IS the token text

`kind()` on the delimiter node returns the literal `"("` / `")"` / `"["` etc., `is_named=false`. So the pair
set is matched by `child.kind()` against `{"(",")"}`/`{"[","]"}`/`{"{","}"}` — no name lookup, no query. A real
impl detail the spike surfaced: **`Node::child(i)` takes `u32`, not `usize`** (a compile error in the
prototype — worth knowing before Implement).

### FORK A — MEASURED, and the decision is (a) with a parse-only path + a filed follow-up

**A full parse of a 2000-line / 48 KB synthetic Rust file: 4.0 ms (best-of-20, release).** Linear in size, so
~1 ms at 500 lines (typical), ~2 ms at 1000. Phase 1 was right that this is NOT free — it is above the
"~2 ms, just do it" bar at large sizes.

**Decision: (a) per-caret recompute, but PARSE-ONLY, keyed on (nonce, version, caret).** Rationale, with the
number in hand rather than a guess:
- The recompute is memo-gated on `(nonce, version, caret)`, so it fires once per DISTINCT caret position, not
  per frame — a static caret costs nothing.
- **Parse-only, not `highlight_full`.** Bracket-match needs only the TREE, not the highlight query + per-line
  spans. So the pure fn owns a bare parse (~4 ms worst case), NOT the `highlight_full` the #329/#330 throwaway
  route pays (parse + query + span build, ~2× more). That is why the seam is `matching_delimiters_in(src,
  pos)` — a free fn that parses internally — NOT the spec's `matching_delimiters(&HighlightSession, …)`: the
  app has no cached session, and a session would drag in the query cost we do not need.
- 4 ms worst case is under a 16.7 ms frame and bounded by file size; for a passive highlight, a few ms of lag
  when the caret settles on a bracket in a huge file is imperceptible, and nobody reads the bracket highlight
  mid-flight during a held arrow.

**Rejected v1: (b) the app-side cached tree.** It makes the walk microseconds and would let #329/#330 stop
throwaway-reparsing too — the *right* long-term shape — but it is real cross-thread plumbing (surface the
worker's tree, or run a second incremental session on the app thread, with the invalidation that implies).
That is its own ticket. **Filed as the follow-up with the 4 ms baseline recorded**, to be taken if profiling
a real large file shows jank. When it lands, it adds the `matching_delimiters(&HighlightSession, pos)` variant
that reuses the cached tree; `_in` stays as the parse-owning path.

**NO size gate in v1** — premature; 4 ms is tolerable and file-bounded, and a gate is easy to add later if the
follow-up's profiling wants one. The number is recorded so that decision has a baseline.

### FORK C — RESOLVED: the mark maps through `raw_span_to_display_bytes`

Traced the render: the `&syntax` arg to `styled_slices_with_marks` is display-byte-mapped via
`raw_span_to_display_bytes` (app.rs:4607); today's find-band marks reach the same display-byte domain by a
different route (`row_selection_cols` → `cols_to_bytes`, the cols path). A bracket delimiter arrives as a RAW
byte span from the tree — exactly like a syntax span — so `raw_span_to_display_bytes` is the domain-correct
mapper: it lands the two delimiter spans in the same display-byte domain the syntax + marks already share, so
`styled_slices_with_marks` aligns them. **F4 resolved: `raw_span_to_display_bytes`, and the reason is domain
alignment, not just precedent.**

### Architecture

A pure walk in `marley_syntax` + a thin app cadence hook + a mark + a chord. §14 honored: all offset math in
the PURE crate (the #336/#339 lesson — byte/char/display units hide in a shim), no panics on the input path
(the walk returns `Option`, `descendant_for_byte_range` returns `Option`).

- **`matching_delimiters_in(src: &str, byte_pos: usize) -> Option<(Range<usize>, Range<usize>)>`** (NEW, pure,
  `crates/syntax/src/lib.rs`, cov/MSI 100): parse `src`, `descendant_for_byte_range(pos, pos)`, then climb
  `.parent()` while the smallest containing node's FIRST and LAST children are a delimiter pair by `kind()`.
  Returns the two children's byte ranges. Total: bad/empty input → `None`, never a panic.
- **The adjacency rule (D-BEFORE-THEN-AT) lives in the PURE fn**, as a two-probe helper: try the byte BEFORE
  the caret first (if a delimiter pair is found there — the "type `)` and the pair lights behind the caret"
  feel), else the byte AT the caret. Pinned by a table so the preference is a decision, not an accident.
- **App cadence hook** (`app.rs`): a `bracket_match: Option<(Range, Range)>` recomputed when `(nonce, version,
  caret)` moves — the #329 memo idiom, parse-only via `matching_delimiters_in(&text, caret_byte)`. No tree ⇒
  `None` (non-Rust files fall out for free). This is a **shim** (`#[cfg_attr(test, mutants::skip)]`); the
  decision is the pure fn.
- **The mark** (`code_view.rs` + `app.rs` render): a NEW `MarkTier::Bracket`, the two delimiter spans mapped
  via `raw_span_to_display_bytes`, a subtle background tint (below `Match`/`Current` in band priority — a
  bracket highlight must not fight a find band or the selection).
- **⌘⇧\ Go to Bracket** (`keymap.rs` + `app.rs`): `chord(true,false,false,true,"\\")`, Editor-scoped. On the
  opener → caret to the closer; inside/on the closer → to the opener. NO NavStack push (D-NO-NAVSTACK —
  intra-expression). Reuses the same `matching_delimiters_in` result the highlight already computed.

### §20 — CONFIRMED, and the `### Prior art` holds
VS Code / Zed = OBSERVED (adjacency, the ⌘⇧\ jump, highlight-not-select). tree-sitter = published-API reuse;
the sweep confirmed it owns the node primitives but NOT the delimiter concept, so `matching_delimiters_in` is
a thin adapter — reading tree-sitter's source to establish that is adoption, outside the wall (the #336
precedent). No copyleft source read.

### File manifest

| File | Change |
|---|---|
| `crates/syntax/src/lib.rs` | **NEW `matching_delimiters_in(src, byte_pos)`** (pure, cov/MSI 100) + the two-probe adjacency + the pair-set-by-`kind()`. `Node::child` takes `u32` (spike-confirmed). |
| `crates/marley_app/src/code_view.rs` | **`MarkTier::Bracket`** variant; the tier-priority `max_by_key` (:410) updated — **Bracket is LOWEST** (Current > Match > Bracket), so a find/selection band wins the slice. |
| `crates/marley_app/src/app.rs` | the `bracket_match` field + the `(nonce,version,caret)`-memo recompute (parse-only); the render — the two spans via `raw_span_to_display_bytes` into the marks channel, the band-priority `match mark` (:4690) gains the `Bracket` arm (a low tint, ~0.10); `go-to-bracket` dispatch. All shim (`mutants::skip`). |
| `crates/marley_app/src/keymap.rs` | `⌘⇧\` = `chord(true,false,false,true,"\\")`, Editor-scoped; the roster assert **66 → 67** with the `#340 ×1` breakdown entry. |
| `crates/marley_app/src/palette.rs` | "Go to Matching Bracket" → the `go-to-bracket` verb (a palette row + `action_for_command`, so it is discoverable and testable through dispatch). |
| `crates/marley_app/src/headless_drive.rs` | the drives (REQ + the render STATE assert). |

### Regression Test Plan

| REQ | Test | Where | |
|---|---|---|---|
| REQ-001 | `matching_delimiters_in("fn f() {}", 4)` → `Some((4..5, 5..6))`; `[]`, `{}` likewise | lib.rs | pure |
| REQ-002 | **the string/comment fixture (Fork B, now formal)**: `let s = "(";` @ the inner `(` → `None`; `// (` → `None` | lib.rs | pure — the correctness pin |
| REQ-003 | **the adjacency table (D-BEFORE-THEN-AT)**: caret AFTER `)` lights the pair (prefers the delimiter before); caret ON `(` lights it; caret in open text → `None` | lib.rs | pure |
| REQ-004 | **the MULTIBYTE mapping row** (the #339 lesson): a pair after an emoji/CJK on the line maps to the right DISPLAY bytes — `raw_span_to_display_bytes` on `"// 😀\nfn f() {}"`'s `(` lands on the right cell | code_view/headless | the byte→display seam |
| REQ-005 | nesting: `f(g())` — caret on the OUTER `(` lights the outer pair, on the inner lights the inner (smallest-containing-node) | lib.rs | pure |
| REQ-006 | ⌘⇧\ jumps: caret on `(` → caret moves to the `)`; on `)` → to the `(`; NO NavStack push (the #312 stack depth unchanged) | headless | |
| REQ-007 | non-Rust / no-tree / empty → `None`, no highlight, no panic (totality) | lib.rs | pure fuzz row |
| REQ-008 | **the render STATE (the #339 HIGH — do not drop the render)**: with the caret on a bracket, `bracket_match` is `Some` AND the mark reaches the render (a headless assert on the computed mark spans); the pixel is DEFERRED with a note | headless | |
| — | MarkTier priority: a `Bracket` slice UNDER a `Match`/`Current` slice yields the find/selection tint, not the bracket tint | code_view | pure |

**What a green test would NOT prove (up front, the batch's discipline):**
- **REQ-002 is the correctness heart** — a test that only checks real brackets match would pass with the
  string/comment class broken. The `None`-in-string rows are the load-bearing ones; the spike proved them, the
  test formalizes them.
- **REQ-004's mapping is byte-vs-display, invisible on ASCII** (the #339/#336 coincidence). The multibyte row
  is the only one that tests the mapper; an ASCII fixture proves nothing about it.
- **REQ-008 is the render, and I dropped exactly this in #339.** State without a reader is dead state — the
  test asserts the mark reaches the render channel, and Phase 3 must build the render, not just the field.

**Uncoverable / deferred:** the actual PIXEL (the tint painting on the right two chars) — chad is at the
machine, synthetic input off-limits. Verified by the headless STATE assert + `raw_span_to_display_bytes`
mechanism; the pixel re-verified when the machine is free (~30 s, no ticket). Documented deferred-not-skipped.

### Risks
1. **The 4 ms per-caret parse on a large file** — named, measured, bounded by file size, under a frame. The
   cached-tree follow-up (b) is filed with the baseline. Not hidden as "cheap" (the Phase 1 catch).
2. **The MarkTier variant has 2 exhaustive consumers** (code_view.rs:410, app.rs:4690) — the manifest lists
   BOTH; a partial update is a compile error (exhaustive `match`), so the type enforces completeness here.
3. **The adjacency two-probe** could double-count or mis-prefer — pinned by the REQ-003 table, in the pure
   crate where it is testable.

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement (PASS 1 of 2 — the pure seam; the render/chord/cadence wiring is pass 2)

### Built (pass 1)

`crates/syntax/src/lib.rs` — `pub matching_delimiters_in(src, byte_pos) -> Option<(Range, Range)>`, plus
`delims_at` (the adjacent-probe), `is_delim_pair`, and the `DELIM_PAIRS` const. Parse-only, mirroring
`parse.rs::collect_spans`'s total idiom (`Parser::new()` + `set_language().is_err() → None` + `parse()?`), so a
grammar/parse failure, empty `src`, or out-of-range `byte_pos` all yield `None` — no panic on the input path
(§14). Offset math stays in the PURE crate.

### THE DEVIATION — I implemented the design's PROSE and a probe caught it disagreeing with the REQ

The design architecture said "the smallest node CONTAINING byte_pos whose first/last children are a delimiter
pair" — an **enclosing-pair** walk (climb `.parent()` from any position until a bracket pair is found). I built
exactly that first. Then the throwaway probe printed the behavior per offset and it was **wrong against
REQ-003**: `matching_delimiters_in("fn m() { f(g()); }", 9)` — a caret in the open space before `f` — returned
the body-brace pair, but REQ-003 says **"caret in open text → None"**.

The design's prose and its own acceptance criterion were in tension. **The REQ wins**: enclosing-pair would
light the function's body braces from *every* position inside it — noise, and not what "the pair the caret
TOUCHES" (the title) means. That is the "highlight enclosing pair" feature, a different and noisier thing.

Rewrote to **adjacent-only**: `delims_at(pos)` requires the node AT `pos` to itself be a delimiter token
(`descendant_for_byte_range` lands on the anonymous `"("` leaf only when the caret is on it), then reads its
parent's first+last children as the pair. Re-probed — now correct:
- open text (offsets 1,2,3,9,16 in the fixture) → `None`;
- caret ON `(` / just AFTER `)` / just inside a bracket → that pair;
- nesting resolves by WHICH delimiter the caret touches (`g`'s inner pair vs `f`'s outer pair), via the
  before-then-at preference.

This is the process working as intended — the design prose was imprecise, the probe made the disagreement
concrete, and the REQ resolved it. **Flag for review:** confirm adjacent-only is the intended semantics (I am
confident it is — REQ-003 + the title + the observed mainstream behavior all point there), and that the design
notes' "smallest node containing" phrasing should be read as "the pair whose delimiter the caret is on".

### Verified (a throwaway probe, run then DELETED — `grep probe_matching` → 0)
The adjacency table printed per-offset (above) and matches REQ-003. `let s = "(";` @ byte 9 → `None` (Fork B,
now confirmed in-code, not just the spike). `cargo clippy -p marley_syntax --all-targets` CLEAN; `cargo nextest
run -p marley_syntax --lib` → **34 passed**; `cargo fmt --all`.

### A §20 trap caught + fixed (gate:14 brand-scrub)
The doc comment said "the observed VS Code/**Zed** feel" — gate:14 greps `-i -w` for reference-app names in
`crates/` and would have blocked the commit. §20 ALLOWS observing Zed's behavior; the WORD in a `crates/`
comment is the violation. Reworded to "mainstream-editor feel". (I read the grep hit rather than trusting the
count — a presence-grep's N is only meaningful once you read what matched.)

## Phase 3 — Implement (PASS 2 of 2 — the wiring; pass 1's pure seam is unchanged)

### Built (pass 2) — every field has a reader (the #339 HIGH is not repeated)

- **`code_view.rs`** — `MarkTier::Bracket` (a THIRD variant), documented as the LOWEST tier. Both consumers
  updated: (1) the tier-priority `max_by_key` (:410) changed from `matches!(t, Current) as u8` to an explicit
  3-rank `match` (Current 2 > Match 1 > Bracket 0 — distinct ranks, so no order-dependent tie); (2) the doc on
  `styled_slices_with_marks` now states the 3-way priority. The band-priority `match mark` in app.rs is the
  compile-forced consumer (the exhaustive `match` refused to build until the `Bracket` arm existed — the type
  enforced the #339 "don't drop half the wiring" lesson for me).
- **`app.rs`** — the deliverable:
  - **The fields** `bracket_match: Option<(Range,Range)>` (absolute doc bytes) + `bracket_match_key:
    Option<(u64, BufferVersion, usize)>` (the memo), both inited `None`.
  - **The cadence hook** `refresh_bracket_match` (mirrors `refresh_sticky_headers`, but the KEY carries the
    caret — a caret move alone invalidates, unlike #330's version-only key). Parse-only via
    `matching_delimiters_in` (Fork A). Stores the key even when the pair is `None` (open code) so an unmoved
    caret does not reparse. Wired into the pump with `if … { dirty = true; }` (the #203 repaint rule).
  - **THE RENDER** (the deliverable): the two `bracket_match` spans are whole-document BYTES; the per-row read
    now also yields `row_byte_start` (`char_to_byte(line_start(row))`), so each delimiter is kept only if it's
    on THIS row, localized to line-local bytes, and mapped via `raw_span_to_display_bytes` (Fork C — the
    CODE-side/#331 map: a delimiter is code, so it hugs the code; the caret-tracking cols path the find bands
    use would smear the tint onto an inlay hint at the boundary). Folded into the SAME `marks` vec as
    `MarkTier::Bracket`; the band-priority `match` gains `Some(Bracket) => Some(0.10)` (below Match 0.16 /
    Current 0.35).
  - **`go_to_matching_bracket`** (⌘⇧\ / palette): refreshes the pair first (the pump is frame-cadenced; a jump
    wants the caret NOW), then moves the primary caret to the OTHER delimiter's start (opener→closer,
    closer→opener) via `Selection::caret`. NO NavStack push (D-NO-NAVSTACK). A no-op off a delimiter.
- **`keymap.rs`** — `⌘⇧\` = `chord(true,false,false,true,"\\")`, Editor-scoped, `"go-to-bracket"`. The roster
  asserts the chord INDIVIDUALLY (`.contains`), THEN the count 66→67 (the #337 discipline — never loosen the
  count to pass); the scoped count 21→22 likewise, both with a `#340 ×1` breakdown entry.
- **`palette.rs` + `app.rs::cockpit_commands`** — `CommandId(22)` "Go to Matching Bracket" (chorded AND listed,
  the #337 pattern) → `action_for_command` arm → the `dispatch_action` verb.

### Verified (pass 2)
`cargo check --workspace` CLEAN; `cargo clippy -p marley -p marley_syntax --all-targets` CLEAN (no dead-code
from the wiring); `cargo nextest -p marley --lib` roster/palette/command tests **47 passed** — incl.
`all_chords_lists_every_binding` (count 67), `every_cockpit_command_resolves_to_a_verb` (id 22 resolves), and
the scoped 22. `cargo fmt --all`. §20 CLEAN (0 zed/warp in the crates/ diff). A throwaway probe (run then
DELETED, `grep probe_bm` → clean) confirmed `matching_delimiters_in` returns ABSOLUTE doc bytes and that the
render's localize-then-`raw_span_to_display_bytes` maps a `)` past a WIDE emoji to the correct cell — the
byte/char/display coincidence class (the #336/#339 batch bug) does NOT bite because localization is by BYTE and
the mapper is width-aware.

### Deviations from the manifest (both deliberate, both toward the standing lessons)
1. **The test hooks (`bracket_match_for_test` / `refresh_bracket_match_for_test`) are DEFERRED to Phase 4**, not
   added here — the design manifest listed a `#[cfg(test)]` reader, but a dead `#[cfg(test)]` accessor with no
   consumer trips clippy's dead-code lint at the gate (the #337 F2 / #339 trap; #338's `auto_close_on_for_test`
   was deleted at implement and re-added at validate for exactly this reason). Phase 4 adds BOTH as its drive
   consumes them: `refresh_bracket_match_for_test()` to populate the field headlessly + `bracket_match_for_test()`
   to read it (the highlight-STATE + jump oracle).
2. **`go_to_matching_bracket` REFRESHES before reading the field** rather than trusting the possibly-stale
   memo. The design said "reuse the `bracket_match` result"; the pump is frame-cadenced, so between a caret
   move and the next tick the field can lag a keystroke. `refresh_bracket_match()` is memo-gated (a no-op when
   already fresh), so this reuses the single source of truth AND guarantees freshness — cheaper than a second
   parse call and correct.

### For Phase 3.5 — Inspect (the lenses to point)
- **The render localization** (`row_byte_start` subtraction + on-row filter + `raw_span_to_display_bytes`) — the
  byte/char/display seam. Probe-verified above, but the inspect's job is the edge rows: a delimiter at byte 0,
  at EOF, a row with a wide glyph BEFORE and AFTER the delimiter, an empty row.
- **`go_to_matching_bracket`'s opener-vs-closer branch** — the `caret_byte == open.start || caret_byte ==
  open.end` test, and the degenerate `()` case (caret between, both delimiters adjacent).
- **The adjacent-only deviation carried into the jump** (from pass 1): REQ-006's "inside→closer" is moot under
  adjacent-only — go-to-bracket only fires when the caret TOUCHES a delimiter. Confirm this is the intended v1.

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect

Two parallel `general-purpose` critics over the full working-tree diff (pass 1 + pass 2), each verifying
concretely with throwaway tests (all run then DELETED, tree re-confirmed clean). Lens split: (1) the pure walk
+ the render byte/char/display localization; (2) the cadence memo + the jump + the wiring. **2 findings fixed
at source, both real; 1 acknowledged-and-documented.** The rest of both lenses came back VERIFIED CLEAN with
reproductions — notably the render localization (the ticket's stated whole risk) including the wide-glyph
DISPLAY-correctness and the empty-span drop, the 3-rank tier priority in both input orders, the memo's no-per-
frame-reparse, the jump byte-table, NavStack-untouched, and the mutants::skip discipline (0 app-shim mutants,
114 pure-fn mutants, no skip-detach).

| # | Sev | Finding | Verdict | Fix |
|---|-----|---------|---------|-----|
| M1 | **MEDIUM** | **Language gate dropped — bracket-match ran on ANY file using the Rust grammar.** `matching_delimiters_in` always parses `tree_sitter_rust::LANGUAGE`; `refresh_bracket_match` had no `is_rust` gate, so a `.json`/`.toml`/`.py` file would show a bracket tint + ⌘⇧\ jump driven by RUST semantics (a `(` in a Python `#` comment would false-match — Rust has no `#` comment). Also violates **REQ-008** ("do no walk when no tree exists / non-Rust") and made my own doc comments ("a non-Rust file yields None") FALSE. Critic proved it by parsing JSON as Rust → `Some((0..1,12..13))`. | **REAL — confirmed.** The spec's Scope is "Rust-only v1"; REQ-008 requires the gate. The design premise "non-Rust files fall out for free" (Phase-2 notes) was FALSE — it assumed no tree for non-Rust, but the pure fn ALWAYS builds a Rust tree. | Added the `is_rust` language gate to `refresh_bracket_match`, mirroring `refresh_syntax_cache` EXACTLY (`language_of(path) == Language::Rust`; `!is_rust` → drop the cache like the no-editor branch). Corrected both doc comments: the non-Rust `None` comes from the GATE, not the pure fn. Key stays complete (nonce → path → language). |
| L1 | **LOW** | **Missing-node phantom pair.** For an unclosed `(` (`fn f( {}`), tree-sitter error-recovery inserts a zero-width MISSING `)` node whose `kind()` is `")"` — `is_delim_pair` passed, so the walk returned `Some((4..5, 5..5))`: a lone `(` still tinted (0.10) and ⌘⇧\ jumped the caret onto whitespace. Contradicts the "returns None" contract AND the spec's cut of unmatched-error tinting. Critic proved it per-byte. | **REAL — confirmed** by a fresh probe (`fn f( {}` @4 → `Some((4..5,5..5))` before). Narrow trigger, render swallowed the empty close span, but a real logic gap. | One guard in `delims_at`: `if first.is_missing() || last.is_missing() { return None }`. Probe-confirmed AFTER: `fn f( {}` @4 → `None`; `fn f() {}`/`[1,2]` real pairs UNAFFECTED; `a[0` → `None`. |
| L2 | LOW | **Degenerate `()` jump is a silent no-op** — caret between the two adjacent delimiters (`caret_byte == open.end == close.start`) classifies as "on the opener" → targets `close.start` == the caret → no move. Reachable via #338 auto-close (`(` → `(|)`). | **REAL but ACCEPTED** — the highlight is unaffected (both delimiters light); only the jump is a no-op in this one geometry. Nudging the caret would be a surprise; a no-op is internally consistent with adjacent-only + before-preference. My own design/notes already flagged it. | **No code change** — documented as a known v1 wart (a candidate for the #340 follow-up if a user reports it). |

**Both fixes verified:** `cargo check -p marley -p marley_syntax` CLEAN, `cargo clippy … --all-targets` CLEAN,
34 `marley_syntax` lib tests pass, the L1 probe watched the behavior FLIP (`Some((4..5,5..5))` → `None`) with
real pairs unaffected, §20 re-checked CLEAN. **Phase 4 owes:** a headless drive opening a non-Rust file and
asserting `bracket_match_for_test()` is `None` (the M1 gate — REQ-008), plus a pure `matching_delimiters_in`
row for the missing-`)` case (L1).

**Lessons (for the AAR / capture):**
- **M1 is the batch's recurring class, applied to a GATE**: a confident design sentence ("non-Rust files fall
  out for free") was a CLAIM, not a fact — the pure fn is language-specific *by construction* (hardcoded Rust
  grammar), so the language gate belongs at the CALLER that knows the file's language, mirroring the sibling
  feature (`refresh_syntax_cache`) that already had it. "Falls out for free" is a claim to VERIFY, not assume —
  and the verifier is trivial (parse a non-Rust bracket, watch it return `Some`).
- **L1**: a `kind()`-string match on a tree node is not enough — tree-sitter's error recovery inserts MISSING
  nodes with a REAL `kind()` but zero width; a walk that trusts `kind()` alone admits phantoms. `is_missing()`
  is the precise guard.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

**13 tests added; GATE GREEN [diff] (15/15, coverage 100%, mutation MSI 100%). Receipt valid (hash
`700c3995…`). The FIRST gate run was RED on coverage + mutation — both fixed at source (§0), no suppressions.**

### Tests written (≥1 per REQ + the inspect rows + the byte/char/display discipline)
- **(A) `crates/syntax/src/lib.rs` — 8 pure-walk tests** (all offsets probe-verified before asserting):
  `bm_basic_pairs_all_three_kinds` (REQ-001, `()`/`[]`/`{}`); `bm_string_and_comment_delimiters_never_match`
  (REQ-002 Fork B — the correctness heart); `bm_adjacency_before_then_at_and_open_text_none` (REQ-003 — the
  before-then-at table + open-text→None); `bm_nesting_resolves_to_the_touched_pair` (REQ-005); `bm_totality_
  never_panics` (REQ-007 — empty/OOR/BOF); `bm_missing_node_delimiter_is_not_a_pair` (L1 inspect fix);
  `bm_guard_rejects_non_delimiter_caret_inside_brackets` + `is_delim_pair_only_ordered_same_pairs` (the two
  mutation-killers, below).
- **(B) `crates/marley_app/src/code_view.rs` — 2 tests**: `raw_span_to_display_bytes_bracket_after_wide_glyph`
  (REQ-004 — the byte→display seam; the `\t`-prefixed variant makes raw≠display, so it PROVES width-awareness,
  not identity — the #336/#339 batch bug); `marks_bracket_tier_is_lowest_under_find` (the tier differential —
  Bracket under Match→Match, in BOTH input orders; Current>Match>Bracket; a lone Bracket still renders).
- **(C) `crates/marley_app/src/headless_drive.rs` — 3 drives** + the re-added `#[cfg(test)]` hooks
  (`refresh_bracket_match_for_test` + `bracket_match_for_test`, deferred from implement per the #337 F2 rule,
  added here because these drives consume them): `bracket_match_lights_the_caret_pair_and_memoizes_headless`
  (REQ-008 the render STATE — the pair reaches the field the render folds into `marks`; + REQ-007 the memo →
  second refresh is a no-op; + open-code → None); `bracket_match_off_for_non_rust_files_headless` (M1 — the
  SAME content is Some in `.rs` but None in `.json`, so the None is the GATE, not the content — the drive that
  would have caught the dropped gate); `go_to_matching_bracket_jumps_both_ways_headless` (REQ-006 — the ⌘⇧\
  round-trip 4→6→4 on a NON-empty pair, via the real `dispatch_action` path; + D-NO-NAVSTACK — nav depth
  unchanged).

### The first gate's 2 reds — both fixed at source
- **gate:4 coverage (lib.rs is NOT excluded; only `parse.rs` is):** two dead-by-design `return None` LINES —
  529 (`set_language().is_err()`, the pinned grammar can't fail) and 561 (the redundant `count < 2` guard).
  Fixed by restructuring to TOTAL one-liners with no dead line: `parser.set_language(&lang).ok()?` and
  `let last_ix = parent.child_count().checked_sub(1)?`. Behavior identical (count is always ≥1 for a
  delimiter's parent; count==1 falls through to first==last, which `is_delim_pair` rejects) — the guard was
  redundant, and removing it removed the untestable line rather than papering over it.
- **gate:5 mutation (3 survivors in the pure fn; the app shims are correctly 0/skipped):**
  `lib.rs:555` `*o == kind` / `*c == kind` (×2) and `lib.rs:585` `&&` in `is_delim_pair`. The open-text test
  (byte 9) returned None but its parent had no delimiter first/last child, so it did NOT distinguish the
  mutant. Killed with `bm_guard_rejects_non_delimiter_caret_inside_brackets` — a caret on `b` INSIDE `(ab)`
  (byte 10, non-delimiter before AND at): correct→None, but the mutant proceeds and lights the ENCLOSING
  parens. The `&&`→`||` mutant needed the mismatched-halves row (`is_delim_pair("(", "]")` → the `||` mutant
  wrongly accepts it) — a DIRECT unit test on the private fn.

### Verification discipline
Every offset/expected value was **probe-verified before asserting** (throwaways run then DELETED — the #339
guess-twice lesson). The two mutation-killers were designed by tracing the mutant's behavior, then confirmed
by the gate's re-run (0 survivors). The M1 drive's two-arm structure (`.rs`→Some, `.json`→None) makes it a
GENUINE discriminator of the gate — without the gate the `.json` arm would return Some and the test would fail.

### Live pixel — DEFERRED, not skipped
The bracket TINT painting on the two delimiter cells is verified by the headless STATE assert (C) + the
width-aware `raw_span_to_display_bytes` unit (B) + the mechanism (the render folds the pair into `marks`).
The actual pixel is deferred (chad at the machine — synthetic input off-limits); re-verify when the machine is
free (~30s, no ticket). Documented deferred-not-skipped (the batch discipline).

### Pre-existing / not in scope
None. `docs/README.md` + `docs/marley_architecture/detached-sessions.md` are pre-existing unrelated changes
(present at session start), excluded from this ticket.

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete
- Docs updated; AAR capture; archive.
