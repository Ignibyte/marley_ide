---
pipeline_id: f23644d2-bead-4537-ae03-a72ab9af2c9b
ticket: forge#390 (b15dd53b-3df2-4f00-a2ae-515afed6eaa6) · local docs/planning/tickets/open/TICKET-390-panes-rail-section.md
aar_id: d347c1e0-0b0b-477e-a373-f27ae2ccd41c
status: Phase 5 — Complete PASS
title: Panes rail section — split views un-nest into the 4th section (arrangement rows)
type: feature
milestone: M27
references:
  - docs/marley_architecture/pane-composition-model.md (#388 — the 4-section model; this ships its DISPLAY half early)
  - docs/marley_architecture/app_shell.md (the #385/#386/#387 rail lineage)
---

## Title
The rail becomes chad's FOUR fixed-order sections — **Editor · Terminal · Panes · Browser** — and the
**Panes section dynamically collects every split view**. chad (2026-07-22, sketch + confirm): "panes
should be anything that opens a split view and they dynamcally allocate there"; the unit is locked =
**one row per split ARRANGEMENT** (not per cell). The #155 pane-nesting-under-tab dissolves: cell rows
move under their arrangement row in Panes; the origin tab stays whole under its own section ("the
original stays open" — model A).

## Scope
### In
- **`RailSection::Panes`** (4th variant): `ALL` order Editor→Terminal→**Panes**→Browser; `label`/
  `from_label` arms ("Panes"). #386 collapse keys are label-based and tolerant (`from_label`→None for
  unknown) → additive, old settings unaffected. **`rail_section()` NEVER returns Panes** — no tab FILES
  there; it is a DERIVED section (note this at the #385 "a new content kind must pick a section" comment).
- **`rail_rows` re-plumb:** per project, after the Terminal section: a Panes section header; under it one
  **arrangement row** per tab whose grid has **≥2 cells** ("PANE n", n = 1-based per project in tab
  order), with that tab's existing `RailLevel::Pane` CELL rows nested under the arrangement (moved —
  they no longer render under the Tab row). A single-cell tab lists nowhere in Panes. Split → the row
  appears; unsplit/close → it disappears (dynamic, derived per render — no stored state).
- **Clicks:** arrangement row → focus its tab (the #174 cross-project idiom + persist); cell rows keep
  their existing jump-to-cell behavior (moved verbatim).
- **#387 totality:** `section_action` gains a Panes arm → `SectionAction::SplitFocused` dispatched to the
  existing `split_focused_pane(Horizontal)` (reuse-only; the ＋ stays total over `RailSection`).
- **Parity:** empty Panes renders the muted header (#385 REQ-004); collapse/chevron/force-expand-active
  per #386. Active wash: the active tab's OWN section (e.g. Terminal) keeps the section highlight; the
  arrangement row carries the row-level active highlight when its tab is active.

### Out (explicitly deferred)
- ContentId identity on the rows (#394 adds it), add-to-pane, one-instance-many-views (train slices).
- Nameable/saved arrangements (train slice 6) — labels are the generated "PANE n" only.
- The GLOBAL cross-workspace Panes section (gated on multi-workspace — #388 D-gate).
- Editor-surface file-tab "splits" (an EditorSurface is one grid-less tab; only real PaneGrid splits list).

## Reference (§20)
**N/A — Marley-specific.** The 4-section sketch is chad's own (2026-07-22); no Warp/Zed analog (Warp
has no type-sectioned rail — docs/warp_architecture/subsystems/03-terminal-session-core.md, research
map; Zed's project panel lists files, not view-arrangements — docs/zed_architecture/subsystems/
07-workspace-panes-palette.md, research map). No copyleft source read.

### Prior art
1. **In-repo owners (the highest-yield leg):** `rail_rows` (tabs.rs — the ONE pure row producer; the
   #155 Pane rows + #385 Section rows + #386 collapse all live there); the Pane-row render arm + its
   cross-project click (app.rs ~16286-16330, the #174 idiom); `split_focused_pane` (#155/M10 #166);
   the #385 empty-header + #386 chevron machinery; #387's `section_action`/`SectionAction` (the arm
   this extends). The #388 design doc's Panes-section rows ("carry BOTH the (tab,pane) coordinate AND
   the ContentId") — this ticket ships the coordinate half.
2. **Behavior maps** — the two research maps above (negative results).
3. **Published material** — IDE convention of derived/dynamic view lists (VS Code's "Open Editors");
   supports derived-per-render over stored state.
4. **Permissive deps** — gpui hover/click stock; no crate owns row derivation. "None: checked gpui, no
   owner" for the arrangement-row seam.

## Locked-In Decisions
- **D1 — Arrangement = the unit** (chad, AskUserQuestion 2026-07-22): one Panes row per multi-cell grid;
  cells nest under it. NOT one row per cell.
- **D2 — Derived, not stored:** Panes rows are computed by `rail_rows` from grid shape each render; no
  new persisted state (the grid codec already round-trips splits — #163/#205).
- **D3 — Display-first, deliberately ahead of the registry:** coordinates only; #394 adds ContentId.
  Recorded as a conscious re-order of the #388 train (the display needs no identity).
- **D4 — The origin tab keeps its row** (model A "original stays open") — dual listing is the point.
- **D5 — Panes＋ = SplitFocused** (reuse `split_focused_pane`); pure seams cov/MSI 100; render masked.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The rail shall render four fixed-order section headers per project — Editor, Terminal, Panes, Browser. | Unit on `RailSection::ALL` + rail_rows order; capture. |
| REQ-002 | WHEN a tab's grid gains ≥2 cells, a "PANE n" arrangement row shall appear under Panes (and disappear when the split collapses/closes). | rail_rows units (split/unsplit/close); capture. |
| REQ-003 | Cell rows shall nest under their arrangement row and shall NOT render under the Tab row (the un-nest). | rail_rows units; capture. |
| REQ-004 | The origin tab shall keep its normal row under its own section while listed in Panes. | rail_rows unit. |
| REQ-005 | WHEN an arrangement row is clicked, its tab shall focus (cross-project sync + persist, the #174 idiom). | Unit on the routing decision; driven/mechanism. |
| REQ-006 | The Panes section shall honor #385/#386 parity: empty muted header, collapse/chevron, force-expand-active; `section_action(Panes)` shall resolve to SplitFocused. | Units; capture. |

## Phase Plan
- **P2 Design** — the arrangement-row shape in `RailRow` (new level vs a flagged Tab-level?), the "PANE n"
  numbering rule, the multi-cell predicate, active-wash details, the section_action Panes arm.
- **P3 Implement** — tabs.rs (`RailSection::Panes`, rail_rows re-plumb, SectionAction::SplitFocused) +
  app.rs (render arms for the arrangement rows + Panes ＋ dispatch).
- **P3.5 Inspect** — critics: numbering stability across closes, filter/search interaction, collapse
  interplay, the #386 collapse-key remap with 4 sections, active-wash correctness.
- **P4 Validate** — rail_rows unit matrix (cov/MSI 100) + `--diff` gate + captures (4 headers, split →
  PANE 1 appears, un-nest proven).
- **P5 Complete** — CHANGELOG, app_shell.md + pane-composition-model.md cross-ref, archive, close #390.
