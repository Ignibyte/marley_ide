# TICKET-760 — Marley calls itself Marley

- **Ticket:** LOCAL #760 (chore, the finish line)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** not started; the design is drafted in [445's spec, Out item 1](../../pipeline/completed/445-marley-release-identity.spec.md)
- **Source ticket:** split from #445 (Marley's release identity, slice 2) on 2026-10-11, when Chad asked to "spec all" of the survey list. Plan: [finish-line shelf](../../design-notes/finish-line-2026-10-11.md).
- **Status:** open

## Summary
Marley's own Wayland app id and X11 class (today `dev.zed.Zed-Dev`), the name it shows for itself in place of "Zed Dev", the desktop entry's `StartupWMClass`, the e2e runner's class, and the single-instance socket's name. It stays on Zed's `dev` channel, so the database (`db/0-dev`), the `"dev"` settings block and the credentials file do not move.

## Acceptance
The window's app id, its title in the About window and the socket's name say Marley; a restart keeps every project, group and setting.
