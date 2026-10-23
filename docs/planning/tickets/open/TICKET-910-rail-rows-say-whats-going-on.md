# TICKET-910 — Rail rows say what is going on

- **Ticket:** LOCAL #910 (feature, the Marley look, the rail)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [910-rail-rows-say-whats-going-on.spec.md](../../pipeline/queued/910-rail-rows-say-whats-going-on.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/B-rail-chrome.md, boards Rail and RailStates)
- **Boards:** Rail, RailStates
- **Size:** M
- **Status:** open

## Summary

A rail row's second line should say what is going on, in the color of the row's state. Today a
thread row always reads "<agent> · <status word>", and the word can disagree with the color: a
finished, unseen thread reads "Marley · idle" in the done color. The Rail board has the line say
what the agent does or asks ("Editing thread_view.rs", "Allow kill-session?", "3 files
changed", "Opus 5.5 · idle"). A shell row's failed command should offer Ask an agent, and an
idle shell in its project's root should show its folder instead of a blank line. A group
header's summary becomes pills worded "needs you". A header with nothing going on says how many
rows it lists while open and "quiet" while folded, and the Default group gets its home tile.

## Acceptance

Each row and header reads as the Rail board draws it, its words and color from the same state,
and the shell row's Ask an agent asks an agent; shot in Marley Night and Marley Day.
