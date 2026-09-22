---
pipeline_id: ae11a2c3-36af-4e7a-aa33-2f08141a8137
ticket: forge#119 (f4bb512a-f31d-4d91-ad59-db32bd2f818b) · local docs/planning/tickets/open/TICKET-119-focus-fix.md
aar_id: e7a700d7-c54f-4001-b0d8-94a7c1a14f3d
status: Phase 5 — Complete PASS
title: fix sticky/ambiguous search focus
type: bug
references:
  - crates/marley_app/src/app.rs (SHIM: clear_input_focus + mutual exclusion + clear-on-outside-click)
---

## Title
Fix the "funky search": the sidebar-search, commit-message, and top-search inputs share three focus flags
that were never cleared, so keystrokes went to the wrong box and focus got stuck. Make focus mutually
exclusive and cleared when you click the terminal.

## Scope
### In
- SHIM `clear_input_focus(&mut self)` (all three focus flags false); called before setting any one true, and
  on terminal-pane click-to-focus.

### Out
- Refactoring the three bools into an enum (a bigger change; the helper fixes the behavior). New pure logic.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — focus is mutually exclusive (clear the others before setting one) + cleared on an outside (terminal)
  click. The fix is masked shim (app.rs); no new pure surface.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN one input is focused, the other two focus flags shall be false. | code review |
| REQ-002 | WHEN a terminal pane is clicked, all input focus flags shall be cleared. | code review |
| REQ-003 | gate GREEN (the fix is masked; no new pure lines). | gate |

## Phase Plan
- **P2** — the clear_input_focus helper + the 4 call sites; no test plan (shim-only).
- **P3** — implement (app.rs).
- **P3.5** — 1 self-review: mutual exclusion + clear-on-outside-click; borrow-safety in the listeners.
- **P4** — gate GREEN (existing tests; the interactive drive is env-blocked → code-reviewed).
- **P5** — docs, AAR, archive, close #119.
