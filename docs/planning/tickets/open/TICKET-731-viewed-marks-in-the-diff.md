# TICKET-731 — Viewed marks in the project diff

- **Ticket:** LOCAL #731 (feature; size small to medium)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** [plannotator-findings.md](../../intake/plannotator-findings.md), item 6; Chad, 2026-10-09: "take a look at https://github.com/backnotprop/plannotator and see what may be to add to the list"
- **Status:** open (deliberate: for future use)

## Summary
A Viewed mark per file in the project diff that survives a restart and clears when an agent rewrites the file. An additive touch to Zed's `git_ui`.

## Acceptance
A file marked viewed stays marked after a restart and loses the mark when changed.
