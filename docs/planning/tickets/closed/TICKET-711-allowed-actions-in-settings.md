# TICKET-711 — Allowed actions in Settings

- **Ticket:** LOCAL #711 (feature; #707's follow-up)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [711-allowed-actions-in-settings.spec.md](../../pipeline/completed/711-allowed-actions-in-settings.spec.md)
- **Source ticket:** Chad, 2026-10-09: "lets do the small things", which includes "a Settings
  screen for `actions_allowed`, which you can only edit in settings.json today".
- **Status:** closed

## Summary
Settings → Marley → Agent Control gets **Allowed Actions**: the names in
`marley.agent_control.actions_allowed`, one per row with a remove button, a field that adds a
name, and a "not an action" mark on a name Zed has no action for.

## Acceptance
- An added name lands in settings.json, and a removed one leaves it.
- A name no action is registered under is marked.
