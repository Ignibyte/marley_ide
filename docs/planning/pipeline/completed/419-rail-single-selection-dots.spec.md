---
pipeline_id: 58584d8d-7d66-448f-8aca-cb5f19ba024a
ticket: docs/planning/tickets/open/TICKET-419-rail-single-selection-dots.md
status: Phase 5 — Complete PASS
title: Rail single-selection highlight + dot indicators — the #418 model ported to the Rust rail
type: feature
milestone: M31
references:
  - docs/planning/design-notes/simple-rail-shelf.md
  - docs/planning/tickets/open/TICKET-418-simple-rail-react-design.md
  - docs/planning/pipeline/queued/418-simple-rail-react-design.spec.md
  - docs/warp_architecture/observed/beautifului-2026-08-12-notes.md
  - docs/warp_architecture/observed/beautifului-sidebar-nav-2026-08-12.png
  - docs/warp_architecture/observed/beautifului-task-rows-2026-08-12.png
  - docs/zed_architecture/subsystems/07-workspace-panes-palette.md
  - crates/marley_app/src/tabs.rs
  - crates/marley_app/src/app.rs
  - marley-web/docs/MARLEY-PARITY.md
  - marley-web/artifacts/marley-ide/src/components/LeftRail.tsx
---

## Title
The M31 simple-rail port slice: kill ancestor lighting, one selected row ever, left-edge dots for
"open elsewhere". Today ONE click lights up to SIX rows with the identical accent fill —
`rail_rows` (tabs.rs:1032-1254) computes `active` independently at every level (Project :1047,
Section :1103/:1111, Arrangement :1146, Pane :1164, Tab :1180/:1213, CrossRef :1241) and all six
render arms share `rail_highlight` (app.rs:1425-1427, accent 0.22 active / 0.10 hover; the arms:
Project :18618, Tab :18748, CrossRef :18843, Arrangement :18924, Pane :18972, Section :19046).
There is no "selected vs ancestor" distinction anywhere. Target model per #418's design (the
beautifului grammar, captured): exactly ONE row — the focused content itself — carries the
selection fill; structural/ancestor rows never fill; rows whose content is open/mounted elsewhere
(pane-mounted cross-refs, the split container of the focused cell, background-active tabs) carry a
small left-edge dot indicator instead. All line numbers re-verified against the tree 2026-08-12.

## Scope
### In
- **The pure model change:** `rail_rows` emits AT MOST ONE row flagged selected; `RailRow.active`
  (tabs.rs:793) renames/splits into a selected flag + an indicator (exact shape = Phase 2, under
  D1/D3); the dot states derive from what the fn already knows (`pane_mounts` tabs.rs:989-1024 +
  the active/focus coordinates) — no new stored state, derived fresh per render.
- **The six render arms** (app.rs:18618/:18748/:18843/:18924/:18972/:19046): fill routes from the
  selected flag ONLY; a left-edge dot element renders from the indicator; the trailing ⊞ retires
  from rail rows (D3); hover wash untouched (D4).
- **The #386 behavior carve-out:** the active-section computation keeps gating force-expand
  (tabs.rs:1103-1107) after the Section header stops filling — collapse/force-expand behavior is
  preserved exactly (D2).
- **Test re-pin:** the rail suite's active-flag pins (tabs.rs:2000, :2013-2016, :2115-2118,
  :2263-2267, :2347-2349, :2386, :2508-2514, :2665) re-pin to the selected/indicator model; the 26
  `rail_rows` test call sites compile again (`cargo check --tests`, D5).
- **Parity:** the React↔Marley parity pair against #418's re-baselined rail captures.

### Out (explicitly deferred)
- **The design itself** — #418 owns the React rail + the MARLEY-PARITY re-baseline; this ticket
  ports a LANDED design (hard ordering constraint, see React-first).
