# The display-map sprint — M32, the B-c chain (2026-08-14)

Chad's directive, verbatim: **"/work 425 to 430 auto approved until finished."** The Queue
was empty and no 425+ tickets existed, so this sprint was minted from the strongest standing
signal of "next": the roadmap's own words — *"the whole remaining M22 surface is the B-c
display-map chain (foundation → soft wrap → multibuffer)"* — and the M22 batch plan's items
13–16 (`m22-editing-bar.md`), of which only folding (#305, item 15) has shipped. The
integrity shelf was run explicitly so that "B-c … deserves a clean, trustworthy gate under
it"; that gate landed (#345/#348/#407 measured-FULL green, #423 pinned toolchain), so the
runway B-c was waiting for is clear. Content chosen by the agent from the roadmap under the
auto-approve directive, not discussed live — **Chad: if 425–430 meant a different six, say
so and this shelf re-targets.**

## The shelf

| Ticket | Type | One line |
|---|---|---|
| [TICKET-425](../tickets/open/TICKET-425-display-map-foundation.md) | feature | The display-map foundation — ONE composable buffer↔display transform stack (folds today, wrap/excerpts tomorrow); byte-identical UI, proven by the existing suite |
| [TICKET-426](../tickets/open/TICKET-426-soft-wrap.md) | feature | Soft wrap — one buffer line → N display rows at the pane width; `editor.soft_wrap` + palette toggle; OFF byte-identical; must revisit the #336 shift site, not layer on it |
| [TICKET-427](../tickets/open/TICKET-427-multibuffer-core.md) | feature | Multibuffer core — the excerpt model over N Buffers + the read-only stitched view (per-file headers, syntax, jump-to-source); ⌘⇧F grows "open results as multibuffer" |
| [TICKET-428](../tickets/open/TICKET-428-multibuffer-editing.md) | feature | Editable multibuffer — typing/undo/save in an excerpt writes through to the real file; cross-excerpt undo is one story; the #275 conflict net holds |
| [TICKET-429](../tickets/open/TICKET-429-search-results-multibuffer.md) | feature | ⌘⇧F phase 2 — results ARE the editable multibuffer; Replace All (with #347's capture groups) lands project-wide as one undoable, save-orchestrated apply |
| [TICKET-430](../tickets/open/TICKET-430-problems-multibuffer.md) | feature | Problems panel, editable form — diagnostics as excerpts you fix in place; the #327 panel's named consumer |

## Order + method

**Strictly 425 → 426 → 427 → 428 → 429 → 430** — each rewrites the seams the next one
reads. That dependency is also why this shelf deliberately does NOT pre-author the six
specs into `pipeline/queued/` (the m22 shelf method): pre-authoring against pre-425 code
would bake in citations each preceding ticket invalidates, and the method's own recorded
lesson is that the confident pre-authored sentences are the dangerous ones. Instead each
`/pipeline:plan` authors its spec fresh at promotion, against the tree the prior ticket
left. Sizes: 425 L · 426 L · 427 M–L · 428 L · 429 M · 430 M.

## Reference posture (§20)

- **Zed is the behavior reference** (same-gpui-stack): `docs/zed_architecture/subsystems/`
  `03-editor-multibuffer.md` §3 (the DisplayMap six-layer transform stack; the uniform
  layer contract — Transform runs + input/output summaries + snapshot + sync) and §6 (the
  multibuffer: excerpt tree, composed anchors, the singleton unification, headers-are-blocks).
  `[Zed-derived]` idea / `[public]` pattern per that doc's provenance tags; research docs,
  never source. The roadmap's standing caveat holds: *"Marley's per-line render admits a
  smaller shape"* — adopt the CONTRACT, not the container (no SumTree obligation).
- **Prior art**: gpui (Apache) text/layout APIs, ropey, and what #305/#331/#336 already
  built IN-TREE are the first sweep targets — the display map generalizes shipped Marley
  code more than it imports anything.

## Load-bearing findings (Explore sweep, 2026-08-14, `main` @ `52a4c4d`)

- **The rim is real and single**: the `uniform_list` row callback converts ONCE —
  `proj.buffer_row(slot)` at `app.rs:6735`, count `proj.visible_count()` at `:6729`
  (`D-PROJECT-AT-THE-BOUNDARY`, comment `:6726-6728`). `FoldProjection`
  (`crates/syntax/src/fold.rs`, 487 lines, pure, cov/MSI 100) is **the only coordinate
  transform in the tree** — already half-open runs; `slot_of` snaps hidden→header;
  `viewport_offset` (#352) anchors the five overlay cards + IME.
- **14 production crossing sites** for the projection in `app.rs` (row count, rim, headers
  inlay, content-width probe, LSP inlay viewport, sticky band, `scroll_editor_to`,
  hover-dwell inverse, 5× `viewport_offset` anchors, `character_index_for_point`) + ~19
  interior sites that read the projected row and assume row==line (gutter label
  `gutter_label(row+1)` at `:7052`, caret bars `top(0)` `:6969-6979`, per-row
  `line_layout_with_inlays` `:6765`, selection/find/bracket bands, git lane, squiggles,
  click/drag reading `line_text(row)`).
- **TWO shift sites, not one**: the code row's `ml(px(-scroll_x))` at `app.rs:6911`
  (clipper at `:7059-7090`) AND the sticky band's own shift at `app.rs:7391` (its clipper
  `:7386`) — the #336 AD's "revisit the shift site" is plural in practice.
- **The column model**: `code_view.rs:41-57` `LineLayout { display, col_starts, col_ends,
  hint_spans }` with the two-boundary accessors `col_of_offset` (caret map, `:72`) /
  `col_of_span_end` (code map, `:88`) — the #331 invariant is API-shaped already;
  `display_cols()` (`:143`) is the #336 cell-width authority.
- **Gate topology**: `app.rs` (23,098 lines) is the coverage-excluded, `mutants::skip`
  SHIM; the pure files it composes (`code_view.rs` 1,817 · `h_scroll.rs` 600 · `fold.rs`
  487 · `editor_search.rs` 156 · `editor_problems.rs` 149 · `sticky_header.rs` 133) carry
  cov/MSI 100. **Every new display-map/wrap/excerpt mechanism must land as pure code in a
  pure file**; app.rs gets only the wiring.
- **Search today**: the ⌘⇧F overlay is a read-only modal card (`search_overlay`
  `app.rs:16000-16111`, preview `String`s, no Buffer); the replace ENGINE already exists
  single-file (`marley_editor` `find.rs:114` `replace_all` / `:286` regex / `:344`
  `replace_all_with` + `resume_after`); results model `OpenSearch`/`SearchFile`
  (`app.rs:933-973`) over `marley_project::search_lines` (`LineMatch{row,col,len,preview}`,
  char-domain). CHANGELOG:2390 records the phase-2 deferral this sprint pays.
- **Problems today**: `ProblemRow{path, line, character, …}` carries RAW LSP positions in
  the owning host's negotiated encoding; `jump_to_problem` (`app.rs:14641-14672`) resolves
  the host by canonical-root match + `position_to_offset` — the diagnostics-multibuffer
  must keep that encoding bridge per excerpt. CHANGELOG:2371 records the deferral.
- **The read-only viewer twin** (`app.rs:7259-7282`, the #246 pane path) has NO projection
  / no `scroll_x` / no geom — wrap's scope decision (editor list only vs viewer too) is a
  design fork to pin explicitly.
- **Zed blueprint anchors**: the 6-layer stack + per-layer notes
  (`docs/zed_architecture/crates/editor.md:79-100`), the singleton unification (`:211`),
  Marley layer order (`:254-260`), and — directly validating 427's shape — *"render as one
  `uniform_list` with excerpt-header block rows between files"*
  (`docs/zed_architecture/crates/multi_buffer.md:219`).

## Recall pins (knowledge ledger, read before each plan)

- **AD-claude-305 (half-open runs)** — "Future projection code (soft-wrap display map,
  multibuffer excerpts) should prefer half-open runs from the start"; boundaries land on
  the exclusive end so boundary mutants stay killable. Binding on 425's core types.
- **AD-claude-two-boundary-maps-for-phantom-text-001 (#331)** — code spans hug the code,
  caret ranges track the caret; every consumer classifies into exactly one accessor. The
  stack must EXPOSE this fork, not flatten it.
- **AD (#336 h-scroll clipper/shift)** — "Binding on the display map (B-c): soft wrap
  breaks row↔line identity … the wrap work must revisit the shift site, not layer on top
  of it." Also: the two geometry probes live in OPPOSITE domains on purpose (x0 rides the
  shift, code_w sits on the clipper); and the rejected `Unconstrained` road becomes the
  better engineering "if the layout ever changes" — wrap IS that change; re-weigh it.
- **PR-claude-every-caret-path-must-reach-the-reveal-hook-001 (F-#305)** — motions and
  jumps take DIFFERENT sinks; enumerate every caret-placement path when a display feature
  hooks "the shared primitive". Wrap + multibuffer both add such features.
- **F-#352 (row-vs-slot equality gate)** — `row == first` compared a buffer row to a slot
  and froze `EditorFrameGeom` whenever a fold sat above the viewport. Every new row space
  multiplies this class; typed row newtypes at the seams are the countermeasure.
- **PR-claude-identity-claims-over-display-maps-probe-zero-width-boundaries-001** — any
  "byte-identical" claim across a display layer gets probed at zero-width char boundaries.
  425's whole acceptance is a byte-identical claim: probe it there.
- **F-#386 shape** — widening a hot signature breaks `#[cfg(test)]` call sites plain
  `cargo check` never compiles; verify with `cargo check --tests` after every seam change.

## React-first (parity)

Zone A's editor surface row (`components/EditorView.tsx` ↔ `editor_surface.rs`/
`code_view.rs`) plus `overlays/ProjectSearch.tsx` and `overlays/ProblemsPanel.tsx` are the
twins. 425 is `N/A — no UI delta` (a byte-identical refactor). 426–430 are UI-affecting:
wrap behavior and the multibuffer surface are designed React-first in the POC (Vite,
screenshot, READ the PNG), then ported 1:1; MARLEY-PARITY.md gains the new-surface rows as
each ships.
