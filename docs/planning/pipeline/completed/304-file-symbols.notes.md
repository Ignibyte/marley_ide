# Go to Symbol in File (⌘⇧O) — Notes

- **Forge ticket:** #304 4b10e6e2-0a24-4835-bcaf-a8be11a432ab
- **AAR:** 044a3850-e2e3-47f8-92c2-2c47780d1574
- **Local ticket doc:** docs/planning/tickets/open/TICKET-304-file-symbols.md
- **Pipeline spec:** 304-file-symbols.spec.md

<!-- Working scratch. Each phase appends its entry. -->

## Phase 1 — Plan
- **Request:** ⌘⇧O fuzzy picker of the current Rust file's symbols via tree-sitter's tags query; jump-centered + NavStack. Promoted from the pre-authored queued spec (the Fable method); FOURTH of `/work 300,302,303,304,305,314,315,316,317,259`.
- **Classification / tier:** work pipeline, one shippable slice (a pure `file_symbols` marley_syntax adapter + the #325-shaped picker + the jump + the chord). M19 editor power-tools.
- **Promotion done:** `git mv` queued→active; pipeline_id `4687b05f-744c-468a-8a86-ff6e51c0e3cc`; aar `044a3850-e2e3-47f8-92c2-2c47780d1574`; TICKET-304 created.
- **Seam re-verification (spec written on Fable; #300/#302/#303 all landed since — app.rs drifted 3×; EVERY cited line treated as stale).** The whole approach rests on ONE claim (tree-sitter-rust ships TAGS_QUERY) — an Explore agent is verifying it against the cargo registry + all other seams. _Findings table below._

### Phase 1 findings (seam re-verification)

**F1 — THE LOAD-BEARING CLAIM IS CONFIRMED (verified inline against the cargo registry).** tree-sitter-rust
**0.24.2** (matches the spec exactly; Cargo.lock) ships `pub const TAGS_QUERY: &str = include_str!("../../queries/tags.scm");`
(bindings/rust/lib.rs:49). The core approach — a query RUN, not a hand-rolled walk — holds. The ACTUAL
`queries/tags.scm` captures (read in full):
| capture | node(s) | `@name` node | → SymbolKind |
|---|---|---|---|
| `@definition.class` | `struct_item`, `enum_item`, `union_item`, `type_item` | `type_identifier` | struct/enum/union/type-alias (ALL map to "class") |
| `@definition.method` | `function_item` INSIDE a `declaration_list` | `identifier` | method (impl/trait body) |
| `@definition.function` | `function_item` | `identifier` | free fn |
| `@definition.interface` | `trait_item` | `type_identifier` | trait |
| `@definition.module` | `mod_item` | `identifier` | mod |
| `@definition.macro` | `macro_definition` | `identifier` | macro |
| `@reference.call` / `@reference.implementation` | call/macro-invocation / `impl_item` | — | **DROPPED** (not this ticket) |

**F2 (the sharp extractor edge the spec GLOSSED — a real design decision) — the METHOD/FUNCTION DOUBLE-MATCH.**
A `function_item` inside a `declaration_list` matches BOTH the method pattern (tags.scm:19) AND the free-function
pattern (tags.scm:25) — tree-sitter patterns match independently, so an impl/trait method yields TWO captures at
the SAME node (one `@definition.method`, one `@definition.function`). The design MUST dedup by node span (line,col)
and PREFER `method` (the more specific), or every method appears twice in the picker. This is the extractor's
central correctness edge — pin it in the truth table (a fixture with an impl method + a free fn).

**F3 — the QUALIFIER (`Type::method`) is NOT free from the tag.** `@definition.method` captures only the method
name, not the enclosing type. To show `Type::method` you must walk UP from the method node to the enclosing
`impl_item` and read its `type:` field (tags.scm treats impl as `@reference.implementation`, dropped). The spec
already made this best-effort ("never block the ticket on perfect qualifiers"); the design decides parent-walk
vs kind-glyph-only. Recommend: v1 kind-glyph-only, defer the qualifier (the walk is a nontrivial node-navigation;
the glyph already disambiguates a method from a free fn).

**F4 — "impls" in the title are NOT symbols; their METHODS are.** `impl_item` is `@reference.implementation`
(dropped). REQ-001's "impl methods" IS covered (`@definition.method`); the title's bare "impls" is loose — an impl
block itself is not a pickable symbol (matches VS Code's ⌘⇧O, which lists methods, not impl blocks).

**F5 — `@definition.class` conflates 4 node kinds.** struct/enum/union/type_item all → `@definition.class`. To
render distinct glyphs (struct vs enum), the design reads the matched node's `kind()` to refine class→{Struct,
Enum,Union,TypeAlias}; else one "type" glyph. A design choice, not a blocker (the `is_header_kind` 4-kind table
is NOT reused — that's sticky-header ranges; this is names).

**F6 — `file_symbols` compiles TAGS_QUERY via the parse.rs OnceLock idiom** (the highlight-query precedent), on a
throwaway parse gated by the CALLER on `Language::Rust`. _(the exact OnceLock/parse seam + the language helper
confirmed by the Explore sweep — see below.)_

**F7 — the reuse seams CONFIRMED present (inline greps; real locations):**
| seam | spec cited | REAL | verdict |
|---|---|---|---|
| ⌘⇧O global open-remote | keymap.rs:177 | `chord(true,false,false,true,"o")`→`"open-remote"` keymap.rs:**177-178**; TERM test keymap.rs:**637-638** | CONFIRMED — the shadow target (⌘O, no shift, :145, is a DIFFERENT chord). Editor-scoped ⌘⇧O free to shadow. |
| roster | 67/22 (stale) | now `chords.len() == 77` / scoped `== 32` (post-#303) | → target **78/33** (one Editor-scoped row). |
| `fuzzy_rank` | finder.rs:56 | `fuzzy_rank(candidates: &[&str], query: &str) -> Vec<Scored>` **marley_search_core/src/lib.rs:34**; empty query = identity (test :81) | CONFIRMED — map symbols→`&[&str]` names, rank, `Scored` carries the index back to the Symbol. |
| `cap_with_tail` | editor_symbols.rs:42 | `cap_with_tail(len, max) -> (usize, usize)` **editor_symbols.rs:42** | CONFIRMED. |
| `FinderState` | — | **finder.rs:10** | CONFIRMED — the modal state to embed. |
| `OpenSymbols` (#325) | app.rs:724 | `struct OpenSymbols` **app.rs:727**, field `open_symbols` **app.rs:235** | CONFIRMED — mirror as `OpenFileSymbols`. |
| `language_of` | marley_syntax | `language_of(path) -> Language` **crates/marley_app/src/code_syntax.rs:27** + `enum Language` :11 (APP-side, not marley_syntax — the caller gate is `code_syntax::language_of`) | DRIFTED home — the gate lives app-side. |

**F8 — NO existing `Symbol`/`SymbolKind`/`file_symbols`** (grep clean across `crates/*/src`). The new pure
`Symbol { name, kind, line, col }` + `SymbolKind` won't clash with #325's workspace-symbols (which use a
different LSP type).

**F9 — the extractor HOME + the exact idiom to mirror (confirmed inline).** The syntax crate is
**`crates/syntax/`** (= "marley_syntax"). `file_symbols` is the 4th node API in `crates/syntax/src/lib.rs`
beside `enclosing_ranges` (:461, #329), `matching_delimiters_in` (:522, #340), `all_headers` (:599, #330).
The query-compile idiom to mirror is `highlight_query() -> &'static Query` (parse.rs:17): a
`static QUERY: OnceLock<Query>` compiling `tree_sitter_rust::HIGHLIGHTS_QUERY` once — `file_symbols` adds a
twin `tags_query()` compiling `TAGS_QUERY`. The parser is `rust_parser()` (parse.rs:96,
`Parser::set_language(tree_sitter_rust::LANGUAGE)`); the query-run uses `QueryCursor` + `StreamingIterator`
(parse.rs:11, the ts-0.24 API). So the pure delta = one OnceLock + one `file_symbols(src) -> Vec<Symbol>` that
parses, runs the query, maps captures→Symbol (dedup the F2 double-match, sort by line/col). **The caller gates
on Rust app-side (`code_syntax::language_of`), the pure fn parses Rust unconditionally (F6/#340 M1).**

**F10 — the memo shape CONFIRMED.** The `(nonce, version)`-keyed cache is the house pattern: `#330` sticky
headers `(nonce, version, [(header_row, end_row)])` (app.rs:632), `#331` inlay hints `(nonce, version, …)`
(app.rs:297), `#329` ladder (app.rs:279), `#272` match memo (app.rs:449). The symbols cache mirrors it: a field
`file_symbols_memo: Option<(nonce, version, Vec<Symbol>)>` recomputed only on a buffer edit (D-MEMO-NONCE-VERSION),
NEVER per keystroke — the picker filters the cached Vec with `fuzzy_rank`.

**F11 — the adjacent ungated-session gap is REAL (filed, not fixed).** `selection_ladder.rs` (#329) and
`sticky_header.rs` (#330) contain NO `language_of`/`Language::Rust` gate (grep clean) — they build a Rust
`HighlightSession` unconditionally, so on a non-Rust file they parse it as Rust. Harmless-ish (a non-Rust file
yields few valid Rust nodes) but semantically wrong, and #304 must NOT copy it (D-CALLER-GATES-LANGUAGE — the
#340 M1 lesson). **Filed as forge #351** (bug, not this ticket's fix — #304 adopts the correct caller-gate pattern #351 back-fills onto #329/#330).

**F12 — no app-side cached tree CONFIRMED** (app.rs:3414: "no cached tree app-side; #349 is the [shared fix]").
Each syntax query pays a throwaway parse (the #330/#340 pattern); #304 pays the same memoized ~4ms/2000-line
cost — the spec's correction of the original ticket's "the tree is free" claim HOLDS. #349 is the shared future
fix, not #304's problem.

**Verdict:** the spec's core approach HOLDS (TAGS_QUERY confirmed, all seams present). The design must own THREE
extractor edges the spec glossed: F2 (the method/function double-match → dedup, prefer method), F3 (the qualifier
needs a parent-walk → recommend defer v1), F5 (`@definition.class` conflates 4 kinds → read node.kind() for a
distinct glyph, or one "type" glyph). An Explore agent independently swept the same claims for cross-check
(corroborated on return; the inline findings above are authoritative).

- **§20 + prior art:** CONFIRMED — the tree-sitter tags query is a PUBLISHED API + the grammar's OWN MIT
  `tags.scm` (adoption, outside the §20 wall; reading `~/.cargo/registry/.../tree-sitter-rust-0.24.2/queries/tags.scm`
  is the highest-yield prior-art leg, and it PAID: symbol extraction is a query RUN, not a hand-walk). The
  picker/jump plumbing is Marley-original over shipped seams (#325 FinderState, #312 NavStack).
- **AC / decisions:** all 8 REQ hold; the 6 locked decisions hold (D-TAGS-QUERY-IS-THE-EXTRACTOR confirmed by
  F1; D-CALLER-GATES-LANGUAGE reinforced by F11). No reopens.

**Cross-check refinements (the Explore agent corroborated F1-F12 exactly; three design-relevant adds):**
- **The EXACT gate site to mirror is #340's, NOT #329/#330's** — `refresh_bracket_match` gates on
  `code_syntax::language_of(&s.active_file().path) == Language::Rust` (app.rs:**3426**, drops the cache when
  `!is_rust` at :3436-3442). #304's symbol-refresh copies THAT, not the ungated #329 ladder (app.rs:3316) /
  #330 sticky (app.rs:3388) — which is exactly why #351 exists.
- **editor_symbols.rs already owns the symbol-picker GLUE namespace** (not a Symbol type): `SymbolQuery`
  (:17, a query stale-key), `symbol_kind_glyph(kind: u32)` (:28 — takes an LSP `SymbolKind` u32), `cap_with_tail`
  (:42), `MAX_SYMBOL_ROWS = 64` (:9). Design decision for P2: place the new file-local `Symbol`/`SymbolKind` +
  `OpenFileSymbols` here (reusing cap_with_tail/MAX_SYMBOL_ROWS) vs a new module; and whether the new
  `SymbolKind` maps to LSP u32 kinds to REUSE `symbol_kind_glyph`, or carries its own glyph. Name it distinctly
  from `marley_lsp::WorkspaceSymbolResult` (the #325 LSP row — `kind: u32`, NOT a Rust enum; the spec's "reuse
  the existing Symbol type" premise was WRONG, favorably — no clash).
- **`OpenSymbols`'s second field is `results: Vec<WorkspaceSymbolResult>`** (app.rs:727) — the mirror
  `OpenFileSymbols` gets a parallel `symbols: Vec<Symbol>` (a distinct local type, per above).

## Phase 2 — Design

### 1. Architecture / approach
Two halves: a PURE extractor in `marley_syntax` (the 4th node API, cov/MSI 100) + an app-side picker/jump
mirroring the #325 `OpenSymbols` shape. §20: the tags query is a PUBLISHED tree-sitter API run over the
grammar's OWN MIT `tags.scm` (adoption, outside the clean-room wall — the prior-art sweep's highest-yield leg;
no new dep, tree-sitter-rust already ships). VS Code/Zed ⌘⇧O = observed behavior.

**(a) THE PURE EXTRACTOR — NEW `crates/syntax/src/symbols.rs`** (a module, re-exported from lib.rs; the tags
concern is distinct from highlighting, so a cohesive module beats bloating lib.rs):
- `pub struct Symbol { pub name: String, pub kind: SymbolKind, pub line: usize, pub col: usize }` — `line`/`col`
  are **1-based CHAR** positions of the symbol's `@name` (feeding #302's `caret_for_line_col` directly; char-
  computed so a symbol after a multibyte glyph lands right — the #300/#336 discipline).
- `pub enum SymbolKind { Struct, Enum, Union, TypeAlias, Trait, Method, Function, Module, Macro }` — a clean,
  LSP-agnostic Rust enum (the pure crate must not know LSP numbers).
- `pub fn file_symbols(src: &str) -> Vec<Symbol>`: mirror `spans_from_tree` (parse.rs:46) — `rust_parser()`
  parse (throwaway; total via the `Some(tree)` guard → empty Vec on the dead-by-design fail), a `QueryCursor`
  over a NEW `tags_query()` OnceLock (parse.rs, twin of `highlight_query()` compiling `TAGS_QUERY`). Per match:
  scan `m.captures` for the capture whose name `starts_with("definition.")` (→ the kind + its node, for the F5
  split) and the one named `"name"` (→ the symbol text + position); a `@reference.*` match has no
  `definition.*` capture → SKIPPED. Map:
  | capture | node.kind() split (F5) | SymbolKind |
  |---|---|---|
  | `definition.class` | struct_item / enum_item / union_item / type_item | Struct / Enum / Union / TypeAlias |
  | `definition.method` | — | Method |
  | `definition.function` | — | Function |
  | `definition.interface` | — | Trait |
  | `definition.module` | — | Module |
  | `definition.macro` | — | Macro |
  Then: read `@name`'s text (`&src[name_node.byte_range()]`); compute (line, col) char-correct from the name's
  start byte (`src[..start].matches('\n').count()+1`; the intra-line char count after the last `\n`); **DEDUP
  the F2 method/function double-match** — a method's `function_item` matches BOTH the method pattern AND the
  free-fn pattern → two Symbols at the SAME (line, col) differing only in kind; sort by (line, col) then dedup
  adjacent equal-(line,col) PREFERRING Method (Method > Function). Final: document order (sorted by line, col).
  §14 total — empty src / parse-fail / no definitions → empty Vec, never a panic.

**(b) THE APP-SIDE (all app.rs shim + editor_symbols glue):**
- State `open_file_symbols: Option<OpenFileSymbols>` (near `open_symbols` app.rs:235) where
  `struct OpenFileSymbols { finder: FinderState, symbols: Vec<Symbol> }` (mirrors `OpenSymbols` app.rs:727,
  whose 2nd field is `results` — ours is `symbols`).
- **The memo + the LANGUAGE GATE (copy #340, NOT #329/#330):** a `file_symbols_memo: Option<(nonce, version,
  Vec<Symbol>)>` field; the open path gates on `code_syntax::language_of(&s.active_file().path) ==
  Language::Rust` (the `refresh_bracket_match` idiom, app.rs:3426) — **non-Rust → skip the parse, symbols
  empty**; else reuse the memo when `== Some((nonce, version))`, else `file_symbols(&text)` once and cache.
  The picker still OPENS on a non-Rust/empty file, showing a single "(no symbols)" row (REQ-005, the #340 M1
  gate — the pure fn can't self-gate).
- **The filter:** `fuzzy_rank(&names[..], &finder.query)` (names = `symbols.iter().map(|s| s.name.as_str())`),
  `Scored.index` → the Symbol; **empty query = document order** (fuzzy_rank's identity, its own test). Rows
  capped via `cap_with_tail(ranked.len(), MAX_SYMBOL_ROWS=64)`.
- **The render:** mirror the `open_symbols` overlay (app.rs:9146) — a framed modal; each row = a kind glyph +
  the name. Glyph: a NEW small `file_symbol_glyph(kind: SymbolKind) -> &'static str` in editor_symbols.rs
  (beside `symbol_kind_glyph`), reusing its vocabulary (ƒ fn/method, ◆ struct/union, ◇ enum/trait, ◈ module,
  a distinct one for macro/type-alias) — keeps the pure `SymbolKind` LSP-free while sharing the glyph look.
- **The jump (REQ-004):** on Enter, the chosen `Symbol` → capture the origin `(path, active_caret())` FIRST →
  close the picker → `caret_for_line_col(buffer, sym.line, Some(sym.col))` (#302's clamp) → `clear_marked()` +
  `set_single_caret(target)` → `scroll_editor_to_row(sym.line - 1)` (ScrollStrategy::Center) → **push
  `NavLoc{path, offset: origin_caret}`** (the 5-site origin-capture idiom, jump_to_sticky_header class; ⌃-
  returns). Close BEFORE the jump (it replaces the active editor — the #325 idiom app.rs:9238).
- **The chord + dispatch:** ⌘⇧O `chord(true,false,false,true,"o")` Editor-scoped → `"go-to-file-symbol"`,
  SHADOWING the global open-remote (keymap.rs:177; the #325 ⌘T precedent); dispatch_action opens the picker
  (gate+memo+extract). Off an editor tab, ⌘⇧O still resolves to open-remote (the shadow).
- **The 3 easy-to-forget wiring sites (the #339 lesson):** (i) the picker key-ladder arm HIGH in on_key_down
  (mirror `open_symbols` app.rs:9146-9262: escape/enter/up/down/backspace/char); (ii) the `text_input_blocked`
  line (app.rs:7680 — add `|| self.open_file_symbols.is_some()` so typed chars filter, not leak to the buffer);
  (iii) the render call in the overlay stack.

### 2. File manifest
| File | Change |
|---|---|
| `crates/syntax/src/symbols.rs` | NEW — `Symbol`, `SymbolKind`, `file_symbols(src)` + the pure truth-table tests |
| `crates/syntax/src/parse.rs` | NEW `tags_query() -> &'static Query` OnceLock (twin of `highlight_query()`) |
| `crates/syntax/src/lib.rs` | `mod symbols; pub use symbols::{file_symbols, Symbol, SymbolKind};` |
| `crates/marley_app/src/editor_symbols.rs` | NEW `file_symbol_glyph(kind: SymbolKind) -> &'static str` |
| `crates/marley_app/src/app.rs` | `OpenFileSymbols` state + `file_symbols_memo` + the gate/extract + the picker key-arm + the jump + the `"go-to-file-symbol"` dispatch + the render + the `text_input_blocked` line + `open_file_symbols_for_test` |
| `crates/marley_app/src/keymap.rs` | ⌘⇧O Editor-scoped row → "go-to-file-symbol"; roster 77→78, scoped 32→33; individual asserts (#337); a resolution unit (Editor→go-to-file-symbol, TERM→open-remote) |
| `crates/marley_app/src/headless_drive.rs` | the drives (open/filter/jump/gate/memo) |

### 3. Regression Test Plan (≥1 row per REQ)
| # | Test | Kind | Pins |
|---|---|---|---|
| REQ-001 | `file_symbols_extracts_all_kinds_in_document_order` | pure | a fixture with struct/enum/union/type/trait/mod/macro/free-fn/impl-method → the right kinds, document order |
| REQ-002 | `file_symbols_dedups_method_function_double_match` | pure | **THE F2 edge** — `impl Foo { fn bar() }` + a free `fn bar()`: exactly ONE Method(`bar`) + ONE Function(`bar`), NOT 2+2; the class-split (struct vs enum distinct kinds via node.kind() — F5) |
| REQ-001 | `file_symbols_col_is_char_not_byte` | pure | a symbol after a multibyte line → `col` is CHAR (the #336 separator) |
| REQ-003 | `open_file_symbols_filters_by_fuzzy_and_empty_is_document_order` | pure/headless | fuzzy_rank reuse (cite its own tests) + empty query = document order |
| REQ-004 | `go_to_file_symbol_jumps_centered_and_pushes_navstack_headless` | headless | Enter → caret at the symbol, nav depth +1 (⌃- returns) |
| REQ-005 | `file_symbols_non_rust_shows_no_symbols_no_walk_headless` | headless | a `.md`/non-Rust tab → picker opens, "(no symbols)", the gate skipped the parse (the #340 M1 row) |
| REQ-006 | `file_symbols_memo_reuses_on_second_open_headless` | headless | second open, no edit → the memo hit (no reparse; assert via a memo-probe hook or a stable identity) |
| REQ-007 | `cmd_shift_o_resolves_to_file_symbol_on_editor_open_remote_on_term` | keymap unit | the shadow |
| REQ-008 | `file_symbols_caps_with_tail` | pure | cap_with_tail(>64, 64) → (64, N-64) |
| §14 | `file_symbols_empty_and_garbage_no_panic` | pure | empty src, non-Rust text, a truncated fn → empty/partial, no panic |

Coverage/MSI 100 on symbols.rs + `file_symbol_glyph` (pure). The app.rs picker/jump/dispatch are shims
(`#[cfg_attr(test, mutants::skip)]`); the drives prove the wiring. **LIVE modal PIXEL deferred-not-skipped
(chad at machine)** — the symbols list, the filter, the jump caret + NavStack, the gate, the memo are ALL
headless STATE; only the framed-modal rendering is unverified-by-pixel (documented, a plausible mirror of the
shipped OpenSymbols overlay).

### 4. Risks / decisions
- **F2 the method/function double-match** (the correctness heart) — dedup by (line,col) preferring Method;
  the REQ-002 fixture (same-named method + free fn) is the pin. Inspect will re-check.
- **F5 the class-split** — read `def_node.kind()` to split struct/enum/union/type; a wrong split shows the
  wrong glyph (a test per kind).
- **The caller gate copies #340 (app.rs:3426), NOT #329/#330** (which are ungated — #351). The pure fn parses
  Rust unconditionally; the gate lives at the call site.
- **Symbol/SymbolKind live in marley_syntax** (the pure crate — they're the pure fn's output, like TokenKind),
  NOT editor_symbols.rs (app-side glue). The app maps SymbolKind→glyph.
- **The qualifier (Type::method) DEFERRED v1** (F3 — needs a parent-walk to the impl's `type:`; the kind glyph
  already disambiguates a method from a free fn). A follow-up if anyone asks.
- **`line`/`col` are CHAR-based** (feed caret_for_line_col) — computed char-correct in the pure fn; the
  multibyte fixture proves it.

## Phase 3 — Implement

Built to the manifest; `cargo check --workspace` clean, `cargo fmt` applied, `git add -N symbols.rs`.

**What was built:**
- **`crates/syntax/src/parse.rs`** — NEW `pub(crate) fn tags_query() -> &'static Query` (twin of `highlight_query`, compiling `tree_sitter_rust::TAGS_QUERY`).
- **`crates/syntax/src/symbols.rs`** (NEW, pure) — `SymbolKind` (9 variants), `Symbol { name, kind, line, col }` (line/col 1-based CHAR), `file_symbols(src)` (mirrors `spans_from_tree`: `rust_parser` parse → `QueryCursor` over `tags_query` → per match scan `m.captures` for the `definition.*` + `name` captures → `symbol_kind(def_name, node.kind())` splitting class via node.kind() → char-correct `(line, col)` → **sort by (line,col) + `dedup_by` preferring Method** for the F2 double-match). Private `symbol_kind` (the defensive `None` arms covered by a Phase-4 direct test) + `char_line_col`. Total (§14). No `mod tests` yet (Phase 4 owns the tables).
- **`crates/syntax/src/lib.rs`** — `mod symbols;` + `pub use symbols::{file_symbols, Symbol, SymbolKind};`.
- **`crates/marley_app/src/editor_symbols.rs`** — NEW `file_symbol_glyph(kind: SymbolKind) -> &'static str` — EXHAUSTIVE 9-variant match (ƒ fn/method, ◆ struct/union/type, ◇ enum/trait, ◈ module, ! macro; no `_` arm).
- **`crates/marley_app/src/app.rs`** — `OpenFileSymbols { finder, symbols }` (mirrors OpenSymbols); fields `open_file_symbols` + `file_symbols_memo: Option<(u64, BufferVersion, Vec<Symbol>)>` (the StickyHeaderCache shape) + inits; the `"go-to-file-symbol"` dispatch arm; `open_go_to_file_symbol` (the gate copies #340's `language_of == Rust` + the (nonce,version) memo; non-Rust → empty), `file_symbol_rows` (reuses `finder.results(&names)` — the finder ALREADY wraps fuzzy_rank, so no direct call), `handle_file_symbols_key` (mirrors `handle_symbols_key` — escape/enter/up/down/backspace/char, but SYNCHRONOUS: no park/consume, filter is local), `jump_to_file_symbol` (mirrors #302 goto_commit: origin capture → NavStack push → caret_for_line_col + clear_marked + set_single_caret → scroll Center); the `text_input_blocked` line; the on_key_down key-arm; `file_symbols_overlay` render + its render-stack call. All shim methods `#[cfg_attr(test, mutants::skip)]`.
- **`crates/marley_app/src/keymap.rs`** — ⌘⇧O `chord(true,false,false,true,"o")` Editor-scoped → "go-to-file-symbol" (shadows global open-remote); roster **77→78 / scoped 32→33** + a `#304 ×1` breakdown; individual `.contains` asserts (both rosters) before the counts (#337); NEW resolution test `cmd_shift_o_is_file_symbol_on_editor_open_remote_elsewhere`.

**Deviations from design (with reason):**
- **`file_symbol_rows` reuses `FinderState::results(&[&str]) -> Vec<usize>`** instead of calling `fuzzy_rank` directly — the finder ALREADY wraps fuzzy_rank (the ⌘P idiom, finder.rs:56). A cleaner reuse than the design's "map names→fuzzy_rank→Scored.index"; same result (empty query = document order). A WIN to record (the substrate already does it — the #339-class).
- **The picker is SYNCHRONOUS** (no `park_symbol_query`/`consume`/`apply` — those are #325's LSP-async fan-out). The symbols are in hand from `file_symbols`, so `handle_file_symbols_key` just edits the finder + the render re-filters. Smaller than the #325 mirror by design.

**Compile/test as-you-go:** `cargo check --workspace` clean; `cargo nextest -p marley --lib -E 'test(/chord|roster|keymap|cmd_shift_o/)'` → 22 passed (incl. the new resolution test); `cargo nextest -p marley_syntax` → 42 passed. NO test expansion beyond compile + the roster/keymap asserts — Phase 4 owns symbols.rs's tables + the drives + `symbol_kind`'s defensive-arm coverage.

## Phase 3.5 — Inspect

My own independent verification (a throwaway integration test over the extractor edges, since removed;
rustdoc `-D warnings` on both crates; `mutants --list`; clippy; a self-review read of the gate/memo/jump/
leak/enter-consistency) — authoritative below. Two parallel general-purpose critics also swept the same diff
for cross-check (their reports append when they return; any real finding is fixed then). **Result: NO code
defect; ONE documentation-worthy scope limit (bodyless trait sigs) + one Phase-4 coverage owe (symbol_kind
None arms). PR-claude-adopted-query-inherits-its-scope-gaps-document-them-001 recorded.**

**Lenses covered:** the F2 method/function dedup; the F5 class-split; char-correct positions; document order +
@reference-dropped; §14 totality; the defensive `symbol_kind` None arms; the language gate (copy #340); the
(nonce,version) memo; the leak gate; the jump (origin-push + clear_marked + close-before); enter-vs-render
consistency; roster honesty; the mutation surface; clean-room + doc-links.

| # | Finding | Severity | Verdict | Resolution |
|---|---|---|---|---|
| E-1 | file_symbols edges (my throwaway) | — | **ALL PASSED** | F2 (`impl S { fn bar }` + free `fn bar` → exactly ONE Method + ONE Function, the impl one is Method; same-named at different lines both kept); F5 (struct/enum/union/type → the 4 distinct kinds, none dropped); positions char-correct (a `struct After` under a `//😀 é` line → col 8 CHAR, not byte; `fn first` → (1,4)); document order + a `foo()` call + an `impl` block DROPPED (@reference.*); `mod`→Module, `macro_rules!`→Macro; empty/garbage/truncated → no panic. |
| F-DOC | **Bodyless trait method signatures are NOT extracted** | **LOW (doc, not a code defect)** | **REAL — a tags.scm scope limit** | `trait T { fn m(&self); }` → `m` is a `function_signature_item`, which tags.scm's method pattern (matching `function_item`) does NOT capture; a trait DEFAULT method (`fn m() {}`) IS captured (Method). file_symbols faithfully runs tags.scm — not a bug. **Documented as a v1 scope limit** (the sticky-header `all_headers` already notes the same "a body-less `function_signature_item` is NOT a header"). VS Code's ⌘⇧O shows them via LSP documentSymbol (a richer source, out of scope v1). Recorded in the ledger + the CHANGELOG/editor.md at Phase 5; a follow-up if anyone asks. |
| C-COV | `symbol_kind`'s defensive `_ => None` arms are dead-by-design | (Phase 4 owe) | **NOTED** | The pinned tags query never emits an unknown `definition.*` / an unknown class node kind, so the None arms are only reachable via a DIRECT unit test (`symbol_kind("definition.class","bogus")`→None, `symbol_kind("definition.zzz","x")`→None) — Phase 4 MUST add it or the 100% coverage floor fails (the #300 dead-branch class). Flagged for Phase 4. |

**Wiring (self-verified by reading — mirrors the shipped patterns):** the gate copies #340 (`language_of==Rust`
at the caller, non-Rust→empty→no walk, NOT the ungated #329/#330); the memo reuses on `== Some((nonce,version))`
+ stores on miss, nonce/version from the same active buffer; the leak gate (`text_input_blocked` +=
`open_file_symbols.is_some()`) + the key-arm before the router; the jump captures the ORIGIN caret, pushes it
(not the target), `clear_marked` before `set_single_caret`, closes the picker BEFORE the jump; enter + render
both drive off the SAME `file_symbol_rows()` + `cap_with_tail(rows.len(), MAX)` clamp (the #325 render-vs-nav
consistency).

**Mutation surface (verified):** symbols.rs = 29 mutants (the pure target, `git add -N`'d — `A` in git status,
not skipped); editor_symbols `file_symbol_glyph` = 2; app.rs shims = 0 (mutants::skip). **Hygiene:** rustdoc
`-D warnings` CLEAN on both marley_syntax + marley (no #303-style private-module link); no zed/warp in added
lines; clippy clean.

**Critic reconciliation (both returned — CORROBORATED my verification + 2 actionable adds, both fixed):**
- **Critic 1 (correctness): all 6 checks CONFIRMED** (F2 dedup, F5 split, char positions, doc order, ref-drop,
  totality — matching my throwaway exactly, incl. nested `mod x { impl S { fn deep }}`→Method, order-robust
  promotion). **Its MEDIUM sharpened F-DOC → a DOC-ACCURACY fix:** the `SymbolKind::Method` doc "a fn inside an
  impl/trait body" OVERSTATED coverage — bodyless trait sigs AND `extern` fns are `function_signature_item`,
  not captured. **FIXED** — the `Method` doc + `file_symbols` doc now state the bodyless/extern limit
  (PR-claude-adopted-query-inherits-its-scope-gaps-document-them-001). Its LOW (symbol_kind None arms) = my
  C-COV, now covered by the Phase-4 direct `symbol_kind_unknown_captures_are_none` test.
- **Critic 2 (wiring): all 7 gates CONFIRMED** (gate copies #340 not #329/#330; memo per-buffer nonce no
  cross-file staleness; leak gate both halves — text_input_blocked early-returns the IME path + the key-arm;
  jump pushes origin + clear_marked + close-before; enter==render via the same file_symbol_rows+cap; roster
  78/33 honest; clean-room clean). **Two MEDIUMs, both resolved:** (a) `file_symbol_glyph` had zero tests → 2
  live mutants — **FIXED** (the Phase-4 `file_symbol_glyph_all_variants` per-variant test kills both); (b) **the
  `git add -N` EMPTY-BLOB hazard** — `git add -N` stages symbols.rs as git's empty blob (`e69de29…`) with the
  CONTENT unstaged, so a BARE `git commit` would land it empty → a broken build the worktree-reading gates
  never see. NOT a code defect; a commit-time trap handled by /commit's explicit `git add crates/syntax/src/symbols.rs`
  (my #300/#302/#303 new-file staging already does this) — but pinned: PR-claude-git-add-N-stages-empty-blob-content-must-be-re-added-at-commit-001,
  and /commit must verify `git show :crates/syntax/src/symbols.rs | wc -l` ≈ 292 (not 0) before committing.
  Its LOW (throwaway debug noise) was STALE — it saw Critic 1's mid-flight `zzcritic_*` throwaway (since
  deleted); my real tables have zero `println!`/`dump()` (grep-confirmed).

## Phase 4 — Validate

**Tests added (14):**
- **symbols.rs `mod tests` (10)** — the pure truth tables (the 30-mutant surface): all-kinds document order + @reference-dropped; the F2 method/function dedup (impl method + free fn, same-name-different-line, trait default vs bodyless-sig); the class-split; char-correct positions (multibyte); totality (empty/garbage/truncated); the C-COV direct `symbol_kind` None-arm test; the `dedup_double_matches` BOTH-orderings test (covers the promotion line whichever capture sorts first); `char_line_col` arithmetic.
- **editor_symbols.rs (1)** — `file_symbol_glyph_all_variants` (all 9 SymbolKind → glyphs; kills the 2 mutants, per-variant).
- **headless_drive.rs (4)** — REQ-004 jump+NavStack-push+close (caret at 17); REQ-005 the language gate (non-Rust → 0 symbols, no walk); REQ-006 the memo (populated + stable reopen); the leak (typed char stays out of the buffer).

**REQ coverage:** REQ-001..008 all pinned (REQ-007 the chord = the Phase-3 keymap unit). LIVE modal PIXEL
deferred-not-skipped (chad at machine) — the list/filter/jump/gate/memo STATE is headless-proven.

**Two gate reds fixed at SOURCE (§0):**
- **gate:4 coverage** — symbols.rs had 2 dead-by-design lines: the parse-`None` guard (`return Vec::new()` —
  tree-sitter always returns `Some`) and the order-dependent `b.kind = Method` dedup promotion (hit only when
  Method sorts second). **Fixed:** iterate the parse via `.into_iter()` (a plain `option::IntoIter` — the body
  runs on the one `Some`, the loop always ends through `None`, so NO dead region AND no `for_loops_over_fallibles`
  lint), and EXTRACTED `dedup_double_matches` as a helper with a both-orderings direct test. symbols.rs → 100%
  lines. (The #300 dead-branch lesson, twice; the `.into_iter()`-over-Option idiom is the clippy-clean way to
  cover an always-`Some` parse without moving code into the coverage-excluded parse.rs.)
- **gate:5 mutation — exit 4 "baseline failed"** was the #334 load-flaky baseline (gate:3 nextest was green, so
  the unmutated baseline was sound); it cleared on the clean re-run (no retry-until-green — the diagnosis was
  "nextest green ⇒ baseline sound ⇒ transient flake under the earlier parallel-critic load").

**Runs (actual):**
- `cargo nextest -p marley_syntax -E 'test(/symbols::/)'` → 10 passed; editor_symbols glyph → 1; the 4 drives → 4 passed.
- Focused `cargo llvm-cov -p marley_syntax` confirmed symbols.rs **100% lines** before the full gate.
- **Full `scripts/gates.sh --diff` → GATE GREEN [diff]**, 15/15: coverage **100% lines**; mutation **30 caught / 0
  missed → MSI 100.0%**. Receipt `de0da364…` == live `gate_state_hash` (commit-valid).
- No #348 hang; no pre-existing failures. (An unrelated aside: reclaimed ~5G of regenerable build artifacts —
  `target/llvm-cov-target` + `mutants.out` — at chad's request mid-phase; the gate regenerated llvm-cov-target
  cleanly.)

## Phase 5 — Complete
- Docs updated; AAR capture (lessons / failures / prevention rules / ADs); archive.
