# TICKET-662 — Favourites and bookmarks from Rusty

- **Ticket:** LOCAL #662 (feature, Rusty in Marley R4b)
- **Owner:** claude-opus-5-5, 2026-10-06 (Chad: "we need to look into then adding the last things missing", after the R1 to R7b batch)
- **Pipeline doc:** (none yet)
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D3, the slices table); the read-only survey of
  Rusty at `13249a8` and Marley on 2026-10-06
- **Status:** open

## Summary
Rusty keeps bookmarks in the vault (`.rusty/bookmarks.json`; `bookmark_list`, `bookmark_add`, `bookmark_remove`, `bookmark_set`; kinds `file`, `folder`, `search`, `heading`; file and folder bookmarks are the favourites; Rusty's TICKET-037). Marley shows none of them. This ticket adds a Favourites group above the Brain view's tree (pages and folders, a click opens), a star in a Page tab's header that adds or removes the page, `rusty: toggle bookmark` for the page in front, favourites first in the page picker (#654) with a star, and a right-click to rename or remove one; search and heading bookmarks open their search or scroll to their heading.

## Acceptance
A page starred from its tab appears under Favourites in the Brain view and first in the page picker, and is gone from both when unstarred; the list is Rusty's, so a bookmark made in Rusty's app shows in Marley.
