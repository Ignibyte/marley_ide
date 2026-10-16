# TICKET-790 — The agent editor when the data folder has a space

- **Ticket:** LOCAL #790 (bug, macOS)
- **Owner:** c70cc71d-da30-4123-80da-1db398eb7911
- **Pipeline doc:** (set at Plan)
- **Source ticket:** seen in #787's check on the Mac, 2026-10-16.
- **Status:** open

## Summary
The agent editor (#649: Ctrl+G in an agent Marley started opens the agent's draft in a tab) puts
its helper in Marley's data folder and refuses a path with whitespace, since the agents split
`$EDITOR` on spaces (`agent_editor::set_helper`). On macOS the data folder is always
`~/Library/Application Support/Marley`, so the log says "Marley's data directory has a space in
its path" at every start and the agent editor is never on there. The helper goes to a folder
without whitespace when the data folder has some (Marley's state folder on macOS,
`~/.local/state/Marley`, or a link to it), so Ctrl+G works on a Mac.

## Acceptance
On the Mac, Ctrl+G in Claude Code started by Marley opens the draft in a tab, and the warning is
gone from the log.
