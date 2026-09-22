---
pipeline_id: bb6d9957-6b3e-40b2-9309-1839291bd33d
ticket: forge#100 (7af97ea6-a745-4f4c-9246-07c316beaf85) · local docs/planning/tickets/open/TICKET-100-gutter.md
aar_id: b95bfee9-1fd6-49ad-931a-7590205ce5bd
status: Phase 5 — Complete PASS
title: line-number gutter
type: feature
milestone: M4 — The Code Panel
references:
  - crates/marley_app/src/code_view.rs (PURE: gutter_width, gutter_label)
  - crates/marley_app/src/app.rs (SHIM: the viewer gutter render)
---

## Title
A right-aligned line-number gutter in the code viewer — width sized to the file's line count.

## Scope
### In
- PURE `gutter_width(line_count)` (digits of the max line number, min 2) + `gutter_label(n, width)` (right-aligned).
- SHIM: the viewer renders each line's gutter label (muted) sized by gutter_width.

### Out
- The current-line highlight (defers to #101's cursor). Relative line numbers. Clickable gutter.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `gutter_width = line_count.to_string().len().max(2)`; `gutter_label = format!("{n:>width$}")`.
- D2 — the current-line bg is #101 (the movable cursor lands there).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `gutter_width(line_count)` runs, it shall be the digit count of line_count, min 2. | unit |
| REQ-002 | WHEN `gutter_label(n, width)` runs, it shall right-align n in width. | unit |
| REQ-003 | gate GREEN, cov/MSI 100 on the gutter fns; the shim masked. | gate |

## Phase Plan
- **P2** — gutter_width + gutter_label; the viewer render wiring; test plan.
- **P3** — implement.
- **P3.5** — 1 critic: gutter_width digit/min; gutter_label right-align; the render.
- **P4** — the gutter tests (cov/MSI 100) + gate GREEN.
- **P5** — docs, AAR, archive, close #100.
