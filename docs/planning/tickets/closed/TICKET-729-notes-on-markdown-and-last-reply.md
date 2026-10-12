# TICKET-729 — Notes on any Markdown file or the agent's last reply

- **Ticket:** LOCAL #729 (feature; size medium to large, after #726 slice 1 and #732; the review batch after the finish)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [729-notes-on-markdown-and-last-reply.spec.md](../../pipeline/completed/729-notes-on-markdown-and-last-reply.spec.md)
- **Source ticket:** [plannotator-findings.md](../../intake/plannotator-findings.md), item 4; Chad, 2026-10-09: "take a look at https://github.com/backnotprop/plannotator and see what may be to add to the list"; Chad, 2026-10-11, on the [finish-line list](../../design-notes/finish-line-2026-10-11.md): "11.) yes"
- **Status:** closed

## Summary
#726's review tab opens on any Markdown file (`marley: review markdown`) and on an agent's last
reply (`marley: review last reply`, or Review Last Reply in the rail's menus): a Zed agent thread's
last turn, or Claude Code's last reply in a terminal, read from its transcript. A file's notes go
through #522's picker with their quoted passages and lines; a reply's go back to the agent that
wrote it, by #522's delivery to a terminal or the thread's own send. #732's template wraps both.
The notes are made in Marley's tab rather than inside Zed's Markdown preview.

## Acceptance
A note on a Markdown file and one on a thread's last reply reach the chosen agent.
