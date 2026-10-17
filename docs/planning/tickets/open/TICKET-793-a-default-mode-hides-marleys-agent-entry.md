# TICKET-793 — A default mode hides Marley's own agent entry

- **Ticket:** LOCAL #793 (bug, the Marley and Rusty agents)
- **Owner:** c70cc71d-da30-4123-80da-1db398eb7911
- **Pipeline doc:** (set at Plan)
- **Source ticket:** found in #791's Test phase, 2026-10-16: #780's own scenario fails on this
  build ("Rusty started in its vault: FAIL"), and the New Agent picker finds no Rusty.
- **Status:** open

## Summary
Marley puts its Marley and Rusty entries in the settings' defaults (`assistant.rs`, `put_in`,
`agent_servers.<name>` as a custom agent with the adapter's command). When the user makes a mode
a thread's default, Zed's `set_default_config_option` (`agent_servers/src/custom.rs:158`) writes
`agent_servers.<name>` into the user's settings file, made with `default_settings_for_agent`
when the user's file has none: `{"type": "registry", "default_config_options": {...}}`. The
user's entry stands in place of the default one, so the entry becomes a registry agent named
"Rusty", which the registry does not have, and Rusty leaves the New Agent picker. The box's
settings hold exactly that (`"mode": "bypassPermissions"`), so the installed Marley has no Rusty
entry. Marley's own entry would go the same way.

## Acceptance
With a mode made the default for the Rusty or the Marley entry, the entry stays in the New Agent
picker, starts its agent with the adapter's command, and the chosen mode is its default.
