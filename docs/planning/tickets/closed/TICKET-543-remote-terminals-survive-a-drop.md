# TICKET-543 — Remote terminals that survive a dropped link

- **Ticket:** LOCAL #543 (feature, prong 2 remote)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/543-remote-terminals-survive-a-drop.spec.md
- **Source ticket:** Chad, 2026-09-25, on the Orca survey: "we will be taking what it does well and bring it in here" (docs/orca_architecture/README.md, which lists "remote terminals that survive a dropped link through tmux" among the smaller things worth a day). Report 04 §3.2 item 5 and §3.4 rows 5 and 6; the embedded rustal-harness is the long-term answer.
- **Status:** closed
- **Backlog:** Queue. Nothing blocks it, and it does not wait for the harness: the tmux wrapper is the stopgap until the embedded rustal-harness's remote entry replaces it.

## Summary
A remote shell in Marley ends when its link drops, and a Claude Code agent running in it ends with
it. `marley_remote` builds Marley's ssh argv, but nothing has called it since the fork: the crate
was ported on 2026-09-18 "not yet wired into the app", and its connect action, badge and saved hosts
were gpui-era features that did not come across. This ticket wires it. `marley: open remote
terminal` lists the SSH hosts saved in Zed's settings and opens a terminal whose shell runs inside
`tmux -L marley new-session -A -s <name>` on the host. A dropped link leaves the shell and its
agent running, and Rerun on the terminal reattaches. Claude Code's notifications from inside that
tmux reach Marley: Claude Code wraps a hook's sequence for tmux by itself, so the plugin only has
to accept Marley's own variable where `TERM_PROGRAM` reads `tmux`, and Marley's tmux server allows
the passthrough.

## Acceptance
A remote terminal opened on a saved host shows a shell in a tmux session named for it; after the
ssh process dies, Rerun shows the same shell with the output it made meanwhile; a Claude Code
notification raised inside that session shows in Marley.
