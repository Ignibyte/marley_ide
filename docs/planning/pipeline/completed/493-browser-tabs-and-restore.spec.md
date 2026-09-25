---
pipeline_id: 4a8d13c3-78e6-4bb8-9e4c-79484bf0abf3
ticket: docs/planning/tickets/open/TICKET-493-browser-tabs-and-restore.md
status: Phase 4 — Complete PASS
title: "B1b: Browser tabs as pages"
type: feature
slice: prong 3 B1b (restore split to #494, the select picker to #495)
references: [docs/marley/three-prong-plan.md, docs/marley_architecture/embedded-browser-model.md, docs/planning/pipeline/completed/488-browser-pane.spec.md, docs/planning/pipeline/completed/490-browser-navigation.spec.md, docs/planning/pipeline/completed/492-browser-tools-for-agents.spec.md]
---

## Title
One Browser tab per page of Marley's Chromium: pages opened by pages, by agents and by Ctrl+T
each get a tab, and closing a tab closes its page.

## Scope
### In
- **Tabs as pages.** The hub keeps every `page` target of Marley's Chromium, each attached with
  its own session and its own frame, viewport, history, loading, dialog, rings and refs; each
  Browser tab shows one page. A page opened by a page (`window.open`, a `_blank` link: the new
  target carries an `openerId`) opens a Browser tab next to its opener's and takes the focus, as
  a browser does. A page with no opener (an agent's, or any CDP client's `Target.createTarget`)
  opens a tab that leaves the focus where it is (D5).
- Closing a Browser tab closes its page (`Target.closeTarget`); moving a tab to another pane does
  not; a page closed elsewhere closes its tab.
- `marley::NewBrowserTab` (Ctrl+T in a Browser tab, and the palette) opens a blank page in a new
  tab with the address bar focused. `marley: open browser` activates the workspace's newest
  Browser tab, gives a page with no tab a tab, or opens a new page.
- **The agent tools** (#492) take an optional `tab`, a page's id from the new `browser_tabs`
  (each tab's id, title, URL, whether it loads, and which one the user focused last); left out,
  they act on the Browser tab the user focused last. `browser_navigate` takes `new_tab`, which
  opens the page in a new tab without the focus and answers with its id.

### Out (explicitly deferred)
- Restoring tabs on relaunch (#494) and the `<select>` picker (#495).
- Tab groups, pinning, moving a page between Marley windows, a private profile.

## Reference (§20)
Upstream Zed for the tab behavior: Browser tabs are `workspace::Item`s and follow Zed's pane
rules (activation, closing, moving between panes; `Pane::add_item` with a destination index,
`Item::on_removed`, which Zed calls on a close and on a move alike). For what a page does when it
opens another page, Chromium's own behavior, as CDP reports it: a new page target per window,
with `openerId` when a page opened it. Warp: N/A.

### Prior art
- **Published material.** CDP's Target domain: `targetCreated` (`openerId`, `canAccessOpener`),
  `targetDestroyed`, `createTarget`, `closeTarget`, `attachToTarget` with `flatten`.
- **Observed (the probe, 2026-09-25).** A `_blank` link and `window.open` both create a page
  whose `openerId` is the opener's target id (`canAccessOpener` false for the link, which is
  `noopener`, and true for `window.open`); a `Target.createTarget` page has none, and its
  `targetCreated` arrives before `createTarget` answers; `closeTarget` reports `targetDestroyed`.
- **Code we already ship.** #488's hub and page session, #490's navigation state, #492's
  observers, rings, refs and chip, all of which become per page; `Pane::add_item`,
  `Pane::remove_item`, `Workspace::items_of_type`, `activate_item`; `Item::on_removed`.

## UI proof
UI-AFFECTING. `script/e2e/493-browser-tabs.sh` with `COMPOSITOR=sway`, Chromium offline.
Fixtures: a loopback site whose page has a `_blank` link and a `window.open` button. Steps:
open the browser and go to the page; click the link (a second tab, focused, beside the first);
the stand-in agent opens a page with `browser_navigate` and `new_tab` (a third tab, not focused)
and lists the tabs; close the second tab with the mouse (its page closes: the agent's list);
close the agent's page through CDP (its tab closes); Ctrl+T (a blank tab, the address bar
focused). Shots: `493-01-new-tab`, `493-02-agent-tab`, `493-03-closed`, `493-04-ctrl-t`.

## Locked-In Decisions
- D1 — One tab per page: what an agent opens, Chad sees, in the tab bar, without losing his
  focus.
- D2 — Closing a tab closes its page, as in a browser; an agent's page closes the same way.
- D3 — A move between panes is not a close: `on_removed` checks, after the effect cycle, whether
  any Browser tab still shows the page.
- D4 — The agent tools act on the tab the user focused last unless the call names one.
- D5 — A tab that must not take the focus never hides the tab that has it (Code phase, from
  Zed's focus-loss rule: a pane whose focused item goes behind gives the focus to its new front
  item). It comes to the front of a pane without the focus; in the pane with the focus it joins
  the tab bar behind the active tab; and when no Browser tab is open and the pane with the focus
  shows other work, such as the agent's own terminal, it opens in a new pane split beside that
  one, and the focus goes back where it was.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a page opens another page, a Browser tab for it shall open next to the opener's tab and take the focus. | Shot `493-01-new-tab` |
| REQ-002 | WHEN an agent or another CDP client opens a page, a Browser tab shall open for it without taking the focus. | Shot `493-02-agent-tab` |
| REQ-003 | WHEN the user closes a Browser tab, its page shall close; WHEN a page closes elsewhere, its tab shall close. | Shot `493-03-closed`; the run log's tab list |
| REQ-004 | WHEN the user presses Ctrl+T in a Browser tab, a new blank page shall open in a new tab with the address bar focused. | Shot `493-04-ctrl-t` |
| REQ-005 | WHEN an agent calls `browser_tabs`, the answer shall list every page with its id, title and URL and mark the one the user focused last; a browser tool given a `tab` shall act on that page. | The run log |

## Phase Plan
- **P1 Plan** — this spec, and the design and E2E plan in the notes.
- **P2 Code** — the hub's pages, the tab lifecycle, `NewBrowserTab`, the tools' `tab` and
  `browser_tabs`; fmt and clippy clean; a review of the diff.
- **P3 Test** — write and run `493-browser-tabs.sh`, and the #488 to #492 scenarios again on the
  new hub; read every shot; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, the architecture notes, the plan's status, ledger capture, close,
  archive, commit.
