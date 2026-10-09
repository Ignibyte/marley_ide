---
status: superseded
created: 2026-08-12
ticket: <unassigned>
pipeline_spec: <unassigned>
note: audit (2026-10-09): obsolete; both the marley-web POC's overlays and the gpui app's are gone, and Zed's finder and palette replaced them
---

# POC overlay parity nits — the three pre-existing divergences the #416 inspect surfaced

## What
Three small marley-web POC divergences from shipped Marley overlay behavior,
found (and two-sided-verified against the Rust) by the #416 inspect critics.
None introduced by #416; all pre-existing:

1. **Render-cap divergence** — Marley's file finder + history render
   `results.iter().take(20)` (app.rs ~21112/~21142, "capped at 20 rendered
   rows"); the POC's FileFinder/HistorySearch render ALL rows. Visible only at
   window heights ≥ ~746px (POC shows 22 fixture rows where Marley shows 20).
   The palette is genuinely unwindowed on BOTH sides (exact match). Note
   Marley's own split: it clamps selection against the FULL result length
   while rendering 20 — the highlight can leave the rendered set; the POC fix
   (if taken) is `filtered.slice(0, 20)` in the two renders while keeping the
   hook count at full length, mirroring the warts-and-all contract.
2. **Enter on an empty result set** — Marley's enter arms close the overlay
   unconditionally; the POC's `if (!count) return` swallows Enter and stays
   open (OverlayShell.tsx:52).
3. **Selection re-anchor keying** — Marley resets selection on every query
   EDIT (finder.rs push/backspace → selected=0); the POC re-anchors on
   `[count]` change only. Repro: finder, ArrowDown×5, type "c" (all 22
   fixture paths contain "c") → count unchanged, POC keeps row 5, Marley
   resets to row 0.

## Why
Zone A frozen-shell parity (MARLEY-PARITY.md) — each is a small
behavior-parity hole of the same family #416 just closed; cheap to fix
together in one POC-side pass.

## Notes
Full evidence chains in the #416 inspect ledger
(docs/planning/pipeline/completed/416-poc-overlay-drifts.notes.md after
archive) — Rust anchors: palette.rs:105-113, finder.rs:31-52, app.rs
finder/history render sites. POC anchors: OverlayShell.tsx:43 (re-anchor
effect), :52 (empty guard); FileFinder.tsx / HistorySearch.tsx renders.

## Promotion
This is NOT an active pipeline doc — it is a candidate. Promote it via `/work`
when ready: it becomes a ticket (`docs/planning/tickets/open/`) + an active
pipeline doc pair (`docs/planning/pipeline/active/`). On promotion, set
`status: promoted` and fill `ticket:` + `pipeline_spec:`.
