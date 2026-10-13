# TICKET-780 — Marley and Rusty start in their own folders, set in settings

- **Ticket:** LOCAL #780 (feature)
- **Owner:** 9623da6e-a5cd-4dcb-b048-40009324e9f0
- **Pipeline doc:** none yet
- **Source ticket:** Chad, 2026-10-12: "instead of marley choosing a folder we should default in
  the location that makes sense", then "this should be a settings as well and we dont ask for the
  folder for marley and rusty. by default choose the ones you recommended and if the user wants to
  somewhere else they can change it. The rusty agent probably should open in the place in which
  rusty has chosen."
- **Status:** open

## Summary
New Agent asks Where for the Marley and Rusty entries as for any agent, and Talk to Marley/Rusty
start them in hidden folders under Marley's data folder. Two settings name their folders instead:
Marley's defaults to Marley's settings folder (`~/.config/marley`), Rusty's to the vault folder
Rusty stores (`brain_vault_path`, else `~/.rusty/brain`). Every way to start either uses its
setting, and New Agent never asks Where for them. The agent they run on stays
`marley.assistant.agent`.

## Acceptance
New Agent → Marley starts a conversation in `~/.config/marley` with no Where step; Rusty's starts
in Rusty's vault folder; changing either setting moves the next conversation there.