- **Per-file editor rows** (#420) and **add-project** (#421) — sibling shelf slices.
- **`pane_mounts` semantics** — the #398 foreign-host/editor-mount signals stay byte-identical as
  inputs; only their presentation changes.
- **Keyboard/multi selection** — the rail stays click-driven, single-select; no roving focus model.
- **Persistence** — nothing new serializes; selection stays derived from workspace focus state.

## Reference (§20)
**N/A — Marley-specific selection model, no reference-app analog.** The Warp behavior maps carry
ZERO sidebar/rail coverage (docs/warp_architecture/subsystems/ has no sidebar material — checked);
Warp's left panel has no project→section→tab tree to light. The DESIGN SOURCE is our own: #418's
queued spec (docs/planning/pipeline/queued/418-simple-rail-react-design.spec.md) + the observed
captures docs/warp_architecture/observed/beautifului-sidebar-nav-2026-08-12.png (ONE selected row,
subtle rounded fill, nothing else lights) and beautifului-task-rows-2026-08-12.png (the left-edge
circle indicator grammar), with the capture notes (beautifului-2026-08-12-notes.md) recording the
intent: grammar, not skin — palette/type/radius stay Marley's own. Zed is cited at the BEHAVIOR
level only: docs/zed_architecture/subsystems/07-workspace-panes-palette.md maps one active item per
pane (`active_item_index` :132) and one active panel per dock (:204) — the one-lit-thing invariant,
never ancestor lighting. Clean-room §20 untouched: no Warp (AGPL) / Zed (GPL) source consulted.

### Prior art
1. **Behavior maps — checked.** Warp: nothing (no sidebar subsystem coverage — an honest gap, not
   thin coverage). Zed 07-workspace-panes-palette.md: `active_item_index`/`active_panel_index`/
   "highlight the active workspace" (:132/:204/:238) — Zed's navigator lights the active ITEM, not
   its ancestors; behavior-level support for the invariant. In-house: the beautifului captures +
   #418's spec are the binding design source.
2. **Published material.** The WAI-ARIA tree/listbox pattern is the grammar being adopted:
   single-select trees hold `aria-selected` on EXACTLY ONE treeitem, and selection is distinct from
   expansion state — precisely D1 + D2. Observed convention (VS Code explorer, ChatGPT sidebar):
   one selected row ever; "open elsewhere" reads as a decoration (VS Code's open-editor dot /
   filled activity badge), never as a second selection fill; folders don't light when a child is
   selected. Described from observation/published docs only.
3. **Our permissive deps — none: checked gpui, our own seam.** gpui 0.2.2
   (~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-0.2.2) ships NO selection
   affordance — no `Selectable` trait, no selected state in styled.rs/interactive.rs; elements/
   {list,uniform_list}.rs are virtualization-only. Reading gpui source is sanctioned adoption and
   there is nothing to adopt: the seam is our own pure `rail_rows`, which is where the invariant
   lands (D1). ropey/regex/alacritty_terminal own nothing near this seam.

## React-first (parity)
**UI-AFFECTING — Zone A (rail).** marley-web file: `artifacts/marley-ide/src/components/LeftRail.tsx`
(the port-map row, MARLEY-PARITY.md:572 — `LeftRail.tsx` ↔ app.rs (rail) / tabs.rs /
context_menu.rs). Build & visually verify in marley-web first (`pnpm --filter @workspace/marley-ide
run dev` → localhost:5173) against #418's re-baselined design, then port 1:1; validate captures the
React↔Marley parity pair. **Ordering constraint: #418 must have landed the design** — the POC's
current rail deliberately encodes the ancestry model this ticket kills (`projectActive = true`,
LeftRail.tsx:169) and MARLEY-PARITY's rail rows still point at the old captures (:26); until #418
re-baselines both, there is nothing correct to port. /work must not promote #419 while #418 is
unshipped. The POC's `RailFill { active }` (:54) and `PaneMark` ⊞ (:11-14) are the exact primitives
whose #418 replacements this port mirrors.

## Locked-In Decisions
- **D1 — the single-selection invariant lives in the PURE model.** `rail_rows` emits at most one
  row flagged selected, for every input; the render arms consume the flag, they never re-derive
  activity. `RailRow.active` renames/splits into `selected` + `indicator` (or an equivalent
  three-state mark — exact shape is Phase 2's call; the invariant + the pure-fn placement in
  tabs.rs are locked). Selection stays DERIVED from workspace focus per render — no stored
  selected-index (the F-#236 class: index-keyed view state aliases on remove).
- **D2 — structural rows never select.** Project, Section, and Arrangement rows never carry the
  selected flag (the ticket's ancestor set: project, section, tab-of-pane, arrangement). The #386
  active-section computation SURVIVES the fill's death: `is_active_section` keeps gating
  `section_collapsed` (tabs.rs:1103-1107) so the active section still force-expands — behavior
  preserved, presentation removed. Whatever non-fill emphasis the active project keeps (bright
  text etc.) is #418's design call, not a selection.
- **D3 — the dot replaces the old duplicate lighting AND ABSORBS the rail-row ⊞.** Indicator
  states (content-bearing rows — Tab/Pane/CrossRef): **(i) mounted-elsewhere** — the row's content
  also lives in a pane cell (`pane_mounts` signals unchanged: editor cell mounts; terminal
  foreign-hosted by ≥2 distinct tabs, tabs.rs:1194-1201 — today's `pane_marked`); **(ii)
  container-of-focus** — a multi-cell Tab row whose focused cell's row (Pane/CrossRef) carries the
  selection; **(iii) background-active** — a non-active project's active tab / focused cell (open
  and live, not focused — today lit nowhere). The trailing ⊞ (U+229E) retires from the rail's Tab
  arm (app.rs:18740-18742) and CrossRef arm (:18841) — one indicator vocabulary, per the #418
  grammar; whether the `pane_marked` field survives as indicator INPUT is Phase 2's shape call.
  The per-row truth table over all states is a Phase 2 deliverable.
- **D4 — hover treatment unchanged.** The 0.10 accent wash (`rail_highlight`, app.rs:1425-1427)
  stays on every non-selected row exactly as shipped; only the 0.22 fill's ROUTING changes.
- **D5 — the F-#386 pin is part of the plan, not a hope.** Any `rail_rows`/`RailRow` shape change
  fans out to `#[cfg(test)]` call sites plain `cargo check` never compiles — 26 test callers today
  (tabs.rs:1990-3060) + 2 runtime callers (app.rs:18502, :18540). `cargo check --tests` is a
  REQUIRED exit step of P3 Implement and P4 Validate.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN any single click lands on any rail row, at any RailLevel, `rail_rows` shall emit at most one row flagged selected. | pure unit sweeping every RailLevel click target over multi-project / sectioned / split fixtures — count(selected) ≤ 1 asserted per case |
| REQ-002 | Project, Section, and Arrangement rows shall never be selected, and the #386 active-section force-expand shall hold unchanged after the header fill is removed. | structural-sweep unit (no fixture makes them selected) + the re-pinned #386 collapse/force-expand tests green |
| REQ-003 | WHEN content is mounted in a pane cell but not focused, its rail row shall carry the indicator, not the selection. | units over `pane_mounts` fixtures: pane-mounted editor file (CrossRef), foreign-hosted terminal tab |
| REQ-004 | WHEN the focused content is a cell of a multi-cell tab, the cell's row (Pane/CrossRef) shall be the selected row and the hosting Tab row shall carry the indicator. | unit (split fixture: selected lands on the cell row; the container Tab row indicates) |
| REQ-005 | WHILE a project is not the active project, none of its rows shall be selected and its active tab's row shall carry the indicator (background-active). | unit (two-project fixture) |
| REQ-006 | The six render arms shall route the selection fill from the selected flag ONLY and the dot from the indicator ONLY; the ⊞ marker shall no longer render on rail rows. | §18.1 inspect of all six arms (app.rs:18618/:18748/:18843/:18924/:18972/:19046) + app smoke |
| REQ-007 | WHEN Validate runs, the Marley rail and #418's re-baselined POC shall present the same selection/dot behavior — the React↔Marley parity pair captured. | React capture (localhost:5173) + Marley capture, side by side |
| REQ-008 | WHEN the REQ-001 click sweep replays against the built app, no frame shall render two selection fills nor a filled Project/Section header; and the reshaped crate shall compile under `cargo check --tests`. | negative smoke (drive clicks, assert the absence) + `cargo check --tests` exit 0 + gates green |

## Phase Plan
- **P2 Design** — verify #418 LANDED (design + MARLEY-PARITY rail re-baseline) and bind to its
  as-built grammar, not this spec's predictions; settle the `RailRow` shape (D1/D3: selected +
  indicator vs a three-state mark — pick the shape that makes two-selected unrepresentable or
  awkward); write the per-state selected/indicator truth table over all six levels; the test
  re-pin map over the 26 callers + the active-flag pins (tabs.rs:2000-2665); the render-arm
  manifest (fill re-route, dot element, ⊞ retirement, #386 force-expand preservation, Project
  text-emphasis per #418).
- **P3 Implement** — confirm the #418 baseline visually at localhost:5173 (this slice PORTS; new
  React work only if #418 left a gap); pure model first (`rail_rows` + shape + truth table), then
  the six arms; **`cargo check --tests` before phase exit (D5)**.
- **P3.5 Inspect** — independent critics vs the diff: the two-fills hunt (adversarial fixtures —
  cross-project splits, pane-only files, collapsed sections + selection); the #386 force-expand
  regression lens (5653c72 behavior); the dot truth table vs `pane_mounts` semantics (the #398
  inspect-corrected foreign-host rule); provenance (§20).
- **P4 Validate** — write + RUN the REQ units incl. the every-RailLevel sweep and the negative
  smoke; **`cargo check --tests` (D5)**; the re-pinned rail suite green; the React↔Marley parity
  pair (REQ-007); gates green (`--diff`).
- **P5 Complete** — CHANGELOG; ledger capture (§19); archive; close #419 + drop its BACKLOG row.
