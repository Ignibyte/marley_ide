---
pipeline_id: 094d43c1-790b-413a-9215-06d3b7cdf1fc
ticket: docs/planning/tickets/closed/TICKET-492-browser-tools-for-agents.md
status: Phase 4 — Complete PASS
title: "B2: The agent sees and drives the browser"
type: feature
slice: prong 3 B2
references: [docs/marley/three-prong-plan.md, docs/marley_architecture/orchestration-shell.md, docs/planning/pipeline/queued/491-marley-mcp-in-the-app.spec.md, docs/planning/pipeline/queued/488-browser-pane.spec.md]
---

## Title
The browser tool family on Marley's MCP server: read what the user sees, act where the user
watches.

## Scope
### In
- **Read tools**, answered from the page the Browser tab shows:
  - `browser_look`: the URL, title, viewport size, scroll offsets, whether the page is loading,
    the focused element (role, accessible name, tag; its value unless it is a password field),
    the selected text, and the frame on the user's screen as an image;
  - `browser_snapshot`: the page's accessibility tree as indented text, interactive elements
    only by default (`full` for all), each with a ref (`e12`) the write tools take; same-site
    iframes through the page's frame tree and cross-site iframes from the sessions
    `Target.setAutoAttach` opens; no field values (Chromium's masked password shows its
    length); capped at 30,000 characters with a note when cut;
  - `browser_console`: the latest 200 console messages and uncaught exceptions (level, text,
    source and line, time);
  - `browser_network`: the latest 200 requests (method, URL, status, type, duration, failure),
    with no headers or bodies, and the values of query parameters whose names look secret
    (`token`, `key`, `secret`, `password`, `auth`, `code`, `sig`, `session`) replaced by `…`.
