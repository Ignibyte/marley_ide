# TICKET-761 — Marley's keyring label

- **Ticket:** LOCAL #761 (chore, the finish line)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** not started; the design is drafted in [445's spec, Out item 2](../../pipeline/completed/445-marley-release-identity.spec.md)
- **Source ticket:** split from #445 (slice 3) on 2026-10-11, when Chad asked to "spec all" of the survey list. Plan: [finish-line shelf](../../design-notes/finish-line-2026-10-11.md).
- **Status:** open

## Summary
Marley's own Secret Service label in place of the fixed `zed-github-account`, System One's keys included, so Marley's secrets no longer sit under stock Zed's name, with the existing entries carried over.

## Acceptance
After the change the keyring lists Marley's entries under Marley's label, and sign-in and System One still work without asking again.
