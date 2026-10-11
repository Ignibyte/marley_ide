---
pipeline_id: c934ec84-6827-4688-bc24-1d04522f47e9
ticket: docs/planning/tickets/closed/TICKET-743-the-rails-top.md
status: Phase 4 — Complete PASS
title: The rail's top - the Rusty group leaves, Home becomes Default
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), tabs everywhere (docs/planning/design-notes/tabs-everywhere-2026-10-11.md)
references:
  - docs/planning/pipeline/completed/700-home-and-rusty-groups-from-the-start.spec.md
  - docs/planning/pipeline/completed/739-rustal-icons-in-the-status-bar.spec.md
---

## Title
The rail stops making a Rusty group, takes out the one an earlier build made, and opens Rusty's
screens and pages as tabs of the group you are looking at; the Home group is called Default. Chad,
2026-10-11: "The rusty panel shouldnt be there anymore on start up", and "i cant manually remove it.
but rusty now is in the bottom panel which is what we want". Since #739 the status bar's Rusty button
opens Rusty's home page wherever you are, so the group only takes a row.

## Scope
### In
- No window makes a Rusty group, at start (#700) or on a Rusty open (#675).
- Every Rusty opener (the screens, Open Page, page links, the graph, the home page) opens its tab in
  the group the window shows, deferred as before; Rusty's home page is no longer forced first.
- A Rusty group restored from an earlier build, shown at the quit or not, is closed and forgotten;
  a window showing it shows Default first.
- The Home group's name in the rail is Default, a restored record named Home included. The status
  bar's Home button and the Home page keep "Home".
- Rusty's home page gets the four entries only the Rusty group's + menu had: Open Page…, Capture to
  Today…, Capture to Inbox…, Capture a URL….
- `docs/marley/guide.md` and `docs/marley/walkthrough.md`: no Rusty group, Default in place of Home.

### Out (explicitly deferred)
- The + menu hook in Zed's `pane.rs` (`MarleyNewItemMenu`, #699) loses its only user; #744 decides
  whether one New Agent… uses it or the hook leaves Zed.
- Moving Marley and Rusty to the title bar (#746), Threads in the rail (#745).
- Renaming the code's Home kind and the record's `home` field: the name is a display name only.

## Reference (§20)
N/A — Marley-specific: the rail's projectless groups (#600), its Home and Rusty groups (#675, #700)
and Rusty are Marley's own; Zed's sidebar has no group without a folder. Upstream Zed's behavior kept:
`MultiWorkspace::remove` with `RemovalIntent::CloseProject` closes the retired group's workspace with
Zed's own close prompts, as the rail's Remove Group already does (#600).

### Prior art
- The code we ship: `groups.rs` already has every piece: `with_group(Home)` makes Default on demand,
  the rail's `remove_group` closes a group through `MultiWorkspace::remove` and forgets it, and
  `in_group` holds the deferred open that F-claude-679 needed. `adopt` and `reopen` (#601) bring a
  saved group back, so the retired group meets the same path every other group does.
- `rusty::in_rusty_group` is the one funnel every Rusty opener calls (L-claude-675), so the route
  changes in one place.
- The status bar's Rusty button (#739, `home_tab::open_here`) already opens the home page in the
  shown group; the home page's screen buttons (#679) already reach every screen.
- Behavior maps: `docs/zed_architecture/` has nothing on folderless groups; none in the Warp or Orca
  maps either (Warp's tabs have no app-owned group).

## UI proof
Scenario `script/e2e/743-the-rails-top.sh` (`compositor sway`). The installed build (19dda499a2,
with #700's Rusty group) runs first on the run's profile with Rusty on through its stand-in, shows
the Rusty group and quits; the debug build then restores that session.
- `old`: the installed build's rail, Home then Rusty on top (the starting point).
- `restored`: the debug build after the restore: Default first, no Rusty row, Default shown (REQ-001,
  REQ-003, REQ-004).
- `brain-here`: a Rusty screen opened from the palette while the repo shows, as a tab in the repo
  (REQ-002).
- `rusty-home`: Rusty's home page with Open Page… and the three captures (REQ-005), and
  `open-page`: Open Page… clicked opens the page picker.

## Locked-In Decisions
- D1 — No group of the Rusty kind is made any more; `GroupKind::Rusty` goes. The record's `rusty`
  field stays (serde default) only to find an earlier build's group.
- D2 — Rusty's openers run in the shown group: `groups::in_shown(asked_from, …)` defers, then runs
  the open in the window's shown workspace in the Marley layout, and in `asked_from` in Zed's, as
  `in_group` does today.
- D3 — One path retires an old Rusty group: it is adopted as any saved group is (#601; a hidden one
  is reopened first), and the rail's refresh closes each live one through `remove_group` after the
  window shows Default (made when missing). A group whose close the user cancels (a running
  terminal's prompt) is kept as a named group called Rusty, which Rename and Remove reach.
  Rejected: dropping Rusty records when the store is read: a Rusty group shown at the quit comes
  back as the window's workspace with no record, a folderless workspace with no row.
- D4 — Default is the Home kind's display name; the kind, `is_home` and the record's `home` field
  keep their names, so #700's start rules and saved records carry over. `adopt` names a Home record
  Default whatever it was saved as. Rejected: renaming the kind across the crate (churn, no
  behavior).
- D5 — The Rusty group's quick links move to Rusty's home page, which the status bar's Rusty button
  opens; no + menu lists them. Rejected: Rusty's links in every + menu (#699 kept them to the
  Rusty group on Chad's word).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a window starts or Rusty turns on, the system shall make no Rusty group, and the rail shall list Default above the projects. | shot `restored` |
| REQ-002 | WHEN a Rusty screen or page opens, the system shall open it as a tab of the group the window shows, without showing another group. | shot `brain-here` |
| REQ-003 | WHEN a window restores a Rusty group an earlier build made, the system shall close the group and leave it out of the rail, showing Default in its place when it was shown. | shot `restored` |
| REQ-004 | The rail shall name the window's Home group Default, a restored record named Home included; the status bar's Home button and the Home page shall keep the name Home. | shot `restored`; the diff |
| REQ-005 | Rusty's home page shall offer Open Page…, Capture to Today…, Capture to Inbox… and Capture a URL…. | shot `rusty-home` |
| REQ-006 | IF the user cancels the close of a retired Rusty group, THEN the system shall keep it as a named group called Rusty. | review of the diff (a scenario cannot hold a running process in a group the old build made without driving that build further) |
| REQ-007 | The change shall pass `script/gates.sh --diff`. | the gate's exit code |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `groups.rs`, `rail.rs`, `rusty.rs`, the Rusty openers and `home_tab.rs`; a review of
  the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run `script/e2e/743-the-rails-top.sh`, read every shot.
- **P4 Complete** — CHANGELOG, guide and walkthrough (§21), ledger capture (§19), close the ticket,
  archive, commit.
