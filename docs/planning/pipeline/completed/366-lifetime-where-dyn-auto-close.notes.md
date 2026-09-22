# Auto-close ' at where/dyn lifetime positions (the #362 deferred half) — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-366-auto-close-where-clause-lifetimes.md
- **Pipeline spec:** 366-lifetime-where-dyn-auto-close.spec.md
- **pipeline_id:** 4e610839-3f3d-4dbc-9ad5-b969c71f71c3

<!-- Working scratch for the pipeline. Each phase appends its entry. -->

## Phase 1 — Plan
- **Request:** `/work` next-ticket (auto-approved goal): TICKET-366 — suppress `'`-pairing at the two
  lifetime-position classes #362 deferred: `where T: 'a` (all variants) and `dyn Trait + 'a` / `Ref<T, 'b>`
  type-usage positions. Feature, M22, editor/auto-close/syntax.
- **Classification / tier:** work pipeline, one shippable slice (both classes fall to ONE new probe — see the
  spike). Systems: marley_syntax (new pure probe), marley_app (Context-closure branch + headless drive).
  marley_editor expected UNCHANGED (LifetimeBound already gates `'` — #362).
- **Recall (§18.3):** ledger + archive surfaced: (a) #362's spec/notes — the shipped `in_type_parameters`
  ancestry probe + `Context::LifetimeBound` + the app closure, and its spike table (where → NO ancestry;
  dyn → `type_arguments`); (b) **PR-…-suppression-test-must-prove-the-baseline-would-not-suppress** (the #362
  vacuous drive: `>` after the caret made `opens_here` false — fixtures must put whitespace after the caret,
  the double-space trick); (c) **PR-…-no-adhoc-python-splice-on-source-use-edit-tool** (+ spikes live in
  scratch, never in-tree); (d) PR-1556-family: verify a claimed node EXISTS at the pre-insert caret before
  scoping (spike at plan) — followed; (e) #346's Context seam + #338's blocked_quote char rule (prev `&`/`<`/
  word already blocks; the GAP class is space/`,`/`:`/`+`-preceded positions).
- **Discovery:** seam confirmed against HEAD — `blocked_quote` auto_close.rs:199 (context == LifetimeBound
  under `typed == '\''`), `Context` :210, `pair_action` doc :117-125 (names where/dyn/`'static` deferred),
  the app Context closure app.rs:16873-16893 (captures `src` AND `text`; StringOrComment → in_type_parameters
  → Code today), `in_type_parameters` syntax/lib.rs:556, `node_kind_at` :538, `parser_for` parse.rs:101.
- **★★ THE SPIKE (scratchpad crate `spike366`, pinned tree-sitter 0.26.11 + tree-sitter-rust 0.24.2 — the
  exact Cargo.lock versions; 19 fixtures × 3 probe modes). Results:**
  - **Pre-insert ancestry** (what an in_type_parameters-style probe sees): every `where` variant — body
    present or EOF, +Send, 2nd predicate, struct/impl — has NO bound ancestry (`function_item`/`struct_item`/
    `impl_item`/`source_file`). ⇒ #362's ancestry approach can NEVER reach `where`; the ticket's premise
    confirmed at HEAD's pinned grammar. `Box<dyn Trait + §>` / `Ref<T, §>` → `type_arguments` ancestry exists;
    `&(dyn Trait + §)` (paren, no `<>`) and `-> impl Iterator + §` → nothing. Const-generic blocks show
    `block < type_arguments` (a guard WOULD be needed on that path).
  - **★ Speculative post-insert parse** (splice `'a` at the caret, parse the copy, kind of the node covering
    the spliced span): **`lifetime` at ALL 13 lifetime positions** — every where variant (EOF recovers via
    `function_signature_item`; the trailing-comma predicate start recovers as `lifetime < ERROR` — correctly
    TRUE: `where T: 'a, 'b: 'c` is legal), both dyn-in-Box forms, `Ref<T, §>`, dyn-paren, impl-trait return —
    and **`label`/ERROR (never `lifetime`) at ALL 6 controls** — both const-generic blocks, char-literal,
    expr-plus, call-arg, `<`-comparison. The Rust token grammar's lifetime-vs-label position split, read back
    out of the parser's error recovery.
  - **Bare-`'` splice mode** (informational): discriminates the body-present cases but ERRORs at EOF — the
    `'a` splice is the robust form; adopted.
  - ⇒ **ONE mechanism covers BOTH ticket halves + bonus positions** (dyn-paren, impl-return, where-EOF), with
    the const-generic guard free. The ticket's option (a) backward text scan DIES (would false-positive at
    `x + '`); recorded as the #339 substrate-win pattern.
