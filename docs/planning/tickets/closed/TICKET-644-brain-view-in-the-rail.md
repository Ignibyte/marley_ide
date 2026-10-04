# TICKET-644 — The rail's Brain view: Rusty's vault behind a switch in the header

- **Ticket:** LOCAL #644 (feature, Rusty in Marley R4: R-D9, R-D10)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/completed/644-brain-view-in-the-rail.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md`, slice R4, R-D9 and R-D10; Chad,
  2026-10-02: "Switch in the rail header"
- **Status:** closed

## Summary
Rusty's knowledge workspace moves into Marley (the plan's D11, amended), and Chad chose how its
screens are reached: a switch in the rail's header. While `marley.rusty.enabled` is on and Rusty
is connected (#643), the header's PROJECTS label becomes two icon buttons, Projects and Brain,
and one key, `secondary-alt-v`, flips between them. Brain swaps the rail's content for Rusty's
vault: a fixed row with Today, a brain search field that asks `brain_search` on Enter, and the
vault tree over `brain_tree`, drawn with Zed's `ui` list parts. A click opens a page (a preview
on one click, kept on two) through one function, which opens the page's file in Zed's editor
until #645 points it at the Page tab; the right-click menu and a drag make, rename, move and
delete pages and folders through Rusty's tools, never the disk; `list_changed` keeps the tree
current. Favourites wait for Rusty's TICKET-037, since Rusty serves none today. The ticket comes
after #643 and before #645, in Chad's order.

## Acceptance
With Rusty off, the header reads PROJECTS and the key answers with a toast. With Rusty on and
connected, the header shows Projects and Brain; Brain shows Today, the search field and the
vault tree; a page opens in a preview tab or a kept one; search lists Rusty's hits on Enter
only; the list keys walk the tree; new page, new folder, rename, move and delete go through
Rusty's tools, a refusal shows Rusty's message, and a delete asks first; a change Rusty
announces appears with no click; Projects carries a dot while something there needs the user;
the key flips back; and a lost connection returns the rail to Projects.
