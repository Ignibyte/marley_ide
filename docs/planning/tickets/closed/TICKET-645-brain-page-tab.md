# TICKET-645: A brain page in a center tab

- **Ticket:** LOCAL #645 (feature, Rusty in Marley R2: the Page tab; decision R-D4)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/completed/645-brain-page-tab.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D3's NoteTab row, R-D4, R-D9, R-D10, the
  slices table's R2); Chad, 2026-10-02 and 2026-10-03: "lets make a plan to begin the work and spec
  out the tickets", "lets make sure we use the gpui components we found here", "Queue all five"
- **Status:** closed

## Summary
A page of Rusty's brain opens as a center tab, a `workspace::Item` beside Marley's Decisions and
Agent tabs. The tab shows the page's title and properties above its body, rendered from
`brain_render`'s `raw` by Zed's own `markdown` crate after a small pure pass in `marley_rusty` turns
each wikilink into an ordinary link that carries Rusty's own `rusty:page/` or `rusty:new/` address,
resolved from `brain_render`'s `links`. A click on a link opens the linked page in the same tab,
with Back and Forward; an unresolved link is drawn muted. One click on a page in the rail's Brain
view (#644) opens it as Zed's preview tab and a double click keeps it, through Zed's own pane API.
Edit turns the tab into a Zed editor over the page's vault file, and Read shows the saved file
rendered; Rusty's watcher picks up the save as it does an Obsidian edit. The action every caller
uses, `rusty::OpenPage { slug, preview }`, lands here, and #644's `open_page` is pointed at it; the
picker over `brain_list_pages` is a second slice of its own. Behind `marley.rusty.enabled` (#643),
off by default.

## Acceptance
A single click on a page in the rail's Brain view shows it in an italic preview tab, with its
properties, headings, lists, task boxes, highlighted code and tables; another single click replaces
it and a double click keeps it. Wikilinks open their page in the tab, a heading link lands on the
heading, Back and Forward walk the tab's history, and an unresolved link is drawn muted and says the
page does not exist yet. Edit shows the vault file in a Zed editor, an edit keeps a preview tab,
and Read saves and shows the result. A change Rusty announces is shown without input.
