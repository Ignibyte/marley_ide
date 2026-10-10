# TICKET-735 — The New Agent picker

- **Ticket:** LOCAL #735 (feature, the Marley layout: agents anywhere)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** Chad, 2026-10-10: "it should auto detect what project you are in or you need
  to Open Agent In Path or something like that. Or open agent in a path finder." Plan:
  [agents-anywhere shelf](../../design-notes/agents-anywhere-2026-10-10.md).
- **Status:** open

## Summary
`marley: new agent` asks three things: the agent, how (a thread in a tab, or the CLI in a terminal)
and where. Where starts on a guess (the active file's project, else the active terminal's folder,
else the shown project, else the home folder) and also lists the window's projects, recent projects
and Browse…, Zed's own path prompt. Every + menu and the Home page offer it. A folder in the project
panel and a terminal's rail row get Open Agent Here, which skips the where.

## Acceptance
The picker opens a Claude Code CLI in a folder chosen with Browse…, and a thread tab on the guessed
folder; Open Agent Here on a terminal row starts the agent in that terminal's folder.
