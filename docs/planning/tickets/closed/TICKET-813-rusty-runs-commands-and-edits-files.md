# TICKET-813 — Rusty's agent runs commands and edits files

- **Ticket:** LOCAL #813 (feature, Prong 2: Rusty in Marley)
- **Owner:** 2f48853a-c455-455f-9b94-114158170228
- **Pipeline doc:** ../../pipeline/completed/813-rusty-runs-commands-and-edits-files.spec.md
- **Source ticket:** Chad, 2026-10-18: he asked Rusty in the Agent Panel to check a system process,
  and it refused and told him to use the command line.
- **Status:** closed

## Summary
Rusty's entry in the Agent Panel (#696) was built like the Marley agent: told "You do not edit
files, run commands or change code here", with Claude Code's Bash, Edit, Write, NotebookEdit and
MultiEdit turned off, Codex held in read-only, and Zed's agent given no built-in tool. That suits
the Marley agent, which only explains and configures Marley, but not a personal assistant: Rusty
cannot check a process or run its own machine skills (`/devbox`, `/plex`, `/mac`). Chad,
2026-10-18: Rusty gets commands and file edits, they come with Rusty being on (no new setting),
and it asks before each one unless the permission settings (#779) say bypass. The Marley agent
stays read-only.

## Acceptance
While Rusty is on, Rusty's agent runs commands and edits files on Claude Code, Codex and Zed's
agent, asking first unless the thread's mode skips the asking, and still changes Rusty's own data
through Rusty's tools; the Marley agent keeps every limit it has today.
