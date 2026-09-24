---
pipeline_id: 4a8d13c3-78e6-4bb8-9e4c-79484bf0abf3
ticket: docs/planning/tickets/open/TICKET-493-browser-tabs-and-restore.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "B1b: Browser tabs, restore on relaunch, and the select picker"
type: feature
slice: prong 3 B1b
references: [docs/marley/three-prong-plan.md, docs/marley_architecture/embedded-browser-model.md, docs/planning/pipeline/queued/488-browser-pane.spec.md, docs/planning/pipeline/queued/490-browser-navigation.spec.md]
---

## Title
One Browser tab per page, tabs that survive a relaunch, and a `<select>` picker drawn by
Marley.

## Scope
### In
- **Tabs as pages.** Each `page` target in Marley's Chromium is at most one Browser tab. A
  page opened by a page (`window.open`, a `_blank` link) opens a Browser tab next to its
  opener's and takes the focus, as a browser does; a page an agent opens (#492's
  `browser_navigate` with `new_tab`, or any CDP client's `Target.createTarget`) opens a tab
  without taking the focus. Closing a Browser tab closes its page (`Target.closeTarget`); a
  page closed elsewhere closes its tab.
- `marley::NewBrowserTab` (Ctrl+T in the `MarleyBrowser` context, and the palette) opens a
  blank page with the address bar focused; `marley: open browser` still activates the
  newest Browser tab.
- #492's tools act on the Browser tab the user focused last, and `browser_tabs` lists them
  (id, title, URL, which one the user sees).
- **Restore.** Browser tabs are saved with the workspace through Zed's
  `SerializableItem`, in a table of their own holding each tab's page id and URL, never in
  the layout's codec. On relaunch each tab reattaches to its page while that page lives, and
  otherwise opens a new page at the saved URL.
- **The `<select>` picker.** A press on a `<select>` (found with `DOM.getNodeForLocation`)
  opens a list of its options under it in Marley, with the current one marked; choosing one
  sets the element's value and fires `input` and `change`, in an isolated world; Escape or a
  click elsewhere closes it. The keyboard still changes a closed select in the page, as
  Chromium does.

### Out (explicitly deferred)
- Tab groups, pinning, moving a page between Marley windows, a private profile.
- `<select multiple>` and `<datalist>`.
- Restoring history (back and forward stacks) across a Chromium restart.

## Reference (§20)
Upstream Zed for the tab behavior: Browser tabs are `workspace::Item`s and follow Zed's pane
rules (activation, closing, drag between panes), and restore follows Zed's `SerializableItem`
contract (the `onboarding` item is the lightest model). For what a page does when it opens
another page, Chromium's own behavior: a new page target per window, as CDP reports it.
Warp: N/A.

### Prior art
- **Published material.** CDP's Target domain (`targetCreated` with `openerId`,
  `targetDestroyed`, `createTarget`, `closeTarget`, `activateTarget`); `DOM.getNodeForLocation`,
  `DOM.describeNode`, `DOM.resolveNode`, `Runtime.callFunctionOn` in an isolated world
  (`Page.createIsolatedWorld`).
- **Observed (the probe).** A `<select>` popup never reaches a headless frame, and no target
  appears for it.
- **Code we already ship.** `workspace::register_serializable_item` and the `db` crate's
  domains (Onboarding's item, `workspace.rs:1309`); `Workspace::items_of_type`,
  `activate_item`, `Pane::add_item`; Zed's `ContextMenu` or a `Picker` for the option list;
  #488's app-wide entity, which now holds a session per page; the embedded-browser model's
  rule to keep URLs out of the layout's codec (#403), kept by the side table.

## UI proof
UI-AFFECTING. `script/e2e/493-browser-tabs.sh` with `COMPOSITOR=sway`. Fixtures: a loopback
site with a `_blank` link, a `window.open` button and a `<select>` that prints its value on
`change`; the stand-in agent opens a page with `Target.createTarget`. The scenario relaunches
Marley in the same profile (a harness step that stops and starts Marley, keeping the
profile). Shots: `493-01-new-tab`, `493-02-agent-tab`, `493-03-closed`, `493-04-restored`,
`493-05-select-open`, `493-06-select-chosen`.

## Locked-In Decisions
- D1 — One tab per page: what an agent opens, Chad sees.
- D2 — Closing a tab closes its page, as in a browser; an agent's page is closed the same way.
- D3 — Restore keeps page ids and URLs in the item's own table, never in the layout's codec.
- D4 — Marley draws the `<select>` list and sets the value in an isolated world, so the
  page's own handlers see an ordinary change.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a page opens another page, a Browser tab for it shall open next to the opener's tab and take the focus. | Shot `493-01-new-tab` |
| REQ-002 | WHEN an agent or another CDP client opens a page, a Browser tab shall open for it without taking the focus. | Shot `493-02-agent-tab` |
| REQ-003 | WHEN the user closes a Browser tab, its page shall close; WHEN a page closes elsewhere, its tab shall close. | Shot `493-03-closed`; the run log's target list |
| REQ-004 | WHEN Marley relaunches, its Browser tabs shall return, each on its page while it lives and at its saved URL otherwise. | Shot `493-04-restored` |
| REQ-005 | WHEN the user presses on a `<select>`, Marley shall show its options, and choosing one shall set the page's value and fire its change event. | Shots `493-05-select-open`, `493-06-select-chosen` |
| REQ-006 | WHEN the user presses Ctrl+T in a Browser tab, a new blank page shall open in a new tab with the address bar focused. | Shot `493-03-closed` (taken after Ctrl+T) |

## Phase Plan
- **P1 Plan** — this spec, and the design and E2E plan in the notes.
- **P2 Code** — a session per page in the app-wide entity, the tab lifecycle, `NewBrowserTab`,
  the serializable item and its table, the select picker, `browser_tabs`; fmt and clippy
  clean; a review of the diff.
- **P3 Test** — write and run `493-browser-tabs.sh` (with the relaunch step), read every shot;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `marley_browser.md`, the plan's status, ledger capture, close,
  archive, commit.
