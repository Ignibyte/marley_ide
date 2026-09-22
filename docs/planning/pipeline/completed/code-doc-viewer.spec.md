---
pipeline_id: 7a12e987-3610-4cb4-8a28-c635c88f81d8
ticket: forge#97 (14d9c7b3-0025-4b0d-9919-678f6594fa6d) · local docs/planning/tickets/open/TICKET-097-code-doc-viewer.md
aar_id: 7e428549-7220-4cf4-9360-23753befef6c
status: Phase 5 — Complete PASS
title: the code-doc model + viewer overlay
type: feature
milestone: M4 — The Code Panel (FOUNDATION)
references:
  - crates/marley_app/src/code_view.rs (NEW PURE: CodeLine, code_lines, CodeViewState)
  - crates/marley_app/src/lib.rs (mod code_view)
  - crates/marley_app/src/app.rs (SHIM: code_view Option + the overlay render)
---

## Title
A read-only code VIEWER overlay (like the ⌘P finder — not a PaneGroup pane) that shows a file's contents
with line numbers. The FOUNDATION for M4 (#98-106 wire opening, syntax, diff, …).

## Scope
### In
- NEW pure `code_view.rs`: `CodeLine { number, text }`, `code_lines(text, tab_width, max_cols)`,
  `CodeViewState { path, lines, scroll }` + `new`.
- SHIM: `RootView.code_view: Option<CodeViewState>`; render a centered overlay of numbered lines when Some;
  Esc closes.

### Out
- Opening a file (seq-2). Syntax (seq-3). Gutter styling depth (seq-4). Scroll (seq-5). Diff (seq-6+).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `code_lines` splits on '\n' (a trailing '\n' yields a final empty line); 1-based numbers.
- D2 — tab-stop expansion (`tab_width - col%tab_width` spaces per '\t'); char-truncate over max_cols (+'…').
- D3 — the viewer is an overlay (reuses the finder pattern), NOT a pane — no pane-kind surgery.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `code_lines(text, tw, max)` runs, it shall return 1-based numbered, tab-expanded, truncated lines. | unit |
| REQ-002 | WHEN text has a trailing '\n' or is empty, the line set shall be well-formed (final empty line / one empty line). | unit |
| REQ-003 | WHEN `CodeViewState::new` runs, scroll shall be 0 and lines shall be `code_lines(text,…)`. | unit |
| REQ-004 (visual) | WHEN `code_view` is Some, a numbered-line overlay shall render; Esc closes it. | self-test (deferred to seq-2's open path) |
| REQ-005 | gate GREEN, cov/MSI 100 on code_view.rs; the shim masked. | gate |

## Phase Plan
- **P2** — code_view.rs API; the app.rs code_view field + overlay render + Esc; test plan.
- **P3** — implement (code_view.rs + lib.rs + app.rs).
- **P3.5** — 1 critic: code_lines MSI (split/number/tab-stop/truncate/empty); CodeViewState::new; the render.
- **P4** — code_lines + new tests (cov/MSI 100) + gate GREEN (live proof deferred to seq-2).
- **P5** — docs, AAR, archive, close #97.
