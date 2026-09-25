---
pipeline_id: 6988c870-dbe1-443c-82e1-f67b3c2543ea
ticket: docs/planning/tickets/closed/TICKET-490-browser-navigation.md
status: Phase 4 — Complete PASS
title: "B1a: The address bar and navigation"
type: feature
slice: prong 3 B1a
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/queued/488-browser-pane.spec.md, docs/planning/pipeline/queued/489-browser-input.spec.md]
---

## Title
A toolbar across the top of the Browser tab (back, forward, reload or stop, and an address
bar), the browser's keys, and JavaScript dialogs drawn by Marley.

## Scope
### In
- **The address bar**, a Zed single-line `Editor`, showing the page's URL whenever it does not
  have the focus. Enter navigates to what was typed:
  - text with a scheme Chromium navigates to (`http`, `https`, `file`, `about`, `data`,
    `chrome`, `view-source`) loads as typed;
  - a host, with or without a port and path (`localhost`, an IPv4 or bracketed IPv6 address,
    or a name with a dot and no spaces) gets `http://` for loopback and `https://` otherwise;
  - anything else is a DuckDuckGo search (`https://duckduckgo.com/?q=…`).
- **Back, forward, reload and stop** as icon buttons: back and forward move through the
  page's history (`Page.getNavigationHistory`, `navigateToHistoryEntry`) and are disabled at
  its ends; reload becomes stop while the main frame loads (`Page.reload`, `stopLoading`).
- **The loading state**: from the main frame's start of loading to its stop, the reload
  button is a stop button and a thin bar runs under the toolbar.
- **Keys** in a `MarleyBrowser` key context: Ctrl+L focuses the address bar and selects its
  text; Alt+Left and Alt+Right go back and forward; Ctrl+R and F5 reload; Escape in the
  address bar restores the URL and gives the page the focus back.
- **JavaScript dialogs**: `alert`, `confirm`, `prompt` and `beforeunload` open a card over the
  page in the tab, with the message, OK and Cancel (a text field for `prompt`, with its
  default); Enter answers OK and Escape Cancel, through `Page.handleJavaScriptDialog`.
- The tab's tooltip is the page's URL, and a navigation made by anyone (a link, an agent)
  updates the address bar.

### Out (explicitly deferred)
- Tabs (#493), bookmarks, history search, downloads, find in page, zoom.
- A setting for the search engine or a home page (each needs the `marley` settings block;
  DuckDuckGo and `about:blank` until someone asks).
- A connection-security indicator.

## Reference (§20)
N/A for Warp and upstream Zed (neither has a browser). The behavior reference is Chromium's
own: navigation, history and dialogs go through Chromium's Page domain, so what back, forward
and reload do is what they do in Chrome. The address bar follows Chrome's omnibox rules for
URL, host and search, in their simplest form.

### Prior art
- **Published material.** CDP's Page domain in Chromium 152: `navigate`,
  `getNavigationHistory`, `navigateToHistoryEntry`, `reload`, `stopLoading`, the events
  `frameNavigated`, `navigatedWithinDocument`, `frameStartedLoading` and
  `frameStoppedLoading`, `javascriptDialogOpening` (message, type, `defaultPrompt`) and
  `handleJavaScriptDialog` (accept, `promptText`). A dialog left unanswered blocks the page's
  script.
- **Code we already ship.** Zed's single-line `Editor` (`Editor::single_line`), as the search
  bars and #481's rich input use it; `ui::IconButton` with `IconName::ArrowLeft`,
  `ArrowRight` and `RotateCw`; Zed's key contexts and Marley's `keymap.json` (bindings in a
  deeper context beat Zed's workspace bindings for the same keys); the rich input's
  editor-in-a-container and its key filter (#481).

## UI proof
UI-AFFECTING. `script/e2e/490-browser-navigation.sh` with `COMPOSITOR=sway` (#487). Fixtures:
a loopback HTTP server (a small python3 script, so one path can answer slowly) serving page A
(a link to B), page B (it counts its loads in `sessionStorage`), a dialogs page (buttons for
alert, confirm and prompt that print their answers) and `/slow` (three seconds), and a Chromium
that resolves no host but the loopback ones. Shots: `490-00-address-selected`,
`490-01-typed-url`, `490-02-link`, `490-03-back`, `490-03b-forward`, `490-04a-host`,
`490-04-forward-reload`, `490-06b-restored`, `490-05-loading`, `490-05b-stopped`,
`490-06-search`, `490-07-alert`, `490-08a-prompt`, `490-08-prompt-answered`,
`490-09-agent-navigated`.

## Locked-In Decisions
- D1 — The address bar is a Zed `Editor`, so it edits like every other field in Marley (Vim
  mode included when it is on).
- D2 — The URL rules stay small and predictable: a known scheme, then a host, then search.
- D3 — DuckDuckGo searches until a setting exists: it needs no account and no consent page.
- D4 — Dialogs are drawn inside the Browser tab, over the page they block, rather than as a
  window-wide modal; the rest of Marley stays usable while a page waits.
- D5 — The browser's keys live in a `MarleyBrowser` context, which beats Zed's workspace
  bindings for the same keys only while the Browser tab has the focus.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user enters a URL in the address bar, the page shall load it, and the address bar and the tab shall show its URL and title. | Shot `490-01-typed-url` |
| REQ-002 | WHEN the user enters a host with no scheme, the page shall load it over `http` for loopback and `https` otherwise; WHEN the text is neither a URL nor a host, the page shall load a DuckDuckGo search for it. | Shots `490-04-forward-reload` (loopback host) and `490-06-search` (the address bar shows the search URL) |
| REQ-003 | WHEN the user goes back or forward, by button or with Alt+Left or Alt+Right, the page shall move through its history, and each button shall be disabled at its end of the history. | Shots `490-02-link`, `490-03-back`, `490-04-forward-reload` |
| REQ-004 | WHEN the user reloads, by button, Ctrl+R or F5, the page shall load again. | Shot `490-04-forward-reload`: page B's load count went up |
| REQ-005 | WHILE the main frame loads, the reload button shall be a stop button and a loading bar shall run under the toolbar, and stop shall end the load. | Shot `490-05-loading` |
| REQ-006 | WHEN the page opens a JavaScript dialog, the tab shall show it over the page, and the user's answer shall reach the page. | Shots `490-07-alert`, `490-08-prompt-answered` |
| REQ-007 | WHEN the user presses Ctrl+L, the address bar shall take the focus with its text selected; WHEN the user presses Escape in it, it shall show the page's URL again and the page shall take the focus. | Shots `490-01-typed-url`, `490-06-search` |
| REQ-008 | WHEN something other than the user navigates the page (a link, an agent), the address bar shall show the new URL. | Shot `490-09-agent-navigated` |

## Phase Plan
- **P1 Plan** — this spec, and the design and E2E plan in the notes.
- **P2 Code** — the toolbar and address bar, the URL rules, history and loading, the key
  context and its bindings, the dialog card; fmt and clippy clean; a review of the diff.
- **P3 Test** — write and run `490-browser-navigation.sh`, read every shot;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `marley_browser.md`, the plan's status, ledger capture, close,
  archive, commit.
