---
pipeline_id: 32916ce1-950f-4632-8616-17b4d2f63154
ticket: docs/planning/tickets/open/TICKET-679-the-rusty-home-page.md
status: Phase 4 — Complete PASS
title: "The Rusty home page, and one Rusty button in the rail"
type: feature
slice: Rusty in Marley, after #675 and #678
references: [docs/planning/pipeline/completed/678-the-brain-tab.spec.md, docs/planning/pipeline/completed/675-the-rusty-group.spec.md, docs/planning/pipeline/completed/672-rusty-screens-in-the-rail-header.spec.md]
---

## Title
A Rusty home page, a dashboard tab always first in the Rusty group, and one Rusty button in the
rail in place of the row of screen buttons. Chad, 2026-10-07 (quoted in the ticket): the buttons go
"into the rusty home page card for now", with the vault's recents and a tasks table "just to have
some visual".

## Scope
### In
- **The header**: while Rusty is connected, one Rusty button (a placeholder icon, `RUSTY_ICON`) and
  PROJECTS; the row of eight screens and its `…` go. The Rusty group's header takes the same icon.
- **The home tab** (`rusty: open home`, the Rusty button): in the Rusty group, put first among its
  tabs whenever anything opens there. Cards:
  - **Rusty**: a button per screen (Brain, Today, Graph, Tasks, Decisions, Memory, Skills, Secrets);
  - **Recent pages**: the pages opened lately, newest first, each opening in the Brain tab;
  - **Follow-ups due**: `brain_due`'s decisions, the date and whether overdue, each opening its
    page in the Brain tab;
  - **Tasks**: the open tasks of every list as a table, task and list, each opening Tasks on its
    list.
- **Fresh**: the cards read again when Rusty announces a change and when it connects.

### Out (explicitly deferred)
- **Chad's own Rusty icon**: one constant to change.
- **More cards** (memory, skills to review, favourites): "for now" is these.

## Reference (§20)
N/A — Marley-specific: a home over Rusty's data. Its parts are Marley's own tabs (#658's Tasks,
#659's Decisions, #678's Brain tab) and the Knowledge panel's project view (#655), whose reads it
repeats.

### Prior art
- **Behaviour maps:** none.
- **Published material:** none applies.
- **The code we ship:** the Knowledge panel's reads (`brain_due`, `list_task_groups`,
  `list_tasks`) and `open_tasks`; the page picker's recents (`RecentPages`); `brain::Screen` and
  `open_screen`; `groups::in_group`; `ui::Button`.

## UI proof
`script/e2e/679-the-rusty-home-page.sh` (`compositor sway`), Rusty's stand-in over a scratch state
folder with a list and tasks, a decision due, and a page. Shots: `679-01-header` (one Rusty button
and PROJECTS); `679-02-home` (Rusty clicked: the home tab first in the Rusty group, its cards);
`679-03-recent` (a recent page clicked: the Brain tab on it); `679-04-screen` (back on home, the
Graph button: the Graph tab, Home still first).

## Locked-In Decisions
- D1 — **Home is forced first**: every open in the Rusty group makes it if missing, at the first
  place, without bringing it forward.
- D2 — **Rows open where their data lives**: pages and follow-ups in the Brain tab, tasks in Tasks.
- D3 — **A placeholder icon of its own** (Blocks), not the Brain tab's book.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Rusty is connected, the rail's header shall show one Rusty button in place of the screen buttons. | `679-01-header` |
| REQ-002 | WHEN the Rusty button is clicked, the system shall open the Rusty home tab in the Rusty group, first among its tabs. | `679-02-home` |
| REQ-003 | The home tab shall show a card of every screen's button and cards of recent pages, open tasks and follow-ups due. | `679-02-home` |
| REQ-004 | WHEN a recent page is clicked, the system shall show it in the Brain tab. | `679-03-recent` |
| REQ-005 | WHEN a screen's button is clicked, the system shall open that screen, the home tab staying first. | `679-04-screen` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `rusty/home_tab.rs` (new), `rusty.rs`, `rusty/brain_tab.rs`, `rusty/page_picker.rs`,
  `rail.rs`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the scenario and its shots.
- **P4 Complete** — CHANGELOG, the guide, the architecture notes, ledger capture, close, archive,
  commit.
