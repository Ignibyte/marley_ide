---
pipeline_id: 917c17e5-275d-4416-85ef-25891bd3310e
ticket: docs/planning/tickets/open/TICKET-662-rusty-favourites.md
status: Phase 4 — Complete PASS
title: "Favourites and bookmarks from Rusty"
type: feature
slice: Rusty in Marley R4b (docs/marley/rusty-in-marley.md)
references: [docs/marley/rusty-in-marley.md, docs/planning/pipeline/completed/644-brain-view-in-the-rail.spec.md, docs/planning/pipeline/completed/654-rusty-open-page-picker.spec.md]
---

## Title
Favourites and bookmarks from Rusty. Rusty keeps bookmarks in the vault (its TICKET-037:
`bookmark_list`, `bookmark_add`, `bookmark_remove`, `bookmark_set`; kinds `file`, `folder`, `search`
and `heading`; file and folder bookmarks are the favourites). Marley shows none. This ticket shows
them above the Brain view's tree, puts a star on a page's tab, lists favourite pages first in the
page picker, and edits the list through Rusty's tools.

## Scope
### In
- **Favourites in the Brain view.** While Rusty holds any bookmark, a Favourites group above the tree
  lists them in Rusty's order, each with its kind's icon and title: a page opens as a tree row does
  (one click a preview tab, two a kept tab), a folder is revealed and opened in the tree, a search
  runs its query in the Brain view's search, and a heading opens its page scrolled to that heading.
  A right-click gives Rename… (the title, edited in place) and Remove.
- **The star.** A Page tab's header shows a star, filled when the page is a bookmark; a click adds or
  removes it. `rusty: toggle bookmark` (Ctrl+D in a Page tab in Read) does the same for the page in
  front.
- **The page picker.** On an empty query, favourite pages come after the page in front and before the
  recent ones, each with a star.
- **Live.** The list is Rusty's: read when the Brain view or a Page tab needs it, again on Rusty's
  announcement and after each write, so a bookmark made in Rusty's app shows in Marley.
- **`marley_rusty::bookmarks`** and the stand-in's four tools, with Rusty's rename and delete rules.

### Out (explicitly deferred)
- **Reordering by drag**; `bookmark_set` keeps Rusty's order, and Rename keeps the place.
- **Bookmarking a search or a heading from Marley** (they show and open; Rusty's app or an agent
  makes them). A heading bookmark from the outline column is a later step.
- **Keyboard navigation inside the Favourites group** (it sits above the tree's list).

## Reference (§20)
N/A — Marley-specific: Rusty's own app is the reference for the behaviour (`crates/rusty-app/qml/
BookmarksPane.qml`, `Explorer.qml:196-226`, `NoteTab.qml:390-420`, `QuickSwitcher.qml:12-14`,
`Main.qml:329-365` in Rusty's repository at `9281c35`): favourites above the explorer tree, a star in the
note header and Ctrl+D, favourites first in the quick switcher with a star, a heading opened by its
text, and the list written whole with `bookmark_set`. Zed's own rows, icons and context menus draw it.

### Prior art
- **Behaviour maps:** `docs/orca_architecture/` and `docs/t3code_architecture/` hold no bookmark list
  of a notes vault; `docs/zed_architecture/` none for the project panel.
- **Published material:** Rusty's tool contract (`crates/rusty-mcp/src/main.rs:111-145`, `:1275-1315`)
  and its store (`crates/rusty-core/src/brain/bookmarks.rs`: identity by kind and path, query or
  heading; duplicates dropped and blank titles filled on set; renames carried and deletes dropped,
  `:85-152`).
- **The code we ship:** the Brain view's tree, reveal, search and inline edit (`rusty/brain.rs`);
  `ProjectPages` (`rusty/project.rs:55-158`) as the model of a Rusty-backed global read on need and
  on announcement; the Page tab's outline scroll (`scroll_to_heading_line`, #656); the page picker's
  `switcher::empty_order` (#654); Zed's `IconName::Star` and `StarFilled`. No crate we build owns
  bookmarks.

## UI proof
`script/e2e/662-rusty-favourites.sh` (`compositor sway`: clicks on rows, the star and menus). Setup:
the stand-in over a scratch vault (`notes/alpha`, `notes/beta` with a heading `## Second`,
`projects/atlas`), and `.rusty/bookmarks.json` holding a folder (`projects`), a search (`tag:idea`) and
a heading (`notes/beta`, `Second`); `MARLEY_RUSTY_MCP` names it, never the user's Rusty (R-D8).
Shots:
- `662-01-favourites`: the Brain view: Favourites with the folder, the search and the heading above
  the tree.
