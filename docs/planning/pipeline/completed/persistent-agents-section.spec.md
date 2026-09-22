---
pipeline_id: 07ce495b-84c5-428c-a33f-2515aa969383
ticket: forge#91 (09e43d8c-d681-4353-b32a-3ace356981e3) · local docs/planning/tickets/open/TICKET-091-persistent-agents-section.md
aar_id: 2890437b-be54-4b15-abe8-d8692206dfd0
status: Phase 5 — Complete PASS
title: persistent Agents section in the dock
type: feature
milestone: M2.F — The Persistent Cockpit
references:
  - crates/marley_app/src/agent_view.rs (PURE: agent_row_text, agents_empty_hint)
  - crates/marley_app/src/app.rs (SHIM: the Agents-section render + the Fleet refactor)
---

## Title
Make the agent Fleet always-visible in the #90 right-dock Agents tab — see every running agent (glyph +
label + ticket + status + quiet-age + last line) without toggling the ⌘⇧E overlay; click a row to jump.

## Scope
### In
- PURE `agent_row_text(&AgentRow) -> String` — the exact Fleet row format, extracted from the masked render
  into a tested pure fn; `agents_empty_hint() -> &'static str`.
- SHIM: the Fleet overlay refactors onto `agent_row_text`; the #90 Agents-section renders `agent_rows`
  as clickable rows (→ focus, reuse #81) or `agents_empty_hint`.

### Out
- The Forge section (#92). Rich Details (#93). Any change to agent_rows (already tested). Removing ⌘⇧E.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `agent_row_text` is the single source for the row string (Fleet overlay + Agents section both use it).
- D2 — the Agents-section rows reuse #81's clickable-row → workspace.focus + flash pattern.
- D3 — empty fleet → `agents_empty_hint()` ("no agents — ⌘⇧A to launch").

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `agent_row_text(row)` runs, it shall be "{glyph} {label}{ #N} ({status}{ · age}){ · last}" with each suffix present iff its field is set. | unit |
| REQ-002 | WHEN no agents run, `agents_empty_hint()` shall return the launch hint. | unit |
| REQ-003 (visual) | WHEN the Agents tab is active, the dock shall list the agents (or the hint). | self-test (static live + engine) |
| REQ-004 | gate GREEN, cov/MSI 100 on the pure surface; the shim masked. | gate |

## Phase Plan
- **P2** — agent_row_text + agents_empty_hint; the Fleet refactor + Agents-section render; test plan.
- **P3** — implement.
- **P3.5** — 1 critic: agent_row_text MSI (suffixes + assembly), agents_empty_hint, the render/click reuse,
  the Fleet refactor is output-identical.
- **P4** — the pure tests (cov/MSI 100) + gate GREEN + live static capture (synthetic click env-blocked).
- **P5** — docs, AAR, archive, close #91.
