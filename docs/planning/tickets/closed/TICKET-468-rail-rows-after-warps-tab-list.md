# TICKET-468 — The rail's rows after Warp's tab list

- **Ticket:** LOCAL #468 (feature, workbench shell)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/468-rail-rows-after-warps-tab-list.spec.md
- **Source ticket:** none (Chad's request, 2026-09-23)
- **Status:** closed

## Summary
Chad asked for the rail to look like Warp's vertical tab list: "It basically puts more padding
around the project + terminal to make is a bit better. makes the icons bigger etc." Today a
terminal row is Zed's dense `ListItem` with a 14px icon, and a thread row is Zed's one-line
`ThreadItem`. The rows become padded two-line cards with a large round icon, the selected row a
bordered card, and a line runs between projects. The layout observed is in
`docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`.

## Acceptance
Every terminal and thread row is drawn at one height with a round icon, the selected row as a
bordered card that moves nothing when the selection moves, and a divider between projects;
every click, menu, key and hover the rail has keeps working. The EARS criteria are in the spec.
