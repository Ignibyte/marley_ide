---
pipeline_id: c2e87c90-54b4-474b-821e-de4432baf046
ticket: forge#229 (7062e0a2-6926-415c-8fd2-4ca03d6bdc5c) · local docs/planning/tickets/open/TICKET-229-agent-launcher-dismiss.md
aar_id: efe9cef1-9c72-480c-b7fd-5bbb98b33995
status: Phase 5 — Complete PASS
title: Agent-launcher popup should dismiss on click-outside
type: bug
milestone: M13
references: []
---

## Title
The #181 agent launcher (⌘⇧A) renders as a single `.occlude()` box with no full-screen backdrop, so
clicking outside it does nothing — it only closes via Escape/Enter (chad live feedback #5). Add a
click-outside dismiss by mirroring the proven #166 context-menu backdrop.

## Scope
### In
- `marley_app/src/app.rs` — add a full-screen `.inset_0().occlude()` backdrop as the FIRST child of the
  `if let Some(launcher)` render block (before the box), whose `on_mouse_down(Left)` clears
  `self.agent_launcher = None` + `cx.notify()`.

### Out (explicitly deferred)
- Any pure fn (the dismiss is unconditional — a shim handler, exactly like the #166 backdrop).
- The launch flow, the keyboard nav (↑↓/Enter/Esc), the other overlays.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — mirror the #166 context-menu backdrop** (app.rs:6268-6287): a full-screen `.inset_0().occlude()`
  div with an `on_mouse_down(Left, → agent_launcher = None + notify)`, drawn BEFORE the launcher box so the
  box (also `.occlude()`) hit-tests first — an inside click is swallowed (no dismiss), an outside click hits
  the backdrop → dismiss.
- **D2 — no pure seam** (unconditional dismiss = a shim, like #166). gate-is-shim render fix (app.rs is
  cov-excluded + `mutants::skip` → no cov/MSI delta).
- **D3 — the backdrop is scoped inside the `if let Some(launcher)` block** → it only exists while the launcher
  is open (no interference with the other overlays).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the agent launcher is open AND the user clicks outside its box, the system shall dismiss the launcher. | Driven (⌘⇧A → click outside → closed); ELSE env-considerate → the mechanism (byte-identical to the #166 backdrop) + code review |
| REQ-002 | WHILE the launcher is open, a click INSIDE its box shall NOT dismiss it (the box `.occlude()`s; the keyboard ↑↓/Enter selection is unaffected — the rows have no click handlers). | Code review (box occludes, rows keyboard-driven); driven if unlocked |
| REQ-003 | WHERE the launcher is dismissed via Escape or launched via Enter, that behavior shall be unchanged. | Code review (the existing :1500/:1502 paths untouched); driven if unlocked |

## Phase Plan
- **P2 Design** — read the launcher render block + the #166 backdrop; confirm the box occludes + rows are
  keyboard-driven + modal-exclusivity (the backdrop scoped to the block); file manifest (app.rs only); no pure seam.
- **P3 Implement** — add the backdrop child at the top of the launcher render block.
- **P3.5 Inspect** — critic: does the backdrop dismiss correctly, not break row selection / keyboard nav / the
  launch flow, and not stack confusingly with other overlays? clean-room.
- **P4 Validate** — no new unit tests (gate-is-shim); the existing suite green; driven (or env-considerate →
  mechanism) capture; gate green [diff].
- **P5 Complete** — CHANGELOG + app_shell.md (the launcher backdrop note); AAR; close #229; archive.