- **Write tools**, acting in the tab the user sees:
  - `browser_navigate` (`http` and `https` only), `browser_back`;
  - `browser_click` (a ref, scrolled into view first, or viewport coordinates; button and
    count; a ref inside a cross-site iframe is placed through its owner element's box),
    `browser_type` (text into a ref or the focused element, as key events; `submit` presses
    Enter), `browser_press` (a key or chord, `Ctrl+A`), `browser_scroll` (by pixels, or a ref
    into view).
  A write call with no Browser tab open opens one, and a Browser tab behind another comes to
  the front, without taking the focus from where the user types, so the user sees what the
  agent does. `browser_navigate` and `browser_back` answer once the page has loaded (at most
  15 seconds).
- **The Agent chip.** While a write call runs, and for five seconds after, the Browser tab's
  toolbar shows "Agent" and the last action ("clicked button “Sign in”").
- **The grant.** The browser's write tools carry the grant class `browser.write`, which Marley
  grants when it starts the server (D2).
- **Images in answers.** `marley_mcp`'s `ToolAnswer` carries an image, which reaches the client
  as an MCP image content block (`browser_look`'s JPEG).
- The console and network rings start with the connection (#488's app-wide entity), so an
  agent reads what happened before it asked.

### Out (explicitly deferred)
- Tabs and `browser_tabs` (#493); picks, annotations and traces (wave 2).
- Any tool that evaluates script the agent supplies.
- Screenshots of a region or at another size; file uploads; downloads.

## Reference (§20)
N/A for Warp and Zed. Published neighbors: Playwright MCP's accessibility snapshot with refs
and its navigate, click and type tools (Apache-2.0; its behavior, not its code), and
agent-browser's interactive-only snapshot, measured on the dev box on 2026-09-18 at about
3,400 tokens against Playwright MCP's 11,900 plus a 4,600-token schema: the reason the
snapshot here defaults to interactive elements.

### Prior art
- **Published material.** CDP 1.3 (Chromium 152): `Accessibility.getFullAXTree` (per session,
  so a cross-site iframe needs its own session, observed in the probe),
  `DOM.getBoxModel`, `DOM.scrollIntoViewIfNeeded`, `DOM.resolveNode`, `Runtime.consoleAPICalled`,
  `Runtime.exceptionThrown`, `Log.entryAdded`, `Network.requestWillBeSent`,
  `responseReceived`, `loadingFinished`, `loadingFailed`. MCP tool results with image
  content. Claude Code asks before each MCP tool call unless allowed (the docs agent,
  2026-09-24).
- **Code we already ship.** #488's app-wide entity and page session; #489's key and mouse
  mappings, which the write tools reuse so an agent's click is the user's click; #491's
  deferred calls and registry; `marley_mcp`'s `GrantTable` and `decide`.

## UI proof
UI-AFFECTING. `script/e2e/492-browser-tools.sh` with `COMPOSITOR=sway`, Chromium offline
(`offline_chromium`). Fixtures: a loopback site with a sign-in form that reports the events it
gets, a console message, an uncaught error and a request to `/api?token=abc123&page=2`, and a
cross-site iframe with a field. No Browser tab is open at the start; a stand-in agent run by
the harness speaks JSON-RPC through the plugin's bridge (`MARLEY_MCP_ENDPOINT` at the e2e
profile) and prints each answer to the run log, saving `browser_look`'s image beside the
shots. Shots: `492-01-opened-and-navigated` (the agent's first navigation opened the tab),
`492-02-typed-and-clicked` (with the Agent chip), `492-03-chip-gone`; the saved frame
`492-look.jpg`.

## Locked-In Decisions
- D1 — The agent reads the page the user has, never a second one (plan D6 and the
  amendment's single Chromium).
- D2 — Write tools are allowed by Marley without a grant setting; the checks are the
  client's approval of each call (Claude Code asks by default) and the tab, where each action
  shows with the Agent chip. `marley_mcp`'s grant table grants the `browser` class by default,
  and a settings block can take it away later. This replaces plan D9's deny-by-default for
  this family, on Chad's "cursor like experience".
- D3 — No script evaluation tool; the snapshot, look and rings read through CDP domains, with
  any script Marley needs run in an isolated world.
- D4 — The snapshot defaults to interactive elements; `full` is the exception.
- D5 — Secrets are redacted in Marley before an agent sees them: password values never, query
  values by name, no headers or bodies.
- D6 — An agent's write call opens the Browser tab when none is open: the user sees the page
  the agent acts on.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `browser_look`, the answer shall hold the URL, title, viewport, scroll offsets, focused element and selection of the page in the Browser tab, and the frame the tab shows. | The run log, and `492-look.jpg` against shot `492-01-opened-and-navigated` |
| REQ-002 | WHEN an agent calls `browser_snapshot`, the answer shall list the page's interactive elements with roles, names and refs, including those inside a cross-site iframe. | The run log (the iframe's field is listed) |
| REQ-003 | WHEN an agent calls `browser_console` or `browser_network`, the answer shall hold the recent entries, with secret-looking query values replaced and no headers. | The run log (`token=…`, `page=2`) |
| REQ-004 | WHEN an agent calls `browser_navigate` with an `http` or `https` URL, the Browser tab shall load it; WHEN the URL has another scheme, the call shall fail and the page shall stay. | Shot `492-01-opened-and-navigated`; the run log's refusals (`file:`, `javascript:`) |
| REQ-005 | WHEN an agent clicks a ref or types, including into a cross-site iframe's field, the page in the tab shall receive the same events a user's click and keys send. | Shot `492-02-typed-and-clicked` |
| REQ-006 | WHILE an agent's write call runs and for five seconds after, the tab shall show the Agent chip with the last action. | Shots `492-02-typed-and-clicked` (the chip) and `492-03-chip-gone` |
| REQ-007 | WHEN an agent calls a write tool and no Browser tab is open, one shall open. | Shot `492-01-opened-and-navigated` |
| REQ-008 | The tool list shall hold no tool that evaluates script. | The run log's `tools/list` |

## Phase Plan
- **P1 Plan** — this spec, and the design and E2E plan in the notes.
- **P2 Code** — the read tools and their rings, the snapshot with refs across iframes, the
  write tools over #489's mappings, the chip, the grant; fmt and clippy clean; a review of
  the diff.
- **P3 Test** — write and run `492-browser-tools.sh`, read every shot and the saved frame;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `marley_browser.md` and the MCP note, the plan's status, ledger
  capture, close, archive, commit.
