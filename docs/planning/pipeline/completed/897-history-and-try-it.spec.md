---
pipeline_id: 311ed771-3251-47ad-953c-f5ad92005aeb
ticket: docs/planning/tickets/closed/TICKET-897-history-in-the-inspector-and-the-guides-try-it.md
status: Phase 4 — Complete PASS
title: History in the Changes inspector, and the guide's Try it
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
The Marley Guide's "Try it" links run a command from the guide page in Marley's Browser. The
board's history in the Changes inspector is Zed's own: the git panel's History tab, which the
Inspector's Changes holds; a Try it link opens it.

## Scope
### In
- `guide.rs`: `try_it`, which maps a `#try=` fragment of the guide's own page to a command on a
  fixed list; `browser.rs` asks it on each in-page navigation.
- `guide/index.html`: Try it links in three sections, and the script that clears the fragment.

### Out (explicitly deferred)
- A Marley History view: planning built one, and the first shots showed Zed's git panel already
  has a History tab (each commit's subject, author, when and sha) under Changes, in the Inspector
  too. It was dropped as a duplicate.

## Reference (§20)
N/A — Marley-specific: Marley's guide (#599, #884) in Marley's Browser (#488).

### Prior art
- The Browser hears `Page.navigatedWithinDocument` for a fragment move (browser.rs).
- `guide.rs` (#599) writes the page to `<data>/guide/index.html` and opens its file URL.
- Zed's git panel's History tab, under Changes.
- The brain (consultation 81ca7df7): nothing on this seam.

## UI proof
Scenario `script/e2e/897-history-and-try-it.sh` (sway, the browser fixture).
- `897-01-guide`: `marley: open guide`, at the rail's section: its Try it link (REQ-001).
- `897-02-tried`: the link clicked: the rail folded to its strip, which only the command does
  (REQ-001).
- `897-03-changes`: the guide's "This guide" section, its Changes link clicked: Changes with its
  History tab (REQ-001, REQ-002).
- `897-04-day`: Marley Day, the guide's links (REQ-001).

## Locked-In Decisions
- D1 — Try it runs only from the guide's own file page, and only the commands on its list:
  `marley::OpenThreads`, `OpenSessionsPage`, `OpenSystemPages`, `OpenComponentLibrary`,
  `ToggleRailStrip` and `git_panel::ToggleFocus`. No script binding is added to any page.
- D2 — The page clears the fragment after a Try it, so the same link runs again.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user clicks a Try it link in the Marley Guide, Marley shall run that command in the guide's window, in Marley Night and Marley Day. | `897-01-guide`, `897-02-tried`, `897-04-day` |
| REQ-002 | The guide shall lead to the commits of the Changes inspector: Zed's git panel's History tab. | `897-03-again` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
