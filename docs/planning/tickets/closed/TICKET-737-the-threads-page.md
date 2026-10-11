# TICKET-737 — The Threads page

- **Ticket:** LOCAL #737 (feature, the Marley layout: agents anywhere)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [737-the-threads-page.spec.md](../../pipeline/completed/737-the-threads-page.spec.md)
- **Source ticket:** Chad, 2026-10-10: "lets put the threads as a button … that will provide a home
  page for threads", and "i dont see a way to start a new conversation or how do you view past
  conversations". Plan: [agents-anywhere shelf](../../design-notes/agents-anywhere-2026-10-10.md).
- **Status:** closed

## Summary
A center tab listing every agent conversation Zed keeps, from every project, group and folder,
newest first: title, agent, folder, when. A search field filters by title and folder, and chips
filter by agent. Marley's and Rusty's conversations head the page. A click opens a conversation in
a tab where you are, or brings forward the tab or panel already showing it. Archived ones are listed
under their own heading. Zed's thread history lives in its own sidebar, which the Marley layout
replaces, so today the rail's project rows are the only way back to an old thread.

## Acceptance
`marley: open threads` lists threads from two projects and a group; typing filters them; a click
opens one in a tab.
