---
pipeline_id: aacfebff-909c-4b4b-8a15-23b1f57ff860
ticket: forge#315 (2936c168-b807-48de-8a57-d811d8288e9d) · local docs/planning/tickets/open/TICKET-315-multi-language-syntax.md
aar_id: c0e7cea0-9541-46b1-94bb-18fa850d7bd3
status: Phase 5 — Complete PASS
title: Multi-language syntax highlighting — the language axis through marley_syntax, and a 10-slot taxonomy
type: feature
milestone: M20
references: [THE EXACT RUST-HARDCODED LIST (recon 2026-07-17): highlight_query's OnceLock static, collect_spans, rust_parser, HighlightSession::new (no lang arg — parse.rs:96/lib.rs:328), kind_of_capture's 6-name map, the 21-name pin test (lib.rs:642); SyntaxReq has NO language field (app.rs:11495) and the worker owns ONE permanent Rust session (app.rs:11449); language_of = {rs,toml,json,sh|bash|zsh,md} (code_syntax.rs:27); TWO TokenKind enums (syntax 5-kind, app 6-kind+Hint) bridged by kind_from_syntax (code_syntax.rs:246); tree-sitter core 0.26.11 pinned, NO other grammar crates in the lock; the hand-lexer fallback (highlight_ranges) stays as the no-grammar floor]
---

