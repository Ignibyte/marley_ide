---
pipeline_id: 61dddfb7-2fad-4bb3-9d6d-ff3c7505d548
ticket: forge#217 (2c09c9bc-f7b4-4a29-9cf8-2f6233b92ef0) · local docs/planning/tickets/open/TICKET-217-warp-block-styling.md
aar_id: 8ca6ef52-b09e-48b0-b885-f814a707bebe
status: Phase 5 — Complete PASS (Plan · Design · Implement · Inspect · Validate all PASS; driven at-rest/hover captures confirm REQ-001..003)
title: Warp visual parity — command-block styling (hover-reveal affordances)
type: feature
milestone: M12.2
references: []
---

## Title
Match Warp's command-block styling. The genuine delta (confirmed at discovery):
Marley's block affordances (⧉ copy-cmd / ⧉ copy-out / ↻ rerun) are ALWAYS visible;
Warp reveals block actions on hover. Make Marley's reveal on block-hover — matching
Warp + decluttering the block header.

## Scope
### In
- The block-header affordances (`copy_cmd`/`copy_out`/`rerun`, app.rs ~4404-4456): hidden
  by default, revealed when the block header is hovered (gpui group-hover).
### Out (explicitly deferred)
- The status glyph (○/✓/✗ #36), command emphasis, the `border_t_1` separator — already
  read Warp-like, within tolerance (no churn).
- The block context menu (#175) — unchanged (still the right-click path to the actions).
- Sidebar/prompt/cursor styling — separate warp-parity tickets.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — reveal-on-hover via gpui group-hover: the block header is a hover `group`; the
  affordances start hidden and become visible on the header's group-hover. (Design confirms
  the exact gpui API; fallback = a per-block hovered-state flag if group-hover is unavailable.)
- **D2** — the actions stay reachable when NOT hovering via the #175 right-click block menu
  (no loss of function — just a cleaner default, like Warp).
- **D3** — auto-approved (/work 195–222): document with the Warp-vs-Marley captures.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a block is not hovered, its ⧉/↻ affordances shall be hidden. | Driven capture (block at rest = clean header) |
| REQ-002 | WHEN a block header is hovered, its affordances shall appear and remain clickable (copy/rerun still work). | Driven capture (hover → affordances show) + the actions still function |
| REQ-003 | The change shall not alter the status glyph, command text, or block separator. | Review + capture |

## Phase Plan
- **P2 Design** — confirm the gpui group-hover API; the exact render change (group on the
  header, hidden+group_hover on the 3 affordances); manifest + test plan.
- **P3 Implement** — the group-hover render change.
- **P3.5 Inspect** — critic/self-review: the actions stay clickable when revealed; no
  regression to the #175 menu or selection.
- **P4 Validate** — driven capture (at-rest clean, hover reveals); gate.
- **P5 Complete** — CHANGELOG + doc; AAR; close.
