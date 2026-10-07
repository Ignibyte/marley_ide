# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-683](open/TICKET-683-the-marley-agent-in-the-agent-panel.md) | feature | Marley agent, phase 1 item 5: the "Marley" entry in the Agent Panel, found from Claude Code, Codex or Zed's agent, offered once, no file tools |
| [TICKET-684](open/TICKET-684-the-marley-agent-in-a-terminal.md) | feature | Marley agent, phase 1 item 6: the same agent as Claude Code in a terminal tab |
| [TICKET-686](open/TICKET-686-keymap-changes-accepted-as-a-diff.md) | feature | Marley agent, phase 1 item 4's second half: `keymap_change`, applied only when the user accepts, through Zed's keymap updater |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then). Chad, 2026-10-01: wait, so bugs found before the first release do not mean cutting several |
