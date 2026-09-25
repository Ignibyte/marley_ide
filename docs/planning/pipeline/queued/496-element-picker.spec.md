---
pipeline_id: 586c6a28-c005-4f0f-bc43-dabe695f23cf
ticket: docs/planning/tickets/open/TICKET-496-element-picker.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "B3a: Pick an element in the Browser tab and send it to the agent"
type: feature
slice: prong 3 B3a (wave 2)
references: [docs/marley/browser-handoff.md, docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/492-browser-tools-for-agents.spec.md, docs/planning/pipeline/completed/493-browser-tabs-and-restore.spec.md]
---

## Title
Pick mode in the Browser tab: Chad points at an element, Marley captures a durable bundle for
it, stages it for a caption, and sends it to the agent in his terminal, which reads the bundle
over MCP.

## Scope
### In
- **Pick mode.** A toolbar button and `marley::PickElement` (Ctrl+Shift+C in a Browser tab, as
  Chrome's inspect key) turn on `Overlay.setInspectMode` (`searchForNode`) for the tab's page:
  Chromium's own inspect highlight, with its tooltip, follows the pointer in the frames. A click
  picks the element (`Overlay.inspectNodeRequested`) and ends pick mode; Escape ends it with no
  pick. The toolbar button shows pick mode is on.
- **The durable bundle**, captured at the pick so it outlives the page's changes: the element
  walked up to its nearest interactive ancestor (a control, a link, an element with a role or a
  click listener), open shadow roots pierced; ranked locators (a test id, an id, role and
  accessible name, visible text, a CSS path); the accessibility role and name; the listeners on
  it and its ancestors, each with its event type, script URL, line and column
  (`DOMDebugger.getEventListeners`, `Debugger.scriptParsed`); what blocks a click on it
  (`pointer-events`, `visibility`, `opacity`, `disabled`, and the element on top at its middle);
  its box; the page's URL and title; and a crop of the frame around it.
- **Staged picks.** A tray in the tab lists the session's picks, newest first, each with its
  summary, a caption field, Send and Discard. Send types one line into the terminal the user
  used last, as attaching a file does (#480): `[browser pick N: <role> "<name>" on <host/path>]
  <caption>`, and marks the pick sent.
- **For agents.** Two read tools on Marley's MCP server: `browser_picks` (each pick's id,
  summary, caption, whether it was sent) and `browser_pick {id}` (the bundle, with the crop as an
  image).

### Out (explicitly deferred)
- The listener's original source through its source map, and opening it (#497).
- Picks kept across launches; picking in a cross-site iframe (its highlight works, and the pick
  names the iframe's element only when the iframe's session reports it).

## Reference (§20)
Chrome DevTools' "Select an element" (Ctrl+Shift+C): the inspect highlight with its tooltip,
the click that picks, Escape to leave. Cursor's browser, where a picked element goes into the
agent's context. Marley matches the first with Chromium's own inspect mode, whose highlight is
in the screencast frames (the 2026-09-24 probe), and the second with a staged pick Chad sends,
per `browser-handoff.md` open decision 3 ("The intake leans toward staged"). Warp: N/A.

### Prior art
- **Observed (the probe, 2026-09-25).** In headless Chromium, `Overlay.setInspectMode` with a
  click dispatched through `Input.dispatchMouseEvent` raises `Overlay.inspectNodeRequested` with
  the element's `backendNodeId`; `DOM.describeNode` gives its attributes (a `data-testid`
  included); `DOMDebugger.getEventListeners` on `DOM.resolveNode`'s object gives the click
  listener's `scriptId`, line and column, and `Debugger.scriptParsed` that script's URL and its
  `sourceMapURL`; `Accessibility.getPartialAXTree` gives role and name; `elementFromPoint` finds
  an element covering another.
- **Published material.** CDP's Overlay, DOM, DOMDebugger, Debugger and Accessibility domains;
  Playwright's locator ranking (test id, role, text) as the published idea of a durable locator.
- **Code we already ship.** #492's `Page::accessibility_tree`, `box_center`, `screenshot`,
  `isolated_context` and refs; #493's per-page hub and its tool `tab`; #480's
  `AttachFile`, which types into the focused terminal; `marley_mcp`'s browser family.

## UI proof
UI-AFFECTING. `script/e2e/496-element-picker.sh` (`compositor sway`, offline): a fixture page
with a card holding a button (a test id, a click listener in `app.js`) and an element covered by
a transparent layer. Steps: the pick button; the pointer over the button (`496-01-hover`: the
inspect highlight and its tooltip); a click (`496-02-staged`: the tray lists the pick of the
button, pick mode off); a caption and Send (`496-03-sent`: the line in the terminal); the
stand-in agent's `browser_picks` and `browser_pick` (the run log: locators, role and name, the
listener at `app.js`, the covered element's blocker); pick mode and Escape (`496-04-cancelled`).

## Locked-In Decisions
- D1 — Picks are staged: Chad captions and sends each; nothing reaches the agent on its own.
- D2 — Chromium's own inspect highlight, since it is in the frames; Marley draws none.
- D3 — The bundle is captured at the pick and kept for the session; the terminal line is the
  reference, the MCP tool the content.
- D4 — Send types into the terminal the user used last, as attaching a file does.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE pick mode is on, the page shall show Chromium's inspect highlight on the element under the pointer. | Shot `496-01-hover` |
| REQ-002 | WHEN the user clicks an element in pick mode, Marley shall stage a pick of its nearest interactive element and end pick mode. | Shot `496-02-staged` |
| REQ-003 | WHEN the user sends a captioned pick, Marley shall type the pick's reference and caption into the terminal the user used last. | Shot `496-03-sent` |
| REQ-004 | WHEN an agent calls `browser_pick` with a pick's id, the answer shall carry its locators, role and name, listeners with their script locations, what blocks a click on it, its box and a crop image. | The run log |
| REQ-005 | WHEN the user presses Escape in pick mode, pick mode shall end with no pick. | Shot `496-04-cancelled` |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design (whether the tray is a bar in the tab
  or a popover; the interactive-ancestor rule).
- **P2 Code** — pick mode, the bundle, the tray and Send, the two tools; fmt and clippy clean.
- **P3 Test** — the scenario, every shot; #492's and #493's scenarios again; the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
