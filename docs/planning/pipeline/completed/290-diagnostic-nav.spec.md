---
pipeline_id: 20b1a4a1-d7af-469e-8d9c-1ca6f21a513a
ticket: forge#290 (af6d6c64-5811-4ecc-ba98-8603cb2654e5) · local docs/planning/tickets/open/TICKET-290-diagnostic-nav.md
aar_id: 4799d92a-ca36-4108-9fcd-e2fb23c35b16
status: Phase 5 — Complete PASS
title: Next/prev diagnostic navigation — F8 walks the caret through the error rows
type: feature
milestone: M18
references: [docs/planning/intake/editor-as-peer-and-terminal-fusion.md]
---

## Title
F8 / ⇧F8 walk the caret through the #289 diagnostic rows (next / previous), in
source order, wrapping, scroll-following. Pure next/prev over the sorted set.

## Scope
### In
- `code_view.rs` (pure): `next_diagnostic(rows: &[usize], current: usize) -> Option<usize>` (the first
  row strictly `> current`, else wrap to `rows.first()`); `prev_diagnostic` (the last row strictly
  `< current`, else wrap to `rows.last()`); `None` when `rows` is empty.
- `keymap.rs`: `"f8" → "next-diagnostic"` + `"shift-f8" → "prev-diagnostic"`, both `KeyContext::Editor`.
- `app.rs` `dispatch_action`: the two arms — `rows = open_file_diagnostic_rows()` (#289), `current =`
  the active editor's caret row (`buffer.line_col(caret).0`), `target = next/prev_diagnostic`; if
  `Some(row)`, set the caret to `buffer.line_start(row)` + `follow_editor_caret()` (#270).

### Out (explicitly deferred)
- Selecting the diagnostic range / showing the message inline — v1 just moves the caret to the row start.
- A block-side "next failure" affordance — the editor F8 is v1 (the block has #213's Jump-to-Failure).
- Cross-file navigation (jumping into a DIFFERENT file's diagnostics) — v1 walks the open file's rows.

## Reference (§20)
Zed / VS Code / the universal F8-next-error convention: F8 jumps to the next
diagnostic, ⇧F8 the previous, wrapping. Marley matches over its own diagnostic row
set (#289). Clean-room §20: the nav math + bindings are Marley's own; no Zed source read.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — strictly after / before the CURRENT caret row, then wrap.** `next` = the
  first row `> current` (so repeated F8 advances even when the caret is ON a
  diagnostic), wrapping to the first; `prev` = the last row `< current`, wrapping to
  the last. Empty rows → `None` (no-op).
- **D2 — the target is the row START** (`line_start(row)`), selection collapsed —
  v1 places the caret at the error line; selecting the span is a follow-up.
- **D3 — F8 / ⇧F8, Editor-scoped** (the #265 KeyContext), so they don't collide
  with any terminal/global chord; `chords_unique_scoped` stays green.
- **D4 — reuse #289 + #270 + #212 end to end** — the rows from `open_file_diagnostic_rows`,
  the follow from `follow_editor_caret`; no new state.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN there are diagnostic rows and the caret is before the last, `next_diagnostic` shall return the first row strictly after the caret row. | code_view.rs unit: rows [2,8,14], current 5 → 8; current 8 → 14 (strictly after). |
| REQ-002 | WHEN the caret is at/after the last diagnostic, `next_diagnostic` shall wrap to the first; symmetrically `prev_diagnostic` before the first wraps to the last. | unit: rows [2,8,14], current 20 → 2 (next wrap); current 0 → 14 (prev wrap). |
| REQ-003 | WHEN there are no diagnostic rows, `next/prev_diagnostic` shall return `None` (a no-op). | unit: `[]` → None. |
| REQ-004 | WHEN F8 (⇧F8) is pressed on the editor, the caret shall move to the next (previous) diagnostic row and scroll into view. | Headless: F8 on the editor resolves to `next-diagnostic` (routes to the dispatch); with an injected/known row set the caret moves (else the routing smoke + the pure fixtures). |
| REQ-005 | `next_diagnostic` + `prev_diagnostic` shall be pure at 100% line coverage + MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — `next/prev_diagnostic` + the keymap bindings + the dispatch arms; the test plan.
- **P3 Implement** — the pure fns + bindings + dispatch.
- **P3.5 Inspect** — the strict-after/wrap edges, the F8 routing/context, the caret+follow.
- **P4 Validate** — pure fixtures + a headless F8-routes smoke; gate green [diff].
- **P5 Complete** — CHANGELOG + editor.md, AAR, close #290.
