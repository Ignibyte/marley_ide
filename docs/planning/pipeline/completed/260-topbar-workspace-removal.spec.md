---
pipeline_id: 4b0b40cf-e8b0-4b2e-93cc-fc5c1cd11a31
ticket: forge#260 (7d0d3756-9c85-4fdd-8fee-34e6b69d3566) · local docs/planning/tickets/open/TICKET-260-topbar-workspace-removal.md
aar_id: 0dcc41ed-4c50-4d1c-b22a-44c86752d953
status: Phase 5 — Complete PASS
title: Remove the top-bar workspace indicator/switcher — workspace lives in the left rail
type: chore
milestone: M16
references: []
---

## Title
chad (2026-07-11 live review): "I don't want the project workspace at the top.
we opt in for the left." Remove the #235 focused-workspace indicator
("Marley · main") and the #244 click-to-switch popover from the TOP BAR; the
left Workspace rail is the canonical home (#233 rail-click switches, #236
highlights). Removing the duplication loses no capability.

## Scope
### In
- app.rs: delete the #235 indicator render block + its #244 toggle listener;
  delete the #244 popover render block (backdrop + rows); delete
  `workspace_switcher_open` (field + init); remove the two now-unused imports;
  delete `TOPBAR_INDICATOR_MAX_CHARS` + `TOPBAR_INDICATOR_WIDTH`; re-anchor
  the cockpit-tab cluster to the vacated slot-3 x (no 180px ghost gap).
- titlebar.rs: delete `focused_workspace_indicator`, `workspace_switcher_rows`,
  `SwitcherRow`, and their two test blocks (dead code under -D warnings
  otherwise). `branch_from_git_head` STAYS if any other caller remains
  (verify at design).
- CHANGELOG + app_shell.md at Phase 5.

### Out (explicitly deferred)
- The window/OS titlebar M8 #142 `~/…/Marley · main` status label — SEPARATE,
  untouched (per the ticket).
- The left-rail workspace UX itself (#233/#236) — unchanged, it is already
  the single source of truth.
- Any rail-side switcher addition — the rail already switches; nothing to
  relocate (ticket's "likely just remove" confirmed).

## Reference (§20)
N/A — Marley/IDE-specific chrome (the multi-workspace rail model is Marley's
own M13 direction; no reference-app analog is being matched — this REMOVES a
Marley-specific top-bar affordance per owner feedback).

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — Pure removal, no relocation: the #233/#236 rail is the switch
  affordance; no new switcher UI anywhere.
- D2 — The cockpit tabs (Details/Agents/Forge) re-anchor to the slot-3 x the
  indicator vacated (the #235 relocation's intent — "group with the workspace
  actions" — survives without the label).
- D3 — Delete-don't-deprecate: fns/type/state/consts/tests all removed in the
  same change (§0 no-dead-code).
- D4 — The #142 OS-titlebar label and the footer cwd·branch are untouched.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The top bar shall render NO workspace name/branch indicator and NO click-to-switch popover; a click where the indicator sat shall open nothing. | driven capture (top bar) + code removal review |
| REQ-002 | Clicking a left-rail workspace row shall still switch the focused workspace with the active highlight following (#233/#236 unchanged). | existing rail tests green + driven capture |
| REQ-003 | The cockpit tabs shall render in the left cluster at the vacated slot-3 anchor (no 180px gap) and still open their sections. | driven capture + click |
| REQ-004 | `focused_workspace_indicator`, `workspace_switcher_rows`, `SwitcherRow`, `workspace_switcher_open`, `TOPBAR_INDICATOR_*` shall be absent from the crate. | grep empty + clippy -D warnings green (no dead code) |
| REQ-005 | The #142 OS-titlebar label and footer cwd·branch shall be unchanged. | grep `titlebar_label` callers intact + capture |

## Phase Plan
- **P2 Design** — confirm branch_from_git_head's surviving callers; the exact
  removal hunks; cockpit-tab anchor edit; test-removal list; regression plan.
- **P3 Implement** — the deletions + re-anchor; cargo check.
- **P3.5 Inspect** — critics (removal completeness, no orphaned handler/state,
  rail regression).
- **P4 Validate** — tests + driven captures (REQ-001/002/003/005); gate --diff.
- **P5 Complete** — CHANGELOG, app_shell.md (#235/#244 entries updated),
  AAR, archive, close.
