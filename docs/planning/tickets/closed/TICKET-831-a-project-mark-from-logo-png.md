# TICKET-831 — A project's mark from its logo.png

- **Ticket:** LOCAL #831 (feature, the Marley look, W1.2)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** [831-a-project-mark-from-logo-png.spec.md](../../pipeline/completed/831-a-project-mark-from-logo-png.spec.md)
- **Source ticket:** Chad, 2026-10-20: "lets file all the tickets now and lets start on this", on
  [the Marley look plan](../../../marley/look-plan.md), item W1.2. The design canvas is
  https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn; its source is kept in the private design record,
  `docs/design/marley-canvas-2026-10-20`.
- **Boards:** ProjectLogo
- **Size:** M
- **Status:** closed

## Summary

Look for `logo.png`, then `logo.svg`, in a project's top folder; a file picked in Settings wins; otherwise the name's first letter on a tint of its own (`marley.project_logo`, proposed). Read once and cached, a remote project's file once over SSH; a change to the file updates the mark. Shown in the rail's group header, the shrunk strip (#830), the title bar, Home's recent projects, the switcher and the project's notifications.

## Acceptance

A project with `logo.png` shows it in each of those places; one without shows its letter; replacing the file updates the mark. The visual check shoots the change in the real app; a color or layout change is shot in
both Marley Night and Marley Day once #828 lands.
