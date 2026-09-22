---
pipeline_id: 116d5595-acd7-4ead-a9b3-e73d089f2a9b
ticket: forge#310 (d82fad8b-f689-440c-9824-6fb61c1cff6a) · local docs/planning/tickets/open/TICKET-310-lsp-diagnostics.md
aar_id: e7c2437d-2428-4afb-b1d1-c6ae46a50790
status: Phase 5 — Complete PASS
title: LSP diagnostics — publishDiagnostics + squiggles + counts, merged into the M18 gutter/F8 lane
type: feature
milestone: M20
references: [docs/planning/pipeline/completed/309-lsp-document-sync.spec.md, docs/planning/pipeline/completed/308-lsp-client-core.spec.md, docs/zed_architecture/subsystems/05-lsp-language-intelligence.md]
---

## Title
**Real compiler truth in the editor as you type — the first user-visible LSP payoff.** rust-analyzer
pushes `textDocument/publishDiagnostics`; Marley already lights a gutter from a failed TERMINAL
block's file:line refs (M18 #289-291) and walks it with F8 (#290). LSP becomes a SECOND, richer
producer into the SAME lane — not a parallel mechanism — adding severity, a message, and a squiggle
range under the offending span, mapped through #309's position bridge.

## Scope

### In
- **The diagnostic store (PURE, `marley_lsp` or a new app module)** — per-uri `replace(uri, Vec<Diag>)`
  with REPLACE/CLEAR semantics (a new publish for a uri replaces the prior set; an EMPTY publish
  clears it — missing this is the classic stale-squiggle bug).
- **A Marley `Diag` value** `{ row: usize, span: Range<CharOffset>, severity: Severity, message:
    String, source: Lsp|Terminal }` — the app never sees `lsp-types`. `lsp-types` is re-added,
    CONFINED to a `diagnostics` parse seam: `parse_publish_diagnostics(params, text, encoding) ->
    (uri, Vec<Diag>)` maps each LSP `Diagnostic { range, severity, message }` to a `Diag` via #309's
    `position_to_offset` (start+end) under the negotiated encoding.
- **`merged_diagnostics(lsp: &[Diag], terminal_rows: &[usize]) -> Vec<Diag>` (PURE)** — the LSP rows
  + the M18 failed-block rows (as `source: Terminal`, severity Error, no span/message), deduped per
  row, severity-ranked (Error > Warning > Info > Hint). The gutter + F8 consume `merged.rows()`.
- **`underline_runs(diags, line_texts, encoding) -> Vec<(row, start_col, end_col)>` (PURE)** — a
  diagnostic's span → per-row underline spans (a multi-line span splits per row; a ZERO-WIDTH span
  renders a 1-char-wide squiggle, the LSP convention), for the render's squiggle paint.
- **Status-bar counts** — `diagnostic_summary(merged) -> Option<String>` = `"N errors, M warnings"`
  for the focused file; `0/0` → `None` (no segment).
- **App wiring (SHIM)** — the pump captures `publishDiagnostics` → the store (set `dirty` — the #203
  repaint rule); `open_file_diagnostic_rows` now unions the M18 terminal rows with the store's rows
  for the open file (feeds the shipped gutter tint + F8); the code_view render paints squiggle
  underlines (severity color) under the spans; the status bar shows the counts; hovering a squiggle
  (or the caret on a diagnostic row) shows the message in the #221 rounded overlay card (plain text
  v1 — markdown hover is #311).

### Out (explicitly deferred)
- Code actions / quick fixes (a later ticket); `relatedInformation`; diagnostic TAGS
  (deprecated/unnecessary dimming); the pull-diagnostics model (`textDocument/diagnostic`).
