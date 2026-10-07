---
pipeline_id: bfc6b4dd-7f0b-4cbe-ba8f-d8df8145114f
ticket: docs/planning/tickets/open/TICKET-678-the-brain-tab.md
status: Phase 4 — Complete PASS
title: "The Brain tab: the vault's navigation inside its own tab"
type: feature
slice: Rusty in Marley R-D9, superseding the rail's Brain view (#644)
references: [docs/planning/pipeline/completed/644-brain-view-in-the-rail.spec.md, docs/planning/pipeline/completed/658-rusty-tasks-tab.spec.md, docs/planning/pipeline/completed/675-the-rusty-group.spec.md]
---

## Title
The vault's tree moves from the rail into a Brain tab with its own navigation column, the page on
its right. Chad, 2026-10-07: "the tasks has its own sub navigation system (doesnt overtake the left
panel) but brain does. Lets make brain have the sub navigation instead of taking ove the left", and
on where a page shows, "lets proceed with" the page inside the tab.

## Scope
### In
- **The Brain tab** (in the Rusty group, #675): on the left the Brain view as it was in the rail
  (search, favourites, the tree, its menus and keys), on the right the page a click picks, with the
  page's own back, forward, Edit, outline and star; "Pick a page" until one is picked. One per window.
- **Opening it**: the header's Brain button and `ToggleBrainView` (Ctrl+Alt+V) open or focus it;
  Today opens it and shows today's note there.
- **A page's menu** in the tree gains Open in New Tab: a Page tab of its own, in the Rusty group.
- **Followers**: the Knowledge panel, the Graph tab's Local and the page picker read the Brain
  tab's page as the page in front.
- **The rail**: the Brain view, the Projects/Brain switch and New Page leave the header; Brain is
  the first screen button. While Rusty is connected the header holds the screens and the folder +.

### Out (explicitly deferred)
- **Restoring the Brain tab after a restart**.
- **Opens from elsewhere** (the page picker, the Knowledge panel, the Graph tab, Decisions, links in
  a Page tab) keep opening Page tabs in the Rusty group, as now.

## Reference (§20)
N/A — Marley-specific: Rusty's app had no in-tab navigation; the shape follows Marley's own Tasks
tab (#658), lists beside the selected list, and Obsidian's file explorer beside the note (R-D9's
note on Obsidian's ribbon, `docs/marley/rusty-in-marley.md`).

### Prior art
- **Behaviour maps:** none for the vault; the Tasks tab's two columns are the house shape.
- **Published material:** none applies.
- **The code we ship:** `BrainView` (#644) whole; `PageView` (#645) and its `navigate`;
  `groups::in_group` (#675, #676); `Item::act_as_type`, which a page's followers can read through.

## UI proof
`script/e2e/678-the-brain-tab.sh` (`compositor sway`), Rusty's stand-in over a scratch vault. Shots:
`678-01-header` (the rail with no switch, Brain the first screen); `678-02-tab` (Brain clicked: the
tab in the Rusty group, the tree on its left, "Pick a page" on its right); `678-03-page` (the folder
opened, the page clicked: the page on the right); `678-04-new-tab` (Open in New Tab from the page's
menu: a Page tab of its own beside the Brain tab).

## Locked-In Decisions
- D1 — **The Brain view is reused whole**, hosted by the tab; its opens go to the tab, deferred.
- D2 — **The tab carries one page view** and navigates it; its followers find it through
  `act_as_type` and a shared `page::page_in`.
- D3 — **The header's switch goes**: with no Brain view in the rail, Projects has nothing to switch
  from.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The rail shall never show the vault's tree, and its header shall offer Brain as a screen button. | `678-01-header` |
| REQ-002 | WHEN Brain is clicked or Ctrl+Alt+V pressed, the system shall open or focus the Brain tab in the Rusty group, its navigation on the left. | `678-02-tab` |
| REQ-003 | WHEN a page is clicked in the Brain tab's tree, the tab shall show that page on its right. | `678-03-page` |
| REQ-004 | WHEN Open in New Tab is chosen on a page, the system shall open the page in a tab of its own. | `678-04-new-tab` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `rusty/brain_tab.rs` (new), `rusty/brain.rs`, `rusty/page.rs`, the followers,
  `rail.rs`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the scenario and its shots.
- **P4 Complete** — CHANGELOG, the guide, the architecture notes, ledger capture, close, archive,
  commit.