- `662-02-starred`: `notes/alpha` opened from the tree and its star clicked: the star filled, alpha
  under Favourites.
- `662-03-picker`: Ctrl+Alt+U: alpha after the page in front, starred, before the recent pages.
- `662-04-folder`: the folder favourite clicked: `projects` open in the tree, selected.
- `662-05-search`: the search favourite clicked: `tag:idea` in the field, its hits listed.
- `662-06-heading`: the heading favourite clicked: `notes/beta` with `Second` at the top.
- `662-07-renamed`: Rename… on alpha's favourite, "Alpha notes" typed: the new title in place.
- `662-08-ctrl-d`: Ctrl+D in alpha's tab: the star empty, alpha gone from Favourites.
- `662-09-outside`: `bookmarks.json` changed from outside (as Rusty's app would): the group follows.
- `662-10-removed`: Remove on the search favourite: gone.

## Locked-In Decisions
- D1 — **One global, read as `ProjectPages` is**: `Bookmarks` in a new `rusty/favourites.rs`, read when
  the Brain view or a Page tab first needs it, on Rusty's announcement, on reconnect and after each
  write; cleared when Rusty turns off.
- D2 — **Writes go through Rusty's add and remove for the star and Ctrl+D**, and through `bookmark_set`
  for Rename (the whole list with one title changed, in Rusty's order); each answer is the list.
- D3 — **The group sits above the list**, drawn only while it holds a bookmark, so the tree's keys,
  rows and every earlier scenario's coordinates are unchanged when there is none.
- D4 — **A heading opens by its text against the page's outline**, then `scroll_to_heading_line` (#656),
  as Rusty's app matches `scrollToHeadingText`; an `open_later` variant carries the heading to a new
  or an existing tab.
- D5 — **Ctrl+D in `RustyPage`**: free in Read, where the tab's own focus has no `Editor`; in Edit,
  Zed's select-next wins, as in Rusty's app.
- D6 — **The picker's order**: the page in front, then favourite pages, then recent, then the rest;
  a favourite also recent shows once, among the favourites.
- D7 — **The stand-in follows Rusty**: the list at `.rusty/bookmarks.json`, identity by kind and
  path, query or heading, a blank title filled, duplicates dropped, renames carried, deletes
  dropped, and its watcher announcing the file's changes.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Rusty holds bookmarks, the Brain view shall show a Favourites group above the tree listing them in Rusty's order with each kind's icon and title. | Shot `662-01-favourites` |
| REQ-002 | WHILE Rusty holds no bookmark, the Brain view shall show no Favourites group. | Review; earlier scenarios unchanged |
| REQ-003 | WHEN a page's star is clicked, the system shall add the page as a bookmark, or remove it when it is one, and the star shall show which. | Shots `662-02-starred`, `662-08-ctrl-d` |
| REQ-004 | WHEN Ctrl+D is pressed in a Page tab in Read, the system shall do what the star does. | Shot `662-08-ctrl-d` |
| REQ-005 | WHEN the page picker opens with an empty query, it shall list favourite pages after the page in front and before the recent pages, each with a star. | Shot `662-03-picker` |
| REQ-006 | WHEN a folder favourite is clicked, the Brain view shall open that folder in the tree and select it. | Shot `662-04-folder` |
| REQ-007 | WHEN a search favourite is clicked, the Brain view shall put its query in the search field and show its hits. | Shot `662-05-search` |
| REQ-008 | WHEN a heading favourite is clicked, the system shall open its page with that heading at the top. | Shot `662-06-heading` |
| REQ-009 | WHEN Rename… is chosen on a favourite and a title entered, the system shall send the list with that title changed and show it. | Shot `662-07-renamed`; the stand-in's log |
| REQ-010 | WHEN Remove is chosen on a favourite, the system shall remove it. | Shot `662-10-removed` |
| REQ-011 | WHEN Rusty announces a change, the group and the stars shall show the list as it now is. | Shot `662-09-outside` |
| REQ-012 | WHILE Rusty is off, nothing of the bookmarks shall show. | Review (#661's filter; the group and star read nothing while off) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_rusty::bookmarks`; the stand-in's four tools and rename/delete rules;
  `rusty/favourites.rs` (the global, the group, its menu, the action); the Brain view's hook, its
  menu split under the line cap; the Page tab's star and Ctrl+D; the heading opener; the picker's
  order; the keymap; the guide page; a review; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, the plan's R4b, the architecture notes, the guide, ledger capture,
  the brain decision, close, archive, commit.
