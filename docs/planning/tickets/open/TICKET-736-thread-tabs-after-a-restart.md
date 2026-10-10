# TICKET-736 — Thread tabs after a restart

- **Ticket:** LOCAL #736 (feature, the Marley layout: agents anywhere)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** the agents-anywhere plan (Chad, 2026-10-10); #697's tab is gone after a restart.
  Plan: [agents-anywhere shelf](../../design-notes/agents-anywhere-2026-10-10.md).
- **Status:** open

## Summary
A thread tab comes back after a restart in the group and pane it was in, loading its thread from
Zed's thread store with its agent and folder, as the Agent Panel loads a thread from its history.
A thread that can no longer load leaves no empty tab.

## Acceptance
A thread tab in Home, quit and started again, is a tab in Home again with its earlier turns.
