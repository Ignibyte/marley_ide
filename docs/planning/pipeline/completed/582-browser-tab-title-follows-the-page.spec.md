---
pipeline_id: 8df91e6c-7d19-484c-99f5-956ae0886f01
ticket: docs/planning/tickets/open/TICKET-582-browser-tab-title-follows-the-page.md
status: Phase 4 — Complete PASS
title: "A Browser tab's title follows a title its page's script sets"
type: bug
slice: prong 3, the Browser tab (B1b, #493); found in #523's Plan
references: [docs/planning/pipeline/completed/523-saved-playwright-scripts.notes.md, docs/planning/pipeline/completed/495-select-picker.spec.md]
---

## Title
A page that sets `document.title` after it loads (an unread count, a build's state, a title set
after the page fetched its data) keeps its old title in its Browser tab, in its rail row and in
`browser_tabs`, because Chromium sends no CDP event for such a change and Marley reads a title
only at DOMContentLoaded, load, a move within the document and `Target.targetInfoChanged`. A
watcher in an isolated world of the page's main frame now reports each new title through a CDP
binding, and the hub takes it, so every place that shows the page's title follows it.

## Scope
### In
- `crates/marley_browser/src/title.rs` (new): the watcher (`WORLD` `marley-title`, `BINDING`
  `marleyTitle`), a `MutationObserver` that reports the main frame's title when it differs from
  the last it reported, and `Page::watch_title(session)`, set up as `watch_actions` is.
- `crates/marley_browser/src/marley_browser.rs`: the module.
- `crates/marley_workbench/src/browser.rs`: `attached`'s watchers start the title watcher; the
  `Runtime.bindingCalled` routing in `follow_observed` sends `marleyTitle` to a new
  `title_reported`, which sets the page's title and emits `PageInfoChanged`.
- `script/e2e/browser-fixture.sh`: the login site's `#read` move within the document, #523's
  stand-in for this ticket, goes, so #507's and #581's scenarios rest on the watcher.
- `script/e2e/582-browser-tab-title-follows-the-page.sh`.

### Out (explicitly deferred)
- The title of a cross-site iframe (it is not the page's).
- A page's favicon that a script changes after the load (#504 reads the icon at each load).

## Reference (§20)
Upstream Zed: none; a Browser tab's title is Marley's, drawn through Zed's `Item::tab_content`.
The mechanism is Marley's own #495 pattern (L-claude-495): `Runtime.addBinding` with
`executionContextName`, `Page.addScriptToEvaluateOnNewDocument` with `worldName`, and
`Page.createIsolatedWorld` for the document already loaded. Chrome shows a tab's title as the
page sets it, which is the behavior matched. Warp: N/A, Warp has no browser.

### Prior art
- **Behavior maps and reports.** #523's Plan: a scratch Chromium 152 with discovery on reported a
  page's first title in `Target.targetInfoChanged` and nothing when a timer changed it
  (L-claude-523-chromium-sends-no-event-for-a-scripts-title-001).
- **Published material.** CDP's Target domain reports title changes only as target info changes,
  which Chromium does not send for a script's `document.title`; the Page domain has no title
  event. `Runtime.bindingCalled` is the documented way for a page's script to reach a client.
- **The code we already ship.** `marley_browser::select` (`watch_selects`) and
  `marley_browser::recorder` (`watch_actions`) set up a binding and an isolated world per page
  session; `follow_observed` routes `Runtime.bindingCalled` by name since #506; the hub's
  `target_changed` sets a page's title and emits `PageInfoChanged`, which the tab, the rail's row
  and `browser_tabs` already follow. Does a crate we build own the seam? `marley_browser` owns the
  page's session and its worlds; the watcher is one more world there.

## UI proof
UI-AFFECTING (the Browser tab's title, the rail's row). `script/e2e/582-browser-tab-title-follows-the-page.sh`
(`compositor sway`). Setup: the offline Chromium; a loopback site whose `timer.html` sets its title
three seconds after it loads, whose `stored.html` sets its title after an IndexedDB read, and
whose `framed.html` embeds a same-site iframe that sets the iframe's own title. Shots: the timer
page's tab and rail row with its new title (`582-01-timer`); a second tab in front while the
first page, behind it, changes its title, its rail row following (`582-02-behind`); the stored
page's new title (`582-03-stored`). The run log carries `browser_tabs` after each step, the
framed page's title unchanged by its iframe's, and `half.html`'s title, which its script sets
with half of a surrogate pair, with a replacement character in its place.

## Locked-In Decisions
- D1: A watcher in an isolated world of the page's main frame reports the title: Chromium sends
  no event for it, and polling the title of every page would cost a call a page a second.
- D2: It reports only a title that differs from the last it reported, cut at 1,000 characters,
  and it observes the whole document only until `DOMContentLoaded`, then the head alone, so a
  loaded page that changes its body often pays nothing for it (F-claude-506).
- D3: It runs in the top frame only: an iframe's title is not the page's.
- D4: The hub takes a reported title as `target_changed` takes a target's: the page's title
  changes and `PageInfoChanged` goes out, so the tab, the rail and `browser_tabs` follow it with
  nothing of their own.
- D5: The login site's `#read` stand-in in the fixture goes: #507's and #581's scenarios then
  check the watcher every time the golden set runs.
- D6 (added in Test): the title is made well formed before it is cut, as #518's texts are:
  half of a surrogate pair in a CDP message fails the whole message
  (L-claude-518-a-lone-surrogate-fails-the-whole-cdp-message-001).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a page's script sets its `document.title` after the page loaded, the system shall show the new title in the page's Browser tab and its rail row within a second. | Shot `582-01-timer` |
| REQ-002 | WHILE the page's tab is behind another, the system shall follow the page's title in its rail row and in `browser_tabs`. | Shot `582-02-behind`; the log |
| REQ-003 | WHEN a page sets its title after an asynchronous read, the system shall name that title in `browser_tabs`. | Shot `582-03-stored`; the log |
| REQ-004 | WHEN an iframe of a page sets the iframe's own title, the system shall keep the page's title. | The log: `browser_tabs` for `framed.html` |
| REQ-005 | WHEN the golden set runs, the system shall pass #507's and #581's scenarios with the login site's title set by its script alone. | The golden set's run |
| REQ-006 | WHEN a page's script sets a title that holds half of a surrogate pair, the system shall show that title with a replacement character in the half's place. | The log: `browser_tabs` for `half.html` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes.
- **P2 Code:** the watcher and its setup in `marley_browser`, the hub's routing and
  `title_reported`, the fixture; fmt, clippy and the gate's dylint stage; a review of the diff.
- **P3 Test:** write and run the scenario and read every shot; the golden set with 582 added;
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG (Fixed), `marley_browser.md`, `marley_workbench.md`, the ledger
  capture, close the ticket, archive, commit and push.
