# TICKET-890 — Port rows, the Containers panel and Collab as the board draws them

- **Ticket:** LOCAL #890 (feature, the Marley look, W5)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** [890-port-rows-containers-collab.spec.md](../../pipeline/completed/890-port-rows-containers-collab.spec.md)
- **Source ticket:** split from TICKET-883 while planning it (2026-10-22): #883 kept the Ctrl Tab
  switcher.
- **Boards:** Splits
- **Size:** S
- **Status:** closed

## Summary

The port rows under a project (the port in the tile, the URL as a link over what serves it, Open
or Copy), the Containers panel's rows with Stop, and Collab's sign-in card ("Work with others in
Zed", Sign In with GitHub), as the Splits board draws them.

## Acceptance

Each part looks as the board shows and keeps its behavior; shot in Marley Night and Marley Day.

## Notes

- The rail's round 28 px tile cannot hold a five-digit port; the port row needs a tile of its own
  width, or keeps its icon.

## Chad's answer (2026-10-23)

- The Containers panel groups its rows by container, as the Splits board draws it, even though
  that changes what a click on a row does.
