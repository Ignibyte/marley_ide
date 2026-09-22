---
pipeline_id: 682e87d7-28cc-4c02-bb89-83ee3e16c825
ticket: forge#266 (4f9c9b01-73c8-442d-9eb6-d64c3b7563fd) · local docs/planning/tickets/open/TICKET-266-b2-render.md
aar_id: 7f891f5a-8208-4474-889e-0c41264d288d
status: Phase 5 — Complete PASS
title: B2 render — the editor tab on uniform_list + StyledText.with_highlights
type: feature
milestone: M16
references:
  - docs/zed_architecture/crates/gpui.md
---

## Title
Replace the editable editor tab's hand-rolled render (a fixed-40-row window
of per-line flex divs, one colored div per syntax span, hand column math)
with the gpui-native substrate: `uniform_list` (true viewport
virtualization over `buffer.len_lines()` — retiring the editor's
`VIEWER_ROWS=40` cap) + one `StyledText::with_highlights` per row (syntax
tints + the #255 selection as `HighlightStyle` ranges) + `TextLayout`
geometry (`position_for_index` for the caret bar, `index_for_position` for
click/drag mapping) + a `UniformListScrollHandle` (retiring the editor
wheel-handler/`scroll_code` path). All four APIs verified present in the
vendored gpui 0.2.2. This is the substrate B3 tree-sitter renders into.
(Plan correction from the render map: the editor NEVER used
`split_at_caret` — that idiom is the terminal prompt's; the editor caret is
already an overlay bar, which stays, repositioned via TextLayout.)

## Scope
### In
- app.rs `code_view_body`'s EDITOR branch only → the uniform_list render
  (per-row StyledText, selection via `HighlightStyle.background_color` over
  the selected byte ranges [matches today's chars-only tint], caret overlay
  positioned by `position_for_index`, per-row TextLayout capture for the
  #254/#255 mouse mapping [byte→CharOffset], listeners rebuilt through the
  entity handle inside the list closure).
- A pure, tested span adapter: `highlight_ranges(display, lang) ->
  Vec<(Range<usize> /*bytes*/, TokenKind)>` (folding `highlight_line`'s
  sequential chunks into byte ranges; Plain dropped).
- Scroll: `UniformListScrollHandle` + `.track_scroll` replaces the editor
  wheel handler + `code_scroll_remainder` use for the editor tab;
  `line_layout` STAYS (tab expansion is the app's job — StyledText renders
  literal \t).
- Keep `offset_for_click`/`line_layout` tests as the column-mapping oracle.
- Driven + headless verification (typing, selection highlight, caret
  position, click-to-caret, a >40-line file scrolling past row 40).

### Out (explicitly deferred)
- The #246 read-only pane branch (the `None` branch keeps the old render;
  #268 coordinates the highlight retirement across both).
- Caret-follow scroll (#270 — the handle makes it cheap; not this slice).
- The terminal prompt's `split_caret_char` render (a different, working
  path); `split_at_caret`'s dead-code removal IS in (trivial, input.rs).
- tree-sitter (#268); scroll-position persistence.

## Reference (§20)
**Zed (the editor reference)** — behavior matched: the virtualized
line-list + styled-text-run editor rendering model (one shaped text element
per visible line, highlights as ranged styles, geometry queries against the
shaped line) as described behaviorally in `docs/zed_architecture/crates/
gpui.md` (the 1:1 element map). Clean-room: the MECHANISM is gpui's own
Apache-2.0 public element API consumed as shipped; no GPL editor source is
read — Marley's row closure, span adapter, and mouse plumbing are original.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — Editor branch only; the shared fn forks internally (the read-only
  branch byte-identical).
- D2 — Selection = `HighlightStyle.background_color` ranges (28% accent, as
  today); the full-height/past-EOL look is explicitly NOT kept (today's v1
  already tints chars only — the render map confirmed parity).
- D3 (revised at design) — Caret stays an overlay div positioned by the
  PURE monospace cell math (`ccol * cell.w` — exact for the mono grid over
  the tab-expanded display, already tested); `position_for_index` is
  deferred until a proportional font/ligatures exist (it requires
  post-paint layout capture and buys nothing on a mono grid).
- D4 (revised at design) — Click/drag mapping keeps the pure stateless
  `x0 + round(rel/cell_w)` → `offset_for_click` path for the same reason;
  `index_for_position` deferred with D3.
- D5 — All byte↔char conversions happen at the seam edges; the editor
  model stays CharOffset-pure.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `highlight_ranges` shall fold `highlight_line`'s chunks into contiguous byte ranges covering the display string, dropping Plain, preserving order/kinds (multibyte + tab-expanded inputs included). | pure unit tests, mutants killed |
| REQ-002 | The editor tab shall render >40-line files scrollable to EOF (the 40-row cap gone), with the visible slice chosen by the real viewport. | headless boot + driven capture on a 100-line file |
| REQ-003 | Typing, backspace, undo/redo shall render identically (text content per row unchanged vs the buffer). | headless_drive keystroke asserts + captures |
| REQ-004 | The #255 selection shall render as a background tint over exactly the selected chars per row (multibyte-safe). | driven capture (⌘D on a word — reusing the #265 flow) + unit (range math) |
| REQ-005 | The caret bar shall sit at the caret's glyph x-position (incl. after tabs + multibyte). | driven capture + the oracle tests |
| REQ-006 | Mousedown/drag shall set caret/selection at the clicked glyph (byte→char correct on multibyte lines). | driven capture (click a word mid-line) |
| REQ-007 | The scroll wheel shall scroll the editor list (handle-driven); the read-only #246 pane shall be pixel-unchanged. | driven captures both surfaces |

## Phase Plan
- **P2 Design** — the exact closure structure (entity plumbing, per-row
  layout registry), adapter signature, wheel/scroll wiring, manifest, tests.
- **P3 Implement** — adapter + render swap + scroll + mouse.
- **P3.5 Inspect** — critics (byte/char seams, first-frame layout None,
  listener leaks, read-only-branch untouched, §20).
- **P4 Validate** — units + mutants; headless lane extends; driven captures;
  gate --diff.
- **P5 Complete** — CHANGELOG, editor.md/app_shell.md, AAR, archive, close.
