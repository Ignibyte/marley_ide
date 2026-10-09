# TICKET-697 — An agent thread in a center tab

- **Ticket:** LOCAL #697 (feature)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** [697-an-agent-thread-in-a-center-tab.spec.md](../../pipeline/completed/697-an-agent-thread-in-a-center-tab.spec.md)
- **Source ticket:** Chad, 2026-10-08: "is there any way that its possible we are able to move the
  agent panel into the main content if we wanted? … lets do #3"
- **Status:** closed

## Summary
`marley: open thread in center` moves the Agent Panel's active thread into a tab of the center
pane, beside terminals and files.
- The tab hosts the same `ConversationView` the panel showed. The thread keeps running and keeps
  its history.
- The panel turns to a new draft (`clear_base_view`, which keeps the thread retained), so the view
  is never drawn twice.
- `marley: move thread to panel`, run from the tab, gives it back (`activate_retained_thread`)
  and closes the tab.
- Zed's own tests already host a `ConversationView` as a center item (`ThreadViewItem` in
  `agent_ui/src/conversation_view.rs`). Marley's item lives in a Marley crate, so Zed's code is
  not changed.

## Acceptance
With a thread open in the Agent Panel, the command shows it as a center tab titled as the thread,
where it can be typed into and keeps streaming, and the panel shows a new draft. Moving it back
shows it in the panel again and closes the tab.
