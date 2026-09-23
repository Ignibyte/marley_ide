# TICKET-460 — The Marley layout by default

- **Ticket:** LOCAL #460 (feature, workbench shell)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/460-marley-layout-by-default.spec.md
- **Source ticket:** ../../../marley/workbench-shell.md (the Default paragraph)
- **Status:** closed

## Summary
Chad, 2026-09-23: "when the program is installed the user shouldnt have to choose Marley at
first it should swap it over." The fork starts in Zed's layout today, because `marley.layout`
defaults to `zed`. The shell plan left the choice open: "if Chad meant Marley by default, the
design is the same with the default flipped." Marley becomes the default; a user who writes
`zed` keeps Zed's layout, and Zed's own tests keep testing Zed's layout.

## Acceptance
With no `marley.layout` in the settings, windows open in the Marley layout. The EARS criteria
live in the pipeline spec.
