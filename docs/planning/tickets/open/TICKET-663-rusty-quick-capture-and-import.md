# TICKET-663 — Quick capture and import from the palette

- **Ticket:** LOCAL #663 (feature, Rusty in Marley R-D3's Main.qml dialogs)
- **Owner:** claude-opus-5-5, 2026-10-06 (Chad: "we need to look into then adding the last things missing", after the R1 to R7b batch)
- **Pipeline doc:** (none yet)
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D3, the slices table); the read-only survey of
  Rusty at `13249a8` and Marley on 2026-10-06
- **Status:** open

## Summary
Rusty's app has three dialogs Marley lacks: capture a line into today's note or the inbox (`brain_capture { text, target, date }`, a timeline entry), capture a URL as a source page (`source_capture { url }`, fetched by Rusty, up to 20 s), and import a vault (`brain_import_plan { path }`, then `brain_import { path }` after a review of the plan's counts). This ticket adds `rusty: capture to today`, `rusty: capture to inbox`, `rusty: capture url` and `rusty: import vault` (a folder picker, then the plan, then Import), and `rusty: open today`. Marley's 5 s tool deadline is too short for `source_capture` and `brain_import`, so a call names its own deadline.

## Acceptance
A line captured from the palette lands in today's note's timeline; a URL captured opens as its source page (or says why it failed); a vault folder shows its import plan and, on Import, its pages appear in the Brain view.
