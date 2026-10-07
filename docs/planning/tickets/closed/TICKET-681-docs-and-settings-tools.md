# TICKET-681 — Docs and settings tools on Marley's MCP server

- **Ticket:** LOCAL #681 (feature, prong 2 C: Marley's MCP server)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** ../../pipeline/completed/681-docs-and-settings-tools.spec.md
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 1, item 3
- **Status:** closed

## Summary
The Marley agent explains and configures Marley, so it needs to read what Marley is and how it is
set: Zed's docs (`docs/src`, `all-actions.md` among them) and Marley's guide, shipped in the build
so they match the running version; every setting's type, default and description, from the
schema Zed already generates; the user's and the project's settings, with which value overrides
which; and every action with the key bound to it. Read tools only: `docs_search`, `docs_read`,
`settings_schema`, `settings_read`, `actions_list`, sized as TICKET-680 sizes results.

## Acceptance
An agent asks how to change the terminal's font and gets the docs page and the setting's schema
entry; asks what a setting is now and gets the value and where it came from; asks which key opens
the rail's filter and gets the action and its binding. No tool writes.
