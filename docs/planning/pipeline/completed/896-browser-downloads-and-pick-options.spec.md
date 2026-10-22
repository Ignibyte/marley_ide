---
pipeline_id: c5d47a0a-7587-4167-a809-896360ad924c
ticket: docs/planning/tickets/closed/TICKET-896-browser-downloads-and-pick-options.md
status: Phase 4 — Complete PASS
title: The Browser's download bar, and a pick's options
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
A Browser tab shows its page's newest download in a bar under the page (its name, where it was
saved, Show, and a close), and the picks panel ends with the Browser board's options: what goes
with the picks (a screenshot of each, its source line, the console error), whom they go to (the
terminal used last or a new agent), and one Send for every pick not yet sent.

## Scope
### In
- `browser.rs`: the download bar from `BrowserHub::downloads` (#767); the picks panel's foot
  (Goes with it, To, Send N picks); the picks' line with what goes with them; `Pick` records what
  went with it.
- `browser_tools.rs`: `browser_pick` leaves out the screenshot and the source places a send left
  out.

### Out (explicitly deferred)
- A choice among several agent terminals in To: the terminal used last is the target Send has
  had since #496.

## Reference (§20)
N/A — Marley-specific: Marley's Browser picks (#496, #879) and downloads (#767); the Browser and
BrowserTools boards draw them.

### Prior art
- `BrowserHub::downloads` and `Download` (#767): the page's downloads with their state and path.
- `send_pick`, `sendable`, `send_line`, `pick_line`, `console_line` (#496, #892).
- `agents::start_cli_with_prompt`, which Send Block uses for a new agent (#826).
- `send_selection::agent_targets` and `Target::label` name an agent terminal.
- The brain (consultation 3b751417): nothing on this seam.

## UI proof
Scenario `script/e2e/896-browser-downloads-and-pick-options.sh` (sway, the browser fixture): a
served page with a card, a Save button and a download link; the terminal used first.
- `896-01-download`: the link clicked: the bar with the file's name, the folder, Open and Show,
  and no toast (REQ-001).
- `896-02-options`: Save picked: the panel's foot with Goes with it, To and Send 1 pick
  (REQ-002).
- `896-03-sent`: the screenshot unticked, the console error ticked, Send 1 pick: the terminal
  shows the line naming the pick without its screenshot and the console (REQ-003).
- `896-04-day`: Marley Day, Save picked again so the foot shows (REQ-001, REQ-002).

## Locked-In Decisions
- D1 — The bar shows the newest download of the tab's page until closed; a newer one shows again.
- D2 — Goes with it starts as the board draws it: the screenshot and the source line ticked, the
  console error not. The choice holds for the tab.
- D3 — A new agent is Claude Code, started with the line as its first prompt, as Send Block's
  new agent is.
- D4 — A pick's Note Enter still sends that pick alone, with the options.
- D5 — While a Browser tab holds the page, the bar says the download is done and #767's toast
  does not; the bar has the toast's Open beside Show. A page with no tab (an agent's) keeps the
  toast.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a page saves a file, its Browser tab shall show a bar with the file's name, the folder it is in, Show and a close, in Marley Night and Marley Day. | `896-01-download`, `896-04-day` |
| REQ-002 | WHILE the picks panel holds a pick not yet sent, it shall end with Goes with it (three ticks), To (the terminal used last, or a new agent) and Send N picks. | `896-02-options`, `896-04-day` |
| REQ-003 | WHEN the user sends picks, the line shall name each pick and say what goes with it, and `browser_pick` shall leave out what was unticked. | `896-03-sent` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
