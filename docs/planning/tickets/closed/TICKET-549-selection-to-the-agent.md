# TICKET-549 — Send the editor's selection to a terminal agent

- **Ticket:** LOCAL #549 (feature, prong 2 with prong 1: the editor feeds the CLI agent in a terminal)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/completed/549-selection-to-the-agent.spec.md
- **Source ticket:** The Warp second pass of 2026-09-25, finding 1 and its first recommendation (`docs/planning/design-notes/warp-second-pass-2026-09-25.md`), with Chad's answer of 2026-09-26: with several agents running, a picker chooses the target. Specced because Chad decided on 2026-09-26 that every remaining Orca and Warp finding gets built.
- **Status:** closed

## Summary
Zed's `ctrl->` quotes the editor's selection into an Agent Panel thread and reaches nothing
else; Claude Code in a Marley terminal gets a file only through Attach File's chooser. Warp
sends a selection from its editor into the running CLI agent's prompt with one key, as the
path, the line numbers and the text, not submitted. Marley adds `marley: send selection to
agent`, bound to `ctrl->` in the editor: it types a reference into the agent's terminal as one
paste with no Enter, `@src/auth.rs#L12-40` for Claude Code (the form its JetBrains plugin
inserts) with the path relative to the agent's working directory, and `src/auth.rs:12-40` for
the other CLIs until each one's mention syntax is checked. One agent terminal in the window
takes it; several open a picker; none falls through to Zed's own action, so the key keeps its
meaning. While rich input is open on the target the reference lands there instead, and the
send refuses while the agent waits on a permission prompt, since a paste would answer it.

## Acceptance
With Claude Code in one terminal, `ctrl->` on a selection puts `@<relative path>#L<a>-<b>` at
the agent's prompt, unsent, and focuses that terminal; with two agents the picker lists both
and Enter sends to the chosen one; with none the selection goes to Zed's Agent Panel as
before; an agent started in a subfolder gets a path relative to that folder; a waiting agent
gets nothing and a toast says why.
