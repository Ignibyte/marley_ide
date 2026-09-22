---
pipeline_id: 21b66e5b-2840-403d-a608-7292acfdaf88
ticket: forge#386 (8ac2f904-6043-4c1b-8b63-e101a6bd3084) · local docs/planning/tickets/open/TICKET-386-section-interaction.md
aar_id: — (opened at promotion)
status: Phase 5 — Complete PASS
title: Section interaction — active-section highlight, collapse/expand with persistence, footer vocabulary reconciled
type: feature
milestone: M26
references:
  - docs/planning/pipeline/queued/385-sectioned-rail.spec.md (HARD DEP — promote #385 first)
---

## Title
The #385 sections become interactive: active-section highlight (the workspace-centric
"rail highlights the active thing", applied at section level), click-to-collapse with a persisted
collapsed set, auto-expand on activation into a collapsed section, and a small footer-vocabulary
reconciliation audit. Slice 2 of the M26 sectioned shell.

## Scope
### In
- **Active-section highlight:** the section header whose section contains the active tab renders
  the accent-wash treatment (#219's `rail_highlight` idiom;
  PR-claude-selection-bg-distinct-from-container — never surface-on-surface); the other two
  headers stay quiet. Pure decision: which section is active, from the active tab's kind (#385's
  `section_of`).
- **Collapse/expand:** clicking a section header toggles that section's tab rows. Mirrors the
  existing per-project collapse (`rail_rows(ws, collapsed)` already threads a project-collapse
  set — tabs.rs:601; app.rs render loop honors it). The collapsed-set model extends to sections
  (keyed per project × section).
- **Persistence:** the collapsed-section set round-trips through the settings idiom (the same
  family as the M14 collapse-persist for projects); restored at boot.
- **Auto-expand invariant:** WHEN a tab inside a collapsed section becomes active (palette,
  keyboard switch, open-file, ＋-action), the section expands — an active tab is never invisible
  (the #305 auto-reveal lesson applied at rail scale).
- **Footer vocabulary audit:** #382's `focus_label` says "cockpit" while that tab now files under
  **Browser**. Decide in design: rename the label, keep it (documented), or contextualize —
  smallest honest change; the outcome is recorded, not silently skipped. No regression to #382's
  pane-kind derivation either way.

### Out (explicitly deferred)
- Per-section ＋ actions (→ #387).
- Any reordering/storage change (D1 of #385 stands).
- Collapse animations; drag-reorder; per-workspace collapse profiles.

## Reference (§20)
**N/A — Marley-specific.** Section-level active highlight + collapse-persist on a type-sectioned
rail is chad's own design (2026-07-22; the 2026-07-10 workspace-centric "the SIDE RAIL should
HIGHLIGHT the open/active workspace" applied one level down). Zed's dock/panel toggling
(docs/zed_architecture/subsystems/07-workspace-panes-palette.md — research map) and general IDE
tree-collapse conventions informed the interaction shape only; no source read.

### Prior art
1. **Behavior maps** — docs/zed_architecture/subsystems/07-workspace-panes-palette.md (dock
   open/close state persistence in Zed's workspace — research); Marley's own shipped precedent is
   stronger: the per-project collapse set already threading `rail_rows(ws, collapsed)` and the
   M14 collapse-persist ticket family.
2. **Published material** — standard IDE tree-section collapse (VS Code sidebar sections persist
   collapsed state per workspace) — supports persistence as the expected default.
3. **Our permissive deps** — gpui (Apache-2.0): click handling + hover are plain gpui
   `on_mouse_down`/hover; no collapse primitive exists or is needed. No crate owns the seam. The
   in-repo owners: the project-collapse set (`rail_rows(ws, collapsed)` tabs.rs:601 +
   `self.collapsed_projects` app.rs:15973) is the pattern to extend, and the typed settings
   round-trip (settings.rs `define_setting!` family, e.g. `ShellLayoutSetting` :144/:241) is the
   persistence idiom. **Reuse both; invent neither.**
4. **In-house lessons** — #219 (`rail_highlight` accent-wash;
   PR-claude-selection-bg-distinct-from-container), #305 auto-reveal (an affordance that hides the
   active thing must auto-reveal on activation), PR-claude-pump-state-change-must-set-dirty (any
   non-input state change affecting render sets `dirty`).

## Locked-In Decisions
- **D1 — Highlight = the #219 accent-wash idiom**, applied to the section header row; contrast
  proven by the existing `contrast_ratio` helpers if a new color pairing appears.
- **D2 — Collapse state is per (project, section)**, persisted via the settings round-trip; boot
  restores it. Default: all expanded.
- **D3 — Auto-expand on activation is an invariant**, not a preference — no setting gates it.
- **D4 — Footer outcome is a recorded design decision** (rename vs documented no-op); #382's
  exhaustive `PaneKind` match stays exhaustive.
- **D5 — Pure seams:** active-section decision, collapse-set model + codec, auto-expand decision —
  cov/MSI 100. Render/click wiring masked, capture-validated.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a project renders in the rail, the section header containing the active tab shall carry the active accent-wash and the other section headers shall not. | Unit on row flags; capture (switch tabs → highlight follows). |
| REQ-002 | WHEN a section header is clicked, its tab rows shall toggle collapsed/expanded. | Unit on the toggle model; driven click + capture. |
| REQ-003 | The collapsed-section set shall persist across relaunch (settings round-trip; boot restores). | Unit round-trip; relaunch smoke. |
| REQ-004 | WHEN a tab within a collapsed section becomes active by any path, that section shall auto-expand so the active tab row is visible. | Unit (activate into collapsed → expanded); driven palette-switch check. |
| REQ-005 | The footer focus vocabulary shall be reconciled with the section names — the divergence (cockpit/Browser) resolved by a recorded design decision with tests updated to match (or a documented no-op). | Design record + unit if changed; review. |

## Phase Plan
- **P2 Design** — collapsed-set shape (extend `collapsed` param vs a second set), the setting key,
  highlight styling; the footer decision (D4).
- **P3 Implement** — tabs.rs model + settings codec + app.rs render/click arms.
- **P3.5 Inspect** — critics (persistence collision with project-collapse; auto-expand from every
  activation path — palette, ⌘-switch, open-file, close-refocus; dirty-flag on toggle).
- **P4 Validate** — unit matrix + round-trip + `--diff` gate + driven captures (highlight follows
  active; collapse survives relaunch).
- **P5 Complete** — CHANGELOG, app_shell.md, archive, close #386.
