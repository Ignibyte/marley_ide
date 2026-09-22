# Multi-language syntax highlighting (the language axis + a 10-slot taxonomy) — Notes

- **Forge ticket:** #315 2936c168-b807-48de-8a57-d811d8288e9d
- **AAR:** c0e7cea0-9541-46b1-94bb-18fa850d7bd3
- **Local ticket doc:** docs/planning/tickets/open/TICKET-315-multi-language-syntax.md
- **Pipeline spec:** 315-multi-language-syntax.spec.md

<!-- Working scratch. Each phase appends its entry. -->

## Phase 1 — Plan
- **Request:** multi-language syntax highlighting — a `Lang` axis through marley_syntax + the 5→10-slot token
  taxonomy + 6 grammar crates + the hand-lexer floor. Promoted from the pre-authored queued spec (the Fable
  method); SEVENTH of `/work 300,302,303,304,305,314,315,316,317,259` (auto-approved, autonomous-through-commit).
- **Classification / tier:** work pipeline — a LARGE but COHERENT single slice (the spec's phase plan: taxonomy
  first [compiler-forced], then the language axis + pin tests, then the app plumbing). NO reshape/split needed
  (assessed below); the genuine Phase-2 risk is the 6 grammar-crate version resolution against core 0.26.11.
- **Promotion done:** `git mv` queued→active; pipeline_id `aacfebff-909c-4b4b-8a15-23b1f57ff860`; aar
  `c0e7cea0-9541-46b1-94bb-18fa850d7bd3`; TICKET-315 created.

### Seam re-verification vs LIVE `main` @ 050f7e9 (spec on Fable; #300..#305/#314 landed since)
Re-verified INLINE with targeted greps (the earlier Explore agents misfired on skill-guidance). **Every
load-bearing claim CONFIRMED; only line numbers drifted.**

