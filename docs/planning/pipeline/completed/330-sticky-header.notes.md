# Sticky context header — the enclosing fn/impl pinned at the viewport top — Notes

- **Forge ticket:** #330 (4de9f2f2-86a7-4142-a54b-5a12ee1cba6d)
- **AAR:** dd0191e6-e9f6-438c-9cef-a442027258cc
- **Local ticket doc:** docs/planning/tickets/open/TICKET-330-sticky-header.md
- **Pipeline spec:** 330-sticky-header.spec.md

## Phase 1 — Plan
- **Request:** the editor twin of the terminal `sticky_block` — pin the enclosing fn/impl/mod at the viewport
  top while scrolling inside it, clickable to jump. Hard dependency on #329 `enclosing_ranges` (shipped
  `a4352c5`). #305 fold is the sibling consumer of the node API.
- **Classification / tier:** work pipeline, one shippable slice — a pure header-collector (marley_syntax) +
  a pure `sticky_rows` pin decision (marley_app) + an app render/scroll/settings/palette shim.
- **Forge recall:** the #329 AD (`AD-claude-syntax-node-range-api-foundation-001`) + the M21 gate lessons
  surfaced; no blocking bulletins.
- **Discovery (the Explore seam map — verbatim for Design):**
  - **Terminal sticky_block (the pin precedent)** — a PURE fn `nav.rs:121` `sticky_block(output_line_counts,
    folds, viewport_top) -> Option<usize>`: pins `Some(RowKind::Output(block,_))`, unpins on
    `Some(RowKind::Header(_))` (real header at top) or `None` (past-last/prompt) — the no-double-render edge,
    off-by-one-proof via the match; tests `nav.rs:282`. Render `app.rs:13005`: `div().absolute().top(px(0.0))
    .left(px(0.0)).w_full()…block_mouse_except_scroll().bg(colors.surface).border_b_1().border_color(colors
    .border)`. **`.block_mouse_except_scroll()` NOT `.occlude()`** — the #185 inspect-HIGH: occlude makes a
    scroll dead-zone. My `sticky_rows` generalizes the single-block pin to N nested headers.
  - **Overlay recipe** — `completion_popup_overlay` (`app.rs:9053`) / `hover_card_overlay` (`8978`) return
    `Option<gpui::Div>`, positioned `.absolute().left(px).top(px).bg(colors.surface).border_1().border_color(
    colors.border).rounded(colors.corner_radius)`; composited at render END `app.rs:14441` as
    `root = root.child(overlay)` (later child = higher z). For a top band: `.top(px(y0 + n*cell_h)).w_full()`.
  - **Scroll model / `first_visible_row`** — **`self.editor_geom.get().first`** (`EditorFrameGeom{x0,y0,first,
    last,cell_w,cell_h}` `app.rs:572`, captured from `uniform_list`'s range callback on the first row's canvas
    `app.rs:4065`; the field `app.rs:371`; the established read e.g. `app.rs:7577`). NOT a scroll_y math.
    `scroll_editor_to_row(row)` `app.rs:9801` = `editor_scroll.scroll_to_item(row, ScrollStrategy::Center)`
    — **no band offset today**; I must bias the center to `row + sticky_height` myself. `follow_editor_caret`
    `9812`, `pending_center_row` `app.rs:282` + `consume_pending_center` `8860` (the #312 deferred-center).
    `editor_scroll_y_for_test` `9825`.
  - **Settings** — `define_setting!(StickyHeader: bool = true, "editor.sticky_header")` (macro `macros.rs:8`;
    bool exemplars `settings.rs:27` DockLeft/`101` FilesOpen). Wire `AppliedSettings` field + `applied_from`
    (`settings.rs:251`) + `applied_defaults` (`217`) [compile forces both] + `persist_sticky_header`
    (mirror `persist_files_open` `322`). App reads applied → a `RootView` field. FLAG: `editor.*` is a NEW
    TOML namespace (first key — fine).
  - **#329 reuse** — `enclosing_ranges` body confirmed (`syntax/lib.rs:461`, ancestry walk + adjacent-dedup).
    `Node::kind() -> &'static str` available (0.26.11). Factoring options: a private `enclosing_nodes` iterator
    both consume (a hand-rolled `struct Climb<'t>{node: Option<Node<'t>>}` avoids closure-lifetime friction),
    OR duplicate the 8-line walk with a kind filter. NOTE the collector may want a FULL-tree walk (all_headers)
    not an ancestry walk — see D-CACHE below.
  - **Palette toggle** — the "Toggle Theme" twin (palette-only, `binding: None`): `cockpit_commands`
    `app.rs:6080` (next free id `CommandId(14)`; `CommandId(3)` is a removed gap), `action_for_command`
    `palette.rs:127`, `dispatch_action` `app.rs:5439` + a `toggle_*` helper that flips + persists. A
    completeness test `app.rs:14536` + `palette.rs:267` asserts every command maps to an action — add all
    three sites together.
- **The load-bearing design realization (shapes D-CACHE):** sticky recomputes EVERY scroll frame
  (`first_visible_row` changes as you scroll). A throwaway-`HighlightSession` parse per frame would be O(file)
  per scroll tick = janky. So the header list MUST be cached per `(nonce, version)` (one reparse per EDIT,
  the #329 sync route), and the per-frame step MUST be a pure filter (`sticky_rows`). This reframes the
  ticket's `enclosing_headers(byte)` + "cache per (version, first_visible_row)" → **lean v1 = `all_headers(&
  session)` (all file headers, one parse per version) + `sticky_rows(all_headers, first_visible_row,
  max_depth)` (pure filter per frame)**. Design confirms (all_headers vs cached-session + enclosing_headers).
- **Decisions:** D-COLLECT (header-kind filter), D-CACHE (per-version list + per-frame pure filter),
  D-PIN (strictly-inside + header-off, nested, max_depth=2, no-double-render edge), D-RENDER (absolute overlay
  + block_mouse_except_scroll + click-gate), D-SCROLL (editor_geom.first + sticky_height center/margin),
  D-SETTING (editor.sticky_header default-on + palette toggle). §20 N/A.

## Phase 2 — Design

### §20 confirm
N/A — Marley composition. The sticky-scope-header BEHAVIOR is a universal editor affordance (VS Code sticky
scroll = observed reference, informing `max_depth = 2`; Zed's `BlockMap` display-stack deliberately NOT used).
Built from Marley's OWN parts: the #329 tree access, the shipped terminal `sticky_block` pin/overlay precedent
(#185, Marley's own code), and published `tree_sitter` node-kind reads. No copyleft source read. Clean-room holds.

### (1) The header collector — `all_headers` (marley_syntax, `crates/syntax/src/lib.rs`)
```rust
fn is_header_kind(kind: &str) -> bool {
    matches!(kind, "function_item" | "impl_item" | "mod_item" | "trait_item")   // Rust v1; #315 extends
}
/// All header-bearing nodes in the file, in document order, as byte ranges. Empty on a fresh session.
pub fn all_headers(session: &HighlightSession) -> Vec<std::ops::Range<usize>> {
    let Some(tree) = session.tree.as_ref() else { return Vec::new(); };
    let mut out = Vec::new();
    collect_headers(tree.root_node(), &mut out);   // preorder recursion (Rust nesting is bounded)
    out
}
fn collect_headers(node: tree_sitter::Node, out: &mut Vec<std::ops::Range<usize>>) {
    if is_header_kind(node.kind()) { out.push(node.byte_range()); }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) { collect_headers(child, out); }
}
```
- A FULL-tree walk (NOT the #329 ancestry walk) — the D-CACHE fork RESOLVED: `all_headers` runs ONCE per
  `(nonce, version)` (a throwaway-`HighlightSession` reparse per edit, the #329 sync route); the per-frame step
  is the pure `sticky_rows` filter. This avoids holding a `Tree` on the app AND a per-scroll-frame parse.
- Returns BYTE ranges (pure, text-free → cov/MSI 100 over fixtures); the APP maps byte→row once per version
  (`line_col(byte_to_char(ByteOffset::from(b))).0`). `is_header_kind` is its own pure unit (each kind + a
  non-header → the `matches!` mutation surface). Coverage: aimed at lib.rs like #329; the `node.children()`
  iterator is a plain child walk (not a `QueryCursor`), so no phantom-region risk — parse.rs-split fallback if
  validate disagrees.

### (2) The pin decision — `sticky_rows` (PURE, new `crates/marley_app/src/sticky_header.rs`)
```rust
/// The header rows to PIN at the viewport top, outermost→innermost, capped at `max_depth`.
/// `headers`: every header's (header_row, end_row) span (any order). A header pins while `first_visible_row`
/// is strictly PAST its header row (it scrolled off) and still within its span; nested scopes stack, and when
/// more than `max_depth` enclose, the INNERMOST `max_depth` are kept (the immediate context, not the module).
pub fn sticky_rows(headers: &[(usize, usize)], first_visible_row: usize, max_depth: usize) -> Vec<usize> {
    let mut enclosing: Vec<(usize, usize)> = headers
        .iter()
        .copied()
        .filter(|&(h, e)| h < first_visible_row && first_visible_row <= e)   // scrolled-off + still inside
        .collect();
    enclosing.sort_by_key(|&(h, _)| h);                 // outermost first (container's header_row is smaller)
    if enclosing.len() > max_depth {
        enclosing.drain(0..enclosing.len() - max_depth); // keep the innermost max_depth
    }
    enclosing.into_iter().map(|(h, _)| h).collect()
}
```
- **Boundary decisions (the table pins them):** `first_visible_row == header_row` → `h < first` false → NOT
  pinned (the sticky_block no-double-render edge); `first_visible_row == end_row` → `first <= e` true → still
  pinned (you're on the scope's last line); `first_visible_row == end_row + 1` → past → unpinned. Nested chain
  (impl>fn both enclose) → both pinned, impl at row 0, fn at row 1. `> max_depth` → keep innermost (drain the
  outermost extras). Free code / empty → `[]`.
- The enclosing set is always a proper ancestry chain (nested scopes have distinct header_rows; siblings don't
  co-enclose), so `sort_by_key(header_row)` is a total outermost→innermost order.
- Mutation surface: the `<`/`<=` predicate, the `sort`, the `drain(0..len - max_depth)` arithmetic + the `>`
  guard — all killed by the boundary table (at-edge, nested, depth-cap-of-3).

### (3) Cache + refresh (app.rs shim, mutants::skip)
- Field `sticky_headers: Option<(u64, marley_editor::BufferVersion, Vec<(usize, usize)>)>` — the (nonce,
  version, row-spans) cache. `refresh_sticky_headers(&mut self) -> bool` in the PUMP (mirrors `refresh_git_marks`
  #328): if the active editor's (nonce, version) matches the cache key → false; else reparse (throwaway
  `HighlightSession::highlight_full(text)` → `all_headers` → map each byte range to `(header_row, end_row)`
  via the buffer) + store, return true (→ dirty). Gated on `self.sticky_header` (the setting) — off → clear
  the cache, skip. So an idle scroll frame does NO parse; only an edit (version bump) reparses.
- Field `sticky_header: bool` — read from `AppliedSettings` at boot (default true).

### (4) Render (app.rs shim, mutants::skip)
- In the editor render, after the `uniform_list`: `let geom = self.editor_geom.get(); let rows =
  sticky_header::sticky_rows(&cached_spans, geom.first, STICKY_MAX_DEPTH);` — gated on `self.sticky_header` &&
  non-empty `rows`. For each pinned row `n`: `div().absolute().top(px(geom.y0 + n as f32 * geom.cell_h))
  .left(px(0.0)).w_full().bg(colors.surface).border_b_1().border_color(colors.border)
  .block_mouse_except_scroll()` (the #185 recipe — NOT `.occlude()`), rendering that buffer row through the
  SAME line render path (syntax-highlighted, gutter-aligned) as the list rows. Composited via `pane.child(...)`
  / `root.child(...)` (later child = higher z). An `.on_mouse_down` per pinned row → jump.
- **Click = jump:** `open_and_place_caret(active_path, |buf| buf.line_start(header_row))` (#312 — re-opens the
  already-open file as a no-op, places the caret, pushes the NavStack, centers). Reuses the shipped path.

### (5) D-SCROLL — RESOLVED as a NO-OP for v1 (a simplification)
`scroll_editor_to_row` uses `ScrollStrategy::Center` (mid-viewport) for EVERY scroll-to-row (F8, find,
go-to-def, `follow_editor_caret`). A centered row lands mid-viewport, never at the very top edge — so it is
NEVER under the pinned band (even a 2-row band). Therefore REQ-008/009 are SATISFIED by the existing center
strategy with NO scroll-math change: the sticky_height bias would only matter for a hypothetical "reveal at
top" strategy, which Marley doesn't have. Documented; inspect/validate confirm. (If a top-reveal scroll is
ever added, it must inset by `sticky_rows.len() * cell_h`.)

### (6) Setting + palette toggle
- `settings.rs`: `define_setting!(StickyHeader: bool = true, "editor.sticky_header")`; `AppliedSettings`
  gains `sticky_header: bool` wired into `applied_from` (`get::<StickyHeader>()`) + `applied_defaults`
  (`StickyHeader::default_value()`) [compile forces both]; `persist_sticky_header(manager, bool)`.
- `palette.rs`: `action_for_command(CommandId(14)) => Some("toggle-sticky-header")` + the completeness test row.
- `app.rs`: `cockpit_commands` adds `Command { id: CommandId(14), title: "Toggle Sticky Header", keywords:
  ["sticky","header","scroll"], binding: None }`; `dispatch_action` arm `"toggle-sticky-header" =>
  self.toggle_sticky_header()`; `toggle_sticky_header` (mutants::skip) flips `self.sticky_header` + clears the
  cache + `persist_sticky_header`. ALL THREE sites together (the `app.rs` + `palette.rs` completeness tests).

## Architecture / file manifest
| File | Change |
|---|---|
| `crates/syntax/src/lib.rs` | ADD `pub fn all_headers(&HighlightSession) -> Vec<Range<usize>>` + private `collect_headers` (preorder recursion) + private `is_header_kind(&str) -> bool`. Pure, cov/MSI 100. |
| `crates/marley_app/src/sticky_header.rs` | NEW. `pub fn sticky_rows(&[(usize,usize)], usize, usize) -> Vec<usize>` + `pub const STICKY_MAX_DEPTH: usize = 2`. PURE, cov/MSI 100. |
| `crates/marley_app/src/lib.rs` | `mod sticky_header;`. |
| `crates/marley_app/src/app.rs` | Fields `sticky_headers: Option<(u64, BufferVersion, Vec<(usize,usize)>)>` + `sticky_header: bool` (init from settings). `refresh_sticky_headers` (pump, mutants::skip). The render overlay + click-jump (mutants::skip). `toggle_sticky_header` (mutants::skip). `dispatch_action` arm + the `cockpit_commands` `CommandId(14)` row. Test hooks `sticky_rows_for_test`/`sticky_header_spans_for_test`. |
| `crates/marley_app/src/settings.rs` | `StickyHeader` setting + `AppliedSettings.sticky_header` + `applied_from`/`applied_defaults` + `persist_sticky_header`. |
| `crates/marley_app/src/palette.rs` | `CommandId(14) => "toggle-sticky-header"` + the completeness test row. |

## Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| 001 | `all_headers_over_fixtures` — nested `mod m { impl T { fn f(){} } }` → the 3 header byte ranges in doc order; a fn at file top; free code (`let x = 1;`) → empty; each kind recognized. `is_header_kind_each_kind` (the 4 kinds + a non-header). | marley_syntax pure (cov/MSI 100) |
| 002 | `sticky_rows_encloses_and_stacks` — an impl+fn chain, `first_visible_row` inside both → `[impl_row, fn_row]` outermost-first. | pure unit |
| 003 | `sticky_rows_header_at_top_no_pin` — `first_visible_row == header_row` → `[]` (no double-render). | pure unit |
| 004 | `sticky_rows_depth_cap` — 3 enclosing (mod>impl>fn), `max_depth=2` → the innermost 2 (`[impl_row, fn_row]`, mod dropped). | pure unit |
| 005 | `sticky_rows_and_all_headers_total` — empty headers / no enclosing / a fresh session / a reversed all_headers input → `[]`, never a panic. | pure unit |
| 006 | `sticky_overlay_renders_gated_headless` — with sticky on + a scrolled fn, the overlay is present (block_mouse_except_scroll, non-empty gate); off → absent. | headless + review (live env-blocked) |
| 007 | `sticky_click_jumps_headless` — clicking a pinned header row places the caret at that row + pushes the NavStack. | headless |
| 008 | REQ-008/009 — center strategy already places jumps mid-viewport (below the band); no band under-lap. | review (mechanism — see D-SCROLL) |
| 009 | `sticky_headers_cache_per_version_headless` — `refresh_sticky_headers` recomputes only on a version change (idempotent across scroll; recomputes after an edit). | headless + review |
| 010 | `sticky_setting_roundtrip` (default true; persist/reload) + `sticky_toggle_off_no_overlay_headless` (toggle → no sticky). | unit (setting) + headless |
| — | LIVE pixel capture (scroll a real fn → pinned header, bg+hairline, gutter-aligned; click → jump) — **env-blocked (screen locked)** → units+mechanism. | uncoverable-live |

## Risks / decisions
- **D-CACHE = all_headers per (nonce, version) + pure sticky_rows per frame** — the load-bearing choice; avoids
  a per-scroll-frame parse. The one throwaway reparse per EDIT is the accepted v1 cost (worker-async is the
  named follow-up). Reversible.
- **D-PIN = keep innermost max_depth, `<`/`<=` boundary** — the immediate scope is the useful context; the
  boundary is pinned by the table.
- **D-SCROLL = no-op (center strategy already avoids the band)** — reversible; a top-reveal scroll would need
  the inset.
- **all_headers coverage placement** — lib.rs (like #329); parse.rs-split fallback if the child walk
  phantom-uncovers at validate.
- **CommandId(14) + the completeness tests** — the `cockpit_commands`/`action_for_command`/`dispatch_action`
  trio must land together (two self-guarding tests).
- **`git add -N` the new `sticky_header.rs`** before the `--diff` gate (the #329 mutation-gap lesson).

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement
Built to the manifest; `cargo check --workspace` + test-compile clean.
- **`crates/syntax/src/lib.rs`** — `is_header_kind` (`matches!` over the 4 Rust kinds), `all_headers(&session)`
  (full-tree preorder recursion via `collect_headers` over `node.children(&mut cursor)`, pushing header-kind
  byte ranges in document order; empty on no tree). Pure.
- **`crates/marley_app/src/sticky_header.rs`** (NEW) — `STICKY_MAX_DEPTH = 2` + `sticky_rows(headers,
  first_visible_row, max_depth)`: filter `header_row < first_visible_row && first_visible_row <= end_row`,
  sort by header_row (outermost first), keep the innermost `max_depth` (drain the outer extras), map to rows.
- **`crates/marley_app/src/lib.rs`** — `mod sticky_header;` (after `status_bar`).
- **`crates/marley_app/src/settings.rs`** — `define_setting!(StickyHeader: bool = true, "editor.sticky_header")`;
  `AppliedSettings.sticky_header` wired into `applied_from`/`applied_defaults`; `persist_sticky_header`; the 3
  test constructors gained `sticky_header: true`.
- **`crates/marley_app/src/palette.rs`** — `CommandId(14) => Some("toggle-sticky-header")`.
- **`crates/marley_app/src/app.rs`** — a `StickyHeaderCache` type alias (clippy `type_complexity` fix) + the
  fields `sticky_headers`/`sticky_header` (init: cache None, setting from `applied.sticky_header`).
  `refresh_sticky_headers` (pump, mutants::skip — cache-gated on `(nonce, version)`, off/no-editor drops the
  cache, else a throwaway `HighlightSession` reparse → `all_headers` → byte→row via `line_col(byte_to_char)`);
  wired into the pump beside `refresh_git_marks`. `sticky_header_overlay` (mutants::skip — the absolute band
  over the `uniform_list`: `block_mouse_except_scroll` + surface bg + hairline, per-row `line_layout` +
  syntax-cache highlight via `styled_slices_with_marks(&syntax, &[], &[])`, gutter via `gutter_label`, click →
  `jump_to_sticky_header`); `body` gained `.relative()` + the band child after the list; `jump_to_sticky_header`
  (mutants::skip — `open_and_place_caret` to `line_start(header_row)`). `toggle_sticky_header` (mutants::skip)
  + the `dispatch_action` arm + the `cockpit_commands` `CommandId(14)` entry. Test hooks
  `sticky_header_spans_for_test`/`sticky_rows_for_test`.

**Deviations from design:** none material. **D-SCROLL confirmed a no-op** (center strategy already places
jumps mid-viewport — no scroll-math change, as designed). The overlay `colors` needed a dedicated clone
(`sticky_colors`) because the list closure MOVES its own `colors` clone. The `sticky_headers` tuple got a
`StickyHeaderCache` type alias to satisfy clippy `type_complexity` (-D warnings).

**Known transient warnings (resolve at Validate, NOT suppressed):** `sticky_header_spans_for_test` +
`sticky_rows_for_test` are `#[cfg(test)]` and unused until the Validate drives call them (the #329 pattern) —
clippy's `-D warnings` sees them in the lib-test build now; the pure units + headless drives land at Phase 4.

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect
Three parallel critics (general-purpose) over the diff — lenses: (1) pure correctness, (2) app state/cache,
(3) render/provenance/wiring. Each verified concretely (throwaway tree-sitter probes with measured stack
ceilings, `cargo test`, `cargo mutants --list`, greps). **THREE real defects found and FIXED.**

