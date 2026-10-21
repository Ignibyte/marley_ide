# TICKET-869 — Toasts and prompts as the boards draw them

- **Ticket:** LOCAL #869 (feature, the Marley look, W3.4 follow-up)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** split from TICKET-840 while planning it (2026-10-21): the toasts come through
  Zed's notification frame (`MessageNotification`, 97 `show_toast` calls in Marley), and the
  prompts are a dozen surfaces; each is a change of its own.
- **Boards:** Inbox (toasts), Dialogs, MoreForms
- **Size:** M
- **Status:** open

## Summary

Toasts at the bottom right as the Inbox board draws them (a dot, a title over a line, their
actions), through a hook in Zed's notification frame; and the trust, close, quit, second-launch,
Rusty-not-connected and apply prompts, the pairing prompt and the prompt when one agent starts
another, as the Dialogs and MoreForms boards draw them.

## Acceptance

Each toast and prompt looks as its board shows and keeps its behavior; shot in Marley Night and
Marley Day.
