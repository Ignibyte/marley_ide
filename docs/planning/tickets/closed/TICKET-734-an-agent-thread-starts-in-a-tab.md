# TICKET-734 — An agent thread starts in a tab

- **Ticket:** LOCAL #734 (feature, the Marley layout: agents anywhere)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [734-an-agent-thread-starts-in-a-tab.spec.md](../../pipeline/completed/734-an-agent-thread-starts-in-a-tab.spec.md)
- **Source ticket:** Chad, 2026-10-10: "you can use cli or use the agent panel but it can be used
  anywhere with and panel on there and not isolated or forced to the folder. We need to specify
  where we want to open." Plan: [agents-anywhere shelf](../../design-notes/agents-anywhere-2026-10-10.md).
- **Status:** closed

## Summary
An Agent Panel style thread (Claude Code, Codex, Zed's agent, a custom entry) starts in a center tab
of whatever group is shown, projects and groups without a folder alike, working in a folder Marley
names for it. #697 can only move a running thread out of the panel; this starts one in a tab. Zed's
ACP file handlers refuse a path outside the project (`acp_thread.rs`, `read_text_file`), so the
thread's folder joins the hosting workspace's project as a hidden worktree. The thread's rail row sits
in the group that holds its tab.

## Acceptance
From a group with no folder (Home), New Agent Thread starts a thread in a tab there, in the home
folder; it answers, and it reads a file of that folder.