| Claim | Verdict | Live evidence |
|---|---|---|
| highlighting is RUST-ONLY — `highlight_query` OnceLock, `HighlightSession::new` (NO lang arg), `kind_of_capture`/`highlight_lines` no lang | **VERIFIED** | parse.rs:17 `highlight_query` OnceLock (+ :33 a 2nd OnceLock = the #304 tags query), :109 `rust_parser`; lib.rs:333 `HighlightSession::new()` no arg, :39 `kind_of_capture(name)` no lang, :112 `highlight_lines(src)` no lang. The threading target: add `lang` to new/highlight_lines/kind_of_capture + a per-Lang (grammar, query) table |
| the TWO TokenKind enums — syntax 5-kind, app 6-kind (+Hint) — + `kind_from_syntax` bridge | **VERIFIED** | marley_syntax `enum TokenKind` lib.rs:23 = {Keyword,Str,Comment,Number,Plain} (5); app `enum TokenKind` code_syntax.rs:58 = {Keyword,Str,Comment,Number,**Hint**,Plain} (6); bridge `kind_from_syntax` code_syntax.rs:246 + the 1:1 test :314 |
| **the taxonomy blast radius (5→10)** | **BOUNDED — compiler-forced** | ~8 exhaustive match/`token_color` sites on `code_syntax::TokenKind` in marley_app; `kind_of_capture` (lib.rs:41-45) a 5-name map. Growing both enums +5 forces every arm (the #340 MarkTier discipline) — a compiler-guided change, NOT a hidden fan-out. **No reshape** |
| `kind_of_capture` + the ~21-name pin test | **VERIFIED** | lib.rs:39 the map; :673-677 the pin test over `parse::query_capture_names()` (fails loudly on a grammar bump). The per-grammar pin idiom (D-PER-GRAMMAR-PIN-TEST) generalizes it ×N |
| `language_of` extension map | **VERIFIED** | code_syntax.rs:27 `language_of(path)` + `enum Language` :11 + the by-ext test :296. Grows py/js/ts (the ONE-table rule, #299) |
| the worker — one permanent Rust session, `is_rust` gate | **VERIFIED (drift)** | `syntax_worker` field app.rs:496; `HighlightSession::new()` built at :3359/:3431; the `is_rust` gate :3466/:3479. Make lang-aware: `SyntaxReq` gains `lang`, rebuild the session on a lang switch (D-REBUILD-SESSION-ON-LANG-SWITCH — a switch is a new nonce → full parse anyway) |
| grammar crates — ONLY tree-sitter-rust in the lock | **VERIFIED** | Cargo.lock: `tree-sitter` (6114) + `tree-sitter-language` (6128) + `tree-sitter-rust` (6134); NO other grammar. #315 adds 6 (python/js/ts/json/toml/bash); versions resolved at implement against core **0.26.11** (tree-sitter-rust rides 0.24.2 vs the same core → ABI-line mixing known-fine); gate:8 cargo-deny enforces MIT provenance |
| the hand-lexer floor — `highlight_ranges` | **VERIFIED** | code_syntax.rs:261 `highlight_ranges(line, lang)` + test :458 — the no-grammar fallback (D-FLOOR-IS-THE-HAND-LEXER: no grammar ⇒ no tree path at all, the #340 M1 class by construction) |

- **Decisions confirmed (no reopens):** D-TEN-SLOT-TAXONOMY (Rust richer as a named side effect; #316 colors the
  slots — hard ordering #315→#316), D-ONE-LANGUAGE-TABLE (#299 — detection/comment/grammar off the one
  `code_syntax` table), D-QUERIES-ARE-ADOPTED (upstream `.scm` trimmed + provenance-tagged, never hand-written),
  D-PER-GRAMMAR-PIN-TEST (each closed capture set truth-tabled), D-REBUILD-SESSION-ON-LANG-SWITCH,
  D-MD-DEFERRED (injection model named out), D-FLOOR-IS-THE-HAND-LEXER. All 10 EARS AC hold.
- **Prior-art sweep (§20):** the whole ticket IS the sweep (the spec's own framing). (1) behavior maps — n/a
  beyond "a .py file is colorized". (2) published — tree-sitter multi-grammar client docs; each grammar's capture
  conventions. (3) OUR PERMISSIVE DEPS — every grammar crate ships its own `HIGHLIGHTS_QUERY` (upstream-maintained
  `.scm`); **Marley writes NO queries, only the capture→taxonomy maps + per-grammar pin tests.** tree-sitter core
  + the grammar crates are published/MIT ADOPTION (outside the §20 wall). No Warp/Zed source anywhere. gpui/ropey
  own nothing here. `is_header_kind`'s doc already says "other languages extend when #315 lands" — the seam was
  built for this.
- **Phase-2 flags:** (a) the grammar-crate VERSION RESOLUTION against core 0.26.11 is the real spike (a compile);
  (b) the tsx variant (tsx under the TS grammar); (c) reading each upstream `.scm` for the exact capture→taxonomy
  map (the adopted-query trim); (d) confirm the `(nonce, version)` cache key stays sufficient (language is a pure
  fn of the path; the nonce is per-buffer — the chain path→language→nonce).

**Phase 1 PASS.** The Rust-only premise + the seams hold; the taxonomy growth is compiler-forced (bounded); the
single-slice scope holds. Next: `/pipeline:design`.

## Phase 2 — Design

**§20 confirmed:** tree-sitter core + the grammar crates are published-API / MIT ADOPTION (outside the wall); each
grammar's `HIGHLIGHT(S)_QUERY` is the upstream `.scm` (taken + trimmed onto the taxonomy, provenance-tagged in-file
— D-QUERIES-ARE-ADOPTED); Marley writes NO queries. No Warp/Zed source anywhere. gpui/ropey own nothing here.

### ⚠️ THE GRAMMAR-VERSION SPIKE — RESOLVED (the make-or-break de-risk, done at design)
Temporarily `cargo add`'d the 6 grammar crates + `cargo check -p marley_syntax` → **ALL SIX COMPILE against the
pinned tree-sitter core 0.26.11** (no core bump forced; they ride the `tree-sitter-language` 0.1.7 ABI shim, like
tree-sitter-rust@0.24.2). Spike reverted (design doesn't commit code; implement re-adds). **Resolved versions +
API (RECORDED — the const name VARIES per grammar, a real implement gotcha):**

| Lang | crate @ version | LanguageFn const | query const |
|---|---|---|---|
| Python | tree-sitter-python 0.25.0 | `LANGUAGE` | `HIGHLIGHTS_QUERY` |
| JavaScript | tree-sitter-javascript 0.25.0 | `LANGUAGE` | `HIGHLIGHT_QUERY` (singular!) |
| TypeScript | tree-sitter-typescript 0.23.2 | `LANGUAGE_TYPESCRIPT` + `LANGUAGE_TSX` | `HIGHLIGHTS_QUERY` |
| JSON | tree-sitter-json 0.24.8 | `LANGUAGE` | `HIGHLIGHTS_QUERY` |
| Bash | tree-sitter-bash 0.25.1 | `LANGUAGE` | `HIGHLIGHT_QUERY` (singular!) |
| TOML | tree-sitter-toml-ng 0.7.0 | `LANGUAGE` | `HIGHLIGHTS_QUERY` |

(tsx = the TS grammar's `LANGUAGE_TSX` variant, confirmed. TOML uses the maintained `-ng` fork.) The per-Lang query
const name is NOT uniform — the Lang→(LanguageFn, query &str) table hard-codes each; no `format!("...QUERY")` guessing.

### Architecture
- **(a) THE TAXONOMY (do FIRST — compiler-forced, small, precedes the axis):**
  - `marley_syntax::TokenKind` (lib.rs:23) grows +5 → `{Keyword, Function, Type, Str, Number, Comment, Attribute,
    Punctuation, Property, Plain}` (constant/constant.builtin KEEP folding into Number — today's precedent).
  - `code_syntax::TokenKind` (:58) grows the SAME +5 (its `Hint` stays) → 11 variants.
  - `kind_from_syntax` (:246) + `token_color` + `highlight_ranges` gain exhaustive arms (the compiler forces every
    one — the #340 MarkTier discipline; ~8 sites). `token_color` v1 maps the 5 new kinds to reasonable EXISTING
    theme colors (#316 owns per-theme values — HARD ordering #315 before #316, D-TEN-SLOT-TAXONOMY).
  - REQ-006 side-effect: Rust's own captures `type`/`function`/`property`/`attribute`/`punctuation` (today → Plain)
    now map to the new slots → Rust gains richer color. A before/after fixture diff pins EXACTLY which spans moved.
- **(b) THE LANGUAGE AXIS (`marley_syntax`, a NEW `src/lang.rs`):**
  - `pub enum Lang { Rust, Python, JavaScript, TypeScript, Tsx, Json, Toml, Bash }` (Tsx a distinct variant — the
    TS grammar's tsx `LanguageFn`).
  - `Lang::grammar() -> tree_sitter::Language` (from each crate's `LANGUAGE`/`LANGUAGE_TYPESCRIPT`/`LANGUAGE_TSX`)
    + `Lang::highlight_query() -> &'static Query` (a per-Lang OnceLock, compiling the crate's `HIGHLIGHT(S)_QUERY`
    ONCE — the parse.rs OnceLock idiom generalized to a per-Lang static/table).
  - `HighlightSession::new(lang: Lang)` (lib.rs:333) — `parser.set_language(&lang.grammar())`; `highlight_lines(src,
    lang)` (:112); `kind_of_capture(lang, name)` (:39) — a per-Lang capture→TokenKind map (each grammar's capture
    base-name → the slot; the shared base map covers most, per-Lang overrides where a grammar names differently).
    Unmapped → Plain (drop-before-sweep holds). The AGNOSTIC core (sweep_disjoint/clip_to_lines/lines_from_spans/
    the splice family/point_at/syntax_edit) is UNTOUCHED.
  - **Per-grammar CLOSED-capture pin test (D-PER-GRAMMAR-PIN-TEST):** the 21-name Rust idiom ×N — each Lang's
    `query.capture_names()` truth-tabled, so a grammar bump that renames captures fails LOUDLY per language.
- **(c) THE APP PLUMBING (`marley_app`):**
  - `SyntaxReq` gains `lang: Lang` (mapped from the file's `language_of` at request time). The worker's ONE
    permanent session becomes lang-aware: REBUILD (`HighlightSession::new(req.lang)`) when `req.lang` differs from
    the session's current lang (D-REBUILD-SESSION-ON-LANG-SWITCH — a switch is a new nonce → full parse anyway, so
    incremental continuity loses nothing). The `is_rust` gate (app.rs:3466/:3479) widens to
    `language_of(path).grammar_lang().is_some()` (a grammar-supported language).
  - `language_of` (code_syntax.rs:27) grows `py`→Python, `js|mjs|cjs`→JavaScript, `ts`→TypeScript, `tsx`→Tsx
    (D-ONE-LANGUAGE-TABLE, #299 — ONE table for detection/comment/grammar; a `Language::grammar_lang() -> Option<Lang>`
    bridges the app's `Language` to marley_syntax's `Lang`).
  - the `(nonce, version)` cache key stays SUFFICIENT (language is a pure fn of the path; the nonce is per-buffer;
    a different file = a different nonce → no cross-language cache bleed).
- **(d) THE FLOOR (§14, D-FLOOR-IS-THE-HAND-LEXER):** a path whose `language_of` maps to no grammar_lang → the
  worker takes NO tree path (the widened gate returns false) → the shipped `highlight_ranges` hand-lexer / plain
  render, byte-identical. A parse that yields no tree → the same floor. Never a panic, never a mis-grammar parse.

### File manifest
| File | Change |
|---|---|
| crates/syntax/Cargo.toml | +6 grammar deps (the resolved versions above) |
| crates/syntax/src/lang.rs | NEW — `Lang` enum + `grammar()` + per-Lang `highlight_query()` OnceLock table + `kind_of_capture(lang, name)` + the 6 per-grammar pin tests |
| crates/syntax/src/lib.rs | `TokenKind` +5; `HighlightSession::new(lang)`; `highlight_lines(src, lang)`; re-export `Lang`; the Rust pin test stays |
| crates/syntax/src/parse.rs | the per-Lang parser factory (`parser_for(lang)`) — generalize `rust_parser` |
| crates/marley_app/src/code_syntax.rs | `TokenKind` +5; `kind_from_syntax`/`token_color`/`highlight_ranges` arms; `language_of` +py/js/ts/tsx; `Language::grammar_lang()` |
| crates/marley_app/src/app.rs | `SyntaxReq.lang`; the worker rebuild-on-lang-switch; the is_rust→grammar-supported gate |
| crates/marley_app/src/headless_drive.rs | the worker async + rebuild-on-switch drives |
| Cargo.lock | the 6 grammars (+ transitive) |

### Regression Test Plan (≥1 per REQ)
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | a Python fixture (def/class/string/comment/decorator) → the taxonomy kinds | pure fixture (lang.rs) |
| REQ-002 | TS (interface/type/arrow) + a tsx fixture + a JS fixture → kinds | pure fixtures |
| REQ-003 | JSON keys-vs-strings + TOML keys/tables → Property vs Str | pure fixtures |
| REQ-004 | a Bash fixture (keywords/vars/strings/comments) → kinds | pure fixture |
| REQ-005 | each grammar's CLOSED capture set pinned (fails loudly on a bump) | pure — 6 pin tables |
| REQ-006 | Rust output IDENTICAL except the named enrichments (before/after span-kind diff) | pure — the side-effect pin |
| REQ-007 | a no-grammar language (.txt / .md) renders byte-identical (the floor) | pure + headless |
| REQ-008 | the worker serves a non-Rust tree (a >1000-line .py async) + rebuilds across a lang switch | headless |
| REQ-009 | a no-grammar extension takes NO tree path (the by-construction gate) | pure + headless |
| REQ-010 | cargo-deny passes with the 6 new MIT grammar crates | gate:8 |

**Uncoverable / deferred:** the pixel (LIVE off-limits, chad at machine) — the highlight SPANS are the pure seam
(fixtures) + the render's span map (headless); no driven capture. Markdown grammar DEFERRED (D-MD-DEFERRED — the
block+inline injection model fights the per-line pipeline; its own ticket).

### Risks / decisions
- **Grammar-version co-resolution — SETTLED by the spike** (all 6 compile against core 0.26.11; recorded versions).
- **The query const name is per-grammar** (HIGHLIGHTS_QUERY vs HIGHLIGHT_QUERY) — the Lang table hard-codes each;
  no name-guessing. The tsx variant is a distinct `Lang::Tsx` → `LANGUAGE_TSX`.
- **The taxonomy 5→10 is compiler-forced** — do it FIRST (both enums + the ~8 arms), commit-clean, before the axis;
  #316 colors the slots after (hard ordering).
- **Session rebuild-on-switch** — a lang switch is a new nonce → the rebuild parses fresh; no incremental-continuity
  loss. The P3.5 critics attack: a switch mid-parse, the cache-key sufficiency chain (path→language→nonce), the
  floor byte-identity, and each adopted `.scm`'s exact capture names (the pin tests are the guard).
- **Each adopted `.scm` is trimmed onto the taxonomy + provenance-tagged** — the pin test per grammar catches a
  capture the map missed (→ Plain, visible in the fixture).

## Phase 3 — Implement

**Built (manifest, in taxonomy-first order):**
- **`crates/syntax/Cargo.toml`** — +6 grammar deps at the spike-resolved versions (python 0.25, javascript 0.25, typescript 0.23, json 0.24, bash 0.25, toml-ng 0.7; all co-resolve against core 0.26).
- **`crates/syntax/src/lang.rs` (NEW)** — the language axis: `pub enum Lang { Rust, Python, JavaScript, TypeScript, Tsx, Json, Toml, Bash }`; `Lang::grammar() -> tree_sitter::Language`; `Lang::highlight_query() -> &'static Query` (one `OnceLock` per Lang; the const name VARIES — `HIGHLIGHTS_QUERY` for rust/python/ts/tsx/json/toml, `HIGHLIGHT_QUERY` for javascript/bash); `pub fn kind_of_capture(name: &str) -> TokenKind`.
- **`crates/syntax/src/lib.rs`** — `TokenKind` grown 5→10 `{ Keyword, Function, Type, Str, Number, Comment, Attribute, Punctuation, Property, Plain }`; `mod lang` + `pub use lang::{kind_of_capture, Lang}`; `HighlightSession` gained a `lang` field, `new(lang)`, `Default → new(Lang::Rust)`; `highlight_lines(src, lang)`; the Rust 21-name pin table updated for REQ-006 + 8 exact-row Rust fixtures updated (see side effect below); a test-local `highlight_lines(src)` shadow keeps the pre-#315 Rust fixtures reading cleanly.
- **`crates/syntax/src/parse.rs`** — `parser_for(lang)` (was `rust_parser`) + a Rust-pinned `rust_parser()` convenience delegating to it; `collect_spans`/`spans_from_tree`/`spans_in_window` take `lang` and use `lang.highlight_query()`/`lang.grammar()`; `query_capture_names(lang)`.
- **`crates/marley_app/src/code_syntax.rs`** — `Language` +4 (Python/JavaScript/TypeScript/Tsx); `language_of` +4 exts (py, js|mjs|cjs, ts, tsx); `Language::grammar_lang(self) -> Option<marley_syntax::Lang>` (the ONE bridge; Shell→Bash; Markdown/Plain→None); `TokenKind` +5; `kind_from_syntax` +5 exhaustive arms; `lang_spec` +Python/JS/TS/Tsx (line_comment + keyword sets for ⌘/ + the transient floor).
- **`crates/marley_app/src/app.rs`** — `token_color` +5 arms (fixed hues for function/type/attribute/property; **Punctuation→`foreground`** so brackets/delimiters read EXACTLY as pre-#315); `SyntaxReq.lang`; the worker rebuilds its `HighlightSession` when `req.lang` differs (D-REBUILD-SESSION-ON-LANG-SWITCH, `at = None` forces a full parse); `refresh_syntax_cache`'s gate widened from `== Rust` to `grammar_lang().is_some()` + the sync fallback threads `lang` into `highlight_lines`; the two Rust-only throwaway sessions (#329 expand-selection, #330 sticky-headers) pass `Lang::Rust`.

**Deviations from design (with reason):**
1. **`kind_of_capture` is lang-AGNOSTIC (`name: &str`), not the design's `kind_of_capture(lang, name)`.** The tree-sitter capture-name convention is shared across grammars (base-name-before-`.` drives the map), with ONE full-name special case (`string.special.key` → `Property` for JSON keys, REQ-003). A per-Lang signature would be dead ceremony; the per-grammar CLOSED-capture pin tests (Phase 4) still verify each grammar's ACTUAL capture set maps correctly.
2. **The widened gate is `refresh_syntax_cache` (app.rs:~12643), NOT `refresh_bracket_match` (app.rs:~3466).** The design's line-ref `(:3466/:3479)` was STALE — that gate feeds `matching_delimiters_in`, a Rust-ONLY pure fn (it always parses as Rust); widening it would run the Rust bracket-matcher on Python. Its own comment already flagged "#315 extends bracket-match per grammar" as a FUTURE item. Bracket-match, expand-selection, sticky-headers, fold, and tags all stay Rust-only in #315 (hence the retained `rust_parser()` convenience).
3. **The grammar-backed set is BROADER than "add python/js/ts":** Json/Shell(Bash) ALSO move from the hand-lexer to the tree path (they now have grammars). The hand-lexer floor (D-FLOOR-IS-THE-HAND-LEXER) is now the fallback for Markdown/Plain + **TOML** (see below) + transient large-file-async / parse-failure states. **REVISED at inspect (C1-2):** TOML was ALSO going to the tree path at implement, but its adopted query mis-colors `key = value` under our sweep → reverted to the floor; so the tree-routed set is Rust/Python/JS/TS/TSX/Json/Bash (7).

**REQ-006 named side effect (which Rust captures changed kind):** in the 21-name pin table — `type`/`type.builtin`/`constructor`→`Type`, `property`→`Property`, `function`/`function.method`/`function.macro`→`Function`, `punctuation.bracket`/`punctuation.delimiter`/`operator`→`Punctuation`, `escape`→`Str`, `attribute`→`Attribute` (12 of 21 moved off `Plain`). 8 exact-row Rust fixtures updated to match (semicolons/brackets now `Punctuation`, `fn` names now `Function`).

**Compile/test state:** `cargo check -p marley_syntax --tests` + `cargo check -p marley --tests` clean; `cargo fmt` clean; marley_syntax 64/64 pass; the 5 touched app tests (token_color/language_of/kind_from_syntax/highlight_ranges) pass. `git add -N crates/syntax/src/lang.rs` done (mutants not silently skipped). No test EXPANSION (Phase 4 owns the 6 fixtures + 6 pin tables + the worker drives); only existing tests updated to compile/pass.

**Notes for Phase 4 (mutation surface — `cargo mutants --list --in-diff` RUN at inspect, the REAL set):**
- **VIABLE new mutants to kill (the Phase-4 fixtures/tables must cover these):**
  - `lang.rs kind_of_capture` — whole-fn→Default (1), `==`→`!=` on the `string.special.key` guard (1), + 9 delete-match-arm (keyword/function/type|constructor/string|escape|char/number|constant|boolean/comment/attribute/punctuation|operator/property). Killed by: the capture-map test (assert the special-key→Property AND a plain string→Str for the `==` mutant; assert one capture per base-name arm). **= 11 mutants, the densest surface.**
  - `lang.rs Lang::grammar`→Default (1), `Lang::highlight_query`→Box::leak(Default) (1), `highlight_query::compile`→Default (1). Killed by: the per-grammar highlight fixtures (a Default/empty grammar or query → empty/wrong spans → fixture fails).
  - `code_syntax grammar_lang`→None (1, VIABLE). Killed by: a `grammar_lang` per-variant test (any `Some` assert kills None; assert all 10 for correctness).
  - `code_syntax language_of` delete-arm ×4 (py/js/ts/tsx). Killed by: `language_of_by_ext` new-ext asserts.
  - `code_syntax lang_spec`→None (1, VIABLE). Killed by: `line_comment_for(Python/JS/TS)` asserts. (Behavioral, not mutation-forced — the new keyword arms add NO per-arm mutant [inspect]: Phase 4 SHOULD still add ≥1 `line_comment_for` per new lang + ≥1 `highlight_line` keyword-floor fixture [e.g. Python `def` highlights via the transient floor] for REQ coverage, since these specs ARE reachable — `line_comment_for` at app.rs ⌘/ + `highlight_ranges` while a large grammar-backed file parses off-thread.)
- **UNVIABLE (no `Default` derive → compile-fail → cargo-mutants skips; do NOT write dead tests for these):** `language_of`→Default, `kind_from_syntax`→Default, `grammar_lang`→Some(Default), `lang_spec`→Some(Default) (Language/TokenKind/marley_syntax::Lang/LangSpec have no Default).
- **Already killed by EXISTING MSI-100 tests (in-diff only because the lines were touched):** `highlight_lines`/`highlight_full`/`highlight_incremental` (lib.rs), `collect_spans`/`spans_from_tree`/`spans_in_window`/`parser_for`/`rust_parser` (parse.rs), `token_color`→Default (app.rs — `token_color_maps_each_kind` asserts several kinds; extend it with the 5 new kinds defensively). Re-confirm green at the `--diff` gate.
- **INSPECT-DRIVEN regression tests Phase 4 MUST add (guarding the C1 fixes):** (a) **TS + TSX fixtures** asserting a STRING, COMMENT, NUMBER, and a base `const`/`function` keyword all color (the C1-1 concatenation regression — the naive per-grammar pin test PASSES while TS is broken, so this behavioral fixture is the real guard); (b) **`grammar_lang(Toml) == None`** asserted + `grammar_lang` for every other variant → `Some(expected)` (C1-2 deferral + the whole-fn None mutant); (c) **JSON `{"k":"v","n":1}`** → key=Property, `"v"`=Str, `1`=Number (C1-3 determinism). Also: the `sweep_disjoint` sort change (`sort_unstable`→`sort_by_key`) is in-diff now — its existing sweep tests cover it (64/64 green), re-confirm at the gate. The Tsx query CONCATENATION means the Phase-4 TS/TSX per-grammar pin test should pin the JS∪TS capture set, not just the TS overlay's 5.

Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.

## Phase 3.5 — Inspect

**4 parallel general-purpose critics** over the working-tree diff, distinct lenses: (1) grammar capture-map correctness, (2) worker rebuild race / state integrity, (3) provenance / clean-room, (4) simplification / hygiene + deviation soundness. The adversarial phase EARNED ITS KEEP: critic 1 found **1 CRITICAL + 1 HIGH** that my self-review AND the other 3 critics all missed — because it alone READ the 8 shipped `highlights.scm` files + ran every language through a probe. Lesson reinforced: for an adopt-upstream-queries change, the load-bearing verification is *running each grammar's actual query*, not reasoning about the map.

### Findings + verdicts

| # | Sev | Finding | Verdict | Fix |
|---|-----|---------|---------|-----|
| C1-1 | **CRITICAL** | TS/TSX render almost blank — `tree_sitter_typescript::HIGHLIGHTS_QUERY` is the `inherits: ecma` OVERLAY only (5 captures: keyword/punctuation.bracket/type/type.builtin/variable.parameter); no string/comment/number/function + no ecma keywords. `grammar_lang(TS)=Some` → the tree path wins, the hand-lexer floor is never consulted → `.ts`/`.tsx` show only type annotations (WORSE than the floor). Per-grammar pin test would PASS while the language is broken. | **REAL** | `lang.rs highlight_query`: TS = `format!("{JS_HIGHLIGHT_QUERY}\n{TS_HIGHLIGHTS_QUERY}")`; TSX = `JS + JSX_HIGHLIGHT_QUERY + TS` — the standard `inherits` resolution. **VERIFIED**: probe shows TS now colors const/let/function (Keyword), `"str"` (Str), `// hi` (Comment), `42` (Number), `f` (Function); TSX colors const/let/5. The concatenation **COMPILES** against `LANGUAGE_TYPESCRIPT`/`LANGUAGE_TSX` (no panic) for the pinned versions. |
| C1-2 | **HIGH** | TOML mis-colors every `key = value` — the query's `(pair (bare_key)) @property` covers the WHOLE pair; our outer-wins-clip-forward sweep paints the VALUE Property (`foo = 1` → key=Type, `1`=Property — REQ-003 inverted). Also a REGRESSION (hand-lexer colored TOML correctly). Independently corroborated by my own read of `toml-ng/queries/highlights.scm`. | **REAL** | `code_syntax::grammar_lang(Toml) = None` → `.toml` stays on the hand-lexer floor (correct, no regression). `marley_syntax::Lang::Toml` + the grammar dep RETAINED (version-pinned, still pin-tested); re-routing awaits an innermost-wins sweep (**follow-up** — a deliberate sweep-semantics change that also touches Rust `@attribute` absorption, hence out of #315). |
| C1-3 | LOW | JSON key Property-vs-Str rests on `sort_unstable`'s "may reorder equal elements" (identical-range double-capture `string.special.key`+`string`). Latent (couldn't reproduce a flip). | **REAL** | `sweep_disjoint`: `sort_unstable_by_key` → `sort_by_key` (stable) — preserves query match order (special.key first) → Property DETERMINISTIC. **VERIFIED**: JSON `{"key":"val","n":42}` → keys=Property, `"val"`=Str, `42`=Number. Equal-kind dups (Rust doc comment) unaffected; 64/64 syntax tests still pass. |
| C1-4 | LOW | 2 stale comments: `multibyte` test says "escape maps Plain" (now Str); `TokenKind::Attribute` doc cites `@decorator` (Python decorators → Function; Attribute is effectively Rust `#[…]`-only). | **REAL** | Reworded both (lib.rs) + the app `TokenKind::Attribute` doc. |
| C2-1 | LOW | "stay Rust-only" comment on the #329/#330 throwaway sessions is imprecise — they're Rust-PARSED (run the Rust grammar on any file), NOT Rust-GATED like `refresh_bracket_match`. Pre-existing behavior. | **REAL** | Reworded the comment (app.rs) to "Rust-PARSED … not language-gated … degraded structural results on a non-Rust file, never a crash". |
| C3-1 | LOW | Spec `## Reference (§20)` / D-QUERIES-ARE-ADOPTED prose says the queries are "trimmed, provenance-tagged in-file" — the code references each crate's `HIGHLIGHT(S)_QUERY` const LIVE (vendors NOTHING). Shipped posture is CLEANER than the spec claims. | **REAL** (doc) | Corrected the spec prose (3 spots) to "referenced live from the crate; copies/trims/vendors nothing". |
| C4-1 | LOW | Stale intra-doc link `parse.rs:19` — `tags_query` doc links `[`highlight_query`]`, moved to `Lang::highlight_query` by #315. Gate-green (`pub(crate)` + gate omits `--document-private-items`) but a real dangling ref. | **REAL** | Plain backticks `` `Lang::highlight_query` ``. |

**Worker-race critic (C2): NO integrity defect** — the rebuild is triple-guarded (`at=None` + `tree:None` fallback + edit-nonce-gating); `nonce` is globally unique per open file (never reused, no in-place `path` write) ⟹ one nonce = one file = one language ⟹ cross-language cache collision is IMPOSSIBLE; the pump's nonce check subsumes a language check. The Tsx-compiles-TS-query risk was probed OK. **Provenance critic (C3): CLEAN BILL** (all 6 MIT, cargo-deny passes, nothing vendored). **Simplification critic (C4): all 3 deviations SOUND** with evidence-backed rejection of every speculative concern (lang-agnostic map, the widened-gate choice, `rust_parser` retention, `Punctuation→foreground`, the test shadow).

**Accepted-as-is (documented, not fixed):** the `lang_spec` TS-arm keyword duplication (floor-only data; concat of two `&'static [&'static str]` isn't ergonomic in plain Rust — a `constcat` dep would be over-engineering for the transient floor); the 8-arm `highlight_query` OnceLock (correct + explicit; an array-indexed form trades clarity for brevity). Both are quality-only.

**Blast-radius change from a fix:** deviation #3 (Toml/Json/Shell move hand-lexer→tree) is now **Json/Shell only** — TOML reverted to the floor (C1-2). Grammar-backed set is now Rust/Python/JavaScript/TypeScript/Tsx/Json/Bash (7); Toml/Markdown/Plain → floor.

Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.

## Phase 4 — Validate

**Tests added** (all RUN + green — see counts below):

*marley_syntax (`crates/syntax/src/lib.rs` tests):*
- `kind_of_capture_table` (REQ-004) — the SHARED base-name map, direct: every arm + `string.special.key`→Property override + a plain `string.special`→Str. Kills the 9 delete-match-arm mutants + the `==`→`!=` guard mutant, independent of grammar output.
- `queries_compile_for_every_lang` (REQ-005 + inspect C1-1) — all 8 Langs' `highlight_query()` compiles (the TS/TSX arms concatenate the JS base — a cross-version break panics HERE, in CI) + exposes non-empty captures.
- `python_fixture` / `javascript_fixture` / `typescript_fixture` / `tsx_fixture` / `bash_fixture` (REQ-002/006) — BEHAVIORAL: each colors its expected kinds. `typescript_fixture_colors_base_and_type_layers` is the **C1-1 regression guard** (asserts the ecma BASE layer — strings/comments/numbers/keywords/functions — colors, not just the TS type overlay).
- `json_keys_are_property_values_are_str` (REQ-003 + inspect C1-3) — keys→Property, string values→Str, numbers→Number, and the key is Property DETERMINISTICALLY across 8 runs (guards the stable-sort fix).
- `session_highlights_per_its_lang` (REQ-008 pure half) — `HighlightSession::new(lang)` parses with THAT grammar (the primitive the worker rebuilds on a lang switch).
- (from implement, REQ-006): the Rust 21-name pin table + 8 exact-row Rust fixtures updated for the taxonomy growth.

*marley_app (`code_syntax.rs` + `app.rs` tests):*
- `language_of_by_ext` extended (REQ-001) — py/js/mjs/cjs/ts/tsx + case-insensitive `.TS`. Kills the 4 `language_of` delete-arm mutants.
- `grammar_lang_maps_every_variant` (REQ-009 + inspect C1-2) — all 10 `Language` variants; **Toml→None** (the deferral) + Markdown/Plain→None; the 7 grammar-backed→Some. Kills the `grammar_lang`→None mutant.
- `line_comment_for_new_langs` — Python `#`, JS/TS/Tsx `//`. Kills the `lang_spec`→None mutant.
- `kind_from_syntax_maps_one_to_one` extended — all 10 syntax→app kinds.
- `token_color_maps_each_kind` extended — the 5 new kinds' exact v1 colors (Punctuation→foreground). Kills the whole-fn Default mutant on the newly-grown match.

**Covered by mechanism / the gate (NOT unit tests — stated, not silently skipped):**
- **REQ-007** (no-grammar → floor, byte-identical): `highlight_ranges` (the floor) is UNCHANGED; `grammar_lang(Markdown/Plain/Toml)=None` routes there (pinned by `grammar_lang_maps_every_variant`). Byte-identity holds by construction (no line of the floor changed).
- **REQ-008** (the worker's async rebuild-on-switch): the worker is an `app.rs` shim (`ensure_syntax_worker_and_send`, `#[cfg_attr(test, mutants::skip)]`); its PURE primitive is `session_highlights_per_its_lang`, and the rebuild's correctness is mechanism-verified (inspect critic 2: `nonce` globally-unique-per-open-file ⟹ one nonce = one file = one language ⟹ no cross-language cache collision; `at=None` on rebuild forces a full parse).
- **REQ-010** (cargo-deny): gate:8 (all 6 grammar crates MIT — provenance critic confirmed `cargo deny check licenses` = ok).
- **LIVE pixel** (env-blocked): chad is AT THE MACHINE → synthetic-input drives are OFF-LIMITS (they'd hit his frontmost window). #315 is a PURE span-computation change — the STATE (which kinds each language yields) is headless-proven by the per-language fixtures; the render plumbing (`token_color`→styled runs) is UNCHANGED from pre-#315 (just more kinds), so the pixel is deferred-not-skipped (mirrors #305/#314). Re-verify colors live when the machine is free (30s, no ticket).

**Test run:** marley_syntax 9 new (fixtures + `kind_of_capture_table` + `queries_compile_for_every_lang` + `session_highlights_per_its_lang`) — all pass, full suite green; marley_app 5 new/extended — all pass.

**Gate: `GATE GREEN [diff]` — 15/15, 0 failed.** coverage **100%** + mutation **MSI 100%** + miri + visual/AX + cargo-deny + docs all green; receipt written (commit-valid). **One coverage red fixed at source (§0):** the JSON-determinism loop's `assert_eq!` had a lazily-evaluated `r[0]` MESSAGE arg on its own line (lib.rs:811) — only executed on assertion FAILURE, so uncovered while the test passes → replaced with a static message (the value being asserted is still evaluated every iteration). Mutation was already MSI 100 on the first (coverage-red) run — the in-diff set (kind_of_capture's 9 delete-arms + the `==`/`!=` guard, grammar_lang→None, language_of's 4 delete-arms, lang_spec→None, the lang.rs grammar/query fns, token_color) all killed; the `Default::default()` mutants on Language/TokenKind/Lang/LangSpec were unviable (no Default derive), exactly as predicted at inspect.

## Phase 5 — Complete

**Docs (§21):** CHANGELOG.md #315 entry (the multi-language axis + the 4 load-bearing parts incl. the two inspect finds); `crate-map.md` marley_syntax row (multi-language, 10-kind palette, TS/TSX concat, TOML floor); `roadmap.md` M20 line + B3 line (multi-language highlighting; structural node APIs still Rust-only); `editor.md` marley_syntax section (the `Lang` axis, `kind_of_capture` + the JSON-key override, the stable sweep, the worker rebuild, the TOML deferral). All drafted during the gate wait, verified present.

**Knowledge captured (forge wired):**
- AAR `c0e7cea0` submitted — outcome completed, effectiveness 4 (shipped, but the CRITICAL + HIGH were caught ONLY at inspect — plan/design/implement all missed them; the adversarial phase earned its keep, hence 4 not 5).
- Failures (recorded at inspect): `BF-claude-adopted-inherits-overlay-query-shipped-alone-blanks-the-language-001` (CRITICAL — TS/TSX blank), `BF-claude-broad-container-capture-plus-outer-wins-sweep-steals-the-value-001` (HIGH — TOML).
- Prevention rule: `PR-claude-verify-adopted-grammar-by-running-its-query-001` — verify an adopted grammar by RUNNING its query over a real fixture (assert string/comment/number/keyword/function color), NOT by pinning its capture-name set (a capture-set pin PASSES while a language is blank).
- Architecture decision: `AD-claude-outer-wins-sweep-defers-broad-container-grammars-001` — the sweep is outer-wins; a grammar with broad container captures stays on the hand-lexer floor until an innermost-wins sweep (a separately-ticketed follow-up, since it changes Rust attribute rendering).

**The durable lessons:**
1. **The spec's "hard half already shipped" HELD** — the language axis threaded cleanly through the #268/#274 seam; the taxonomy growth was compiler-forced + bounded (2 exhaustive matches). The genuine risk was NOT the plumbing but the ADOPTED QUERIES' semantics, which only a grammar-map critic reading the `.scm` + running each language surfaced.
2. **An `inherits:` overlay grammar (TypeScript) ships an INCOMPLETE query** — you must concatenate the inherited base. A per-grammar capture-set pin test gives FALSE confidence (it passes on the overlay's 5 captures while the language renders blank). The real guard is a behavioral fixture per language.
3. **Adopting a query verbatim means routing AROUND its incompatibilities, not editing it** — TOML's broad `(pair) @property` fights our outer-wins sweep, so we defer TOML to the floor rather than trim the query (which would break the clean-room "vendor nothing" posture) or flip the shared sweep (which would change Rust attribute rendering).
4. **A lazily-evaluated `assert!` MESSAGE arg on its own line is an uncovered-line trap** (the coverage red — lib.rs:811) — use a static message or inline capture, since the positional value arg only executes on failure.

**Follow-ups (open, un-ticketed — noted for the shelf):** the innermost-wins sweep (re-enables TOML tree highlighting; needs Rust `(attribute_item)` fixtures first); per-grammar STRUCTURAL node APIs (fold/sticky/bracket/tags/expand-selection are still Rust-only — the `rust_parser()` convenience marks them); JSX element-tag coloring is sparse in TSX v1 (base ecma layer colors; the `highlights-jsx.scm` tag captures are minimal). The #316 theme palette pulls the 10 kinds' v1 fixed hues into per-theme colors (hard ordering #315→#316).

Status: Phase 5 — Complete PASS.
