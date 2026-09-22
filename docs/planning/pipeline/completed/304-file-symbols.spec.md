---
pipeline_id: 4687b05f-744c-468a-8a86-ff6e51c0e3cc
ticket: forge#304 (4b10e6e2-0a24-4835-bcaf-a8be11a432ab) · local docs/planning/tickets/open/TICKET-304-file-symbols.md
aar_id: 044a3850-e2e3-47f8-92c2-2c47780d1574
status: Phase 5 — Complete PASS
title: Go to Symbol in File (⌘⇧O) — the tags-query picker over the current file
type: feature
milestone: M19
references: [tree_sitter_rust::TAGS_QUERY (registry-verified — the crate SHIPS tags.scm), the #325 OpenSymbols/FinderState picker (app.rs:724/:8941), marley_search_core::fuzzy_rank (finder.rs:56 — the ⌘P local-rank idiom), the NavStack 5-site push idiom (editor_nav.rs:41), the (nonce,version) memo (app.rs:3296), PR-claude-language-specific-pure-primitive-needs-caller-side-gate-001 (#340 M1), #349 cached-tree]
---

## Title
⌘⇧O opens a fuzzy picker of THIS file's symbols — fns, structs, enums, traits, impls, mods, macros — and
Enter jumps to the chosen one, centered, with ⌃- returning. The first time the tree answers a MEANING
question in-file (sticky headers use ranges; this needs names). Rust-only v1 (the shipped grammar reality;
#315 extends per grammar with the language axis).

**A claim the original ticket makes that is FALSE, corrected here:** "the LIVE tree… so the tree is free."
There is NO cached tree app-side — the #340/#330 callers pay a throwaway parse per (nonce, version) miss
(~4 ms/2000 lines, measured at #340). This ticket pays the same memoized cost; **#349** collapses it later.

## Scope
### In
- **The FOURTH `marley_syntax` node API (pure, cov/MSI 100):** `file_symbols(src) -> Vec<Symbol { name:
  String, kind: SymbolKind, line: usize, col: usize }>` — **a thin adapter over the grammar's OWN tags
  query.** tree-sitter-rust 0.24.2 SHIPS `TAGS_QUERY` (tags.scm, registry-verified): `@definition.class`
  (struct/enum/union/type), `@definition.method` (fn inside a declaration_list), `@definition.function`,
  `@definition.interface` (trait), `@definition.module` (mod), `@definition.macro` — each with an inner
  `@name` capture. Compile it once (the parse.rs `OnceLock` idiom the highlight query uses), run over a
  fresh parse, map `@definition.*` → `SymbolKind`, read `@name`'s text. `@reference.*` captures are DROPPED
  (call hierarchy is not this ticket). Document order; a parse failure or empty file → empty, never a panic.
- **Qualifier best-effort:** a method row reads `Type::name` when the enclosing impl/trait's type identifier
  is cheaply reachable from the tags match (design confirms the capture layout); otherwise the kind glyph
  disambiguates. Never block the ticket on perfect qualifiers.
- **The picker = the #325 shape, ranked LOCALLY:** an `OpenFileSymbols { finder: FinderState, symbols:
  Vec<Symbol> }` modal mirroring `OpenSymbols` (app.rs:724) — but where #325 re-queries the LSP server per
  keystroke, this list is already in hand, so filtering is `marley_search_core::fuzzy_rank` over the names
  (the ⌘P idiom, finder.rs:56; empty query = document order — the "outline glance"). Rows capped via the
  shipped `cap_with_tail` (editor_symbols.rs:42; 64 like #325). Row: kind glyph + name (+ qualifier).
- **The jump:** Enter → caret to (line, col) + `scroll_editor_to_row` (Center) + **PUSH the NavStack** —
  the uniform origin-capture idiom all 5 production jump sites use (capture `(path, active_caret())`
  before, push on a landed jump; same-file precedent: jump_to_sticky_header). ⌃- returns.
- **The chord: ⌘⇧O, Editor-scoped, SHADOWING the global `open-remote` (#84)** — verified: the global row is
  keymap.rs:177 with a TERM-context test at :577; the shadow is exactly #325's ⌘T-over-new-tab precedent.
  Off an editor tab, ⌘⇧O still opens a remote. Roster + scoped counts grow by 1 (assert the chord
  individually FIRST — the #337 discipline).
- **The memo + the language gate:** symbols cached per `(nonce, version)` (the refresh_sticky_headers
  shape, app.rs:3296); the CALLER gates on `language_of(path) == Language::Rust` — the pure fn parses Rust
  unconditionally and cannot self-gate (#340 M1; PR-claude-language-specific-pure-primitive-...-001).
  Non-Rust / no symbols → the picker opens with a "(no symbols)" row; no error, no walk.
### Out (explicitly)
- Non-Rust queries (#315 threads the language axis; a language without a tags query → empty by contract).
- A persistent outline PANEL (this is a picker); breadcrumbs; `@reference.*` / call hierarchy; workspace-
  wide symbols (shipped, #325); symbol RENAME (shipped, #322); nested-depth indentation in rows (v1 flat).

## Reference (§20)
VS Code / Zed = OBSERVED behavior (⌘⇧O; empty query lists document order; type-to-filter; Enter jumps
centered). tree-sitter tags = PUBLISHED API + the grammar's own MIT query file, adopted. The picker/jump
plumbing is Marley-original over shipped seams (#325 picker shape, #312 NavStack).

### Prior art
1. **Behavior maps / observed** — the ⌘⇧O flow above; VS Code shows "No symbols" for an empty file rather
   than erroring (adopted).
2. **Published material** — tree-sitter's tags documentation (tags.scm is the standard code-navigation
   query form; `@definition.*`/`@name` is its documented capture convention).
3. **OUR PERMISSIVE DEPS — the sweep PAID again:** tree-sitter-rust **ships `TAGS_QUERY`**
   (bindings/rust/lib.rs:49 + queries/tags.scm, read in the registry) — so symbol extraction is a QUERY
   RUN, not a hand-rolled walk; do NOT extend `is_header_kind` (that 4-kind table stays for sticky
   headers, which need ranges not names). `marley_search_core::fuzzy_rank` owns ranking; `FinderState`
   owns the modal; `cap_with_tail` owns honest truncation. The genuinely new code is the capture→Symbol
   mapping + the wiring.

## Locked-In Decisions (design confirms; deltas → the notes)
- **D-TAGS-QUERY-IS-THE-EXTRACTOR** — adapt the crate's tags.scm; never hand-walk node kinds for names.
- **D-RUST-ONLY-V1 + D-CALLER-GATES-LANGUAGE** — the #340 M1 lesson, applied at authoring time not found
  at inspect. (Adjacent pre-existing gap, NOT this ticket: selection-ladder + sticky-headers build a Rust
  session ungated — file as its own bug at promotion.)
- **D-LOCAL-FUZZY** — fuzzy_rank client-side (the list is in hand); empty query = document order.
- **D-PUSH-NAVSTACK** — a real navigation (jumps rows); the 5-site origin-capture idiom.
- **D-SHADOW-CMD-SHIFT-O** — Editor-scoped shadow of global open-remote (the #325 ⌘T precedent).
- **D-MEMO-NONCE-VERSION** — throwaway parse per miss; #349 is the shared fix, not this ticket's problem.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | extract fns/structs/enums/traits/impl methods/mods/macros from a Rust fixture, names + kinds, document order | pure fixture |
| REQ-002 | distinguish a method (inside an impl/trait) from a free fn by kind (+ qualifier when cheaply available) | pure fixture |
| REQ-003 | rank by fuzzy match when a query is typed; list document order on an empty query | pure (fuzzy_rank reuse) |
| REQ-004 | jump to the chosen symbol on Enter, centered, PUSHING the NavStack (⌃- returns to the origin) | headless |
| REQ-005 | show "(no symbols)" and do NO walk on a non-Rust file (the language gate) or an empty result | headless — the #340 M1 row |
| REQ-006 | reuse the (nonce, version) memo — a second open with no edit reparses nothing | headless second-call |
| REQ-007 | resolve ⌘⇧O to the symbol picker ON an editor tab and to open-remote OFF it (the shadow) | keymap unit |
| REQ-008 | cap rendered rows with an honest "+K more" tail (cap_with_tail) | pure |

## Phase Plan
P2 confirm the tags-query capture layout on a real parse (a spike like #340's — does `@name` bind per
definition? how does the qualifier fall out?) + the picker-state shape + the chord shadow; P3 the
marley_syntax adapter first (fixture truth table), then picker + jump + chord; P3.5 critics on the capture
mapping edges (a macro_definition, a nested mod, an impl-for-generic), the language gate, the NavStack
origin capture; P4 tables + headless drives + gate; P5 docs (crate-map's 4th node API, editor.md).
Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
