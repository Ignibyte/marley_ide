# TICKET-923 — The Inspector and Files as the SidePanels board draws them

- **Ticket:** LOCAL #923 (feature, the Marley look, the side panels)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [923-side-panels-as-the-board.spec.md](../../pipeline/queued/923-side-panels-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/D-panels-editor.md, boards SidePanels), build order line 10
- **Boards:** SidePanels (the tab bar's Inspector button, the Inspector's switcher, the Files column)
- **Size:** M
- **Status:** open

## Summary

The Inspector and Files as the SidePanels board draws them. The Inspector's switcher becomes a
segmented control: equal tabs in a rounded track. Its Changes tab carries the number of changed
files ("Changes 3"). An Inspector button at the right of the tab bar shows it and lights while it
is on. Files, the left pop-out tree (#868), gets a "Find a file" field at its top that narrows the
tree as you type. A changed file shows a mono "M" at its row's right. The column opens 320 px
wide, not about 570. The Changes panel's own layout, Outline's header and the breadcrumb are the
board's other half and go to a follow-up ticket. The Inspector stays off by default (#837).

## Acceptance

The tab bar's Inspector button opens the Inspector, whose switcher is a segmented track with a
count on Changes. Files opens 320 px wide with a filter field and "M" on changed files, and typing
in the field narrows the tree. Shot in Marley Night and Marley Day.