- **Decisions:** D1 the speculative-splice probe is the single signal; D2 new pure marley_syntax fn
  (combinator, cov-100 shape); D3 `'`-gated in the app closure via the captured `text`, precedence
  StringOrComment → in_type_parameters → splice → Code, ZERO editor-crate delta; D4 labels stay pairing
  (kind gate exactly `"lifetime"`); D5 cost bounded to `'` keystrokes; D6 probe cov/MSI 100 via units, app
  closure inherits `replace_text_in_range`'s mutants::skip (zero mutants — #362 verified), drive non-vacuous
  per the PR (double-space fixture).
- **EARS:** REQ-WHERE-SUPPRESSES, REQ-DYN-SUPPRESSES, REQ-CONST-BLOCK-PAIRS, REQ-CHAR-LITERAL-PAIRS,
  REQ-PRIOR-UNCHANGED (see spec).
- **Backlog:** TICKET-366 row removed from the Queue (promotion, §19). No intake doc involved.

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Architecture / approach.** One new PURE `marley_syntax` probe + one app-closure arm; the editor crate's
DECISION surface is untouched (#362's `Context::LifetimeBound` already suppresses only `'`). §14: total fn (a
`is_char_boundary` guard makes the splice slicing panic-free; combinators for the None flows); no IO; no new
shared types. **§20 confirm:** the spec's `N/A` HOLDS — the mechanism is Marley-specific over tree-sitter-rust
error recovery (adoption); zed docs gave inventory only, no behavior read.

