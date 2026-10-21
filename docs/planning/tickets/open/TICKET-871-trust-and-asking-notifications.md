# TICKET-871 — The trust modal and the agents' questions as the boards draw them

- **Ticket:** LOCAL #871 (feature, the Marley look, W3.4 follow-up)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** split from TICKET-870 while planning it (2026-10-21): the prompts are
  `window.prompt`s, drawn by `ui_prompt`; these are a Zed `ModalView` and Marley's app
  notifications, each its own frame.
- **Boards:** Dialogs (trust, the apply question), MoreForms (pairing, one agent starting another)
- **Size:** M
- **Status:** open

## Summary

Zed's trust modal (`SecurityModal`, "Unrecognized Project") as the Dialogs board's trust card
(a title naming the folder, a line on what trust allows, Not now and Trust); the questions agents
ask through app notifications (settings and keymap changes, harness seats, `agent_start`,
pairing) as the boards' cards: a title, a body, the change or code, and buttons quiet / plain /
primary.

## Acceptance

Each surface looks as its board shows and keeps its behavior; shot in Marley Night and Marley Day.
