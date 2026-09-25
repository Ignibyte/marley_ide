---
pipeline_id: 40d7b9e1-5486-4de6-b226-4e1032f160e8
ticket: docs/planning/tickets/open/TICKET-494-browser-tabs-restored.md
status: Phase 4 — Complete PASS
title: "B1c: Browser tabs return after a relaunch"
type: feature
slice: prong 3 B1c (split from #493)
references: [docs/marley/three-prong-plan.md, docs/marley_architecture/embedded-browser-model.md, docs/planning/pipeline/completed/493-browser-tabs-and-restore.spec.md]
---

## Title
Browser tabs are saved with the workspace and come back when Marley relaunches.

## Scope
### In
- `BrowserView` is a `SerializableItem`, registered with `workspace::register_serializable_item`,
  with a table of its own in Marley's database (a `db` domain): per tab, its item and workspace
  ids, its page's target id and its URL. The layout's codec holds only the item, as Zed's items
  do.
- On relaunch each tab is deserialized: when Marley's Chromium still runs its page (the target
  id answers), the tab reattaches to it; otherwise a new page opens at the saved URL. A tab
  whose URL is `about:blank` returns blank.
- The saved URL is the page's committed URL, rewritten whenever it changes; an agent's
  navigation counts as any other. The tab's title is saved beside it, so a returning tab reads
  as it did while its page comes back.
- A start takes stock without opening anything: a Chromium Marley starts opens no page of its
  own (`--no-startup-window` in place of `about:blank`), and the pages a start finds get no tab
  of their own. A restored tab claims its page; `marley: open browser` gives the others tabs; a
  tab opened while the browser starts takes one of them, or opens a blank page when the start
  brings none.
- The e2e harness gains the steps `quit_marley` and `launch_marley`, which quit Marley through
  its palette and start it again on the same profile and path.

### Out (explicitly deferred)
- Back and forward history across a Chromium restart.
- Restoring scroll positions and form contents.

## Reference (§20)
Upstream Zed for the contract: `workspace::SerializableItem` (`serialize`, `deserialize`,
`cleanup`, `should_serialize`) with a `db` domain per item kind; Zed's `onboarding` item is the
lightest real one. The embedded-browser model's rule (#403, `embedded-browser-model.md` Q4)
keeps URLs out of the layout's codec. Warp: N/A.

### Prior art
- **Code we already ship.** `workspace::register_serializable_item`, `SerializableItem`,
  `ItemId` and `WorkspaceId`; the `db` crate's `static_connection!` domains
  (`crates/onboarding`, `crates/terminal_view`'s `TerminalDb`); #493's pages, each a target id.
- **Published material.** CDP's `Target.getTargets` and `Target.attachToTarget`, which say
  whether a saved page still lives.
- **Observed (the probe, 2026-09-25).** Headless Chromium started with a URL opens that page;
  with none it opens `chrome://newtab/`; with `--no-startup-window` it opens no page, and
  `Target.createTarget` still opens one on the same connection.

## UI proof
UI-AFFECTING. `script/e2e/494-browser-restore.sh` (`compositor sway`): two Browser tabs on two
fixture pages, Marley quit through the palette and started again on the same profile (a harness
step that relaunches, keeping the profile), both tabs back on their pages; then Marley's
Chromium unit stopped and Marley relaunched again, both tabs back at their URLs on new pages.
Shots: `494-01-before`, `494-02-reattached`, `494-03-reopened`. The first page holds text
typed into a field, which survives a reattach and not a reopen; the run log prints the tabs'
ids before and after each relaunch, and the profile database's rows for the tabs: the
workspace's `items` rows (kind and id) and the tabs' own rows (URL and title).

## Locked-In Decisions
- D1 — The item's own table holds page ids and URLs; the layout's codec holds only the item.
- D2 — A live page is reattached rather than reloaded, so a relaunch of Marley alone loses
  nothing the page holds.
- D3 — A start opens no page and places no tab for the pages it finds: which page belongs in
  which tab is the restored tabs' to say, and a page no tab claims waits for `marley: open
  browser` or an agent's call.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley relaunches while its Chromium runs, each Browser tab shall return on its page. | Shot `494-02-reattached` |
| REQ-002 | WHEN Marley relaunches after its Chromium stopped, each Browser tab shall return at its saved URL. | Shot `494-03-reopened` |
| REQ-003 | The layout's codec shall hold no URL. | Review: the table and the item's `serialize`; the run log's `items` rows |
| REQ-004 | WHEN Marley starts its Chromium, the browser shall open no page that no Browser tab or agent asked for. | Shot `494-03-reopened` (two tabs, no third); the run log's tab list |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the item, its table, reattach or reopen; fmt and clippy clean.
- **P3 Test** — the relaunch step in the harness, the scenario, every shot; the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
