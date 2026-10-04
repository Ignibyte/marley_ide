# TICKET-646 — The Knowledge panel: a page's backlinks, links and tags, and brain search

- **Ticket:** LOCAL #646 (feature, Rusty in Marley R3 and the Knowledge panel of R-D3; R6's
  project view split out to its own ticket)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/completed/646-knowledge-panel.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D3, R-D5, R-D10, slices R2, R3 and R6);
  Chad, 2026-10-03: "lets make a plan to begin the work and spec out the tickets", "lets make
  sure we use the gpui components we found here", "Queue all five"
- **Status:** closed (2026-10-04)

## Summary
Rusty's Qt app keeps a right pane beside each note: its backlinks with the line each sits on,
its outgoing links (an unresolved one creates the page), the outline and the vault's tags, and a
search pane over the brain with Obsidian's operators. In Marley that becomes the Knowledge panel
in the right dock, drawn from `rusty-mcp`'s tools over #643's connection. While #645's Page tab is
the active item, the panel shows that page's tags with their counts, its backlinks and its links;
a click opens a page through #645's action, and a tag searches for itself. A search field at the
top sends what is typed to `brain_search` as typed, so `tag:`, `path:`, `file:` and `type:` work
as they do in Rusty. The panel re-reads when Rusty announces a change. It exists only while
`marley.rusty.enabled` is on: off, there is no dock button and its toggle says Rusty is off. The
row layouts port Ely GPUI Components' Backlinks and SearchResultItem onto Zed's `ui` crate. The
project view R-D5 describes (the rail's project as a brain page, its follow-ups due and its task
group) is a second slice, deferred to a ticket of its own.

## Acceptance
With Rusty off, no Knowledge button shows and the toggle answers with a toast. With Rusty on and
a brain page in a Page tab, the panel shows the page's tags with counts, its backlinks with their
lines and the link lit, and its links, an unresolved one offered for creation; clicks open pages
and follow them, a tag click searches `tag:<name>`, typed queries list results with their
snippets, and a change Rusty announces shows without input. Turning Rusty off closes the panel
and hides its button without a restart.
