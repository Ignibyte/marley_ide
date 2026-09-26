# TICKET-575 — A terminal keeps its MARLEY_TERMINAL_ID across a restore

- **Ticket:** LOCAL #575 (feature, prong 2, after #520's slice 1)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** (none yet; #520's design, in its notes, is its start)
- **Source ticket:** cut from TICKET-520 at its promotion, 2026-09-26, to keep the slice one piece
- **Status:** open

## Summary
#520's slice 1 mints a `MARLEY_TERMINAL_ID` per terminal per launch. Here it survives Marley's restore of the terminal at the next launch: a Marley table keeps each terminal item's id (`MarleyTerminalIdsDb`, its own database domain like #494's `MarleyBrowserTabsDb`), and `TerminalView`'s serialize, deserialize and cleanup reach it through a hook global (`MarleyTerminalIdentity`, as `MarleyTerminalFooter` is set); `Project::create_terminal_shell_restoring(cwd, id)` hands the saved id to the builder under a private key, which a split never inherits. The design is #520's notes, items 4 and 5, D3 and D4, and its queued REQ-004.

## Acceptance
After a quit and a launch, a restored terminal's `MARLEY_TERMINAL_ID` is the one it had before, and a split of it gets a new one.
