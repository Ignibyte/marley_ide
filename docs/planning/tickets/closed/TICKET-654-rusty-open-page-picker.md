# TICKET-654: Open a brain page by name, or make it, from a picker

- **Ticket:** LOCAL #654 (feature, Rusty in Marley R3a: `rusty: open page`, R-D3's QuickSwitcher
  row)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/completed/654-rusty-open-page-picker.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D3's QuickSwitcher row, R-D10's
  `searchpalette-spotlightsearch-quicklauncher` row, the slices table's R3a, Rusty's triage);
  #645's Out list ("The second slice, its own ticket: `rusty: open page`'s picker"); Chad,
  2026-10-03: "lets make a plan to begin the work and spec out the tickets", "lets make sure we use
  the gpui components we found here"
- **Status:** closed

## Summary
`rusty: open page` opens a picker over every page of Rusty's brain, the screen Rusty's Qt app
draws as `QuickSwitcher.qml` and Obsidian opens on Ctrl+O. It is Zed's own `picker` crate in the
workspace's modal layer, as Zed's file finder and Marley's New Agent picker are, over one
`brain_list_pages` read: titles and slugs matched by Zed's `fuzzy_nucleo` on the machine, the pages
Marley opened most recently first, and a "Create page" row at the end when the query names no
page, which calls `brain_new_page` with the query split into `folder` and `name` (Rusty's
TICKET-041 turns a slashed name into a root page). A click on an unresolved link in a Page tab
creates its page the same way, which #645 left to this slice. `rusty::OpenPage`'s slug becomes
optional, so the command palette lists the action, and `secondary-alt-u` (Ctrl+Alt+U on Linux)
opens it from the editor, a terminal or the rail; Rusty's own Ctrl+O stays the terminal program's
and Zed's. Behind `marley.rusty.enabled` (#643), off by default. Comes after #645.

## Acceptance
Ctrl+Alt+U opens the picker with every page by title and slug; typing ranks title and slug
matches with the matched letters lit; Enter opens the page in a kept Page tab. With an empty query
the recently opened pages lead, the current one first and the selection on the one before it. A
query that names no page ends the list with "Create page: PATH", which makes the page in its
folder through Rusty and opens it; Rusty's refusal shows in a toast with the query kept. A click on
an unresolved link makes that page and shows it. The palette lists `rusty: open page`; with Rusty
off or not connected the key says so and opens nothing.
