---
pipeline_id: 33c054bb-98ef-4739-a155-2640abaa284a
ticket: forge#174 (ad82a4c7-16a7-48ba-b5a0-ed9827d8d10a) · local docs/planning/tickets/open/TICKET-174-agent-jump.md
aar_id: 6d6aac68-38d2-4126-a393-a9615abcda7f
status: Phase 5 — Complete PASS
title: M11 — agent rows jump to their pane; send-to-agent works cross-tab
type: feature
milestone: M11 — Live everywhere + Warp blocks
references:
  - crates/marley_app/src/app.rs (SHIM: jump_to_pane + the two row sites + the send resolve)
---

## Title
An agent row NAVIGATES: clicking it in the Agents tab or the Fleet overlay switches to the agent's
project + tab and focuses its pane, wherever it lives. ⌘⇧S (send-to-agent) also stops silently failing for
a background agent (the #173-routed fix).

## Scope
### In — SHIM only (the pure locate_pane landed + was killed in #173)
- `jump_to_pane(pane) -> bool`: locate_pane → switch_project + sync_active_project + switch_tab +
  the owning grid's focus + persist; false when the pane is gone.
- Both row click sites use it (the cockpit row keeps its ⌘-click diff branch; the Fleet row keeps closing
  the overlay); a gone pane flashes instead of no-oping.
- send-to-agent resolves its target across ALL grids (`grids_mut().find_map(terminal_mut)`).

### Out
- New pure surface (none needed); observing agent OUTPUT from the cockpit (the deeper M3/M5 slice).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 (visual) | WHEN the Agents tab is active and the agent lives in another tab, clicking its row shall activate that tab with the agent pane focused (+ a "jumped to" flash). | driven capture |
| REQ-002 (visual) | WHEN the focused terminal is in another tab, ⌘⇧S shall deliver the composed line to the agent's pty (visible in its pane on return). | driven capture (typed — #172 unblocked) |
| REQ-003 | A gone pane shall flash, not silently no-op. | code/critic |
| REQ-004 | gate GREEN. | gate |

## Phase Plan
P2 folded. P3 implement. P3.5 1 critic (jump ordering vs sync; the send borrow; flash honesty). P4 driven +
gate. P5 docs.
