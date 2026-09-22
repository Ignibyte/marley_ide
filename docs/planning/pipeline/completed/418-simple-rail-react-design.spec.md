---
pipeline_id: a42b63b8-1b81-4690-9031-931685c44467
ticket: docs/planning/tickets/open/TICKET-418-simple-rail-react-design.md
status: Phase 5 — Complete PASS
title: The simple rail, designed in React (+ the Zone A rail re-baseline)
type: feature
milestone: M31
references:
  - docs/planning/design-notes/simple-rail-shelf.md
  - docs/warp_architecture/observed/beautifului-2026-08-12-notes.md
  - docs/warp_architecture/observed/beautifului-sidebar-nav-2026-08-12.png
  - docs/warp_architecture/observed/beautifului-task-rows-2026-08-12.png
  - marley-web/docs/MARLEY-PARITY.md
  - marley-web/artifacts/marley-ide/src/components/LeftRail.tsx
  - marley-web/artifacts/marley-ide/src/index.css
  - marley-web/artifacts/marley-ide/src/App.tsx
  - crates/marley_app/src/tabs.rs
  - crates/marley_app/src/app.rs
  - docs/planning/pipeline/completed/398-add-to-pane-cross-link.spec.md
---

## Title
The M31 rail redo's DESIGN ticket (Chad's call, 2026-08-12: the current left pane was his own idea
and he doesn't like it). Replace the ancestry-lit, four-level rail with a **simplistic ChatGPT-style
rail**, designed and visually settled in the React POC BEFORE any Rust. Fixed constraints, verbatim:
projects listed and addable; panes still visible; **ONLY the selected item highlighted** — the
parent/ancestor lighting removed entirely; a **small circle/dot at the left of rows** indicating
open terminals/files instead of multi-highlight. Target grammar = the captured beautifului.dev
Sidebar Nav + Task Rows (grammar, not skin — palette/type/geometry stay Marley's own).

Why a whole ticket: the multi-highlight is structural on both sides. The POC encodes ancestry
lighting on purpose (`projectActive = true`, LeftRail.tsx:166-169); Marley's `rail_rows`
(tabs.rs:1032-1256) computes `active` per level independently — Project :1047, Section :1103,
Arrangement :1146, Pane :1164, Tab :1180, CrossRef :1241 — so one click lights up to SIX rows
through the one `rail_highlight` fill (app.rs:1421-1423, accent 0.22/0.10). And MARLEY-PARITY.md
freezes the rail as Zone A "Marley is right" (:26), so the contract itself must be re-baselined:
the POC becomes the rail's design source; new captures supersede the old rail shots. **marley-web +
docs only — NO Rust.** The ports land as #419 (single selection + dots), #420 (per-file editor
rows), #421 (add-project).

