# TICKET-730 — Agent findings as diff notes

- **Ticket:** LOCAL #730 (feature; size medium; the review batch after the finish)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [730-agent-findings-as-diff-notes.spec.md](../../pipeline/queued/730-agent-findings-as-diff-notes.spec.md)
- **Source ticket:** [plannotator-findings.md](../../intake/plannotator-findings.md), item 5; Chad, 2026-10-09: "take a look at https://github.com/backnotprop/plannotator and see what may be to add to the list"; Chad, 2026-10-11, on the [finish-line list](../../design-notes/finish-line-2026-10-11.md): "11.) yes"
- **Status:** open (deliberate: the review batch, after the finish)

## Summary
A new MCP tool family, `review`, and its first tool, `review_notes_add`: a reviewing agent or any
MCP client (a linter's wrapper) puts a batch of findings into the project diff as review notes,
each tagged with its source. A finding is a proposal with Keep and Drop until the user keeps it;
only kept notes count and go to the author agent through Send Review to Agent (#522), with a
`Source:` line. The tool asks the user as the editor tools do. Starting the reviewing agent with a
skill as its prompt belongs to #717.

## Acceptance
An agent's findings land in the diff with their source; a kept one reaches the author agent with
its source, and a dropped one is gone.
