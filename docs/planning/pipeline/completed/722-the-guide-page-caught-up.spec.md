---
pipeline_id: 76220578-0570-4054-adfe-81b09c199624
ticket: docs/planning/tickets/open/TICKET-722-the-guide-page-caught-up.md
status: Phase 4 — Complete PASS
title: The guide page caught up
type: docs
slice: the in-app guide (#599)
references:
  - docs/marley/guide.md
---

## Title
The in-app guide describes the Marley that ships: #688 to #711, plus whatever `guide.md` has that
the page lacks.

## Scope
### In
- Fragments from three bots, merged into `index.html`, each replacing an article or inserted after
  one, in the page's own markup.
- Stale statements fixed.
- The page's Reference tables, where the new settings and palette actions belong.

### Out (explicitly deferred)
- New CSS or script.
- The search over the docs, which is #723.

## Reference (§20)
N/A — Marley-specific: Marley's own guide page (#599), written from `docs/marley/guide.md`.

### Prior art
#599 built the page and its contents filter; `guide.md` is the current text; the closed tickets and
CHANGELOG say what shipped.

## UI proof
`script/e2e/722-the-guide-page-caught-up.sh`, under `compositor sway`. `marley: open guide`
opens the page in a Browser tab, the contents filter takes a word, and its first match shows the
article.

Shots:
- `722-01-home`: "Home" finds Home's page article (#701);
- `722-02-agent-control`: "kill switch" finds agent activity and the kill switch (#703);
- `722-03-editors`: "editor_edit" finds the tools table with the editor family (#704, #705);
- `722-04-marley-agent`: "Rusty" or the Marley agent's article (#696).

## Locked-In Decisions
- **D1:** bots draft and I merge, so no two writers touch the page at once.
- **D2:** `guide.md` stays the source of truth; the page follows it.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The guide page shall describe Home, the Rusty group's menu and threads in tabs as shipped (#697, #699 to #702). | Shot 722-01 |
| REQ-002 | The guide page shall describe agent control: the modes, the question, the activity log and the kill switch (#703 to #707, #711). | Shots 722-02, 722-03 |
| REQ-003 | The guide page shall describe the Marley and Rusty agents and the harness seats as shipped (#684 to #698, #709). | Shot 722-04 |
| REQ-004 | The page shall stay well-formed: every id unique, every tag closed. | A parser check in the gate log |
| REQ-005 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan.**
- **P2 Code:** merge, verify the ids, gate.
- **P3 Test:** the scenario.
- **P4 Complete.**
