# TICKET-782 — One Home page, with Rusty's cards while Rusty is on

- **Ticket:** LOCAL #782 (feature, the Marley layout)
- **Owner:** 8a8ae75e-71d7-48ad-bad4-b219ce9581d1
- **Pipeline doc:** [782-one-home-page.spec.md](../../pipeline/completed/782-one-home-page.spec.md)
- **Source ticket:** Chad, 2026-10-13: "whats your thoughts of combining the rusty home page with
  the marley home page and then just show rusty items if it is configured", then "lets do that and
  also in default the Home that always stays should always be at the top."
- **Status:** closed

## Summary
Marley has two home pages: Home (#701), always open in the Default group, and Rusty's home page
(#679), a separate tab behind the status bar's Rusty button. Rusty's cards move onto Home and show
only while `marley.rusty.enabled` is on; with Rusty on but not connected, one Rusty card says so.
Rusty's separate page and its status-bar button go; `rusty: open home` and the `rusty` verb open
Home. In the rail, Home's row comes first under its group, above the group's terminals.

## Acceptance
Home shows Rusty's screens, captures, recent pages, follow-ups due and tasks while Rusty is on and
connected, one not-connected card while it is on and not connected, and nothing of Rusty while it
is off. The status bar holds Home alone. The rail lists Home first under Default.
