---
pipeline_id: 2a6b5293-2778-43e6-ab6d-deb975146f79
ticket: forge#305 (13b140d5-ca11-44a6-80c3-29bc6dbac0f3) · local docs/planning/tickets/open/TICKET-305-code-folding.md
aar_id: 9bd998a9-ccfc-48f8-953d-2428ca72a3db
status: Phase 5 — Complete PASS
title: Code folding — chevrons, ⌥⌘[/⌥⌘], and THE first visible↔buffer row projection
type: feature
milestone: M19
references: [the 17-site visible==buffer-row blast radius (recon 2026-07-17, app.rs:4592-5236), nav.rs FoldState/fold_visible_rows (#184 — the flat projection precedent), FileTree::visible_rows/path_at (#293 — the tree one), anchor.rs (SHIPPED, zero production consumers — folds are the FIRST), the #331 Inlay channel (the "⋯ N lines" marker mechanism), scroll_editor_to_row = scroll_to_item(SLOT) (app.rs:11029), the (nonce,version) memo (app.rs:3296), #349 cached-tree, PR-claude-language-specific-pure-primitive-needs-caller-side-gate-001]
---

## Title
Fold a function body to one line — chevrons in the gutter, ⌥⌘[ folds the innermost region at the caret,
⌥⌘] unfolds, a "⋯ N lines" tail on the header. **The real ticket is not the chevron — it is the FIRST
projection between buffer rows and rendered rows.** Today `uniform_list("editor-lines", len_lines(), …)`
hands the slot index straight to `line_text(row)`, and a recon sweep found **~17 sites** that silently
assume visible-row == buffer-row (inlays, syntax spans, find bands, bracket localize, carets, squiggles,
git lane, diagnostics tint, gutter numbers, click, drag, scroll-to-row, caret-follow, frame geom, sticky
headers). Hiding rows severs that identity everywhere at once — unless the conversion happens exactly ONCE.

## Scope
### In
- **The pure fold model (`marley_syntax`, cov/MSI 100):** `fold_regions(src) -> Vec<FoldRegion {
  header_row, end_row }>` — multi-line nodes from a FOLD_KINDS table (v1: `function_item`, `impl_item`,
  `mod_item`, `trait_item`, `struct_item`, `enum_item`, `match_expression`), document order, nested kept.
  An ITERATIVE cursor walk (the all_headers shape — its 2000-level regression pins why). Parse-only free
  fn (no cached tree app-side; the same throwaway cost #340/#330 pay, memoized; #349 collapses it).
- **THE PROJECTION (the heart, pure, the mutation surface):** `FoldProjection` built from the resolved
  folded ranges — `visible_count()`, `buffer_row(slot) -> usize`, `slot_of(buffer_row) -> usize` (a hidden
  row maps to its fold-header's slot). **D-PROJECT-AT-THE-BOUNDARY:** the conversion happens exactly once
  at the `uniform_list` rim — `total = visible_count()`; first line of the row callback: `let row =
  proj.buffer_row(slot)` — and every one of the ~17 interior sites keeps buffer-row semantics UNTOUCHED.
  The three OUTBOUND crossings convert too: `scroll_editor_to_row(buffer_row)` → `scroll_to_item(
  slot_of(row))` (gpui scrolls SLOTS); the frame-geom `first/last` (recorded as slots) → buffer rows
  before `sticky_rows` consumes them; caret-follow reads the caret's buffer row → slot.
- **Fold STATE rides ANCHORS:** each active fold keyed by its header's `Anchor` (anchor.rs — SHIPPED with
  full bias rules, `anchor_at`/`resolve_anchor`… and **zero production consumers today; folds are the
  FIRST, a named risk**). Resolved per (nonce, version); typing ABOVE a fold keeps it on the same region;
  a region deleted by an edit resolves to nothing and the fold evaporates. Per-file (nonce-keyed) state,
  survives tab switches in-session.
- **AUTO-REVEAL (correctness, not polish):** ANY caret placement into a hidden row — find-next, F8,
  go-to-definition, go-to-line, a symbol jump — unfolds its containing folds first. Without this, every
  shipped navigation feature silently breaks against a fold.
- **Render:** the gutter chevron (▾ open / ▸ folded) on foldable header rows — a NEW per-row gutter click
  target with `stop_propagation` (**the gutter has NO mouse handler today; the row-wide `on_mouse_down`
  would treat the click as a col-0 caret click** — the regression to pin). The "⋯ N lines" tail renders
  through the SHIPPED #331 Inlay channel (phantom EOL text is exactly what it is — no new render path).
- **Chords + palette:** ⌥⌘[ fold-innermost-at-caret / ⌥⌘] unfold — verified FREE, Editor-scoped (roster
  67→69, scoped 22→24). **Fold All / Unfold All are PALETTE-ONLY v1** — the keymap has no multi-stroke
  machinery, and ⌘K-prefix sequences would be a new subsystem (named cut, not a silent one).
- **The language gate at the CALLER** (`language_of == Rust`) — the pure fn parses Rust unconditionally
  (#340 M1; the ladder/sticky ungated-parse gap is a separate pre-existing bug, filed at promotion).
### Out (explicitly)
- **The LSP overlay-card geometry above a fold → SLICE 2 (forge #352 — SHIPPED 2026-08-04, M28; the
  known-limit is closed).** The 5 cards positioned by
  `(buffer_row - editor_geom.first) * cell_h` (hover/completion/signature/rename/hover-dwell) + the content-width
  probe do NOT project buffer↔visible rows — with a fold above the caret on-screen they anchor at the wrong Y.
  A DOCUMENTED KNOWN-LIMIT of this ticket (a bounded VISUAL imperfection, not a correctness break of
  editing/navigation); the Phase-1 sweep found the "3 outbound crossings" undercounted the geom-fanout, so the
  overlay-geometry threading is a separate bounded follow-up (#352). This ticket = the core fold engine.
- Fold persistence across restart (the #163 grid codec could carry it later; named). `#region` comment
  markers. Fold-by-level commands. Multi-stroke chord machinery. Folding in the #246 read-only split pane
  (#259 unifies surfaces first). Non-Rust (#315 extends FOLD_KINDS per grammar). Soft-wrap interplay (the
  display-map chain owns intra-line transforms; THIS ticket is whole-row hiding — deliberately disjoint).

## Reference (§20)
VS Code / Zed = OBSERVED (chevrons, the ⋯ tail, click-to-toggle, auto-reveal on navigate, folds surviving
edits above). tree-sitter = published-API reuse. The projection + anchors are Marley-original over shipped
seams. Zed's FoldMap is an architecture CONCEPT only (source unread — §20); Marley's per-row uniform_list
admits a far smaller row-projection instead of a display-map layer.

### Prior art
1. **Behavior maps / observed** — the folding behaviors above; VS Code auto-reveals on any jump into a fold.
2. **Published material** — tree-sitter's cursor-walk docs (the all_headers idiom this reuses).
3. **OUR OWN CODE + PERMISSIVE DEPS — the sweep paid four times:** (a) `nav.rs` `FoldState` +
   `fold_visible_rows` (#184) is the shipped FLAT projection precedent (visible Vec + `.get(row)` map-back)
   and `FileTree::visible_rows`/`path_at` (#293) the tree one — the projection shape is proven in-repo
   twice; (b) `anchor.rs` ships the exact edit-rebase capability folds need (bias table verified) — unused
   until now; (c) the **#331 Inlay channel already renders phantom EOL text** — the "⋯ N lines" tail needs
   no new mechanism; (d) `enclosing_ranges`' own doc names #305 as an intended reuser. gpui's
   `uniform_list` takes an arbitrary item count — the projection slots in at the rim with no gpui change.

## Locked-In Decisions (design confirms; deltas → the notes)
- **D-PROJECT-AT-THE-BOUNDARY** — one conversion at the uniform_list rim + the three outbound crossings;
  the 17 interior sites stay buffer-row. (The alternative — teaching each site — is the bug factory.)
- **D-ANCHOR-KEYED** — folds survive edits via anchors; first-consumer risk NAMED (expect anchor bugs to
  surface here; they are anchor bugs, not fold bugs — fix at source in anchor.rs).
- **D-AUTO-REVEAL-ON-CARET** — every caret-placement path reveals; hook the SHARED primitive (the #336
  lesson), not each caller.
- **D-MARKER-IS-AN-INLAY** — the ⋯ tail through the #331 phantom channel.
- **D-PALETTE-ONLY-FOLD-ALL** — no multi-stroke keymap machinery v1.
- **D-CALLER-GATES-LANGUAGE** — Rust-only v1 (#340 M1).

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | extract foldable regions (fn/impl/mod/trait/struct/enum/match bodies spanning >1 line) with header/end rows, document order, nesting kept | pure fixture |
| REQ-002 | project rows: visible_count + buffer_row(slot) + slot_of(row), with a hidden row's slot_of = its header's slot — a bijection over visible rows | pure table — the mutation surface |
| REQ-003 | hide a folded region's rows, render "⋯ N lines" on the header, and restore EXACTLY on unfold | headless state + deferred pixel |
| REQ-004 | fold the innermost region containing the caret on ⌥⌘[ and unfold on ⌥⌘] | headless |
| REQ-005 | keep a fold on ITS region when text is typed above it (anchors carry) | headless — the anchors row |
| REQ-006 | auto-reveal when any navigation places the caret in a hidden row (find/F8/goto) | headless |
| REQ-007 | toggle a fold on gutter-chevron click WITHOUT moving the caret (stop_propagation before the row handler) | headless |
| REQ-008 | scroll to the correct SLOT when scroll_editor_to_row targets a buffer row past folds; sticky headers consume converted buffer rows | headless |
| REQ-009 | expose Fold All / Unfold All as palette verbs that resolve (every_cockpit_command_resolves) | unit |
| REQ-010 | do no walk and show no chevrons on a non-Rust file (the language gate) | headless — the #340 M1 row |

## Phase Plan
P2 confirm the FOLD_KINDS table on real parses (spike) + the projection's exact seam (where `slot` enters)
+ the auto-reveal hook point (the shared caret-placement primitive) + the chevron hit-target geometry; P3
the pure model first (fold_regions + FoldProjection truth tables), then state/anchors, then the rim
conversion + render + chords; P3.5 critics on the THREE outbound crossings (scroll/geom/sticky — the ones
a green interior can't prove), the anchor first-consumer edges (delete-across-header, undo past the fold),
the click-vs-caret propagation; P4 tables + headless drives + gate; P5 docs (editor.md's projection
section — the display-map chain's B-c foundation cites this as its precursor).
Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
