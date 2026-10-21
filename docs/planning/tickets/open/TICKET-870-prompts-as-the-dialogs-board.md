# TICKET-870 — Prompts as the Dialogs board draws them

- **Ticket:** LOCAL #870 (feature, the Marley look, W3.4 follow-up)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** split from TICKET-869 while planning it (2026-10-21): the toasts go through
  Zed's notification frame, the prompts through Zed's prompt renderer (`ui_prompt`, which Linux
  uses in place of the system's), so each is a change of its own.
- **Boards:** Dialogs, MoreForms
- **Size:** M
- **Status:** open

## Summary

The trust, close, quit, second-launch, Rusty-not-connected and apply prompts as the Dialogs board
draws them (a when line, a title, a body, buttons quiet / plain / primary / danger), likely by a
Marley prompt renderer set with `App::set_prompt_builder`; the pairing prompt and the prompt when
one agent starts another (MoreForms) in the same frame.

## Acceptance

Each prompt looks as its board shows and keeps its behavior; shot in Marley Night and Marley Day.
