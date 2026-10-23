# TICKET-939 — Rusty's Tasks and brain pages as the board draws them

- **Ticket:** LOCAL #939 (feature, the Marley look, Rusty)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [939-rusty-tasks-knowledge-brain-as-the-board.spec.md](../../pipeline/queued/939-rusty-tasks-knowledge-brain-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/E-rusty.md, boards Tasks, Knowledge, BrainPage)
- **Boards:** Tasks, BrainPage (Knowledge is a follow-up ticket)
- **Size:** M
- **Status:** open

## Summary

Rusty's Tasks tab and brain page differ from their boards in ways met every day. On Tasks, the
Decisions due card's buttons are a plain Decisions (which opens another tab) and Open the page;
the board has a filled Follow Up that opens the follow-up form right there. The tasks are flat
rows where the board draws cards, and the header and lists column are smaller and plainer. On a
brain page, a task checkbox in the body does nothing when clicked, the outline column has no
"Linked here", properties are plain text, favourites have a kind icon where the board puts a ★,
and the outline toggle is an icon where the board has an "Outline" text button. This ticket
builds those. The Knowledge panel's items (asking the brain, the page-type tile, graph settings
in a fixed column) were split into a follow-up ticket to keep this one shippable.

## Acceptance

On Tasks, Follow Up opens the follow-up form and the rows are cards under the board's header; on
a brain page, a checkbox in the body toggles and is saved, "Linked here" lists the backlinks,
properties are coloured pills, favourites start with ★ and Outline is a text button; shot in
Marley Night and Marley Day with Rusty's stand-in.
