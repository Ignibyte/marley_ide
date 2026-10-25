# TICKET-912 — Welcome to Marley on first run

- **Ticket:** LOCAL #912 (feature, the Marley look, first run)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [912-welcome-to-marley.spec.md](../../pipeline/completed/912-welcome-to-marley.spec.md)
- **Source ticket:** the design review of 2026-10-23
  (docs/marley/design-review-2026-10-23/G-system.md, board Dialogs: "First-run screen";
  D-panels-editor.md, board Theme: "No mark on Home or a first-run screen"). #870 and #872 left
  the first-run screen out for a ticket of its own.
- **Boards:** Dialogs, Theme
- **Size:** L
- **Status:** closed

## Summary

On its first start Marley still opens Zed's onboarding page (`crates/zed/src/main.rs`, the
`FIRST_OPEN` branch). The Dialogs board draws Marley's own: the cat mark, "Welcome to Marley",
four rows of choices (Layout, Theme, Agents, Rusty), and Open a Folder and Go to Home. This ticket
puts that screen in place of Zed's onboarding. Each choice writes its setting at once, so the
screen is a short tour of the settings that matter most, and every choice can change later in
Settings.

## Acceptance

A first start shows Welcome to Marley with the cat mark and the four choices; each choice writes
its setting; Open a Folder and Go to Home leave the screen; a later start does not show it; shot
in Marley Night and Marley Day.