## Title
`marley_syntax` highlights RUST ONLY — a Python or TypeScript file renders plain. This threads a language
axis through the crate (grammar + query per language), grows the token taxonomy from 5 slots to 10 (the
set #316 themes), and keeps every no-grammar language on the shipped hand-lexer floor. **The taxonomy
growth is a DECISION this spec makes explicit** — the forge descriptions of #315/#316 jointly assume it
but neither states it: today `type`/`function`/`property`/`attribute`/`punctuation` captures map to
Plain, so Rust itself gains visibly richer color as a side effect. Named, not accidental.

## Scope
### In
- **The taxonomy (the pinned decision):** `marley_syntax::TokenKind` grows to
  `{ Keyword, Function, Type, Str, Number, Comment, Attribute, Punctuation, Property, Plain }` —
  `constant|constant.builtin` keep folding into `Number` (today's precedent). The app's
  `code_syntax::TokenKind` grows the same five (+ its own `Hint` stays); `kind_from_syntax` +
  `token_color` gain arms (compiler-forced exhaustiveness — the #340 MarkTier discipline). `token_color`
  v1 maps the new kinds to reasonable existing colors; **#316 owns per-theme values** (hard ordering:
  #315 before #316).
- **The language axis in `marley_syntax`:** a `Lang` enum (Rust, Python, JavaScript, TypeScript, Json,
  Toml, Bash) + per-language `(grammar, highlight query)` registration — the query compiled once per
  language (the OnceLock idiom generalized to a per-Lang table); `HighlightSession::new(lang)` (the
  parser factory parameterized); `highlight_lines(src, lang)`; `kind_of_capture(lang, name)` — each
  grammar's capture set is CLOSED per version and **pinned by its own truth-table test** (the Rust
  21-name pin generalizes; a grammar bump that renames captures fails loudly, per language). Unmapped
  captures → Plain (drop-before-sweep keeps holding). The language-AGNOSTIC core (sweep_disjoint,
  clip_to_lines, lines_from_spans, the incremental splice family, point_at/syntax_edit) is untouched —
  recon verified it clean.
- **Grammar deps:** `tree-sitter-{python, javascript, typescript, json, toml, bash}` — versions resolved
  at implement against the pinned core **0.26.11** (tree-sitter-rust rides 0.24.2 against the same core,
  so ABI-line mixing is known-fine); licenses MIT, and **gate:8 cargo-deny enforces it** — the provenance
  check is mechanical, not manual. Each grammar's own `HIGHLIGHT(S)_QUERY` const is the adopted source,
  REFERENCED LIVE from the crate (the crate maintains its `.scm`; Marley copies/trims/vendors NOTHING —
  cleaner than an in-file copy, no attribution text to track; §20 adoption of a published API).
- **The app plumbing:** `SyntaxReq` gains `lang`; the worker's ONE permanent Rust session becomes
  lang-aware (simplest honest shape: keep one session, REBUILD when `req.lang` differs from the session's
  — a language switch is a different nonce, hence a full parse anyway, so incremental continuity loses
  nothing); `refresh_syntax_cache`'s `is_rust` gate widens to `lang_of_path is tree-sitter-supported`;
  the `(nonce, version)` cache key is SUFFICIENT (language is a pure function of the path, and the nonce
  is per-buffer — promotion re-verifies that chain). `language_of`'s table grows `py`, `js|mjs|cjs`,
  `ts|tsx` (tsx under the TS grammar's tsx variant — implement confirms); **the ONE-table rule (the #299
  lesson): detection + comment-token + grammar all key off `code_syntax`'s existing table — no second
  source of language facts.**
- **The floor:** a language WITHOUT a grammar (or a parse failure) falls to the shipped hand-lexer /
  plain rendering exactly as today — never a panic, never a mis-grammar parse (the #340 M1 gate class,
  now enforced by construction: the grammar table returns None → no tree path at all).
### Out (explicitly)
- **Markdown grammar — DEFERRED, named:** tree-sitter-md is a two-grammar injection model (block +
  inline) that fights the per-line span pipeline; Markdown stays on today's plain-pass. Its own ticket
  when wanted.
- Extending the OTHER tree-sitter features (bracket-match pair tables, fold kinds, sticky headers,
  file-symbols tags) beyond Rust — each names #315 as its extension point and each is its own small
  follow-up (their gates stay Rust-only meanwhile). Embedded-language injections (JS in HTML). Grammar
  hot-loading / user grammars. The incremental-parse path for non-Rust beyond what the shared session
  gives (full-parse is the v1 floor for the new languages; the >1000-line async route already bounds it).

## Reference (§20)
tree-sitter + each grammar crate = published-API + MIT-query ADOPTION (each grammar's `HIGHLIGHT(S)_QUERY`
const is referenced LIVE from the crate — the crate maintains the `.scm`; Marley copies/trims/vendors
nothing). VS Code / Zed = observed only in the trivial sense (files render colorized). No copyleft source
involved anywhere in this ticket.

### Prior art
1. **Behavior maps / observed** — n/a beyond "a .py file is colorized"; the interesting art is below.
2. **Published material** — tree-sitter's multi-grammar client docs; each grammar's capture conventions.
3. **OUR PERMISSIVE DEPS — the whole ticket IS the sweep:** every grammar crate ships its own
   `HIGHLIGHTS_QUERY` (verified for rust; the family convention) — Marley writes NO queries, only the
   capture→taxonomy maps and their per-grammar pin tests. The recon's exact-list of Rust-hardcoded fns
   (7 items) vs the already-agnostic core (11 items) IS the work plan — the crate was built with this
   seam coming (`is_header_kind`'s doc: "other languages extend when #315 lands").

## Locked-In Decisions (design confirms; deltas → the notes)
- **D-TEN-SLOT-TAXONOMY** — the growth is explicit; Rust gets richer color as a NAMED side effect; #316
  colors the slots per-theme afterward (hard ordering).
- **D-ONE-LANGUAGE-TABLE** — detection/comment/grammar from the one `code_syntax` table (#299 binding).
- **D-QUERIES-ARE-ADOPTED** — each grammar's own `HIGHLIGHT(S)_QUERY` const referenced LIVE from the crate
  (the crate maintains the `.scm`; Marley copies/trims/vendors nothing); never hand-written.
- **D-PER-GRAMMAR-PIN-TEST** — each grammar's closed capture set truth-tabled (the 21-name idiom ×N).
- **D-REBUILD-SESSION-ON-LANG-SWITCH** — one worker session, rebuilt across languages (a switch is a new
  nonce → full parse anyway).
- **D-MD-DEFERRED** — the injection model is named out.
- **D-FLOOR-IS-THE-HAND-LEXER** — no grammar ⇒ exactly today's rendering, by construction.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | highlight a Python fixture (def/class/string/comment/decorator) with the expected taxonomy kinds | pure fixture |
| REQ-002 | highlight TS (interface/type/arrow-fn) and JS fixtures likewise; tsx parses under the tsx variant | pure fixtures |
| REQ-003 | highlight JSON keys vs strings and TOML keys/tables distinctly (Property vs Str) | pure fixtures |
| REQ-004 | highlight a Bash fixture (keywords, vars, strings, comments) | pure fixture |
| REQ-005 | pin each grammar's CLOSED capture set with a per-grammar truth table that fails loudly on a grammar bump | pure — the pin tests |
| REQ-006 | keep Rust output IDENTICAL except the named taxonomy enrichments (a before/after fixture diff pins exactly which spans changed kind) | pure — the side-effect pin |
| REQ-007 | render a no-grammar language (a .txt, Markdown) EXACTLY as today (the hand-lexer/plain floor, byte-identical) | pure + headless |
| REQ-008 | serve non-Rust trees through the worker (a >1000-line .py lands async) and rebuild the session across a language switch | headless |
| REQ-009 | never mis-parse: a file whose extension maps to no grammar takes NO tree path (the by-construction gate) | pure + headless — the #340 M1 class |
| REQ-010 | pass cargo-deny with every new grammar crate (MIT, in-lock) | gate:8 |

## Phase Plan
P2 resolve the grammar-crate versions against core 0.26.11 (a spike compile) + confirm the tsx variant +
the per-Lang query-table shape + the exact capture→taxonomy map per grammar (read each upstream .scm);
P3 the taxonomy growth first (both enums + bridges + token_color arms — compiler-forced, small), then the
marley_syntax language axis + pin tests, then the app plumbing; P3.5 critics on the REQ-006 Rust-diff
(exactly which kinds moved), the session-rebuild races, the cache-key sufficiency chain
(path→language→nonce), the floor byte-identity; P4 fixtures ×6 languages + drives + gate; P5 docs
(crate-map's marley_syntax row, editor.md, the #346/#304/#305/#340 extension notes).
Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