## Scope
### In
- **The new LeftRail built + visually verified in the POC** (artifacts/marley-ide, `pnpm --filter
  @workspace/marley-ide run dev` → localhost:5173): the single-selection fill (D1); the left-edge
  dot indicator grammar (D2); small-caps section headers + the flattened row ladder with pane
  arrangements/cells still listed (D3); per-file editor rows (already present — restyled into the
  new grammar; the Rust side catches up at #420); the add-project affordance (D4, drives the
  existing `twoProjects` mock, App.tsx:132); the quick-search treatment of the existing Search-tabs
  box — **behavior unchanged** (the #390/#399/#403 filter semantics), treatment redesigned
  (constraint: no keycap hint for a binding that doesn't exist).
- **The MARLEY-PARITY.md rail re-baseline** (D5): the POC becomes the rail's design source; a new
  capture set under marley-web/docs/captures/ (continuing the existing numbering); the Zone A rail
  row re-pointed; port-target rows naming #419/#420/#421.
- **The token/geometry sheet** — indent ladder, dot size/inset/color, header type treatment, fill
  geometry — recorded (P2 designs it, P4 lands it in the parity doc) so the Rust ports have numbers
  instead of eyeballs (the MARLEY-PARITY.md:136 "measure, not eyeball" discipline).

### Out (explicitly deferred)
- **Any Rust change** — #419/#420/#421 are the ports. The targets are named for D5 only: the six
  per-level actives in `rail_rows` (tabs.rs:1047/:1103/:1146/:1164/:1180/:1241), `rail_highlight`
  (app.rs:1421-1423), the inline indent literals (app.rs:18606 8px / :18696 28px / :18964 40px /
  :19039 14px).
- **FileDock / Files panel redesign** — the Zone A Files row (MARLEY-PARITY.md:27) stays frozen.
- **Rail row drag-and-drop** — stays the recorded v2 (398-add-to-pane-cross-link.spec.md:76-78).
- **Workspace switcher dropdown behavior** — visual placement only; the switcher's open/select
  mechanics are not this ticket.

## Reference (§20)
**Marley-specific redesign — the behavior reference is an OBSERVED capture, not a reference app.**
The target grammar is the captured beautifului.dev Sidebar Nav + Task Rows:
docs/warp_architecture/observed/beautifului-sidebar-nav-2026-08-12.png,
beautifului-task-rows-2026-08-12.png + beautifului-2026-08-12-notes.md (captured live via
Playwright, 2026-08-12 — saved per the §20 capture discipline). What Marley takes is recorded in the
notes file: one selected row ever; the left dot/circle as the presence indicator; small-caps quiet
section headers; flat hierarchy feel; add-verb as an accent ＋. **Grammar, not skin** — palette,
type roles (`.marley-nav` 12px index.css:211 / `.marley-caption` 11px :206), `--rail-active`
(:95/:142) and the RailFill rounded-fill geometry stay Marley's own. No Warp/Zed analog needed:
Warp has no project-navigator behavior mapped at all, and Zed's project panel is a file tree, not
this rail. Clean-room untouched — no Warp (AGPL) / Zed (GPL) source consulted.

### Prior art
1. **Behavior maps — checked.** Warp: **no sidebar/project-navigator behavior is mapped anywhere**
   in docs/warp_architecture/subsystems/ — 01-ui-framework-rendering.md:65 carries the left dock
   only as one region of the 3-region cockpit; docs/warp_architecture/observed/ holds nothing
   rail-shaped beyond this ticket's own beautifului captures (cited in §20 above). Zed:
   docs/zed_architecture/subsystems/07-workspace-panes-palette.md §1.7 (:218-239) maps
   `MultiWorkspace` + `sidebar` — the workspace/project **switcher rail** (many workspaces per
   window, `NextProject`/`PreviousProject`, `ToggleWorkspaceSidebar`) — the behavioral analog for
   "projects listed and addable" (D4's context; the map's "study MultiWorkspace" pointer is #421's
   Rust-side business, not this ticket's); the :270 comparison row already records Marley
   MATCH/AHEAD on hierarchy shape.
2. **Published material.** ChatGPT's sidebar grammar (behavior described from general knowledge,
   no source consulted): one flat scrolling list under a fixed top band (identity/switcher, search,
   accent new-item affordance), **exactly one selected row** with a subtle rounded fill,
   hover-revealed row actions, quiet group labels, no disclosure tree, no ancestor lighting — the
   grammar Chad named. VS Code's published Open Editors grammar: per-file rows under a small-caps
   header, one active row highlighted — and its **leading dot means DIRTY (unsaved)**. Recorded
   deliberately: Marley's dot means open-elsewhere presence, NOT dirtiness (D2 owns the collision —
   dirty state stays on the editor surface, never the rail dot).
3. **Our permissive deps — thin by design (React-only ticket), checked anyway.** The POC already
   ships a shadcn/ui sidebar substrate (src/components/ui/sidebar.tsx — stock scaffolding,
   **unused**: no import anywhere in src). Verdict: **stay hand-rolled** — the shell is hand-rolled
   Tailwind by house style, and the shadcn Sidebar brings a foreign grammar (SidebarProvider,
   Sheet-on-mobile, its own CSS vars) with no gpui port target — adopting it would make the
   #419-#421 1:1 ports harder, not easier. No new dependency: lucide glyphs already ship
   (LeftRail.tsx:2), and the dot needs no glyph at all — a token-colored circle div here, a quad in
   the Rust port (the vendored-icon discipline of Icon.tsx:38-66 untouched).

## React-first (parity)
**UI-AFFECTING: Zone A (rail) — this ticket IS itself the React-first design pass + the Zone A
re-baseline.** Build & visually verify in marley-web (`pnpm --filter @workspace/marley-ide run dev`
→ localhost:5173) — dark AND light themes; the Rust ports land as **#419–#421** (port-map row
MARLEY-PARITY.md:572: `components/LeftRail.tsx` ↔ app.rs rail / tabs.rs / context_menu.rs). There
is no "then port 1:1" step inside this ticket — the port is the next three tickets; this ticket's
deliverables are the settled POC rail, the new capture set, and the re-baselined contract the ports
will be held to. POC discipline unchanged: the POC never ports its data (MARLEY-PARITY.md:650-656);
selection derives from EXISTING state (activeSection App.tsx:61, openFiles/activeFile :70-71,
terminalTabs :104-106, panes/activePane :108-109) — no new state fields expected.

