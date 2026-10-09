# TICKET-714 — Undo a turn

- **Ticket:** LOCAL #714 (feature; size small to medium)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** [monocode-findings.md](../../intake/monocode-findings.md), item 1; Chad, 2026-10-09: "lets queue up the MonoCode findings for potential future use"
- **Status:** open (deliberate: for future use)

## Summary
Undo and Keep on each turn row of #509. A turn is already a commit under `refs/marley/turns/`, so Undo reverse-applies that turn's diff to the working tree, refusing on conflict and leaving earlier turns in place. Keep marks it reviewed.

## Acceptance
A turn's Undo takes its changes out of the files and leaves the turns before it; a conflicting Undo is refused with why.
