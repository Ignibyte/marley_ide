# TICKET-592 — Show the launch approval verbatim

- **Ticket:** LOCAL #592 (bug, W-series launch configs, after #527)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/592-launch-approval-verbatim.spec.md
- **Source ticket:** found in #527's visual check, after the fact (shot 527-02-approve)
- **Status:** closed

## Summary
Choosing a launch config asks with the config's text, one line per item (#527 REQ-002). On
Linux, Zed's prompt renders its detail as Markdown with smart punctuation, so the approval
runs the items together on one line, shows `--bind` as an en dash and curls the title's
quotes. Worse, the text comes from the repository, so a config could hide part of a command
from the question that approves it as markup (an HTML comment, say). The approval must show
exactly what runs.

## Acceptance
The approval shows each item on its own line, character for character as the config says it,
with nothing in the text read as Markdown; approvals given before the fix still hold.
