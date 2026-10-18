# TICKET-807 — Marley's, Rusty's and Home's own icons

- **Ticket:** LOCAL #807 (feature, the Marley layout)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** ../../pipeline/completed/807-marley-rusty-and-home-icons.spec.md
- **Source ticket:** Chad, 2026-10-17: "Also i have agent-icons folder. one for home, rusty, and
  marley. lets add those as well", then "ok lets build 3 and 4 then reinstall".
- **Status:** closed

## Summary
Chad drew three 16 × 16 monochrome icons (`agent-icons/` at the repository's top): a house for
Home, a cat for Marley and a butler for Rusty. Marley draws Home with Zed's list-tree icon, the
Marley agent with Zed's sparkle and Rusty with Zed's blocks icon. The three SVGs join Zed's icon
set (`assets/icons/`, with names in `IconName`) and take those places: Home's tab, rail row and
status-bar button; the title bar's Marley and Rusty buttons; and the Marley and Rusty agents'
threads, rows and New Agent entries.

## Acceptance
Home, the Marley agent and Rusty show Chad's icons wherever they showed Zed's, tinted by the theme.
