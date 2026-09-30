# TICKET-598 — New Agent in Worktree sits under New Agent Thread

- **Ticket:** LOCAL #598 (feature, workbench shell: the rail's +)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/queued/598-worktree-agent-under-agent-thread.spec.md
- **Source ticket:** Chad, 2026-09-30, testing with the walkthrough: "On the + button on the left lets move New agent in worktree up below New Agent Thread".
- **Status:** open

## Summary
A project's + menu lists New Terminal, New Browser Tab and New Agent Thread, then the Agent CLIs
header with each installed CLI, and only then New Agent in Worktree (#510), where it reads as one
more CLI under that header. The two ways to start an agent with its own context belong together:
New Agent in Worktree moves up to sit directly below New Agent Thread, above the Agent CLIs
header. Nothing else in the menu changes.

## Acceptance
In a local git project with agent CLIs installed, the + menu reads New Terminal, New Browser Tab,
New Agent Thread, New Agent in Worktree, then the Agent CLIs header and its CLIs, then Launch.
