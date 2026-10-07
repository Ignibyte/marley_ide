---
pipeline_id: c06aeeca-0f1c-4492-8f74-f7fec2244129
ticket: docs/planning/tickets/open/TICKET-671-rail-keeps-the-window-order.md
status: Phase 4 — Complete PASS
title: "The rail keeps the window's order"
type: bug
slice: the rail (#542, #602)
references: [docs/planning/pipeline/completed/542-rail-attention-order.spec.md, docs/planning/pipeline/completed/602-drag-to-reorder-the-rail.spec.md]
---

## Title
The rail keeps the window's order. Since #542 the rail sorts projects, and the rows under each,
by what needs Chad (`marley.rail_order: "attention"`). Typing into Claude Code makes it print, the
quiet timer reads that as Working, and Working sorts above Idle, so the project climbs and drops
back when the output stops. Chad on 2026-10-06: "when i start typing in marley the project moves
to the top above another project? Lets remove that. for some reason it jumps back randomly also".
The default becomes `"window"`; the setting stays for anyone who wants the attention order.

## Scope
### In
- **The default**: `marley.rail_order` is `"window"` in `assets/settings/default.json`, in
  `MarleyRailOrder`'s `#[default]` and its doc's `Default:` line (`settings_content`), and in
  `marley_rail::RailOrder`'s `#[default]`.
- **The words**: the default's comment, the guide and the walkthrough's #542 stops say the rail
  keeps the window's order unless `"attention"` is set.

### Out (explicitly deferred)
- **Removing the attention order**: the setting and its code stay; turning it on is one line.
- **The collapsed header's counts** (`1 waiting, 2 working`, #542): they move nothing and stay.

## Reference (§20)
Upstream Zed — the Threads Sidebar (`crates/sidebar`) lists project groups in
`MultiWorkspace::project_groups` order and moves none on activity; the window's order here is that
order with #602's drag on top. The attention order this turns off came from Orca's smart attention
(`docs/orca_architecture/01-agents-and-sessions.md`).

### Prior art
- **Behaviour maps:** Orca's `smart-attention.ts` ordering (`01-agents-and-sessions.md`), the source
  of #542; Warp's vertical tabs keep the user's tab order
  (`docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`).
- **Published material:** none applies.
- **The code we ship:** `RailOrder::Window` already exists (#542) and `project_order` passes the
  window's order through untouched; nothing new is needed.

## UI proof
`script/e2e/671-rail-keeps-the-window-order.sh` (`compositor sway`), which reuses #542's three
stand-in projects a, b and c (window order a, b, c) with no `rail_order` in the copy's settings.
Shots: `671-01-working` (c, the bottom project, working: the rail still lists a, b, c);
`671-02-waiting` (b waiting too: still a, b, c, under each project the rows in tab order);
`671-03-attention` (`rail_order` set to `"attention"`: b and c lead, #542's order back).

## Locked-In Decisions
- D1 — **Flip the default, keep the setting.** The attention order is a working feature some will
  want; "remove that" is met by its being off.
- D2 — **Rows under a project follow the same setting**, as they do today: window order keeps the
  terminals in tab order.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.rail_order` is unset, the rail shall list projects in the window's order whatever their agents do. | `671-01-working`, `671-02-waiting` |
| REQ-002 | WHILE `marley.rail_order` is unset, the rows under a project shall keep their tab order. | `671-02-waiting` |
| REQ-003 | WHEN `marley.rail_order` is `"attention"`, the rail shall order projects by attention as #542 does. | `671-03-attention` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the three defaults and their words; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the scenario and its shots.
- **P4 Complete** — CHANGELOG, the guide and walkthrough, ledger capture, close, archive, commit.