| # | Sev | Finding | Verdict | Action |
|---|-----|---------|---------|--------|
| C1-1 | **HIGH** | `collect_headers` was **unbounded recursion**. Its depth is the FULL tree depth (every nested block/paren/expression level), NOT the header nesting — a probe proved a `fn` wrapping 2000 nested blocks recurses 2000 deep for ONE header. Measured: survives 12k, **overflows + SIGABRTs at 16k** on the 8 MiB main stack (2k–5k on a test thread). That is an UNCATCHABLE abort (not `catch_unwind`-able) taking every open terminal/editor with it, on the UI thread, on EVERY edit (sticky is default-ON). My doc claim "tree-sitter caps parse recursion, so plain recursion is safe" was **factually wrong** — tree-sitter's own parse is iterative *because* of this. | REAL (crash) | **FIXED** — rewrote as an ITERATIVE `TreeCursor` preorder walk (`goto_first_child`/`goto_next_sibling`/`goto_parent`, O(1) stack, identical document order) + deleted the false claim. |
| C3-1 | **HIGH** | `jump_to_sticky_header` **never pushed the NavStack** — REQ-007 requires "caret placed + NavStack pushed". `open_and_place_caret` only opens + places + parks the center; every sibling jump (#312 goto-def, symbols, problems) captures the origin and pushes EXPLICITLY (proof: they'd double-push if the helper pushed). So ⌃- would NOT return after clicking a sticky header, and the planned validate test would fail. | REAL (REQ violation) | **FIXED** — capture `(path, active_caret())` BEFORE the jump; push `NavLoc { path, offset }` only if `open_and_place_caret` returned true (the `jump_to_definition` shape). |
| C1-2 | MED | `if enclosing.len() > max_depth { drain(0..len - max_depth) }` — the `>`→`>=` mutant is **provably EQUIVALENT**: they differ only at `len == max_depth`, where the mutant drains `0..0` (a no-op). No test can kill it → it would survive → MSI < 100 → **gate:5 red** on a pure seam. | REAL (unkillable mutant) | **FIXED** — dropped the guard for `let drop_outer = len.saturating_sub(max_depth); drain(0..drop_outer);`. Identical behavior (max_depth 0 → drain all; max_depth > len → drain none) and `saturating_sub` is a method call (unmutated), so the whole `>`/`>=`/`-`→`+`/`-`→`/` cluster disappears. |
| C1-5 | LOW | Two scopes starting on the SAME line (`mod m { fn f() {`) map to one `header_row` → `sticky_rows` returned `[row, row]` (a cosmetic double-pin). | REAL (minor) | **FIXED** — `dedup_by_key(header_row)` after the stable sort (which keeps outermost-first). |
| C2-2 | — | The marquee risk: `byte_to_char(r.end - 1)` landing MID-codepoint. Traced to ropey → `str_indices::chars::from_byte_idx`, which walks BACK off continuation bytes → returns the containing char's index. Panics only if `byte_idx > len_bytes()`, impossible here (the tree is parsed from the same buffer text). And Rust's 4 header kinds always end in `}`/`;` (ASCII) anyway. | CONFIRMED-SAFE | none |
| C1-3 | — | The pin predicate `header_row < first_visible_row && first_visible_row <= end_row`: strict `<` (a header AT the top does not pin — no double-render), inclusive `<=` (pins through the scope's closing-brace row, releases at `end_row+1`). `sort_by_key` is STABLE (correct — unstable would be a latent bug). `drain` can't underflow. | CONFIRMED-SAFE | none |
| C2-1/4/5/6 | — | Cache `(nonce, version)` correct (scroll → no reparse; edit/file-switch → recompute; off/no-editor → drop). Borrow-safe (owned returns, no `&self` across the parse). Toggle flips + clears + persists, repaint via the dispatch `cx.notify()` (off clears instantly; on appears next pump tick). All four app shims carry `mutants::skip` (verified absent from `cargo mutants --list -f app.rs`). | CONFIRMED-SAFE | none |
| C3-1..7 | — | Click-gate (None → zero band → no click-eat when unpinned); `block_mouse_except_scroll` byte-identical to the terminal `sticky_block` recipe (not `.occlude()`); the `sticky_colors` clone (no double-move, compiles); `.relative()` + band-after-list z-order; settings roundtrip (`cargo test settings::` → 18 passed, default true); the CommandId(14) trio (`every_cockpit_command_resolves_to_a_verb` + `action_for_command_maps_every_row` both ok); §20 grep empty. | CONFIRMED-SAFE | none |
| C3/C1 | LOW | `py_1` vs the band's `top(0)` → a ~4px vertical mis-register. | REAL (cosmetic) | DEFERRED — pixel-verify when unlocked; not worth guessing blind. |
| C1-6 | LOW | The kind table omits `struct_item`/`enum_item`/`const_item` and body-less `function_signature_item` (VS Code includes struct/enum). | BY-DESIGN (v1) | none (a #315-era follow-up) |
| C2 | LOW | A one-pump-tick staleness window after an edit (cached rows vs new text) — panic-free (`line_text` returns "" OOB, `line_start` clamps), self-heals; the accepted #327/#328 pump-cache tradeoff. | BY-DESIGN | none |

**Validate obligations (the critics' exact kill-table):**
- `sticky_rows` (10→fewer mutants post-refactor): first==header_row → NOT pinned (kills `<`→`<=`); first inside → pinned; first==end_row → pinned (kills `<=`→`<`); first==end_row+1 → not; scope wholly above → not (kills `&&`→`||`); first outside all → `[]` (kills `vec![0]`/`vec![1]`); 3 nested @ max_depth 2 → innermost 2, outermost dropped; an UNSORTED input → the outermost is the one dropped (nothing else exercises the sort).
- `all_headers`: the nested `mod m { impl T { fn f(){} } }` fixture asserting the EXACT vec — the critic verified it is **`[0..29, 8..27, 17..25]`** (mod, impl, fn — document order, outermost-first). An exact-COUNT assert kills `is_header_kind`→`true`; non-empty kills `→false`; only a NESTED fixture proves the DFS + order (`collect_headers`→`()` dies to any non-empty result).
- The two `#[cfg(test)]` accessors (`sticky_header_spans_for_test`, `sticky_rows_for_test`) MUST be consumed by the drives or gate:2 stays red (they're currently dead → the documented transient).

**Post-fix:** `cargo check -p marley -p marley_syntax` clean; lib clippy clean; `cargo fmt` clean; brand-scrub EMPTY.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

**Tests added (15 across 4 layers), from the critics' exact kill-table:**
- **`crates/syntax/src/lib.rs` — `all_headers` (5 pure units, cov/MSI 100):**
  `all_headers_nested_fixture_in_document_order` (`mod m { impl T { fn f(){} } }` → EXACTLY
  `[0..29, 8..27, 17..25]` — only a NESTED fixture proves the walk descends; the exact COUNT kills
  `is_header_kind`→`true`), `all_headers_no_tree_is_empty`, `all_headers_free_code_is_empty` (kills
  `is_header_kind`→`true`), `all_headers_kind_table` (a `trait_item` + a default-bodied method ARE headers; a
  body-less `function_signature_item` is NOT), **`all_headers_deep_nesting_does_not_overflow`** (2000 nested
  blocks → 1 header, no stack overflow — the regression test for the inspect C1-1 recursion crash).
- **`crates/marley_app/src/sticky_header.rs` — `sticky_rows` (6 pure units, cov/MSI 100):**
  `sticky_rows_pin_boundaries` (at-header→no pin [kills `<`→`<=`]; inside→pin; at end_row→pin [kills
  `<=`→`<`]; end_row+1→release; above→no; a scope that ENDED above→no [kills `&&`→`||`]),
  `sticky_rows_nested_stack`, `sticky_rows_depth_cap_keeps_innermost` (3 @ cap 2 → innermost 2; cap 0 → `[]`;
  cap > len → all), `sticky_rows_unsorted_input_still_drops_outermost` (the only thing exercising
  `sort_by_key`), `sticky_rows_none_enclosing_is_empty` (kills the `vec![0]`/`vec![1]` bodies),
  `sticky_rows_dedups_same_header_row`.
- **`crates/marley_app/src/headless_drive.rs` — 3 `#[gpui::test]` drives:**
  `sticky_headers_cached_and_pinned_headless` (the 4-row fn fixture → spans `(0,3)`; inside → `[0]`; at the
  header row → `[]`; a second refresh with no edit → **cache HIT, no reparse**),
  `sticky_toggle_off_clears_headless` (on → cached; off → cache dropped + nothing pins; back on →
  recomputed), `sticky_cache_recomputes_after_edit_headless` (an edit bumps the version → recompute).
- **`crates/marley_app/src/settings.rs`** — the sticky-header leg added to `settings_round_trip_survives_reload`
  (persist **false**, the NON-default, → the reload reads false).
- **Test hooks added** so the drives could consume them (clearing their dead-code): `refresh_sticky_headers_for_test`,
  `toggle_sticky_header_for_test` (+ the pre-placed `sticky_header_spans_for_test`/`sticky_rows_for_test`).

**Test run:** `cargo nextest run -p marley -E 'test(/sticky/)'` → **11 passed** (6 units + 3 drives + the 2
pre-existing terminal `sticky_block` tests, still green — the terminal twin is untouched). The gate's full
nextest → all green.

**One gate red found + fixed at source:** gate:5 mutation had **1 survivor** —
`settings.rs persist_sticky_header → Ok(())`: nothing proved the setter actually WRITES. Fixed by adding the
sticky-header leg to the existing round-trip test, persisting the NON-default `false` so a no-op write reloads
as the `true` default and fails the assert. (18 mutants → 18 caught, MSI 100.)

**Mutation scope note (the #329 lesson applied):** `git add -N crates/marley_app/src/sticky_header.rs` BEFORE
the gate — a NEW untracked file is excluded from `git diff HEAD`, so `--in-diff` would have silently skipped
all 8 of its mutants (a false MSI-100 green on the exact new pure seam). Verified with
`cargo mutants --list --in-diff`: **8** sticky_header.rs mutants + **12** syntax header-fn mutants in scope.

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** — 15/15 (coverage 100%, mutation MSI 100, clippy
clean incl. the now-consumed test hooks, gate:14 brand-scrub clean, visual/AX green). The `--diff` receipt is
commit-valid.

**LIVE drive — env-blocked (documented, §7 fallback):** the mac screen is LOCKED — the driven pixel capture
cannot run, so the band's ON-SCREEN appearance (surface bg + bottom hairline, gutter alignment, code scrolling
UNDER it) is unverified. Verified instead via units + mechanism: the pure collector + pin filter are cov/MSI
100; the drives prove the cache + pin rows + toggle end to end; the overlay is byte-identical to the shipped
terminal `sticky_block` recipe (`.absolute().top(0).w_full().bg(surface).border_b_1().block_mouse_except_scroll()`).
**Deferred to an unlocked pass (no ticket):** pixel-verify the band + the known ~4px `py_1`-vs-`top(0)`
mis-register (inspect LOW — not worth guessing blind), and the click→jump→⌃- round trip.

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete
- **CHANGELOG.md** — a "Sticky context header" entry above #329 (the iterative collector + the pure pin
  decision + the cache cadence + the overlay/click + the setting/palette + the named v1 cuts).
- **docs/marley_architecture/editor.md** — a "Sticky context header (#330, M21) SHIPS — the #329 foundation's
  first consumer" paragraph (the sticky_block twin; the load-bearing ITERATIVE walk + why; the pin/unpin
  boundary; cache-per-version + pure-filter-per-frame; the overlay + block_mouse_except_scroll; the D-SCROLL
  no-op finding; the setting + palette; §20 N/A).
- **docs/marley_architecture/crate-map.md** — the `marley_syntax` row notes `all_headers` as the #329
  node-range API's first consumer + the app `sticky_header.rs` / `editor.sticky_header`.
- **Knowledge captured (forge):** AAR `dd0191e6` closed `completed` (effectiveness 5, 8 novel findings).
  Four failures + four rules — three caught at INSPECT, one at VALIDATE:
  - `BF-recursive-tree-walk-stack-overflow-on-deep-input-001` →
    `PR-claude-iterate-cursor-not-recurse-over-unbounded-tree-001` (walk a parse tree ITERATIVELY; recursion
    depth is the full tree depth, and an overflow on the UI thread is an uncatchable abort — never justify
    recursion with an unmeasured "the input is bounded" claim).
  - `BF-jump-helper-assumed-to-push-navstack-001` →
    `PR-claude-verify-helper-side-effects-against-siblings-001` (read the helper AND its existing callers —
    if every sibling does X around it, the helper doesn't do X).
  - `BF-equivalent-guard-mutant-blocks-msi-100-001` → `PR-claude-saturating-sub-instead-of-len-guard-001`
    (`len.saturating_sub(k)` over `if len > k { len - k }` — the guard form is an unkillable equivalent
    mutant under an MSI-100 floor).
  - `BF-persist-setter-body-mutant-survives-no-roundtrip-001` →
    `PR-claude-new-setting-needs-nondefault-roundtrip-leg-001` (a new setting needs FOUR wirings — the fourth
    is a NON-DEFAULT leg in the shared round-trip test, or the `persist_X -> Ok(())` mutant survives).
  - **NO new AD** — #330 CONSUMES `AD-claude-syntax-node-range-api-foundation-001` (#329), the foundation's
    first consumer exactly as that decision predicted; the cache-per-version + pure-filter-per-frame cadence
    is a composition of established patterns (the #327/#328 pump-cache shape), not a novel architecture.

status: Phase 5 — Complete PASS
