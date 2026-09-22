# Expand/shrink selection — tree-sitter node ancestry + marley_syntax's first node-range API — Notes

- **Forge ticket:** #329 (78e4ae26-a46d-4fdf-815e-229d738faede)
- **AAR:** 4cb6a9a2-12a7-4dd8-8259-c5446654dd4f
- **Local ticket doc:** docs/planning/tickets/open/TICKET-329-expand-selection.md
- **Pipeline spec:** 329-expand-selection.spec.md

## Phase 1 — Plan
- **Request:** Grow/shrink the selection by AST ancestry (identifier→call→statement→block→fn), one chord each.
  The heart is a pure tree-sitter `TreeCursor` ancestry walk — but it doubles as the FOUNDATION: marley_syntax
  exposes highlight spans only; the `tree_sitter::Tree` is private to `HighlightSession`. This ships
  `enclosing_ranges`, the first node-range API #305 fold + #330 sticky header reuse.
- **Classification / tier:** work pipeline, single shippable slice — a pure library API + a pure app-side
  ladder state + a thin worker-protocol/keymap shim. FOUNDATION (two downstream consumers named).
- **Forge recall (§18.3):** forge wired. `knowledge-search` surfaced the relevant prevention-rule + AD classes
  (the #313 park/consume deferral, the D2 defer-don't-duplicate stance, the live-identity selection family
  from #327) — all already encoded in the decisions. No blocking bulletins pulled.
- **Discovery (the precise surface for Design):**
  - `crates/syntax/src/lib.rs:309` — `HighlightSession { parser, tree: Option<Tree>, last_src, last_spans }`.
    The `tree` is PRIVATE → `enclosing_ranges` must live in marley_syntax (same module access). `highlight_full`
    (:338) seeds `self.tree`; `enclosing_ranges` reads it (nodes carry byte offsets — no text needed).
  - `crates/marley_app/src/app.rs:9869` — `if line_count > SYNTAX_SYNC_MAX_LINES` forks: LARGE → the off-thread
    worker (`ensure_syntax_worker_and_send`, :9939) that owns a `HighlightSession` and answers `SyntaxReq` →
    `SyntaxResp{generation, nonce, version, lines}` (:9903/:9964); SMALL → the sync `marley_syntax::
    highlight_lines(&text)` (:9921) which DISCARDS its tree. **Crux the ticket glossed:** small files retain NO
    tree/session — so the ladder's tree source differs by tier (→ D-ROUTE hybrid).
  - `crates/marley_app/src/app.rs:402` — `syntax_worker: Option<(Sender<SyntaxReq>, Receiver<SyntaxResp>)>`;
    the pump drains `resp_rx`. A ladder query rides the same channel (a `SyntaxReq` variant/flag + a `SyntaxResp`
    ladder field), consumed one tick later — the #313 park precedent.
  - `crates/marley_app/src/keymap.rs` — `KeyContext::Editor` scoping (:73/:77); `chords_unique_scoped` forbids
    a `(chord, context)` dup. Audit: `cmd-shift-e` TAKEN (#68 Fleet, :516); `ctrl-w`/`ctrl-shift-w` UNBOUND →
    the free ⌃W/⌃⇧W candidate. `⌥↑/↓` + `⌥⇧↑/↓` belong to #300 (move/duplicate line).
- **Decisions:** D-ROUTE hybrid (large=worker+park, small=sync throwaway session — Route B persistent-second-
  session REJECTED); D-LADDER compute-once/navigate-pos, shrink-to-anchor; D-INVALIDATE version+caret identity;
  D-BYTES char↔byte at the app boundary (emoji fixture mandatory); D-CHORD ⌃W/⌃⇧W Editor-scoped (design
  free-checks); D-MULTICURSOR primary-grows-others-collapse (v1). §20: tree-sitter published-API reuse (MIT);
  Zed 04 §6.8 behavior reference only, source unread.
- **The open design question for P2:** the exact `SyntaxReq`/`SyntaxResp` extension shape (a new enum variant
  vs an optional `ladder_query: Option<(usize, usize)>` field + `ladder: Option<Vec<Range>>` reply), and
  whether the small-file path builds a throwaway session inline or reuses a cached one. Both stated; designer
  picks.

## Phase 2 — Design

### §20 confirm
tree-sitter **0.26.11** (`crates/syntax/Cargo.toml:12`) — PUBLISHED-API REUSE (MIT). `enclosing_ranges` uses
only the public `Node` API (`named_descendant_for_byte_range`, `.parent()`, `.is_named()`, `.byte_range()`) —
no copyleft source. Zed 04 §6.8 (`TreeCursor` ancestry, "cheapest high-value feature once a tree exists") is a
BEHAVIOR/shape reference, source UNREAD. Clean-room §20 holds. Confirmed no existing ancestry usage anywhere
in `crates/` (grep) — this genuinely IS marley_syntax's first node-range API.

### (1) The pure foundation — `enclosing_ranges` (marley_syntax, `crates/syntax/src/lib.rs`)
```rust
/// The ancestry ladder of NAMED nodes covering `byte_range`, innermost→root, byte ranges, deduped.
/// Empty when the session holds no tree (a fresh session) or the range is unreachable. #329 (M21).
pub fn enclosing_ranges(
    session: &HighlightSession,
    byte_range: std::ops::Range<usize>,
) -> Vec<std::ops::Range<usize>> {
    let Some(tree) = session.tree.as_ref() else { return Vec::new(); };   // reads the PRIVATE field
    let Some(mut node) = tree
        .root_node()
        .named_descendant_for_byte_range(byte_range.start, byte_range.end)
    else { return Vec::new(); };
    let mut ladder: Vec<std::ops::Range<usize>> = Vec::new();
    loop {
        if node.is_named() {
            let r = node.byte_range();
            if ladder.last() != Some(&r) {           // dedupe identical parent==child span
                ladder.push(r);
            }
        }
        match node.parent() { Some(p) => node = p, None => break }
    }
    ladder
}
```
- **Predicate — "covers" is INCLUSIVE**: `named_descendant_for_byte_range(s, e)` returns the smallest named
  node with `start_byte ≤ s && e ≤ end_byte` (tree-sitter's own contract). Every climbed parent covers it
  transitively, so all returned ranges cover `byte_range`. Innermost node may EQUAL `byte_range` (an exact-span
  selection) — kept here (the pure primitive returns all enclosing ranges; the app's ladder drops the
  equal-to-anchor rung so grow always visibly expands — see (3)).
- **Dedupe** compares to `ladder.last()` (the nearest already-pushed descendant) — tree-sitter nests
  identical-span named nodes (e.g. `expression_statement` == its `call_expression` child), so a naive walk
  double-lists; the dedupe collapses them. `is_named()` filter drops anonymous token parents (punctuation/
  keywords) so the ladder is expansion-useful nodes only.
- **`session.tree` is a private field** (`lib.rs:311`) → `enclosing_ranges` MUST live in `lib.rs` (same-module
  access). It's PURE (no FFI beyond the already-parsed tree's accessor calls) → cov/MSI 100 over fixture
  parses (build a `HighlightSession`, `highlight_full` a real Rust snippet, assert the exact ladder). `parse.rs`
  stays the coverage-shim exclude; this fn does NOT go there.
- Edge cases pinned by tests: no-tree→`[]`; caret (`start==end`)→smallest node at point (REQ-004); range at
  root→ladder tops at `source_file` (REQ-003); an emoji-bearing snippet→byte offsets stay valid (the walk is
  byte-native; char mapping is the app's job).

### (3) `SelectionLadder` (PURE, new `crates/marley_app/src/selection_ladder.rs`, char offsets)
```rust
pub struct SelectionLadder {
    anchor: std::ops::Range<usize>,        // the ORIGINAL selection (char offsets) — shrink's ground
    rungs: Vec<std::ops::Range<usize>>,    // enclosing char ranges STRICTLY containing anchor, innermost→root
    pos: usize,                            // 0 = anchor; 1..=rungs.len() = rungs[pos-1]
}
```
- **Build** (`new(anchor, enclosing_char_ranges)`): `rungs = enclosing.into_iter().filter(|r| *r != anchor).
  collect()` — dropping the rung equal to the selection so the FIRST grow always expands (JetBrains/Zed
  behavior); `pos = 0`.
- **`grow` → `pos = min(pos+1, rungs.len())`**, returns `current()`. First grow: 0→1 = `rungs[0]` (smallest
  strictly-enclosing node). Clamps at the top (root).
- **`shrink` → `pos = pos.saturating_sub(1)`**, returns `current()`. `pos==0` → `anchor` (EXACTLY the
  original, not the smallest node — REQ-007).
- **`current(&self) -> Range`**: `pos==0 ? anchor : rungs[pos-1]`.
- Pure + total; cov/MSI 100 by a direct grow/shrink/clamp/empty-rungs table. Invalidation lives in the app
  shim (version + caret-identity), NOT in the struct.

### Multi-cursor (REQ-009)
The ladder is built from + applied to the PRIMARY selection; on apply, the app collapses the SelectionSet to
the single primary range (the #307 Tab "primary acts, others collapse" v1 precedent). N-ladder deferred.

### (2) The tree-source delivery — D-ROUTE CONFIRMED with a v1 refinement
The Explore pass established the exact seams (verbatim signatures below). It surfaced the decisive fact: the
`tree_sitter::Tree` NEVER crosses the worker→app boundary — the app cache carries only per-line highlight
spans (`SyntaxLines` at app.rs:556). So there is no live tree to query app-side, for EITHER size tier, without
either (a) extending the worker protocol to ship a ladder, or (b) building a fresh session on demand.

**REFINEMENT (designer's call): v1 computes the ladder SYNCHRONOUSLY via a throwaway `HighlightSession` over
the current buffer text, for ALL file sizes.** The spec's D-ROUTE leaned hybrid (worker round-trip for large
files). Generalizing the small-file sync arm to all sizes is chosen for v1 because:
1. **Minimal, always-correct surface** — no `SyntaxReq`/`SyntaxResp` extension, no park/consume, no
   stale-`(nonce,version)`-drop no-op on the first press; the parse is against the exact current text.
2. **Same cost class for the common case** — a ≤`SYNTAX_SYNC_MAX_LINES`(1000) file already parses
   synchronously every keystroke (`highlight_lines`); a one-shot parse on a deliberate ⌃W gesture is
   imperceptible. The ladder is computed ONCE per gesture (first grow) and cached, so subsequent grow/shrink
   never re-parse.
3. **Consistent with the spec's rejection of Route B** — the throwaway session is DISCARDED after the parse
   (not a persistent second session), so no two-tree drift (#313 D2 holds).
- **Named follow-up (deferred):** for VERY large files (>1000 lines) the first grow does a one-time O(file)
  parse (~ms/10ms scale) — the worker-async round-trip (compute the ladder on the worker's live session via the
  `(nonce,version)`-guarded channel, the closer-than-LSP pattern the Explore pass flagged) is the optimization
  if that first-press hitch is ever felt. Reversible: the pure `enclosing_ranges` API is identical either way.

### Verbatim seams (from the Explore pass)
- **tree-sitter 0.26.11 / tree-sitter-rust 0.24.2.** `Node::{named_descendant_for_byte_range(s,e) -> Option<Node>,
  parent() -> Option<Node>, is_named() -> bool, byte_range() -> Range<usize>}` — all NEW (unused today),
  available in 0.26. Node extent is read via `.byte_range()` (the existing idiom; `start_byte`/`end_byte` are
  not used as Node methods here). `session.tree: Option<Tree>` private at `lib.rs:311`.
- **Buffer byte↔char** (`crates/editor/src/buffer.rs`): `char_to_byte(CharOffset) -> ByteOffset` (:105),
  `byte_to_char(ByteOffset) -> CharOffset` (:110) — exact random-access, NOT via line_col. `CharOffset`/
  `ByteOffset` newtypes (`marley_text_offsets`, `.as_usize()`, `From<usize>`). `buffer.text() -> String`,
  `buffer.version() -> BufferVersion`.
- **Selection/SelectionSet** (`crates/editor/src/selection.rs`, CHAR offsets): `Selection::new(anchor, head)`
  (:32), `caret(off)` (:41), `.start()`/`.end()`/`.head()` (min/max), `.is_caret()`. `SelectionSet::single(
  Selection)` (:113), `.primary() -> Selection` (member 0), `.selections()`.
- **Apply** (`crates/marley_app/src/editor_surface.rs`): `set_active_selections(SelectionSet)` (:235, replaces
  the whole set → collapses multi-cursor), `active_selections() -> &SelectionSet` (:230), `active_caret()`
  (:225), `active_nonce()` (:267), `active_buffer()` (:216)/`_mut` (:211). Precedent: #272 select-all
  (app.rs:5417) builds `SelectionSet::single(Selection::new(0, len))`.
- **Dispatch** (`crates/marley_app/src/{keymap.rs, app.rs}`): a keymap row is `(KeyBinding::chord(cmd,ctrl,alt,
  shift,"key"), "action".to_string(), Some(KeyContext::Editor))` in `default_bindings()` (keymap.rs:247+);
  resolved by `action_for(binding, stack)` (:404); dispatched via `dispatch_action(&str)` (app.rs:5327, big
  match). **Roster guards: `assert_eq!(chords.len(), 60)` (keymap.rs:1004) and the scoped-count `== 19`
  (keymap.rs:1071) — TWO new rows bump them to 62 / 21.** ⌃W/⌃⇧W audited FREE.

### The app shim (grow_selection / shrink_selection — app.rs, coverage-excluded)
```
read primary = active_selections().primary(); (cs, ce) = (primary.start(), primary.end()) as usize (char)
nonce = active_nonce(); version = active_buffer().version()
valid = self.selection_ladder.is_some()
     && self.selection_ladder_at == Some((nonce, version))
     && self.selection_ladder.current() == (cs..ce)          // live-identity: caret moved ⇒ stale
if !valid {                                                    // REBUILD (the only re-parse site)
    text = active_buffer().text()
    (bs, be) = (char_to_byte(cs), char_to_byte(ce)) as usize
    let mut s = marley_syntax::HighlightSession::new(); s.highlight_full(&text);   // throwaway
    byte_rungs = marley_syntax::enclosing_ranges(&s, bs..be)
    char_rungs = byte_rungs.map(|r| byte_to_char(r.start) .. byte_to_char(r.end))  // through the rope
    self.selection_ladder = Some(SelectionLadder::new(cs..ce, char_rungs))
    self.selection_ladder_at = Some((nonce, version))
}
new = ladder.grow()  (or .shrink())   // char Range<usize>
active_editor_mut().set_active_selections(SelectionSet::single(Selection::new(new.start.into(), new.end.into())))
```
- Selection changes DON'T bump `version`, so the `(nonce,version)` key stays valid across grows (no re-parse);
  an EDIT bumps version ⇒ rebuild (REQ-008); a caret move leaves version but changes `primary` ⇒
  `current() != (cs..ce)` ⇒ rebuild (REQ-008). `shrink` with an invalid/absent ladder is a NO-OP (you can't
  shrink what you didn't grow).
- The borrow dance mirrors #298 add-next-occurrence: read all values, drop the `&Buffer`, compute, re-borrow
  `_mut` to apply.
- Non-Rust files: v1 parses everything with the Rust grammar (coarse ancestry / ERROR nodes) until #315 lands
  grammars — named, harmless.

## Architecture / file manifest
| File | Change |
|---|---|
| `crates/syntax/src/lib.rs` | ADD `pub fn enclosing_ranges(&HighlightSession, Range<usize>) -> Vec<Range<usize>>` (the pure foundation; reads the private `tree`; `named_descendant_for_byte_range` + `.parent()` climb, `is_named` filter, adjacent-dedupe). cov/MSI 100 over fixture parses. **Coverage note:** aimed at lib.rs (100%); IF the FFI Node-walk shows phantom zero-count regions at validate (as the QueryCursor chain did → parse.rs exclude), split the raw walk into `parse.rs` with a pure dedupe helper in lib.rs. |
| `crates/marley_app/src/selection_ladder.rs` | NEW. `pub struct SelectionLadder { anchor, rungs, pos }` (char `Range<usize>`) + `new(anchor, enclosing)` (filters `rung != anchor`), `grow`, `shrink`, `current`. PURE, cov/MSI 100. |
| `crates/marley_app/src/lib.rs` | `mod selection_ladder;` (+ `pub(crate) use` if needed). |
| `crates/marley_app/src/app.rs` | Fields `selection_ladder: Option<SelectionLadder>` + `selection_ladder_at: Option<(u64, BufferVersion)>` (init None). `grow_selection()`/`shrink_selection()` (the shim above). Dispatch arms `"expand-selection"`/`"shrink-selection"` in `dispatch_action`. Test hooks: `grow_selection`/`shrink_selection` are drivable via `dispatch_action` in headless; add `selection_ladder_pos_for_test() -> Option<usize>`. No render shim touched (SelectionSet drives the existing selection render). |
| `crates/marley_app/src/keymap.rs` | ADD two Editor-scoped rows: ⌃W → `"expand-selection"`, ⌃⇧W → `"shrink-selection"`. Bump the roster guards (60→62, 19→21). |

## Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| 001 | `enclosing_ranges_ladder_on_fixture` — parse `fn f(){ g(x); }`; byte range of `x` → ladder identifier→arguments→call→expression_statement→block→function_item→source_file (deduped). | marley_syntax pure unit (cov/MSI 100) |
| 002 | `enclosing_ranges_dedupes_identical_spans` — a node whose span equals its child's appears ONCE. | marley_syntax pure unit |
| 003 | `enclosing_ranges_root_and_no_tree` — range spanning root tops at `source_file` (no dup); fresh session (no tree) → `[]`. | marley_syntax pure unit |
| 004 | `enclosing_ranges_caret_empty_selection` — `start==end` inside an identifier → ladders from that identifier. | marley_syntax pure unit |
| 005 | `grow_selection_emoji_headless` — a buffer `let x = "😀"; foo(x)`; grow from `x` → the selection lands on CHAR boundaries (byte↔char exact, never mid-emoji). | app headless drive |
| 006 | `selection_ladder_grow_advances` (pure) + `grow_twice_no_reparse_headless` — pos 0→1→2, second grow reuses the cached ladder (no rebuild). | pure unit + headless |
| 007 | `selection_ladder_shrink_returns_to_anchor` — grow×2 then shrink×2 → EXACTLY the original anchor range. | pure unit |
| 008 | `selection_ladder_invalidates_headless` — grow, then an edit (version bump) ⇒ next grow rebuilds; grow, then a caret move ⇒ next grow rebuilds. | headless |
| 009 | `grow_collapses_multicursor_headless` — two cursors → grow → a single primary selection. | headless + review |
| 010 | `expand_shrink_chord_dispatch_headless` (`dispatch_action("expand-selection")`/`("shrink-selection")` grow/shrink) + `keymap_expand_shrink_bound` (⌃W/⌃⇧W → the actions, Editor-scoped, roster 62/21). | headless + keymap pure unit |
| — | LIVE pixel capture (selection growth 4 chords deep, shrink back) — **env-blocked (screen locked)** → units + mechanism fallback (documented). | uncoverable-live |

## Risks / decisions
- **D-ROUTE refined to sync-only v1** (above) — reversible; the worker-async path is a named follow-up. The
  one first-press parse-hitch on >1000-line files is the accepted v1 cost.
- **enclosing_ranges coverage placement** — lib.rs (100%) with a parse.rs-split fallback if the FFI walk
  phantom-uncovers at validate (the #268/#274 precedent). Flagged for the validator.
- **Fixture stability** — the ladder test asserts tree-sitter-rust 0.24.2's node kinds/nesting. Pinned
  versions make this stable; assert RANGES (byte spans), not kind strings, so a grammar cosmetic rename
  doesn't break it — but include ONE kind-naming sanity assert to catch a grammar shape change.
- **Roster-guard bump** — TWO new scoped chords; both `assert_eq!` counts (60→62, 19→21) must update or gate:3
  fails (a self-guarding keymap test — working as designed).

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement
Built to the manifest; `cargo check -p marley -p marley_syntax` clean.
- **`crates/syntax/src/lib.rs`** — `pub fn enclosing_ranges(&HighlightSession, Range<usize>) -> Vec<Range<usize>>`
  after the `impl HighlightSession` block: `session.tree.as_ref()` (private field, same-module) →
  `root_node().named_descendant_for_byte_range(s,e)` → climb `.parent()`, push each `is_named()` node's
  `byte_range()`, dedupe against `ladder.last()`, break at the root (`parent() == None`). No-tree / unreachable
  → `[]`. §20-clean doc comment (no brand words).
- **`crates/marley_app/src/selection_ladder.rs`** (NEW) — `SelectionLadder { anchor, rungs, pos }` (char
  `Range<usize>`): `new` filters `rung != anchor`; `grow` = `pos = min(pos+1, rungs.len())`; `shrink` =
  `pos = pos.saturating_sub(1)`; `current` = anchor at `pos==0` else `rungs[pos-1]`. `#[cfg(test)] pub fn pos`.
- **`crates/marley_app/src/lib.rs`** — `mod selection_ladder;` (alphabetical, after `right_dock`).
- **`crates/marley_app/src/app.rs`** — fields `selection_ladder: Option<SelectionLadder>` +
  `selection_ladder_at: Option<(u64, BufferVersion)>` (init None). `grow_selection`/`shrink_selection` →
  `step_selection_ladder(grow: bool)`: reads the primary `(cs, ce, nonce, version)`; `valid` iff
  `selection_ladder_at == (nonce,version)` AND `ladder.current() == (cs..ce)`; on invalid+grow REBUILDS (a
  throwaway `HighlightSession::highlight_full` → `enclosing_ranges` → char rungs via `byte_to_char`), on
  invalid+shrink returns (no-op); then `grow()`/`shrink()` + `set_active_selections(SelectionSet::single(...))`
  (collapses multi-cursor). Borrow dance: read values, drop the `&Buffer`, build the session, re-borrow `_mut`
  to apply; the ladder-step range is copied out before the `active_editor_mut()` re-borrow (no double borrow).
  Dispatch arms `"expand-selection"`/`"shrink-selection"` after `select-all`. Added `ByteOffset` to the
  `marley_text_offsets` import. Test hook `selection_ladder_pos_for_test`.
- **`crates/marley_app/src/keymap.rs`** — two Editor-scoped rows ⌃W→`expand-selection`, ⌃⇧W→`shrink-selection`
  (both free); roster guards bumped 60→62 and 19→21 (+ the two membership asserts in both roster tests).

**Deviations:** D-ROUTE = sync-only throwaway session for ALL sizes (recorded at design; worker-async deferred).
No other deviations. `enclosing_ranges` placed in lib.rs (not parse.rs) — the coverage placement is
validate-verified (parse.rs-split fallback if the FFI walk phantom-uncovers).

**Known transient warnings (resolve at Validate, NOT suppressed):** `selection_ladder_pos_for_test` + the
`SelectionLadder::pos` accessor are `#[cfg(test)]` and unused until the Validate drives call them — clippy's
`-D warnings` sees them in the lib-test build now; the headless drives + pure units that consume them land at
Phase 4, clearing both.

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect
Three parallel critics (general-purpose) over the diff — lenses: (1) pure-projection correctness, (2) app
state/invalidation, (3) reuse/provenance/keymap. Each traced concrete tree-sitter parses / ran the actual
commands. **No CRITICAL/HIGH defects.** Two doc fixes applied; the rest are Validate obligations.

| # | Sev | Finding | Verdict | Action |
|---|-----|---------|---------|--------|
| C1-1 | LOW | `enclosing_ranges` doc said "empty when … unreachable" — imprecise. Empty comes from no-tree OR a REVERSED range (`start > end`); an out-of-bounds range CLAMPS to the root, it doesn't empty. | REAL (doc accuracy) | **FIXED** — reworded to "degenerate (`start > end`); an out-of-bounds range clamps to the root node." |
| C1-2 | MED | `lib.rs` line 471 (the `named_descendant → None` arm) is reachable ONLY by a reversed range — a valid `start ≤ end` always returns ≥ `Some(source_file)` (probe: even `100..200` on a 9-byte file → `Some(0..9)`). Whole-workspace 100%-line coverage → this line stays uncovered unless a reversed-range test exists. | REAL (coverage, latent gate-red) | **VALIDATE OBLIGATION** — add `enclosing_ranges(&s, 5..2) == []`; the code is correct (Option totality), only the test is owed. |
| C2-1 | LOW | The "apply must not bump `version`" invariant is load-bearing (keeps the ladder cache valid across grows + keeps shrink alive) but was undocumented — a future `set_selection` refactor could silently break shrink with zero unit signal (app.rs is mutation-excluded). Verified TODAY it holds (`set_selection` writes only `self.selection`; `version` bumps only on edits, buffer.rs:191). | REAL (defensive doc) | **FIXED** — added a 4-line invariant comment at the apply site. |
| C1-1b/1c | — | Ladder order innermost→root; `last()`-only dedupe is COMPLETE (ancestor ranges are monotonically non-shrinking ⇒ identical spans are always a contiguous run — proven + traced on `fn f(){ g(x); }` → `[10..11, 9..12, 8..12, 8..13, 6..15, 0..15]`, the `function_item`/`source_file` `0..15` pair correctly collapsed); `is_named` filter cannot break dedupe. | CONFIRMED-SAFE | none |
| C1-3 | — | `enclosing_ranges` in lib.rs (NOT parse.rs) — the phantom-region exclusion is specific to the `QueryCursor`/`StreamingIterator` FFI chain; a plain value-returning Node walk (root_node/named_descendant/is_named/byte_range/parent) is ordinary control flow llvm-cov measures accurately. Stays in lib.rs. | CONFIRMED-SAFE | none |
| C1-4 / C3-4 | — | `SelectionLadder` cannot panic (`rungs[pos-1]` bound proven: `pos ∈ [1, len]` in the else arm; empty-rungs grow stays pos=0→anchor; shrink pos=1→anchor exactly). `new`'s `!= anchor` filter drops only the innermost rung, never a middle gap. | CONFIRMED-SAFE | none |
| C2-4/5/6 | — | Borrow-safe (owned returns, no self-borrow across `highlight_full`, range copied before `_mut`); byte↔char boundary-safe at every hop (emoji lands on char boundaries — tree-sitter node bounds never mid-codepoint in valid UTF-8); total on no-editor/caret/non-Rust/EOF. | CONFIRMED-SAFE | none |
| C3-1/2/3 | — | Roster guards PASS (`cargo test keymap` → 20 passed; 62 total + 21 scoped hand-counted); ⌃W/⌃⇧W genuinely FREE (only other `w` is ⌘W close-pane; no global ctrl-w — terminal ⌃W still deletes-word); §20 grep empty. | CONFIRMED-SAFE | none |
| C2-4b | LOW | `primary()` = member-0 (topmost), so ⌃W acts on the topmost cursor, not the last-added. | BY-DESIGN (#307 v1) | none (noted for the N-ladder follow-up) |
| C2/perf | LOW | The rebuild does a synchronous throwaway full parse on the input thread (once per gesture, cached after). | BY-DESIGN (D-ROUTE v1) | none (worker-async is the named follow-up) |

**Validate obligations captured (from the critics' exact analysis):**
- `enclosing_ranges` fixture (assert the EXACT ladder on `fn f(){ g(x); }`) — kills the `:464` body mutants
  (8, all `vec![]`/`Range::…` replacements) + the `:477` `!=`→`==` dedup mutant.
- The reversed-range test `enclosing_ranges(&s, 5..2) == []` (covers line 471) + a fresh-session→`[]` test
  (covers line 465).
- `SelectionLadder` table test — Critic 3's exact case: `new(5..8, vec![3..10, 0..20])` → `grow()==3..10`,
  `grow()==0..20`, `grow()==0..20` (clamp), `shrink()==3..10`, `shrink()==5..8` (anchor), `shrink()==5..8`
  (clamp) — kills the 5 viable SelectionLadder mutants (grow `+`→`-`/`*`, current `==`→`!=`, `-`→`+`/`/`).
- Headless integration: grow×2 → shrink×2 lands back on the EXACT original selection (pins the cross-module
  version-stability invariant) + an edit mid-gesture invalidates + an EMOJI-line grow lands on char
  boundaries.

**Post-fix:** `cargo check -p marley -p marley_syntax` clean; brand-scrub (`grep -rniwE 'warp|zed'`) over the
changed files EMPTY.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

**Tests added (13 across 3 layers), all from the critics' exact mutant-kill analysis:**
- **`crates/syntax/src/lib.rs` — `enclosing_ranges` (5 pure units, cov/MSI 100):** `ladder_on_fixture`
  (`fn f(){ g(x); }`, byte range of `x` → EXACTLY `[10..11, 9..12, 8..12, 8..13, 6..15, 0..15]` — the
  identical-span source_file/function_item pair deduped; kills the `:465` body mutants + the `:478` `!=`→`==`
  dedup mutant), `reversed_range_is_empty` (built from values to dodge clippy's `reversed_empty_ranges` —
  covers the `named_descendant → None` arm), `no_tree_is_empty` (the `tree` None arm), `caret_ladders_from_
  identifier` (REQ-004), `whole_file_dedupes_root` (REQ-003, single entry).
- **`crates/marley_app/src/selection_ladder.rs` — `SelectionLadder` (3 pure units, cov/MSI 100):`
  `grow_shrink_table` (`new(5..8, vec![3..10, 0..20])` → grow/grow/grow-clamp/shrink/shrink-anchor/shrink-
  clamp; kills grow `+`→`-`/`*`, current `==`→`!=` / `-`→`+`/`/`), `new_drops_anchor_equal_rung`,
  `empty_rungs_grow_stays_anchor`.
- **`crates/marley_app/src/headless_drive.rs` — 5 `#[gpui::test]` drives** through the REAL ⌃W/⌃⇧W chord
  (`simulate_keystrokes` → keymap → dispatch, the end-to-end path): `expand_shrink_round_trip` (grow×2 →
  (9,12), shrink×2 → (10,10) EXACTLY — pins the version-stability invariant), `expand_edit_invalidates`
  (edit bumps version → next grow rebuilds, pos 1 not 3), `expand_emoji_lands_on_char_boundaries` (a
  4-byte 😀 earlier on the line → ⌃W selects exactly `x`, byte↔char boundary-safe), `expand_collapses_
  multicursor` (2 cursors → 1 primary), `shrink_without_ladder_is_noop`.

**Test run:** `cargo nextest run -p marley -p marley_syntax` → **609 passed, 0 failed, 2 skipped**; the full
`--diff` gate's nextest → 1423 tests pass.

**Three gate reds found + fixed at source (no baselines):**
1. **gate:2 clippy** — the literal reversed range `5..2` tripped `reversed_empty_ranges` (a hard `-D warnings`
   error that broke the marley_syntax test build, cascading gate:4/5 to fail). Fixed → build the range from
   `let (start, end) = (5usize, 2usize)` values (the lint fires only on literals).
2. **gate:5 mutation** — `selection_ladder.rs` is a NEW untracked file, so `git diff HEAD -- crates` (the
   `--in-diff` source) EXCLUDED it → its 5 mutants were never tested (a silent false-green). Fixed →
   `git add -N crates/marley_app/src/selection_ladder.rs` (intent-to-add) so the diff includes it. ALSO: the
   three new app.rs shim methods (`grow_selection`/`shrink_selection`/`step_selection_ladder`) lacked
   `#[cfg_attr(test, mutants::skip)]` — app.rs is NOT config-excluded from mutation (only from coverage via
   gates.sh:217); its 217 shim fns each carry the skip, so I added it to the three new ones (the established
   pattern — the pure `enclosing_ranges`/`SelectionLadder` seams carry the real mutation surface). Verified via
   `cargo mutants --list --in-diff`: exactly the keymap `default_bindings`, `enclosing_ranges` (`:465` body +
   `:478` dedup), and `SelectionLadder` grow/shrink/current mutants — NO app.rs.
3. **gate:4 coverage** — `enclosing_ranges` line 481 (the `if node.is_named()` guard's implicit false branch)
   was UNREACHABLE: `named_descendant_for_byte_range` returns a named node and every ancestor of a named node
   is itself named — in tree-sitter, anonymous nodes (grammar string-literal tokens) are always LEAVES, so a
   `.parent()` is never anonymous. Fixed → REMOVED the dead `is_named()` guard (a tree-sitter structural
   invariant, grammar-agnostic; the ladder is byte-identical). This is the [MEDIUM] class the inspect critic
   flagged as dead-but-harmless — it was actually a coverage gap.

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** — 15/15 (coverage 100%, mutation MSI 100%, clippy
clean, gate:14 brand-scrub clean, visual/AX green). The `--diff` receipt is commit-valid.

**LIVE drive — env-blocked (documented, §7 fallback):** the mac screen is LOCKED — the driven pixel capture
cannot run. BUT the 5 headless drives are driven through the REAL ⌃W/⌃⇧W chord via `simulate_keystrokes`
(keystroke → keymap → dispatch → selection), so the full input path IS exercised end to end — stronger than a
masked-shim assertion; only the on-screen pixel render is unverified. Re-verify the visible selection growth
when unlocked (30s, no ticket).

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete
- **CHANGELOG.md** — an "Expand / shrink selection (⌃W / ⌃⇧W)" entry above #328 (the enclosing_ranges
  foundation, the SelectionLadder, the sync-throwaway route, the chords, multi-cursor v1, the named cuts).
- **docs/marley_architecture/editor.md** — an "Expand/shrink selection (#329, M21) SHIPS" paragraph after the
  #328 git-gutter section (enclosing_ranges over the private tree, no-is_named-filter invariant, the
  SelectionLadder shrink-to-anchor, D-ROUTE=sync throwaway, byte↔char, the free ⌃W/⌃⇧W chords, §20).
- **docs/marley_architecture/crate-map.md** — the `marley_syntax` row notes its FIRST node-range API
  (enclosing_ranges) as the #305/#330 foundation + the app selection_ladder.rs.
- **Knowledge captured (forge):** AAR `4cb6a9a2` closed `completed` (effectiveness 5). Failures + rules:
  - `BF-totality-branch-only-degenerate-input-uncovered-001` →
    `PR-claude-totality-branch-needs-degenerate-input-test-001` (a §14 totality arm dead for valid inputs
    still needs a degenerate-input test under 100% coverage).
  - `BF-new-untracked-file-skipped-by-in-diff-mutation-001` →
    `PR-claude-intent-add-new-files-before-diff-mutation-001` (`git add -N` a NEW file before the `--diff`
    gate or its mutants are silently skipped — a false MSI-100 green) + the app.rs-shim-needs-mutants-skip
    reinforcement.
  - `PR-claude-reversed-literal-range-trips-clippy-build-values-001` (a literal reversed range `5..2` trips
    clippy `reversed_empty_ranges` under `-D warnings` → build it from values).
  - **NEW AD `AD-claude-syntax-node-range-api-foundation-001`** — marley_syntax answers node-range QUESTIONS
    (`enclosing_ranges`) rather than exposing the `Tree`; v1 ladder via a synchronous throwaway session for all
    sizes (worker-async deferred); the #305/#330 shared foundation.

status: Phase 5 — Complete PASS
