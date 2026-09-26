# TICKET-550 — Ask before a close or a quit ends a working agent, and hold the closed terminal

- **Ticket:** LOCAL #550 (feature, prong 2: the agents Marley hosts survive its own gestures)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/completed/550-ask-before-ending-a-working-agent.spec.md
- **Source ticket:** The Warp second pass of 2026-09-25, finding 2, and the Orca second pass's finding 1, ranked first there (`docs/planning/design-notes/warp-second-pass-2026-09-25.md`, `orca-second-pass-2026-09-25.md`), with Chad's answers of 2026-09-26: ask first, name the working agents, close idle ones without asking, and also hold a closed working terminal for a configurable number of seconds so an accidental close can be undone. Specced because Chad decided on 2026-09-26 that every remaining Orca and Warp finding gets built.
- **Status:** closed (2026-09-26)

## Summary
Nothing in Marley asks before it ends an agent. `ctrl-shift-w` in Claude Code's terminal, the
tab's close button, the rail's Close, the window's close and the quit that follows every
`just install` all end the agent mid-turn without a word; Zed asks about unsaved files only.
Warp warns before a quit or a close while a process runs and lets a closed tab come back for
60 seconds; Orca names the live agents in its prompt. Marley does both. A close or a quit that
would end a working agent (its `marley_fleet` seat working or waiting on a permission, or the
quiet timer's working when the plugin sends no events) asks first, and the prompt names each
agent with its project and state; an idle agent closes without asking. When a working terminal
is closed anyway, Marley keeps its `Terminal`, PTY and all, for `marley.undo_close_seconds`
(60 by default, 0 for off), and `ctrl-shift-t` or the toast's Undo puts it back in a new view
with the agent still running. A logout or a shutdown is never held up: the question is asked
only for a close made in Marley.

## Acceptance
Quit with a working Claude Code shows a dialog naming it and Cancel keeps Marley running;
`ctrl-shift-w` on that terminal asks the same and, once confirmed, leaves a toast whose Undo
(or `ctrl-shift-t`) brings the terminal back with its scrollback and its agent still working;
an idle agent's terminal closes at once; after the hold passes the agent's process is gone;
with the setting off nothing asks.
