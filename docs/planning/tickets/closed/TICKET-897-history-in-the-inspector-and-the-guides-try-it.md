# TICKET-897 — History in the Changes inspector, and the guide's Try it

- **Ticket:** LOCAL #897 (feature, the Marley look, W5)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** [897-history-and-try-it.spec.md](../../pipeline/completed/897-history-and-try-it.spec.md)
- **Source ticket:** split from TICKET-891 while planning it (2026-10-22): both are new behaviour,
  where #891's other parts restyle what Zed draws.
- **Boards:** SearchMore
- **Size:** M
- **Status:** closed

## Summary

A History view beside Changes in the inspector: the branch's recent commits, each with its dot,
subject, short sha and when (Zed's `Repository::graph_data` and `fetch_commit_data`). And the
Marley Guide's "Try it" links, which run a command from the guide page in Marley's Browser
(`marley: open threads`): a binding only the guide's own file page may call, for a fixed list of
commands.

## Acceptance

Each works and looks as the board shows; shot in Marley Night and Marley Day.
