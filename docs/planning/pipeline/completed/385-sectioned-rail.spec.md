---
pipeline_id: 73ab7594-929f-4679-8b06-da74756115f6
ticket: forge#385 (09006da6-d499-4035-9f47-046accb4fbcd) · local docs/planning/tickets/open/TICKET-385-sectioned-rail.md
aar_id: — (opened at promotion)
status: Phase 5 — Complete PASS
title: Type-sectioned rail — Editor / Terminal / Browser fixed-order sections (display-only grouping)
type: feature
milestone: M26
references:
  - docs/planning/pipeline/queued/386-section-interaction.spec.md (slice 2, depends on this)
  - docs/planning/pipeline/queued/387-section-actions.spec.md (slice 3, depends on this)
---

## Title
The left rail's flat per-project tab list becomes three FIXED-ORDER type sections — **Editor** at
the top, then **Terminal**, then **Browser** — and every tab files under the section matching its
content kind. chad's 2026-07-22 sectioned-shell direction ("force Editor at the Top then Terminal
and last Browser… Any time you open up a certain thing then it goes under that tab"), the first
shippable slice: the *organized* half, with zero storage/model risk.

## Scope
### In
- A pure section model in `tabs.rs`: a `RailSection` (or equiv.) kind — fixed order
  `Editor → Terminal → Browser` — plus a total `section_of(&TabContent) -> RailSection` mapping:
  `CodeView → Editor`, `Terminal → Terminal`, `Cockpit → Browser` (the cockpit/Forge tabs are the
  Browser section's **transitional residents** until Phase E lands a real webview — chad: "Forge
  will open in the browser").
- `rail_rows` (tabs.rs:601-655) emits, per project: the Project header, then for each of the three
  sections IN FIXED ORDER a Section header row, then that section's tab rows (in existing
  `Vec<Tab>` order), with split-pane rows still nesting under their tab row (tabs.rs:635-651
  unchanged in spirit).
- Empty sections still emit their header row (rendered muted) — the fixed skeleton is always
  visible and teaches the model.
- `RailRow`/`RailLevel` (tabs.rs:513-544) grow a Section level/variant; the app.rs render loop
  (~app.rs:16011) renders the new row level (indent, muted caption styling per the #230 Nav role).
- Unit tests proving grouping/order/empty-header/nesting on mixed-kind workspaces.

### Out (explicitly deferred)
- Section highlight, collapse/expand, persistence → #386.
- Per-section ＋ actions + guard re-expression → #387.
- Any storage reordering of `Project.tabs`, any codec change, any `TabContent` change (the map
  flags order-keyed `active: usize` + the order-based `TabLayout` codec as the hazard; this slice
  deliberately never touches them).
- A real Browser content kind (→ #389 spike; Phase E).
- Tab ordering *within* a section beyond storage order (drag-reorder etc. — not asked).

## Reference (§20)
**N/A — Marley-specific.** The three-fixed-type-sections rail is chad's own design (2026-07-22
conversation, extending the 2026-07-10 workspace-centric direction: launcher + multi-workspace +
rail-highlight). Neither reference app behaves this way: Warp's sidebar is a flat session list
(docs/warp_architecture/subsystems/03-terminal-session-core.md — research map), and Zed's dock
panels group by *panel*, not by open-item type (docs/zed_architecture/subsystems/07-workspace-panes-palette.md
— research map). General-IDE conventions (VS Code's "OPEN EDITORS" section; JetBrains tool-window
grouping) were consulted as published-behavior research only; no source read, nothing translated.

### Prior art
1. **Behavior maps** — docs/zed_architecture/subsystems/07-workspace-panes-palette.md (Zed's
   workspace/pane/dock model; confirms Zed has no type-sectioned open-item rail — panels are
   feature-scoped) and docs/warp_architecture/subsystems/03-terminal-session-core.md (Warp's flat
   session sidebar). Research, not source.
2. **Published material** — VS Code "OPEN EDITORS" / JetBrains tool windows: the convention that a
   fixed skeleton with per-type buckets reads as "organized" — supports always-rendering empty
   section headers.
3. **Our permissive deps** — gpui (Apache-2.0): no section/outline list primitive; the rail is a
   plain flex column of rows, ours to shape. `rail_rows` (tabs.rs:601) is Marley's own seam and
   already the single producer both the render loop and its unit tests consume — the section
   change lands in ONE pure fn. No crate we ship owns this seam. **The 2026-07-22 code map is the
   sweep's fourth leg**: `TabContent` (tabs.rs:23-31, 3 variants, kind observable via
   `grid()`/`cockpit_section()`/`editor()`), `RailRow`/`RailLevel` (tabs.rs:513-544), render loop
   app.rs:16011, and the flagged hazard (order-keyed `active` + order-based codec) that fixes this
   slice as display-only.

## Locked-In Decisions
- **D1 — Display-only.** `Project.tabs` storage order, `active: usize` semantics, `switch_tab`,
  and the shell codec (`grid_layout.rs` `TabLayout`) are byte-identical. Sections exist only in
  `rail_rows` output. (The map's flag (a).)
- **D2 — Fixed order Editor → Terminal → Browser**, always all three, per project. An empty
  section renders its header muted. (chad's "force" wording = the skeleton is invariant.)
- **D3 — Cockpit files under Browser.** The native cockpit/Forge tabs are the Browser section's
  transitional residents (chad: "Forge will open in the browser"); #389 plans the real webview
  succession. No fourth section.
- **D4 — Pure seam carries the mutation load.** `section_of` + the row-emission ordering are pure
  (cov/MSI 100); the app.rs render arm for the new row level is masked shim, capture-validated.
- **D5 — Pane rows unchanged** — they nest under their tab row exactly as today (tabs.rs:635-651).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a project holds tabs of mixed kinds, the rail shall list, after the Project header, exactly three section headers in the fixed order Editor → Terminal → Browser, with every tab row under the section header matching its content kind (CodeView→Editor, Terminal→Terminal, Cockpit→Browser). | Unit tests on `rail_rows` output (order + assignment); live capture. |
| REQ-002 | WHEN a new thing is opened (new terminal / open file / open cockpit), its tab row shall appear under its kind's section rather than at the rail tail. | Unit (add_tab then rail_rows); driven live check. |
| REQ-003 | The grouping shall be display-only: `Project.tabs` order, active-tab semantics, and the persisted shell layout shall be unchanged (existing codec + tab tests pass unmodified). | Existing suite green with zero persistence-test edits; restore round-trip smoke. |
| REQ-004 | WHILE a section has no tabs, the rail shall still render that section's header (muted empty state). | Unit on rows; capture. |
| REQ-005 | WHILE a terminal tab has split panes, its pane rows shall keep nesting under that tab row inside the Terminal section. | Unit; capture. |

## Phase Plan
- **P2 Design** — the `RailSection` shape (RailRow field vs RailLevel variant), header-row styling
  (Nav/Caption role), exact insertion points in `rail_rows` + the app.rs render arm; confirm D1's
  zero-codec-touch against `grid_layout.rs`.
- **P3 Implement** — tabs.rs pure model + app.rs render arm.
- **P3.5 Inspect** — critics vs the diff (ordering edge cases: cockpit-only project, empty
  sections at boot, collapse interplay with the existing project-collapse set).
- **P4 Validate** — unit matrix + `--diff` gate + a driven capture (mixed workspace → three
  sections read correctly).
- **P5 Complete** — CHANGELOG, app_shell.md rail note, archive, close #385.
