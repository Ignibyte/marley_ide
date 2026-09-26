---
pipeline_id: 3dd1a382-211c-4d51-b0c8-67c944f7eb25
ticket: docs/planning/tickets/open/TICKET-578-restored-browser-tab-draws-its-reopened-page.md
status: QUEUED — Phase 1 Plan drafted; the first diagnosis ruled out in Test, a reproduction needed before promoting
title: "A restored Browser tab sometimes draws nothing"
type: bug
slice: prong 3 (B1c, #494's restore)
references: [docs/planning/pipeline/completed/494-browser-restore.notes.md, docs/planning/pipeline/completed/576-browser-tabs-table-item-id-not-unique.notes.md]
---

## Title
**Revised in Test (2026-09-26):** the diagnosis below is ruled out; see the notes' Phase 3. The blank
is intermittent (2 of 6 runs of #576's scenario) and its cause open.

A Browser tab restored at a launch paints before its page is back, and `BrowserView::start_viewing`
counts it among the viewers of its saved page id, which the browser no longer has when Marley's
Chromium stopped. When the tab reopens its saved URL in a new page, `show_page` gives it the new
page's id but leaves `viewing` set, so `start_viewing` returns early from then on: the new page
never streams to the tab, which stays blank though its title, URL and `browser_tabs` say the page
loaded (#576's last shot). Here `show_page` lets go of the old page's viewing first, so the tab's
next paint counts it among the new page's viewers.

## Scope
### In
- `crates/marley_workbench/src/browser.rs`: `BrowserView::show_page` stops viewing the old page
  before it takes the new one.

### Out (explicitly deferred)
- Nothing else in the restore.

## Reference (§20)
- **Upstream Zed:** N/A (the Browser tab is Marley's).
- **Warp:** N/A. Marley's own Browser tab.
- **N/A — Marley-specific:** #494's viewer rule (a tab counts as its page's viewer from its first
  paint in front of its pane) is kept; only the page it counts for follows the tab.

### Prior art
- **Behavior maps.** None apply.
- **Published material.** CDP's `Page.startScreencast` streams only while a tab asks; #488's hub
  starts it per page with viewers.
- **The code we already ship.** `start_viewing` (called from `PageElement`'s paint),
  `stop_viewing` and `forget_page`; `show_page` is where every new page reaches a tab
  (`page_created`, and a start's page adopted by a waiting tab). No crate owns more of this seam.

## UI proof
UI-AFFECTING: a restored Browser tab's page drawn.
`script/e2e/578-restored-browser-tab-draws-its-reopened-page.sh` (`compositor sway`; an offline
Chromium; a page with a light background). Shots:
- `578-01-before`: the repository with a Browser tab on the page, drawn.
- `578-02-restored`: after Marley and its Chromium stop and Marley starts again, the restored tab
  drawing the page it reopened; a pixel of the page area is the page's light background.

## Locked-In Decisions
- D1: `show_page` stops viewing before it takes a new page; the next paint in front starts viewing
  the new page, so a tab behind others still counts for none.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a restored Browser tab in front of its pane reopens its saved URL in a new page, the tab shall draw that page. | Shot `578-02-restored`; the run log: a pixel of the page area |
| REQ-002 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec and the notes.
- **P2 Code:** `show_page`; fmt and clippy clean; a review.
- **P3 Test:** the scenario, red on the build before; the shots; `just regress`; the gate.
- **P4 Complete:** CHANGELOG (Fixed), the crate note, knowledge, close, archive, commit.
