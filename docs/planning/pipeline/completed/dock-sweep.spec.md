---
pipeline_id: 78db4af4-b681-41c0-b0cd-bef0c25f4dd6
ticket: forge#171 (be697f04-ce9a-4857-9c12-6b5ea7d6422d) · local docs/planning/tickets/open/TICKET-171-dock-sweep.md
aar_id: none (chore — captured locally)
status: Phase 5 — Complete PASS
title: Sweep the vestigial right-dock plumbing (post-#153)
type: chore
milestone: M11 — Live everywhere + Warp blocks
references:
  - crates/marley_app/src/settings.rs (drop DockRight + persist_dock_right + AppliedSettings.right)
  - crates/marley_app/src/app.rs (drop the import + dead toggle_dock(); toggle_dock_state → left-only)
---

## Title
Delete the dead right-dock code the #153 cockpit-tabs retirement left behind — zero behavior change.

## Scope
- Removed (provably dead): the `DockRight` setting, `persist_dock_right`, `AppliedSettings.right`
  (write-only — the docks init already hardcoded `DockState::Closed`), the pub `toggle_dock()` (no callers),
  and `toggle_dock_state`'s generality (only Left is ever toggled → `toggle_left_dock`).
- Kept (the general layout primitive, honest): `DockSide` + `dock()` + `region_widths`'s left/right params;
  `docks[1]` stays a documented permanently-Closed slot (region_widths yields right=0). The `docks.right`
  TOML key in an OLD settings file is now simply ignored on load (tolerated unknown key).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | No `DockRight`/`persist_dock_right`/`AppliedSettings.right`/`toggle_dock` shall remain; the crate shall build + clippy-clean. | grep + build |
| REQ-002 | The Files (left) dock toggle shall still work + persist (no behavior change). | driven capture |
| REQ-003 | gate GREEN; the settings round-trip test (minus the dropped field) stays cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 the sweep. P3.5 self-review (dead-proof: zero readers of the removed symbols; back-compat of
an old docks.right key). P4 the left-dock driven no-regression + gate. P5 docs.