- A project-wide diagnostics PANEL (only the focused file's squiggles + the merged gutter row set).
- Full markdown in the hover (that's #311's `markdown_runs`).

## Reference (§20)
**Zed (the editor reference) — `docs/zed_architecture/subsystems/05-lsp-language-intelligence.md`
(the `diagnostics` UI + `publishDiagnostics` fan-in, and the UTF-16→anchor clip that puts a squiggle
on the right span); implementation clean-room from the published LSP 3.17 `publishDiagnostics` spec.**
Behavior matched: a diagnostic appears under the offending span as you type and clears when fixed,
error vs warning are visually distinct, and the gutter/F8 navigation already built for terminal
failures now also walks compiler diagnostics — ONE lane, richer source. The Marley-original wedge:
LSP diagnostics MERGE with the command-Block failed-run rows (Zed has no terminal-block lane).

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — LSP is a SECOND producer into the M18 lane, not a parallel one.** `merged_diagnostics` is
  the one set the gutter + F8 read; #310 must not regress the M18 terminal-row path (or #295's
  multi-terminal-grid fix, still open).
- **D2 — REPLACE/CLEAR per uri** — an empty publish CLEARS (pin the stale-squiggle case).
- **D3 — the app never sees `lsp-types`** — it is confined to the `parse_publish_diagnostics` seam;
  the app consumes the pure Marley `Diag`. Spans are `CharOffset` (buffer coords) via #309's bridge.
- **D4 — severity → ThemeColors (v1: 2-tier, no new role).** Error → `danger`; Warning/Info/Hint →
  `muted`. No amber `warning` role in #310 (adding a ThemeColors field expands into every palette);
  the dedicated warning color lands with **#316 (the syntax THEME system, same train)**, which
  reworks the whole palette + roles. No ad-hoc colors — only shipped roles. The squiggle is a COLORED
  UNDERLINE via `HighlightStyle.underline` (a wavy squiggle is a future custom-paint polish).
- **D5 — house seam split** — store/merge/underline-runs/counts/parse = pure cov/MSI 100; the pump
  capture + the squiggle/gutter/status/hover render = the masked shim.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a `publishDiagnostics` for a uri arrives, the store shall REPLACE that uri's set; WHEN the published set is empty, it shall CLEAR that uri. | pure store units (incl. the empty-clears pin) |
| REQ-002 | WHEN an LSP `Diagnostic` range is mapped, the resulting `Diag.span` shall be the buffer `CharOffset` range for that (line, character) range under the negotiated encoding (emoji-column correct). | pure parse units (via #309 bridge) |
| REQ-003 | WHEN LSP diagnostics and M18 terminal rows coexist, `merged_diagnostics` shall union them, dedupe per row, and rank Error > Warning > Info > Hint. | pure merge units |
| REQ-004 | WHEN a diagnostic span crosses N lines (or is zero-width), `underline_runs` shall yield one run per covered row (a zero-width span → a 1-char run). | pure underline-run units |
| REQ-005 | WHEN the focused file has E errors and W warnings, the status bar shall read `"E errors, W warnings"`; WHEN E=W=0 it shall show no diagnostic segment. | pure count unit + driven |
| REQ-006 | WHEN F8/⇧F8 is pressed, the caret shall walk the MERGED row set (LSP + terminal) in row order, wrapping. | pure next/prev over merged rows + driven |
| REQ-007 | WHEN a `.rs` file gains a compiler error, a squiggle + gutter mark + count shall appear; WHEN the error is fixed, ALL of it shall clear. | driven (live rust-analyzer) |

## Phase Plan
- **P2 Design** — the `Diag`/`Severity` types + the parse seam (lsp-types confined); the store shape
  (on RootView or lsp_host, keyed by canonical path per #309); merged/underline/count signatures; the
  severity→color decision (warning role vs muted); the render squiggle mechanism (underline runs in
  code_view); the pump publish→store capture; how `open_file_diagnostic_rows` unions the store.
- **P3 Implement** · **P3.5 Inspect** (the position-mapping + replace/clear are the danger) · **P4
  Validate** (pure cov/MSI 100 + driven: introduce/fix an error live) · **P5 Complete**.
