---
pipeline_id: 316c9831-2e84-4bb1-800a-f340a6f14859
ticket: forge#88 (a406c036-43ea-4adb-b22b-93327514aba7) · local docs/planning/tickets/open/TICKET-088-caret-space.md
aar_id: d67be772-0e94-4563-af71-2b9dec26b719
status: Phase 5 — Complete PASS
title: caret floats a space after the typed text (input-row gap_2)
type: bug
milestone: Terminal Polish
references:
  - crates/marley_app/src/app.rs (SHIM: the prompt input_row render, ~1884)
---

## Title
The prompt caret bar floats an 8px space after your last typed character — because the input row's
`.gap_2()` inserts a gap between the text and the caret. Group the text+caret+after gaplessly.

## Scope
### In
- app.rs prompt `input_row` render: wrap `before` + the caret bar + `after` in ONE gapless inner flex,
  making it a single child of the outer `.gap_2()` row.

### Out
- Any pure-logic / buffer / caret-position change (this is purely a flex-layout fix). The ❯/cwd/git
  segment spacing (kept).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — the caret is FLUSH against the typed text: `before`/caret/`after` are one contiguous text line
  (a gapless inner `div().flex().flex_row().items_center()`), separated from the ❯/cwd/git segments by
  the outer `.gap_2()` (the correct prompt-to-input spacing).
- D2 — SHIM-only render change (app.rs, already cov-excluded + `mutants::skip`) — NO new pure surface,
  so no new unit tests; verified by the gate (masked) + the self-test/capture.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 (visual) | WHEN text is typed at the prompt, the caret bar shall sit immediately after the last character (no gap). | self-test / capture |
| REQ-002 | WHEN the buffer is empty, the caret shall sit at the input start (one prompt-space after the cwd segment). | self-test / capture |
| REQ-003 | `scripts/gates.sh` GREEN (the render change is masked/cov-excluded). | gate |

## Phase Plan
- **P2** — the exact render restructure (the inner gapless flex); no pure surface.
- **P3** — implement (app.rs).
- **P3.5** — 1 quick critic: the grouping is correct, ❯/cwd spacing intact, before/after caret split preserved.
- **P4** — gate GREEN + the self-test (type `abc` → caret flush; else capture the empty-prompt caret + verify the structure — env-block per #86/#87).
- **P5** — docs, AAR, archive, close #88.