**★ The probe (settled — mirrors `node_kind_at`/`in_type_parameters`, syntax/lib.rs after :575):**
```rust
pub fn speculative_lifetime_at(src: &str, lang: Lang, byte_pos: usize) -> bool {
    if !src.is_char_boundary(byte_pos) {   // also false for byte_pos > len — total, panic-free (§14)
        return false;                      // reachable + unit-covered (multi-byte fixture), killable
    }
    let mut probe = String::with_capacity(src.len() + 2);
    probe.push_str(&src[..byte_pos]);
    probe.push_str("'a");
    probe.push_str(&src[byte_pos..]);
    let mut parser = parse::parser_for(lang);
    parser.parse(&probe, None)
        .and_then(|tree| tree.root_node()
            .descendant_for_byte_range(byte_pos, byte_pos + 2)
            .map(|n| n.kind() == "lifetime"))
        .unwrap_or(false)
}
```
Doc carries: why `'a` not bare `'` (EOF recovery — spike: bare `'` ERRORs at EOF, `'a` recovers via
`function_signature_item`); the lifetime-vs-label gate (labels/expr → `label` kind → keep pairing, D4);
the two safe-direction caveats (type positions suppress — field/param spike TRUE; braceless char const args
`Ref<T, 'x'>` suppress — the #362 const-default caveat's twin).

**★ The app closure (app.rs:16873-16893) — the ONLY behavior edit:** before `active_ime_mut`, capture
`let typed_quote = text == "'";` (a bool moved into the closure — no lifetime questions); the branch becomes:
```rust
} else if marley_syntax::in_type_parameters(&src, Rust, byte)
    || (typed_quote && marley_syntax::speculative_lifetime_at(&src, Rust, byte))
{
    Context::LifetimeBound
}
```
Precedence unchanged: StringOrComment first (#346), `in_type_parameters` kept first in the `||` (#362 stable,
short-circuits the splice at decl-sites), the splice LAST and `'`-gated (cost: one extra parse ONLY on `'`
keystrokes outside strings not already in type_params — rare by construction; #363-cache N/A, modified text).
IME compositions/multi-char pastes: `text != "'"` → no splice (auto-close is per-single-char anyway).

**★ Recon settled (confirmed against HEAD):** `replace_text_ctx` (ime.rs:120) takes
`context_at: impl Fn(CharOffset) -> Context` — NOT `'static`, a bool capture is trivially fine.
`open_rs_fixture` (headless_drive.rs:5441) writes per-call content to `sample.rs` — the #362 drive
(:6682) is the template (double-space fixture, `set_editor_caret`, `replace_text_in_range`, exact-buffer +
`!contains("''")` asserts). **Drive discrimination verified by spike**: at `fn f<T>() where T: § {}` the
pre-insert ancestry is `function_item` (in_type_parameters FALSE) → pre-#366 ctx=Code, `opens_here(' ')`
true, `blocked_quote(space)` false → WOULD pair `''`; only the NEW splice arm suppresses. Non-vacuous per
PR-…-suppression-test-must-prove-the-baseline-would-not-suppress.

**File manifest (4 files):**
- `crates/syntax/src/lib.rs` — ADD `speculative_lifetime_at` (after `in_type_parameters`) + 2 units
  (`t366_…_true_at_lifetime_positions`, `t366_…_false_at_char_label_and_degenerate_positions`).
- `crates/marley_app/src/app.rs` — the Context closure: `typed_quote` capture + the `||` splice arm + #366
  comment line.
- `crates/marley_app/src/headless_drive.rs` — ADD `auto_close_suppresses_quote_in_where_clause_headless`
  (fixture `fn f<T>() where T:  {}`, caret `len("fn f<T>() where T: ")`=19, type `'`, assert
  `fn f<T>() where T: ' {}` + `!contains("''")`).
- `crates/editor/src/auto_close.rs` — DOC-ONLY: un-defer the where/dyn deferral notes at `pair_action`
  (:120-125), `blocked_quote` (:193-198), `Context::LifetimeBound` (:215-219 broadens to "a lifetime
  position"); no behavior change (inspect checks doc-vs-behavior — the #362 MEDIUM class).

**★ Test plan.**
| # | Proves | Test |
|---|---|---|
| U1 | REQ-WHERE-SUPPRESSES + REQ-DYN-SUPPRESSES (probe TRUE) | `speculative_lifetime_at` TRUE at: where body/EOF, `+Send` body/EOF, 2nd predicate, struct-where, impl-where, trailing-comma predicate; `Box<dyn +§>` (+EOF), `Ref<T, §>`, `&(dyn +§)`, `-> impl Iterator + §`; field-type + param-type (deliberate safe-direction rows). |
| U2 | REQ-CONST-BLOCK-PAIRS + REQ-CHAR-LITERAL-PAIRS (probe FALSE) | FALSE at: `Foo<{ § }>`, turbofish `::<{ § }>`, `let c = §`, `x + §`, call-arg, `<`-comparison, statement position, empty file @0; degenerate: mid-UTF-8 byte (guard), byte>len, `Lang::Python` (no lifetime kind). |
| U3 | REQ-PRIOR-UNCHANGED (decision surface) | existing #338/#346/#362 unit suites untouched + green (no editor code change). |
| T1 | REQ-WHERE-SUPPRESSES end-to-end (the flagship: the class ancestry can NEVER reach) | the where headless drive (above), non-vacuous baseline traced. |
| R1 | REQ-PRIOR-UNCHANGED end-to-end | full `-p marley_syntax -p marley_editor -p marley` nextest regression incl. the #338/#346/#362 drives. |

**Mutation-kill reasoning (gate:5):** body→true/false killed by U2/U1; boundary-guard flip killed by U1 (all
boundary bytes); `"lifetime"` literal swap by U1; `byte_pos+2` arith mutants by U1's position-rich rows
(inverted/oversized ranges land on non-lifetime kinds); `"'a"` splice-literal swap ("", "xyzzy") by U1 (probe
text degenerates → kind ≠ lifetime). App closure: inside `replace_text_in_range` (already
`#[cfg_attr(test, mutants::skip)]` — #362 verified it enumerates ZERO mutants); carried end-to-end by T1.
**Uncoverable-honestly:** none new — the probe is pure (marley_syntax cov-included → 100 via U1/U2); app.rs is
cov-excluded as ever.

**Risks / decisions.** (a) grammar-pin risk: recovery shapes are 0.24.2-empirical — the units PIN them; a
future grammar bump that shifts a kind breaks units VISIBLY (that is the pin working, #315's multi-language
pass owns generalization). (b) type-position over-suppression (field/param TRUE) — safe direction, documented
+ unit-recorded as deliberate. (c) braceless char-const args suppress — vanishingly rare, safe direction,
doc caveat. (d) labels keep pairing (D4 — out of scope, today's behavior). (e) cost bounded to `'` keystrokes
(typed_quote gate). (f) zero editor-crate behavior delta — REQ-PRIOR-UNCHANGED is structural.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

**React-first: N/A** (spec §React-first — no UI delta; keystroke semantics only), stated per the skill, not
silently skipped.

**Built (to the manifest — 4 files, 3 touched here):**
- `crates/syntax/src/lib.rs` — `pub fn speculative_lifetime_at(src, lang, byte_pos) -> bool` after
  `in_type_parameters`: the `is_char_boundary` totality guard (reachable, documented), the `'a` splice via
  three `push_str`s, `parse → descendant_for_byte_range(pos, pos+2) → kind == "lifetime"` in the no-`?`
  combinator shape. Full doc: why `'a` not bare `'`, the lifetime-vs-label gate, the two safe-direction
  caveats, the 0.24.2 pin note.
- `crates/marley_app/src/app.rs` — `let typed_quote = text == "'";` above `active_ime_mut` + the closure's
  LifetimeBound arm extended with `|| (typed_quote && speculative_lifetime_at(&src, Rust, byte))`; the #362
  comment block extended with the #366 story.
- `crates/editor/src/auto_close.rs` — DOC-ONLY (behavior untouched, verified by clippy/check): `pair_action`
  doc un-defers where/dyn (labels named the deliberately-open miss); `blocked_quote` doc gains the "#366
  closes the rest of the class / one guard serves both probes" paragraph; `Context::LifetimeBound` doc
  broadened to "at a LIFETIME position" with both rare-`'` safe-direction caveats.

**Deviations from design:**
- (1) The 4th manifest file (`headless_drive.rs` — the T1 where-drive) and the U1/U2 probe units are DEFERRED
  to Phase 4 per the implement skill's "do NOT write/expand tests here" rule (the design manifest bundled
  them; #362 wrote units at P3 under the older habit — the skill text wins). No application-code deviation.

**Checks:** `cargo fmt --all` applied; `cargo check --workspace` green; `cargo clippy -p marley_syntax
-p marley_editor -p marley --all-targets -- -D warnings` **exit 0**. (The `block v0.1.6` future-incompat
warning is pre-existing, transitive, and untouched by this diff.)

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

**2 critics (Agent general-purpose, parallel) + my own review. Verdict: 1 MEDIUM + 4 LOW; the MEDIUM + 2 LOWs
FIXED at source, 2 LOWs recorded (no action, reasons below). No HIGH. No correctness bug survived the hunt.**

**Critic 1 (correctness/state-integrity) — scratch-parsed ~60 EXTRA fixtures against the pinned grammar,
replicating the closure's full decision chain. All clean:** match-arm char patterns (`match c { § }` → ERROR
node, not `lifetime` → still pairs — the prompt's scariest hypothesis dead); macro bodies/attributes (`token_tree`
→ pairs); `<`/`<<`/`<=` comparisons (`label`-in-ERROR → pair); char range patterns; `if let` patterns;
break/continue labels (pair — designed); panic paths (boundary guard covers past-len; `+2` can't overflow —
`byte_pos ≤ len ≤ isize::MAX`); the closure's `char_indices().nth()` byte is always a boundary or `len`;
`typed_quote` gate exactness (pairing requires a one-char insert on the no-range/no-marked arm — `text == "'"`
⟺ the pairing char is `'`); token-merge adjacency can't flip an outcome where `opens_here` allows pairing;
kind-spoof impossible (the anonymous `"lifetime"` token can't cover a 2-byte span). Findings: **[LOW]
3-parses-per-`'` cost** (pre-existing #346/#362 family, #349 owns the cached tree; the suggested app-side
`next`-is-word skip would duplicate `pair_action` knowledge — REJECTED, cost noted); **[LOW] probe doc says
"the units keep the pin honest" before the units exist** — transient by design (P4 lands U1/U2); Phase 4 MUST
land them or the doc line is false. Plus P4 fixture guidance adopted: pin the three classes a grammar bump
could flip (match-arm ERROR row, one `token_tree` macro row, one `c < §` comparison row) into U2.

**Critic 2 (simplification/reuse + provenance/doc-consistency):**
- **[MEDIUM] REAL, FIXED — the #362 stale-doc class struck AGAIN, one layer deeper:** the
  `t338_lifetime_guard_does_not_reach_the_bound_positions` TEST-header comment (auto_close.rs:348) still
  declared `T: 'a`, `+ 'a`, `'static` "NOT covered and CANNOT be … needs the grammar (#315)", and its assert
  MESSAGE (:358) repeated it — a direct contradiction of shipped #362/#366 behavior that BOTH pipelines'
  doc passes missed (#362's inspect fixed `pair_action`'s doc; #366's implement fixed the three fn docs; the
  test comment + message hid in the tests module). Fixed: re-scoped to "THE ONE-CHAR RULE'S documented limit"
  — the row pins the bare-`Code` behavior; the context probes (#362/#366) are what separate the positions;
  message updated. Verdict: my own read of :348-360 confirmed word-for-word.
- **[LOW] REAL, FIXED — `in_type_parameters`' doc ended on "where/dyn NOT covered", the exact sentence a
  reader greps for, with no pointer to the sibling that now covers them.** Fixed: "NOT covered here …
  [`speculative_lifetime_at`] (#366) covers them via the post-insert splice."
- **[LOW] REAL, FIXED — `typed_quote` should gate BOTH probes.** `Context::LifetimeBound` is provably inert
  for every typed char except `'` (`blocked_quote` self-gates on `typed == '\''`; the `!= StringOrComment`
  check treats LifetimeBound and Code identically — pinned by `t362_lifetime_bound_suppresses_only_the_quote`),
  yet `in_type_parameters` (a full-file parse) ran for every opener keystroke. Fixed:
  `typed_quote && (in_type_parameters(…) || speculative_lifetime_at(…))` — semantics-preserving (verified:
  `context_at` is consulted only on the pairing plan path, buffer.rs:388-391), one line, and it makes D5's
  "cost bounded to `'` keystrokes" true for the WHOLE lifetime path, a strict improvement on the #362 shape.
  `||` order preserved (decl sites short-circuit the splice).
- Rejected/no-action: parse-then-probe helper extraction (the 3× mirror is the documented module idiom; a
  shared fn fights the `Tree` borrow); `is_some_and` (the probe family's combinator shape + dead-by-design
  comment is deliberate); full-copy splice (no `parse_with`/incremental use in-repo; O(n) copy beside O(n)
  parse); ime.rs `replace_text` comment freshen (pre-existing, out of diff); `blocked_quote` duplicated intro
  paragraphs (pre-existing, out of diff). **Provenance (§20) PASS**: no Warp/Zed source; mechanism from the
  scratch spike over MIT tree-sitter; house idiom throughout.

**Fixes verified:** `cargo fmt --all` + `cargo clippy -p marley_syntax -p marley_editor -p marley
--all-targets -- -D warnings` exit 0 after all three fixes.

**Ledger appends:** `PR-claude-undefer-sweeps-test-comments-and-assert-messages-001` (prevention-rules.md) —
the class fired twice across #362/#366. No F- append: the stale comment never shipped (caught in-phase, doc-only).

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests written (to the Phase-2 plan + the inspect pins):**
- **U1** `t366_speculative_lifetime_true_at_where_and_type_usage_positions` (syntax/lib.rs) — 15 table rows:
  8 where variants (body/EOF/`Send + ` body+EOF/2nd predicate/struct/impl/new-predicate start), Box-dyn ×2
  (incl. EOF), `Ref<T, >`, parenthesized dyn, impl-trait return, + the two deliberate safe-direction TYPE
  positions (field/param). Per-row labeled asserts; fixtures + bytes are the spike's, pinning 0.24.2 recovery.
- **U2** `t366_speculative_lifetime_false_at_char_label_and_degenerate_positions` (syntax/lib.rs) — 10 table
  rows (const-generic block, turbofish block, char-literal, expr-`+`, call-arg, `<`-comparison upstream +
  direct, statement position, and the inspect-pinned match-arm + macro `token_tree` rows) + 4 degenerates
  (empty file @0, non-char-boundary byte inside `é` [§14 guard], past-the-end byte, `Lang::Python`).
- **T1** `auto_close_suppresses_quote_in_where_clause_headless` (headless_drive.rs) — the REAL editor door
  (`replace_text_in_range` → the app closure → `pair_action`). NON-VACUOUS by construction and doc'd in the
  test: at this caret `in_type_parameters` is FALSE (no `<>` anchor — so #362's shipped code alone yields
  `Code`) and `next` is whitespace (double-space fixture → `opens_here` TRUE) — a Code context WOULD pair
  `''`; only the #366 splice arm suppresses. Asserts the exact buffer `fn f<T>() where T: ' {}` AND
  `!contains("''")` (per PR-…-suppression-test-must-prove-the-baseline-would-not-suppress).

**Tests RUN (actual output):**
- Targeted: `cargo nextest run -p marley_syntax -p marley_editor -p marley -E 'test(t366) +
  test(auto_close_suppresses_quote_in_where_clause)'` → **3 tests run: 3 passed** (2 units + the drive).
- Regression (U3/R1): `cargo nextest run -p marley_syntax -p marley_editor -p marley` → **1343 tests run:
  1343 passed, 2 skipped** (the 2 skips are pre-existing, unrelated). The #338/#346/#362 auto-close units +
  drives all green — REQ-PRIOR-UNCHANGED holds; no ripple (the editor crate's behavior surface was not
  touched).

**Live drive: NONE — stated deliberately (the #362 precedent, same seam, same door).** The change is
keystroke SEMANTICS in the editor buffer with no visual/layout/render delta (spec React-first: N/A — no UI
delta; no `visual_acceptance` clause). The headless drive IS the end-to-end proof: it exercises the real
`replace_text_in_range` → real app Context closure (the `mutants::skip`-inherited shim) → `in_type_parameters`
→ `speculative_lifetime_at` → `pair_action` chain against the real buffer — a tree/text read that needs no
font metrics or pixels. The selftest harness drives the terminal cockpit surface (type:<cmd> enter) and has no
editor-caret verbs for a mid-file where-clause position; a live synthetic GUI drive would add nothing over the
headless assertion and would steal focus (chad may be at the machine).

**Full `--diff` gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] — 15/15 PASS** (first run), receipt
`f7bf5740004d8dc8427583731028cbc325b06031`. gate:4 coverage ≥100% (`speculative_lifetime_at` fully exercised
by U1/U2 incl. the boundary guard's early return [the multi-byte-`é` row] — same combinator family as
`node_kind_at`/`in_type_parameters`, both at 100 with the identical shape); gate:5 mutation MSI ≥100% on the
diff (`cargo mutants --in-diff`, all mutants killed — the probe's body/guard/kind-literal/splice-literal/
arith mutants die to U1/U2's position-rich rows; the app closure enumerates ZERO mutants, inherited
`mutants::skip` via `replace_text_in_range`, the #362-verified shape); gate:3 ran the full workspace
(1343+158 green); gate:14 rustdoc clean (the new intra-doc links resolve). No red, no pre-existing
exclusions, nothing suppressed.

**Pre-existing, not in scope:** the 2 nextest skips (unrelated, present before this diff); the `block v0.1.6`
future-incompat cargo note (transitive dep).

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete

**Docs (§21).**
- **CHANGELOG.md** — a `### Fixed` entry at the head of Unreleased: auto-close no longer over-pairs `'` at the
  where/dyn/type-usage lifetime positions (the #362 deferred half; the speculative-splice mechanism, the
  quote-gating of both probes, the label/const-generic non-suppression, the 0.24.2 pin).
- **docs/marley_architecture/editor.md** — the #346/#362 Context paragraph extended: #366's second probe for
  the anchor-less positions, the `text == "'"` gate on BOTH probes (inertness argument), labels still the
  open miss, the "where/dyn stay deferred" sentence GONE (the un-defer sweep, per the new PR rule).
- **Parity sync:** N/A — no UI delta (spec §React-first); nothing to back-port to marley-web.

**Capture (§19 — local ledger appends, by code):**
- `L-claude-366-speculative-post-insert-parse-classifies-unanchored-positions-001` (lessons.md, appended at
  design) — the splice-and-reparse technique for positions the pre-insert tree can't anchor.
- `PR-claude-undefer-sweeps-test-comments-and-assert-messages-001` (prevention-rules.md, appended at inspect)
  — un-deferring a documented limit is a SWEEP incl. test comments + assert messages; fired twice (#362/#366).
- `AD-claude-366-auto-close-lifetime-context-is-two-probes-quote-gated-001` (architecture-decisions.md,
  appended here) — the Context pipeline shape, why two probes, why the quote gate is provably semantics-free,
  the cost profile, the #349 cache boundary.
- No F- append: no shipped failure (the stale t338 comment was caught and fixed in-phase; recorded in the
  inspect ledger).

**Close + archive.** TICKET-366 → `tickets/closed/` with a dated close note (backlog row left at promotion —
sweep confirmed none stale). Spec status → Phase 5 — Complete PASS; the spec/notes pair archived
`active/` → `completed/`.

**Headline lessons:** (a) ★★ the Plan spike flipped the ticket's own pessimism — option (b) "speculative
post-insert parse", dismissed in the ticket text as fiddly, is a PERFECT discriminator on the pinned grammar
(13/13 vs 6/6), while option (a) backward-scan would have false-positived at `x + '`; the #339 "the substrate
already owns it" pattern, now with its own L- lesson. (b) ★ the inspect MEDIUM was the #362 stale-doc class
ONE LAYER DEEPER (a test-header comment + assert message) → the un-defer-sweep PR rule. (c) ★ Critic 2's
inertness argument turned a cost concern into a one-line strict improvement (both probes quote-gated — cheaper
than #362's shipped shape). (d) the pure/app split held again: probe cov/MSI 100 via units; the app closure
zero-mutant by inherited skip; the drive non-vacuous by construction (baseline traced in its own doc).

**Status: Phase 5 — Complete PASS.**
