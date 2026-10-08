# TICKET-690 — Watch a harness session in a terminal

- **Ticket:** LOCAL #690 (feature, phase 2 item 5 of the manager plan)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** [690-watch-a-harness-session-in-a-terminal.spec.md](../../pipeline/completed/690-watch-a-harness-session-in-a-terminal.spec.md)
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 2, item 5
- **Status:** closed

## Summary
#689 lists the commands that watch a harness session (`session_surface_to_human`), each with
Copy. This ticket adds Open to each view: Marley starts the view's command in a new terminal of
the tab's workspace.
- `rh view WS` for an actor or a Claude seat, where the person claims control with Ctrl-b c to
  type.
- `rh attach WS` on tmux, with direct input.
- `rh codex history` or `rh claude history`, or an observer, for a Codex or Claude session.

Since harness TICKET-109, agent seats have these views too.

## Acceptance
With `marley.harness_writes` on, Open on a view starts its command in a new terminal, and the
terminal shows the session's screen.
