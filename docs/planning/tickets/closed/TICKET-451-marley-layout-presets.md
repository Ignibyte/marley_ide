# TICKET-451 — Zed's layout presets in the Marley layout

- **Ticket:** LOCAL #451 (bug, workbench shell W6b)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/451-marley-layout-presets.spec.md
- **Source ticket:** ../../pipeline/completed/442-rail-persistence.spec.md (Out) · ../../../marley/workbench-shell.md
- **Status:** closed

## Summary
Zed's Panel Layout presets misread the Marley layout. With `agent.dock` patched to the right, the
title bar's menu shows "Custom"; Classic then writes the other panels to the left but not
`agent.dock`, and Agentic writes `agent.dock: left`, which overrides the Marley layout for good
(`agent_settings.rs:92-111`, `:338-396`; `title_bar.rs:1289-1292`). In the Marley layout the
crate now catches `workspace::UseClassicLayout` and `workspace::UseAgenticLayout` in the capture
phase, as #441 catches the terminal actions, and shows a message with a way back to Zed's
layout instead of writing settings. Split at promotion (2026-09-23): the docks across a layout
round trip are TICKET-456, and a first terminal is TICKET-455.

## Acceptance
In the Marley layout, Classic and Agentic change no setting and say why, offering Zed's layout;
in the Zed layout they do what Zed's do. Full EARS in the completed spec. Shipped 2026-09-23.
