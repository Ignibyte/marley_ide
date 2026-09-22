---
pipeline_id: 2fbefb5b-91d4-460c-a9cc-89047bb07cfa
ticket: forge#192 (fc082ebd-f80d-4a47-9178-a14993d6bfb9) · local docs/planning/tickets/open/TICKET-192-context-label-footer.md
aar_id: 52e851c9-c9ac-4da7-b1fe-5829c8d2a9a0
status: Phase 1 — Plan PASS · Phase 2 — Design PASS · Phase 3 — Implement PASS · Phase 3.5 — Inspect PASS · Phase 4 — Validate PASS · Phase 5 — Complete PASS
title: Move the cwd/branch context label from the top-right titlebar to the footer
type: feature
milestone: M12.1
references: []
---

## Title
The "~/…/Marley · main" project cwd + git-branch label (#142) renders top-right in the titlebar. chad wants it
at the bottom — move it into the always-on footer status bar (#94). (chad live-app feedback #1.)

## Scope
### In
- Delete the top-right context render (app.rs ~5360-5370, the `.top(8).right(130)` absolute div).
- In the footer render (app.rs ~5237-5270), after the `cockpit_status` segments, add a `flex_1` spacer + a
  child showing `self.titlebar_context_label()` — so cwd/branch sits RIGHT-aligned in the footer, distinct from
  the left-aligned sprint/agents/focus segments (mirrors its old top-RIGHT placement).

### Out (explicitly deferred)
- Any change to `titlebar_context_label` (the $HOME/.git shim) or the pure `titlebar::titlebar_label` — unchanged
  (already unit+mutation tested). Placement only.
- Adding it to the pure `cockpit_status` vec — that would left-align it among the status segments and needs a new
  param; a right-aligned shim label is the faithful move and keeps `cockpit_status` untouched.
- Restyling (keep `text_size(12)` muted, as the top-right label was).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — right-aligned footer label, not a `cockpit_status` segment.** Faithful to the old top-RIGHT position;
  keeps the pure `cockpit_status` (and its ordering test) untouched. Shim placement (no new pure logic).
- **D2 — `flex_1` spacer** pushes the context to the right within the existing footer flex_row.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the shell renders, the cwd/branch context label shall appear in the footer status bar (bottom), right-aligned. | driven capture — cwd/branch in the footer, right side |
| REQ-002 | WHEN the shell renders, the cwd/branch context label shall NOT appear in the top-right titlebar. | driven capture — the top-right no longer shows it |

## Phase Plan
- **P2 Design** — the exact delete + the footer child insertion; test plan (shim-only → driven captures; the
  pure `titlebar_label` is already covered, unchanged).
- **P3 Implement** — remove the top-right block; add the footer spacer + label.
- **P3.5 Inspect** — critics: does the footer label collide with the clickable agent segment / overflow a narrow
  window; is the top-right cleanly removed; any other consumer of the deleted block.
- **P4 Validate** — driven capture (footer shows it, top-right doesn't); gate.
- **P5 Complete** — CHANGELOG + app_shell.md (#142 placement), archive, close.
