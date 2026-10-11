# TICKET-744 — One New Agent, and Reopen Thread…

- **Ticket:** LOCAL #744 (feature, the Marley layout: tabs everywhere)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [744-one-new-agent.spec.md](../../pipeline/completed/744-one-new-agent.spec.md)
- **Source ticket:** Chad, 2026-10-11: "i wonder if the experience becomes just 1 singular open a
  agent for the left pane and the sparkle. Meaning if its a project then default to the project
  folder chosen but allow to switch. Remove New Agent THread with just New Agent. add a New Agent
  from Thread or something that restarts", and "my thought is that you would re open it". Plan:
  [tabs-everywhere shelf](../../design-notes/tabs-everywhere-2026-10-11.md).
- **Status:** closed (Reopen Thread… and threads in tabs split to #772)

## Summary
The rail's + menu offers New Agent Thread (a submenu that starts a thread on the row's folder
without asking), New Agent… (#735's picker) and a list of agent CLIs. The sparkle at the bottom
right opens Zed's Agent Panel, which draws Open Project and starts nothing in a group with no
folder. New Agent… becomes the one entry: in the row's `+`, in every pane's `+`, behind the
sparkle and Zed's own new-thread keys in the Marley layout, and in the palette. In a project, its
Where starts on the project's folder, with the other places one row down. Reopen Thread… picks an
earlier conversation and opens it in a tab, or brings its tab forward. In the Marley layout every
Marley path that opens a thread (Reopen Thread…, the Threads page, the rail's rows, the Manager,
a notification) shows it in a tab, so the Agent Panel stays hidden; its settings page and Zed's
other ways into the right dock go with #747.

## Acceptance
A row's `+` lists New Agent… and Reopen Thread… and no New Agent Thread or CLI list; New Agent…
from a project row starts on that project's folder; the sparkle in Default opens the picker, not
Zed's panel; the pane's `+` offers both entries; Reopen Thread… brings back an earlier conversation
in a tab; a thread row in the rail opens a tab, not the panel.
