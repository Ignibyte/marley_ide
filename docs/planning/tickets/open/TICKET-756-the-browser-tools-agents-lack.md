# TICKET-756 — The browser tools agents lack

- **Ticket:** LOCAL #756 (feature, prong 3: the Browser tab; Marley's MCP server)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [756-the-browser-tools-agents-lack.spec.md](../../pipeline/queued/756-the-browser-tools-agents-lack.spec.md)
- **Source ticket:** [the T3 Code survey](../../../t3code_architecture/README.md), item 10 (report 05
  items 2 to 4), and [Orca report 03](../../../orca_architecture/03-browser-and-design-mode.md),
  item 11. Chad, 2026-10-11: "18.) build now".
- **Status:** open

## Summary
Marley's agents read a Browser tab as well as T3 Code's do, but act on it with less: they can
navigate, go back, click, type, press, scroll and draw boxes, and nothing else. This ticket adds
the acting tools a test of a web app needs:
- `browser_hover`, `browser_select` (an option of a `<select>` by value or label), `browser_drag`,
  `browser_upload` (files from an open project into a file input), `browser_dialog` (answer the
  tab's alert, confirm or prompt), `browser_resize` (a device preset, a size, or back to the tab)
  and `browser_color_scheme` (dark, light or the system's), under the `browser.write` grant;
- `browser_wait`, which waits for text, an element by role and name, or a URL, and acts on
  nothing.

The tab shows a chip while a page runs at a device size or a forced colour scheme, with Reset.

This is the first of three slices. The second puts MCP annotations (`readOnlyHint`,
`destructiveHint`) on every Marley tool; the third brings file choosers and downloads through CDP
and a Take Over key for the Browser tab, like the terminal's Ctrl-I. Their drafts are in this
spec's notes.

## Acceptance
On a fixture page, a stand-in agent hovers, picks an option, drags a card, uploads a project
file, answers a confirm, waits for late text, resizes to a phone and back, and switches to dark,
each one visible in the tab; an upload from outside the projects is refused.