## Locked-In Decisions
- **D1 — the single-selection invariant.** At most ONE rail row carries the selection fill, ever —
  the row whose content the center surface presents. Ancestry lighting dies wholesale: selecting a
  child never lights project/section/arrangement ancestors (`projectActive = true`,
  LeftRail.tsx:166-169, is removed, and no substitute ancestry signal — bold, accent text —
  replaces it). Section headers NEVER carry the fill (D3); a project row carries it only when the
  overview itself is the selection. The fill stays Marley's own `--rail-active` rounded RailFill
  (LeftRail.tsx:54) — this redesign changes WHO lights, not what lit looks like.
- **D2 — dot semantics: presence, not activity, not dirtiness.** The left-edge dot marks a row
  whose content is open/mounted somewhere OTHER than the current selection: (a) **mounted in any
  pane cell** — the #398 cross-link semantic, relocated from the trailing ⊞ to the left edge; and
  (b) **a section's remembered-active item while the selection lives elsewhere** — the residual
  "where you'd land back" signal the ancestry fill used to carry. Agent-live does NOT earn the dot
  (the existing `getAgentBadge` chips keep that meaning — one indicator, one meaning); dirty does
  NOT (the VS Code collision recorded in Prior art). Whether (a) and (b) get distinct treatments
  (filled vs ring) + exact size/inset/color: settled at design time in the POC.
- **D3 — hierarchy presentation.** Sections stay, as **small-caps tracked headers** (a deliberate
  departure from the sentence-case dock-caption role — the new type treatment is recorded beside
  `.marley-caption`/`.marley-nav` in index.css). Headers are labels + verb homes — the #393 ＋
  create menus and collapse stay reachable — but never selection carriers: a header click focuses
  the section's remembered-active child (the fill renders on that CHILD row; empty section = no-op,
  the #403 discipline). The four-level indent ladder (RAIL_INDENT 6/10/20/30, LeftRail.tsx:24-29)
  flattens to header / item / pane-child — pane arrangements stay visible with their cell rows one
  step deeper (the single second level; "panes still visible" is a fixed constraint). The trailing
  **⊞ glyph retires everywhere**, replaced by the left dot — the #398 cross-list ROWS survive; only
  the marker moves. Exact px: settled at design time, recorded in the P2 geometry sheet.
