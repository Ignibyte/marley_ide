---
pipeline_id: 130b4894-b401-4516-8fcd-429750df7404
ticket: docs/planning/tickets/open/TICKET-427-multibuffer-core.md
status: Phase 5 — Complete PASS
title: Multibuffer core — the excerpt model + the read-only stitched view from ⌘⇧F
type: feature
milestone: M32
references:
  - docs/planning/design-notes/display-map-shelf.md
  - docs/zed_architecture/subsystems/03-editor-multibuffer.md
  - docs/zed_architecture/crates/multi_buffer.md
  - docs/zed_architecture/crates/search.md
  - docs/planning/pipeline/completed/426-soft-wrap.spec.md
---

## Title

The B-c chain's step 3 — the first multi-file surface. From a ⌘⇧F result set, ⌘Enter
(or the palette) materializes a **Multibuffer tab**: one excerpt per match (the match
line ± context, overlapping windows merged), grouped under per-file headers with
line numbers, syntax highlighting, and a highlight band on each match line; Enter/click
jumps to the real file:line. READ-ONLY this slice — #428 makes it editable; the model
stores (path, absolute rows) so it can re-anchor. A new Content kind wired exactly per
the #403 checklist.

## Scope

### In
- A pure excerpt model module (`multibuffer.rs`): `Excerpt`/`ExcerptSet` — per-file
  groups, half-open row windows, overlap merge, match-row marks, caps carried from the
  search state; the pure row model (slot → FileHeader | ExcerptLine) the render rims.
- Materialization from `OpenSearch`: text via the two-source rule (live `editors_under`
  buffers win; else disk through the viewer's stat/size/binary guard chain); per-file
  degradation on failure (flash + skip), never a panic.
- The Content/Tab wiring per the #403 checklist (exhaustive matches at every gate;
  always-insert + dropped-on-last-close lifecycle; rail section = Editor; a
  `MULTIBUFFER_TAB_TITLE`; close/release accounting; NO persistence — the writer drops
  the tab, `tab_survives_shell_entry` agrees, recorded in the wire-format doc).
- The body render: per-file header rows (the search overlay's own group-header idiom —
  repo-relative path + count) + numbered excerpt lines via the hand-lexer highlight
  path (`highlight_ranges`); match-line bands; a footer/count line surfacing drops.
