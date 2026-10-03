# TICKET-659: Rusty's Decisions tab, with System One's log renamed System One calls

- **Ticket:** LOCAL #659 (feature, Rusty in Marley R7: the Decisions tab; open decision 4)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/queued/659-rusty-decisions-tab.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D3's DecisionsPage row, R-D9's fixed row,
  open decision 4, the slices table's R7); Chad, 2026-10-02, recorded there: Marley's System One
  log becomes "System One calls" and Rusty's tab keeps "Decisions"
- **Status:** open

## Summary
Marley's System One log, the tab #565 named Decisions, is renamed System One calls: the tab and
its heading, its source file and type, its action (`marley::OpenSystemOneCalls`, with the old
`marley::OpenDecisions` kept as Zed's deprecated alias so a binding to it still works), the
settings page's link, every string and doc that names it, and the four System One scenarios that
open it from the palette. The name Decisions then goes to Rusty: a center tab over `brain_due`
that lists the follow-ups due first, the overdue ones in the warning colour, then every decision
page with its status (decided, kept, revised, superseded) and its decided and follow-up dates, as
Rusty's `DecisionsPage.qml` does. A click opens the decision's page through #645's opener, and
the rail Brain view's fixed row (#644) gains a Decisions entry. The tab reads again when Rusty
announces a change. Recording a follow-up from the tab is the next slice. Behind
`marley.rusty.enabled` (#643), off by default; after #645.

## Acceptance
`marley: open system one calls`, a keymap's `marley::OpenDecisions` and the settings page's Open
System One Calls all open the System One calls tab, and the palette lists no `marley: open
decisions`. With Rusty on, the fixed row's Decisions entry opens the Decisions tab: the due
follow-ups first, overdue marked, then every decision with its status and dates; a click opens a
decision's page, a change Rusty announces shows without input, a failed read offers Read again,
and with Rusty off the tab says so and calls nothing.