- **D4 — add-project lives in the rail's top band.** The affordance sits in the workspace header
  band above the sections (today's Workspace-caption ＋, LeftRail.tsx:183-185, possibly promoted to
  an accent affordance per the captured accent-action row) — persistent and discoverable, never
  buried in a context menu, never bottom-of-rail. It drives the existing `twoProjects` mock; #421
  wires the real picker/new-workspace verbs (F-#236's index-remap hazard is #421's to carry). Exact
  rendering (＋ alone vs accent row): settled at design time in the POC.
- **D5 — parity re-baseline mechanics.** MARLEY-PARITY.md changes in exactly three places:
  (1) the Zone A rail row (:26) — its reference cell (shots `02` / `27` hidden / `32` tab search /
  `35` two projects) is superseded **for rail rows only** by a new capture set under
  marley-web/docs/captures/ (continuing the existing numbering; one capture per acceptance state:
  default, pane-cell selection + dots, tab search, two projects); those attached_assets shots stay
  ground truth for every OTHER Zone A row that cites them. (2) A rail-inversion note in the Zone A
  preamble: for the rail, as of #418 the POC is the design source and **Marley is the bug** until
  #419–#421 land — the one sanctioned exception to "Marley is right". (3) The port-map row (:572)
  gains the port targets: #419 → the six `rail_rows` actives + `rail_highlight`; #420 → the
  tabs.rs rail projection (per-file rows; the #237/#240 projection reversed, the one-surface model
  kept); #421 → the add-project verb wiring. Exact wording lands at P4; the mechanics are locked.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN any content row (file, terminal, browser tab, arrangement, pane cell, or the project overview) is clicked in the POC rail, exactly ONE rail row shall carry the `--rail-active` selection fill — the clicked row itself. | visual inspection at localhost:5173 (screenshot READ back per state) + DOM assertion: the rail contains exactly one element with the fill class across a full click sweep |
| REQ-002 | WHEN a pane cell row is clicked in the POC, its arrangement row, section header, and project ancestor rows shall show NO selection fill and no substitute ancestry highlight. | screenshot + DOM class assertion on each ancestor row (fill class absent); `grep -n 'projectActive' LeftRail.tsx` returns nothing |
| REQ-003 | WHILE a file or terminal is mounted in a pane cell, its home-section row shall carry the left-edge dot; WHEN the last mounting cell closes, the dot shall disappear; the trailing ⊞ shall no longer render anywhere in the rail. | screenshot pair (mounted / unmounted) + DOM assertion (dot node leads the row); `grep -c '⊞' src/components/LeftRail.tsx` = 0 |
| REQ-004 | WHEN the rail renders, the section headers (Editor / Terminal / Panes / Browser) shall present as small-caps muted labels that carry no selection fill under ANY selection state, WHILE the #393 ＋ create menus and section collapse remain reachable. | screenshot (both themes) + DOM (header rows lack the fill class across the REQ-001 sweep); ＋ menu opened in the browser |
| REQ-005 | WHEN the add-project affordance is invoked, a second project shall appear listed with the same row grammar, and WHEN content under either project is selected, only that one row shall light. | screenshot of the two-project rail + the REQ-001 DOM assertion re-run in the `twoProjects` state |
| REQ-006 | WHEN a query is typed in the quick-search box, content rows shall filter exactly as today (#390/#399/#403 semantics: sections keep their shape; positional PANE labels never match) under the new visual treatment. | screenshot with a query + spot-check vs the pre-change filter rules; `pnpm --filter @workspace/marley-ide run typecheck` green |
| REQ-007 | WHEN Phase 4 completes, MARLEY-PARITY.md shall carry the D5 re-baseline: the rail row pointing at the new captures (files present in marley-web/docs/captures/), the rail-inversion note, and port-map rows naming #419/#420/#421. | grep assertions on MARLEY-PARITY.md (capture names, `419`/`420`/`421`, the inversion note) + `ls docs/captures/` shows the new set |
| REQ-008 | WHEN the Marley-side diff lands (docs only: spec/notes/ticket/shelf pointers), `scripts/gates.sh --diff` shall stay green and the enforce-react-parity.sh evidence trail shall record the POC verification. | gate exit code 0 + the parity hook's evidence trail |

## Phase Plan
- **P2 Design** — the component decomposition (top band / quick-search / project block / section
  groups / one row primitive with a left-dot slot) + the **token/geometry sheet** (indent ladder,
  dot size/inset/color, header type treatment, fill geometry — the numbers #419–#421 will copy);
  settle every "settled at design time" call (dot filled-vs-ring, header chevron visibility,
  add-project rendering, quick-search treatment); per-REQ verification plan.
- **P3 Implement** — build it in the POC: LeftRail.tsx redesigned, index.css tokens/roles added,
  App.tsx untouched unless a derive seam demands it; iterate at localhost:5173 until the grammar
  reads right in dark AND light.
- **P3.5 Inspect** — visual critic pass vs the locked constraints (the REQ-001 single-fill sweep,
  dot semantics, header treatment, filter states, two projects, both themes) + code critic on the
  POC diff.
- **P4 Validate** — capture the new set into marley-web/docs/captures/ (screenshots READ back, the
  MARLEY-PARITY.md:136 measure-don't-eyeball discipline); the DOM assertion sweep; land the D5
  re-baseline rows; typecheck green; Marley-side `scripts/gates.sh` green (docs-only diff — trivially).
- **P5 Complete** — shelf row ticked, BACKLOG row dropped, ledger capture (§19), ticket closed,
  pipeline archived; #419 promotion note (the geometry sheet is its input).
