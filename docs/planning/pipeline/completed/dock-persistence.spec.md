---
pipeline_id: 3bb93099-785f-4293-ab63-82ee17f1a1c7
ticket: forge#95 (8d2c5397-6c55-4586-844f-5fc7b94e1deb) · local docs/planning/tickets/open/TICKET-095-dock-persistence.md
aar_id: 1b3ce9fa-d404-4314-9f2c-a976156482c9
status: Phase 5 — Complete PASS
title: dock-state persistence + M2.F finale
type: feature
milestone: M2.F — The Persistent Cockpit (FINALE)
references:
  - crates/marley_app/src/right_dock.rs (PURE: right_section_key / right_section_from_key)
  - crates/marley_app/src/settings.rs (RightSectionSetting + AppliedSettings.right_section + persist)
  - crates/marley_app/src/app.rs (SHIM: boot-load + click-persist)
---

## Title
Make the cockpit rail remember its active tab across relaunches — persist the right dock's section and
restore it on boot. The FINALE that closes M2.F.

## Scope
### In
- PURE `right_section_key` / `right_section_from_key` (round-trip; unknown → Details) in right_dock.rs.
- `RightSectionSetting` (`cockpit.right_section` = "details"); `AppliedSettings.right_section`; applied_from
  resolves it; `persist_right_section`.
- SHIM: boot `right_section` from AppliedSettings; persist on a tab click (best-effort).

### Out
- Persisting anything else (dock widths, per-pane state). A settings UI. Heavy "polish" beyond what already
  renders consistently (#90/#94 captures).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — the persisted key is a lowercase word ("details"/"agents"/"forge"); unknown/absent → Details.
- D2 — the round-trip is `right_section_from_key(right_section_key(s)) == s` for all sections.
- D3 — the persist is best-effort (a settings write must not crash — the #26/#87 rule).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `right_section_key(s)` then `right_section_from_key(...)` runs, it shall round-trip to `s`. | unit |
| REQ-002 | WHEN `right_section_from_key(k)` gets an unknown key, it shall return Details. | unit |
| REQ-003 | WHEN a manager has a saved `cockpit.right_section`, `applied_from` shall resolve `right_section`. | unit |
| REQ-004 (visual) | WHEN a tab is selected + the app relaunches, the dock shall open on that section. | self-test (engine round-trip; synthetic click env-blocked) |
| REQ-005 | gate GREEN, cov/MSI 100 on the pure surface; the shim masked. | gate |

## Phase Plan
- **P2** — right_section_key/from_key; RightSectionSetting + AppliedSettings + applied_from + persist; the
  app.rs boot + click-persist; fixture updates; test plan.
- **P3** — implement.
- **P3.5** — 1 critic: the key/from_key MSI (arms + unknown); applied_from/persist wiring; fixtures.
- **P4** — the round-trip + applied_from tests (cov/MSI 100) + gate GREEN + static live (relaunch).
- **P5** — docs, AAR, archive, close #95. **CLOSE M2.F + the sprint.**
