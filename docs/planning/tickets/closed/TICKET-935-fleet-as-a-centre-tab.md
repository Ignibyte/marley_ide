# TICKET-935 — The Fleet tab with the snapshot beside the list

- **Ticket:** LOCAL #935 (feature, the Marley look, the Fleet)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [935-fleet-as-a-centre-tab.spec.md](../../pipeline/completed/935-fleet-as-a-centre-tab.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/F-tools.md, board Fleet)
- **Boards:** Fleet
- **Size:** S
- **Status:** closed

## Summary

The Fleet board draws the Fleet as a centre tab: "Fleet" as the page's title with its summary
beside it, the hosts and their agents on the left, and the selected agent's snapshot as a column
on the right. The review read Fleet as a dock panel, but in the Marley layout it already opens as
a centre tab (#748's panel tab). What differs is inside it: the snapshot sits under the list in a
draggable split, which suits the narrow dock and wastes a wide tab, and the header is a small
"FLEET" label. This ticket puts the snapshot beside the list whenever the Fleet is wide, keeps the
split under it when narrow (the dock in Zed's layout, a half-width pane), and gives it the board's
header.

## Acceptance

A wide Fleet tab shows the list on the left and the selected agent's snapshot on the right under
the "Fleet" title; a narrow one keeps the snapshot under the list; shot in Marley Night and Marley
Day.
