# TICKET-906 — Small daily fixes from the design review

- **Ticket:** LOCAL #906 (feature, the Marley look, the Threads page, side panels, theme and rail)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [906-small-daily-fixes.spec.md](../../pipeline/queued/906-small-daily-fixes.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/C-threads.md,
  D-panels-editor.md, B-rail-chrome.md; boards Threads, SidePanels, Theme, Rail, RailStates)
- **Boards:** Threads, SidePanels, Theme, Rail, RailStates
- **Size:** S
- **Status:** open

## Summary

Four small things are wrong every day. In a half-width pane the Threads page keeps its side
columns at 200 and 320 px, so the list shrinks to about 100 px and every title truncates; the
board lets the columns flex and wrap. The Git and Outline tabs say "Git Panel" and "Outline
Panel", while their status-bar buttons say "Changes" and "Outline". Primary buttons fill with the
light Marmalade orange, not the cat's coat. An open project header in the rail shows no summary,
so what its agents do is only visible once it is folded.

## Acceptance

The Threads page keeps its list readable in a half-width pane; the panel tabs read "Changes" and
"Outline"; primary buttons fill with the coat under dark ink in Marley Night; an open project
header shows its agents' summary. Shot in Marley Night and Marley Day.
