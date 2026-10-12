# TICKET-731 — Viewed marks in the project diff

- **Ticket:** LOCAL #731 (feature; size small to medium; the review batch after the finish)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [731-viewed-marks-in-the-diff.spec.md](../../pipeline/completed/731-viewed-marks-in-the-diff.spec.md)
- **Source ticket:** [plannotator-findings.md](../../intake/plannotator-findings.md), item 6; Chad, 2026-10-09: "take a look at https://github.com/backnotprop/plannotator and see what may be to add to the list"; Chad, 2026-10-11, on the [finish-line list](../../design-notes/finish-line-2026-10-11.md): "11.) yes"
- **Status:** closed

## Summary
A Viewed box at the end of each file's header in the project diff. Ticking it folds the file. The
mark is kept with a fingerprint of the file's diff, so it survives a restart and clears, unfolding
the file, when the diff changes, as when an agent rewrites the file. The header's control comes
through a Marley addon on the diff's editor and one additive touch in Zed's `editor` crate (a second
addon's header controls), not in `git_ui`.

## Acceptance
A file marked viewed folds, stays marked after a restart, and loses the mark when it changes.