- Entry points: ⌘Enter in the ⌘⇧F overlay (the ⌘↵ replace-all arm shape;
  `stop_propagation` per F-#553) + a palette command; both close transient overlays
  properly (F-#668).
- Jump: Enter/click on an excerpt line → `open_and_place_caret` + NavStack (the
  `jump_to_match` recipe), the multibuffer tab staying open.
- React-first: the POC designs the surface FIRST (`MultibufferView.tsx` from the
  FilePaneView header+gutter+lines triple + `SyntaxHighlightedLines`; ProjectSearch
  gains ⌘Enter; App state + Workspace routing + a LeftRail row).

### Out (explicitly deferred)
- Editing/write-through, cross-excerpt undo, save (#428).
- Live re-sync of excerpt text after buffer edits (v1 is a SNAPSHOT; #428's re-anchor).
- Replace-all / making the multibuffer ⌘⇧F's default results form (#429).
- The diagnostics form (#430).
- Tree-sitter whole-file highlighting for excerpts (the hand lexer ships v1; the
  parse-per-result-file cost is the recorded reason).
- Persistence across restart (documented drop).
- Routing the multibuffer through the editor's own DisplayMap/uniform_list (it renders
  its own list this slice; the singleton unification is the chain's later prize).

## Reference (§20)

**Zed (the editor reference — same-gpui-stack).** Behavior matched: project-search
results materialize as ONE scrollable surface of per-file excerpt groups — file header
rows ("── path ── N"), excerpted lines with their true line numbers, match highlights,
jump-on-activate — the multibuffer's read-only face. Cited research:
`docs/zed_architecture/crates/multi_buffer.md` (excerpt tree, headers-are-blocks,
"render as one uniform_list with excerpt-header block rows" at :219),
`docs/zed_architecture/subsystems/03-editor-multibuffer.md` §6, and
`docs/zed_architecture/crates/search.md:95-139` — whose phase split this ticket
follows deliberately: phase 1 read-only, phase 2 editable, with "results anchored to
buffers, not offsets" named as the upgrade hinge (our model stores (path, rows) now so
#428 can anchor). Clean-room: deconstruction docs only; no Zed source read.

### Prior art

1. **Behavior maps** — the three Zed chapters above; the phase-1/phase-2 search split
   is adopted as the 427/428/429 sequencing.
2. **Published** — none needed beyond the LSP/None; the grouped-results-list is a
   universal editor convention.
3. **Our permissive deps + IN-TREE (the highest-yield leg, swept 2026-08-14):**
   - **The search overlay's own per-file group headers** (app.rs:16413-16444) are the
     header idiom VERBATIM (last-file change-detection inside a windowed range,
     `rel_under_root` labels, count + `+` truncation) — adopted, not invented.
   - **`hover_run_element`'s loose-lines highlight path** (app.rs:16596-16644):
     `highlight_ranges(line, lang)` → `StyledText::with_highlights` over a bag of
     lines with no Buffer — EXACTLY the excerpt-row shape; adopted.
   - **The viewer's guard chain** (`load_code_view_state`: canonical_under_root →
     stat → size cap → binary sniff → flash) and **`editors_under`** (the ONE
     root-filtered iteration, PR-1691) own text sourcing; adopted.
   - **The #403 Browser kind** is the wiring template (the 10-file checklist recorded
     in the plan notes); `ContentKind::addable`/accessors/persist writer are already
     exhaustive-match gates (PR-closed-enum-gate), so the compiler enumerates the
     sites.
   - gpui `uniform_list`: the editor's rim (the one call site) is the virtualization
     precedent if the design picks it; ropey/regex/tree-sitter: no owner of excerpts.

## React-first (parity)

**UI-AFFECTING — a NEW surface, designed React-first (the #418 inversion: the POC is
the design source; it becomes a parity row once shipped).** POC files:
`components/views/MultibufferView.tsx` (NEW — the FilePaneView header+gutter+lines
triple + `SyntaxHighlightedLines`, read-only), `overlays/ProjectSearch.tsx` (⌘Enter →
materialize), `App.tsx` (state member), `pages/Workspace.tsx` (routing + open helper),
`components/LeftRail.tsx` (the tab row under EDITOR). Implement builds + screenshots
the POC FIRST (dev server, READ the PNG), then ports 1:1; validate captures the
parity pair; complete adds the MARLEY-PARITY.md row.

## Locked-In Decisions

- **D1 — A pure model module owns everything decidable.** `multibuffer.rs`
  (cov/MSI-100): the excerpt builder (matches → merged half-open context windows,
  AD-claude-305), the row model (slot → Header(file) | Line(file, row)), labels,
  caps/drop accounting. `app.rs` gets only wiring (the standing shim rule).
- **D2 — The #403 checklist, exhaustively.** Every closed-enum gate (Content,
  ContentKind, the five accessors, TabContent, rail_section, key_context, persist
  writer, restore, close/release, status bar) gains its arm — the compiler is the
  site enumerator. Lifecycle: ALWAYS-INSERT, dropped-on-last-close (each
  materialization is a fresh result set; no dedupe key exists — the sweep's
  recommendation recorded).
- **D3 — v1 is a read-only SNAPSHOT.** Text captured at materialization via the
  two-source rule (dirty-buffer-wins through `editors_under`; else the viewer guard
  chain). The model stores (canonical path, absolute first_row, match rows) so #428
  re-anchors. No live sync this slice.
- **D4 — The hand lexer highlights excerpts.** `highlight_ranges` per line (the
  hover-fence precedent); tree-sitter whole-file parses per result file are the
  recorded deferral (render-thread cost).
- **D5 — Context = ±2 lines, merged.** Overlapping/adjacent windows within a file
  merge into one excerpt (half-open); the constant lives in the pure module.
- **D6 — No persistence.** The persist writer maps the tab to a drop;
  `tab_survives_shell_entry` agrees; the wire-format doc records it. (PR-1323: a
  future payload would be shell-codec framing-safe, but v1 re-materializes from
  nothing — the tab simply doesn't survive.)
- **D7 — Entry points.** ⌘Enter in the search overlay (guarded arm BEFORE plain
  enter; `stop_propagation`); a palette command acting on the LIVE result set (no
  results → status flash). Both run the transient-overlay close discipline.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the ⌘⇧F overlay holds results and ⌘Enter is pressed (or the palette command runs), the system shall open a Multibuffer tab in the Editor section containing every result file's excerpts and close the overlay. | Headless drive (seeded results → verb → tab exists, overlay gone). |
| REQ-002 | Each file group shall render a header row (repo-relative path + match count, `+` when truncated) followed by excerpt lines numbered with their TRUE file line numbers, syntax-highlighted, with a band on every match line. | Pure row-model units + headless render asserts + the live capture. |
| REQ-003 | WHEN Enter/click activates an excerpt line, the system shall open that file with the caret at the match position (NavStack pushed) while the multibuffer tab stays open. | Headless drive. |
| REQ-004 | Excerpt text shall come from the live buffer when the file is open (dirty-buffer-wins) and from disk (stat/size/binary-guarded) otherwise; a per-file failure shall skip that file with a status flash, never a panic. | Units + a drive with a dirty open buffer + a failure-arm unit. |
| REQ-005 | Overlapping or adjacent context windows within one file shall merge into a single excerpt with no duplicated or reordered rows (half-open runs). | Pure unit table (PR-1370 discriminating fixtures). |
| REQ-006 | Closing the tab shall release every registry view (no leaks), and the tab shall NOT reappear after a persist/restore cycle. | Headless close drive + a restore drive asserting absence. |
| REQ-007 | The row model shall be pure and total: every slot maps to exactly one Header/Line, boundaries at group edges pinned; the caps and drop counts from the search state shall surface in the surface's footer. | Units (boundary probes) + render assert. |
| REQ-008 | The new pure surface shall hold 100% coverage / 100% MSI; the gate shall be GREEN [diff]. | gate:4/5; `scripts/gates.sh --diff`. |

## Phase Plan

- **P2 Design** — the excerpt/row-model types; the exact #403-checklist site list; the
  render shape (uniform_list vs windowed — pick with the rim discipline either way);
  the ⌘Enter arm + palette id; the POC build plan; the test table per REQ.
- **P3 Implement** — POC FIRST (build + screenshot + READ), then Rust to the manifest.
- **P3.5 Inspect** — critics on: merge math, the two-source rule, registry accounting
  (the acquired-view contract), persistence drop, overlay-arm discipline, provenance.
- **P4 Validate** — the REQ tests written + run; live drive + parity pair; gate green.
- **P5 Complete** — CHANGELOG + architecture docs (editor.md/crate-map/roadmap);
  MARLEY-PARITY row; ledger capture; close + archive.
