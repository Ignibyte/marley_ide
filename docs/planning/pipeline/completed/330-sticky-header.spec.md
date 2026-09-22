---
pipeline_id: 3366382c-d71e-4c54-9b35-54edbe9fdafb
ticket: forge#330 (4de9f2f2-86a7-4142-a54b-5a12ee1cba6d) · local docs/planning/tickets/open/TICKET-330-sticky-header.md
aar_id: dd0191e6-e9f6-438c-9cef-a442027258cc
status: Phase 5 — Complete PASS
title: Sticky context header — the enclosing fn/impl pinned at the viewport top while you scroll inside it
type: feature
milestone: M21
references: [forge#330, forge#329, forge#185, forge#312, forge#308]
---

## Title
Scroll into the middle of a long function and the editor still tells you where you are: the enclosing
`fn`/`impl`/`mod` header line pinned at the viewport top, clickable to jump. Marley already ships EXACTLY
this UX for the terminal (`sticky_block`, #185); this is the editor twin, powered by the #329
`enclosing_ranges` node API instead of block extents.

## Scope
### In
- **The header collector (marley_syntax, on the #329 foundation)** — a PURE fn that yields the file's
  header-bearing nodes with their row spans, filtered by node kind (`function_item`, `impl_item`, `mod_item`,
  `trait_item` for Rust v1 — a per-grammar kind table #315 extends). DESIGN picks the exact shape/name
  (`all_headers(&HighlightSession) -> Vec<HeaderSpan{start_byte, end_byte}>` [a full-tree walk] vs
  `enclosing_headers(&HighlightSession, byte)` [the #329 ancestry walk filtered by kind]). `Node::kind() ->
  &'static str` (tree-sitter 0.26.11) drives the filter. Pure, cov/MSI 100 over fixtures.
- **The pin decision (PURE, marley_app)** — `sticky_rows(headers, first_visible_row, max_depth) -> Vec<row>`:
  a header pins while `first_visible_row` is strictly INSIDE its node's row span AND its own header row has
  scrolled OFF (`header_row < first_visible_row`); nested scopes STACK (impl over fn), capped at
  `max_depth = 2` (the VS Code observed default). A header row exactly AT `first_visible_row` does NOT pin
  (the `sticky_block` no-double-render edge). This is the mutation/coverage surface — the boundary rows
  (inside / at-edge / header-visible / nested-stack / depth-cap) are pinned by a table test like
  `popup_origin`'s.
- **Caching (the scroll-hot path)** — the header list is cached per `(nonce, version)` and recomputed only on
  a version change (an edit), NOT per scroll frame. `sticky_rows` filters the cached list every frame (pure,
  no parse). A version-change triggers ONE throwaway-`HighlightSession` reparse (the #329 sync route), lazily
  on the next sticky render — amortized over all scroll frames at that version. (The worker-async header
  extraction is the named follow-up, folding with #329's deferred worker route.)
- **Render (app.rs shim, mutants::skip)** — the pinned rows draw as ABSOLUTE OVERLAY rows above the
  `uniform_list` viewport (the `sticky_block`/overlay recipe: `div().absolute().top(px(y0 + n*cell_h))
  .w_full().bg(colors.surface).border_b_1().border_color(colors.border).block_mouse_except_scroll()`), NOT
  inserted into the list (which would fight uniform_list's row math). `block_mouse_except_scroll()` — NOT
  `.occlude()` — so the band is not a scroll dead-zone (the #185 inspect-HIGH precedent). Each row renders
  through the SAME line render path (syntax-highlighted, gutter-aligned) so it looks like the line it is.
  `first_visible_row = self.editor_geom.get().first` (the established read from uniform_list's range
  callback — NOT a scroll_y/line_height computation). Click on a pinned row = `open_and_place_caret` to its
  header row (#312, a NavStack push — it is a jump).
- **The interactions that must not break (REQs, not afterthoughts)** — (a) the pinned overlay must NOT eat a
  click meant for the row under the viewport top when UNPINNED (gate the overlay render on non-empty
  `sticky_rows`); (b) F8/diagnostic jumps + find-match centering account for the overlay height — center to
  `row + sticky_height` (extend the #312 deferred-center); (c) the caret scrolling INTO the pinned region
  stays visible (the scroll margin gains `sticky_height`).
- **Setting** — `editor.sticky_header = true` (default-on; the FIRST `editor.*` settings key), wired through
  `AppliedSettings` (+ `applied_from`/`applied_defaults` + a `persist_sticky_header`). A palette toggle
  command ("Toggle Sticky Header", `CommandId(14)`, the Toggle-Theme twin — add `cockpit_commands` +
  `action_for_command` + `dispatch_action` together to satisfy the completeness test).

### Out (explicitly deferred)
- **A breadcrumb BAR** (the horizontal path variant — a different surface; maybe #304's picker serves it).
- **`max_depth > 2`**, the **worker-async header extraction** (v1 uses the sync throwaway reparse per version),
  **non-Rust grammars** (inherited when #315 lands), the **terminal `sticky_block` is untouched**.

## Reference (§20)
**N/A — Marley-specific composition.** The BEHAVIOR (a sticky scope header) is a universal editor affordance:
VS Code "sticky scroll" is the OBSERVED reference (its max-region default informs `max_depth = 2`); Zed lands
it via a `BlockMap` display stack (deconstruction 03 §3) which Marley deliberately does NOT use (no
display-stack yet — the overlay shape is Marley's own). The implementation composes Marley's OWN parts: the
#329 `enclosing_ranges` node API, the shipped terminal `sticky_block` pin/overlay precedent (#185, Marley's
own code), and published `tree_sitter` node-kind reads. No reference-app source read or translated; §20 clean.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D-COLLECT → `all_headers(&session)` (full-tree walk) filtered by `Node::kind()`** (function_item/
  impl_item/mod_item/trait_item v1) — RESOLVED at design over the ancestry-walk alternative: it caches per
  version + the per-frame step is a pure `sticky_rows` Vec filter (the clean mutation surface), and it holds
  no `Tree` on the app. Returns BYTE ranges; the app maps byte→row once per version.
- **D-CACHE → header list per `(nonce, version)`, filtered per frame** — one throwaway reparse per EDIT (the
  #329 sync route), NEVER a per-scroll-frame parse; `sticky_rows` is the pure per-frame filter.
- **D-PIN → strictly-inside + header-scrolled-off, nested-stacked, `max_depth = 2`** — a header row AT the
  viewport top does not pin (the `sticky_block` no-double-render edge).
- **D-RENDER → absolute overlay, `block_mouse_except_scroll` (not occlude)** — no scroll dead-zone (#185);
  gated on non-empty `sticky_rows` (no click-eating when unpinned); the same line render path.
- **D-SCROLL → `first_visible_row = editor_geom.get().first`; center/margin = a NO-OP for v1** — RESOLVED:
  every scroll-to-row uses `ScrollStrategy::Center` (mid-viewport), so a jump target never lands at the top
  edge under the band; REQ-008/009 are satisfied with no scroll-math change (a `sticky_height` inset would
  only matter for a hypothetical top-reveal scroll, which Marley lacks).
- **D-SETTING → `editor.sticky_header` default-on + a palette toggle** (`CommandId(14)`, three sites together).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The header collector shall yield every header-bearing node (function_item/impl_item/mod_item/trait_item) with its row span, and NOTHING for code outside any such node. | pure unit cov/MSI 100 (fixtures: nested impl/fn, fn at file top, free code → empty) |
| REQ-002 | `sticky_rows(headers, first_visible_row, max_depth)` shall pin each header whose row span STRICTLY contains `first_visible_row` and whose header row is above it, innermost-nested last, capped at `max_depth`. | pure unit (boundary table) |
| REQ-003 | WHEN `first_visible_row` equals a header's own row, that header shall NOT pin (no double-render — the sticky_block edge). | pure unit |
| REQ-004 | The pinned stack shall never exceed `max_depth` (=2 v1) even when more scopes enclose the row. | pure unit |
| REQ-005 | The header collector + `sticky_rows` shall never panic on malformed/empty/degenerate input (empty file, no headers, a reversed/OOB byte). | pure unit |
| REQ-006 | The pinned rows shall render as an absolute overlay above the list (surface bg + bottom hairline), gated on non-empty `sticky_rows`, using `block_mouse_except_scroll` so the band is not a scroll dead-zone and eats no click when absent. | headless/review + LIVE(fallback) |
| REQ-007 | WHEN a pinned header row is clicked, the editor shall jump to that header row (caret placed + NavStack pushed). | headless + review |
| REQ-008 | WHEN a jump/find centers a row, the center target shall account for the sticky band height (the row lands BELOW the pinned band, visible). | headless (scroll math) + review |
| REQ-009 | The header list shall be cached per `(nonce, version)` — an idle scroll frame recomputes NO parse; a version change (edit) recomputes it. | unit (cache key) + review |
| REQ-010 | WHILE `editor.sticky_header` is false, no sticky overlay shall render; the palette toggle shall flip and persist the setting (default true). | unit (setting round-trip) + headless |

## Phase Plan
- **P2 Design** — RESOLVE D-COLLECT (all_headers vs enclosing_headers + the exact factoring vs #329) and the
  cache placement; the `sticky_rows` state machine + boundary table; the overlay render + the scroll/center/
  margin math (`sticky_height`); the settings wiring; the palette command; the file manifest; the mutation
  surface. §20 confirm.
- **P3 Implement** — the pure collector + `sticky_rows`; the app cache + render overlay + click-jump + the
  scroll math + the setting + the palette toggle; every new pure fn a direct unit.
- **P3.5 Inspect** — critics vs the diff; the pin boundary (at-edge/nested/depth-cap), the cache key, the
  overlay mouse handling (block_mouse_except_scroll, click-gate), the center/margin math, the kind filter get
  the hardest look.
- **P4 Validate** — pure units cov/MSI 100 (the collector fixtures, the `sticky_rows` boundary table, the
  setting round-trip) + headless drives (scroll a seeded 100-line fn → sticky appears/unpins at the right
  rows; click → jump; toggle off → no sticky) + the gate; live pixel drive env-blocked (screen locked) →
  units+mechanism.
- **P5 Complete** — CHANGELOG + editor.md + crate-map.md; AAR; archive; close #330.
