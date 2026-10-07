# TICKET-682 — Settings and keymap changes the user accepts as a diff

- **Ticket:** LOCAL #682 (feature, prong 2 C: Marley's MCP server)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** (set at promotion)
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 1, item 4
- **Status:** open

## Summary
`settings_change` and `keymap_change` let an agent propose a change to the user's or the project's
settings or keymap. Marley shows the change as a diff and applies it only when the user accepts,
through Zed's own settings writer so comments and formatting survive; a value the schema refuses
is refused before anything is shown. The tool answers whether the user accepted, declined or did
not answer. It works for any agent that reaches Marley's MCP server, which is what lets the
Marley agent change settings without file-editing tools.

## Acceptance
An agent proposes `"terminal": {"font_size": 15}`; Marley shows the diff; Accept writes it with the
file's comments kept, Decline leaves the file as it was, and the agent is told which. A value of
the wrong type is refused with the schema's reason and no diff.
