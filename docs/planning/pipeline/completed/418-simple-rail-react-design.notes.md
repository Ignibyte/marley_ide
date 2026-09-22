# The simple rail, designed in React (+ the Zone A rail re-baseline) — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-418-simple-rail-react-design.md
- **Pipeline spec:** 418-simple-rail-react-design.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-08-12 (verbatim intent in docs/planning/design-notes/simple-rail-shelf.md):
  the current left pane was his own idea and he doesn't like it — move to a **simplistic,
  ChatGPT-style rail**. Fixed constraints: projects listed and addable; panes still visible;
  clicking a thing must NOT light its ancestors/relatives — **only the selected item is
  highlighted**; everything else "open somewhere" gets a small **left dot** instead. Visual
  reference he likes: beautifului.dev Sidebar Nav + Task Rows, captured same day to
  docs/warp_architecture/observed/ (beautifului-sidebar-nav-2026-08-12.png,
  beautifului-task-rows-2026-08-12.png, beautifului-2026-08-12-notes.md). This is the DESIGN
  ticket of the M31 batch (418 → 419 → 420 → 421, + #422 independent), pivoted to the top of
  BACKLOG ahead of TICKET-415 by his explicit "fixing these first".
- **Classification / tier:** feature — design ticket. **marley-web POC + docs only, NO Rust**
  (the ports are #419/#420/#421). Dev loop: `pnpm --filter @workspace/marley-ide run dev` →
  localhost:5173; verification is visual (screenshots read back) + DOM assertions + doc greps —
  Marley-side gates stay green trivially (docs-only diff).
- **Recall (§18.3):**
  - Shelf pins carried as DOWNSTREAM context (they gate the ports, not this ticket's code):
    **F-#236** — `collapsed_projects` index-aliasing on project close; anything touching project
    add/close must remap index-keyed view state → #421. **F-#386** — widening `rail_rows`'
    signature breaks ~14 `#[cfg(test)]` call sites plain `cargo check` never compiles; verify with
    `cargo check --tests` → #419/#420. Both must survive into the port specs via the D5 port-target
    rows.
  - **The 2026-07-30 method** — MARLEY-PARITY.md:136 "How to measure, not eyeball" + the
    attached_assets/live/ capture discipline (STEP1-ROADMAP.md:48): geometry is measured off live
    captures at a known viewport, never eyeballed off static shots. The P2 token/geometry sheet and
    the P4 capture set follow it — the sheet is what gives #419–#421 numbers.
  - **#398's rail inheritance** (pipeline/completed/398-add-to-pane-cross-link.spec.md): rail row
    DnD deferred to v2 (:76-78) — stays deferred here; the ⊞ cross-link marker
    (D-OPEN-CROSSLIST-MARKER) is the very glyph D2/D3 retire, while the cross-list ROWS it marks
    survive with the dot instead.
  - **"Editor items stopped showing under the project" is not a regression** — #237 (`abbdd3e`) +
    #240 (`b748fa8`) projected the editor to one rail row on purpose, pinned at tabs.rs:1464-1505.
    This ticket restyles the POC's per-file rows; #420 reverses only the Rust rail *projection*,
    never the one-editor-surface model.
- **Discovery (the precise surface for Design):**
  - **POC files to touch** (all under /Volumes/Offload/Projects/marley-web/artifacts/marley-ide/):
    - `src/components/LeftRail.tsx` — the redesign. Everything replaced or restyled: RAIL_INDENT
      6/10/20/30 (:24-29), RailFill (:54), `projectActive = true` (:166-169), the Workspace-caption
      ＋ (:183-185), SectionToggle (:477), SubItem (:535), the PaneMark ⊞ (:11-17, retired).
    - `src/index.css` — tokens kept: `--rail-active` (:95 light / :142 dark), `--sidebar`
      (:76/:122); added: the dot color token(s) + the small-caps header type treatment beside
      `.marley-caption` 11px (:206) / `.marley-nav` 12px (:211).
    - `src/App.tsx` — expected READ-ONLY: the one selected row derives from existing state
      (activeSection :61, openFiles/activeFile :70-71, terminalTabs/activeTerminalTab :104-105,
      panes/activePane :108-109, twoProjects :132); dot inputs derive from `panes[].items` (the
      #398 pane_mounts analog already computed in LeftRail.tsx:143-146). No new state fields
      expected; deviation here needs a written reason in Phase 3.
    - `src/components/ui/sidebar.tsx` — stays unused (Prior art leg 3: hand-rolled wins).
  - **Docs to touch:** marley-web/docs/MARLEY-PARITY.md (:26 Zone A rail row, Zone A preamble
    inversion note, :572 port-map row) + marley-web/docs/captures/ (new rail set, continuing the
    existing numbering). Marley-side: only this pipeline pair + shelf/BACKLOG bookkeeping at P5.
  - **Rust port targets (named for D5, NOT touched):** `rail_rows` tabs.rs:1032-1256 with six
    per-level actives (:1047/:1103/:1146/:1164/:1180/:1241); `rail_highlight` app.rs:1421-1423
    (accent 0.22 active / 0.10 hover); inline indent literals app.rs:18606 (8px) / :18696 (28px) /
    :18964 (40px) / :19039 (14px).
  - **Sweep deltas vs the shelf note:** `--sidebar` dark is :122 (shelf said :120); FleetDock's
    dock recipe comment sits at :10-12 with the chrome at :37-44 as cited. Everything else
    verified byte-for-byte at promotion time, per the shelf's own warning.
- **Decisions:** D1 single-selection invariant (one fill ever; headers never; project row only as
  its own selection); D2 dot = presence (pane-mounted + section's remembered-active-elsewhere; not
  agent-live, not dirty); D3 small-caps non-selectable headers, header/item/pane-child ladder, ⊞
  retired for the left dot; D4 add-project in the rail's top band; D5 three-point MARLEY-PARITY
  re-baseline (rail captures superseded, inversion note, #419–#421 port rows). Pixel-level calls
  (dot filled-vs-ring, chevron visibility, ＋ vs accent row, exact px) deliberately left to P2/P3
  in the POC — the model is locked, the pixels are design-time.
- **Promotion (2026-08-12, `/work 418-422` batch):** queued pair promoted to active; every
  load-bearing cite re-verified against the live tree at promotion time — LeftRail.tsx :11-17 ⊞ /
  :24-29 RAIL_INDENT / :54 RailFill / :143-146 pane derive / :166-169 projectActive / :183-185 ＋;
  index.css :95/:142 `--rail-active`, :206/:211 type roles; App.tsx :61/:70-71/:104-106/:108-109/:132;
  MARLEY-PARITY.md :26 rail row / :136 measure / :572 port-map / :650-656 rules; the three
  beautifului observed captures present. Zero drift. BACKLOG row removed (§19 promotion).

## Phase 2 — Design

### Approach + §20 confirm
React-only design pass (the Zone A rail re-baseline); no crates touched. **Reference (§20)
confirmed as speced:** the grammar source is the OBSERVED beautifului.dev capture set
(sidebar-nav + task-rows, 2026-08-12) — Marley-specific redesign, no reference-app source
consulted, clean-room untouched. Prior-art verdict re-confirmed at design: hand-rolled Tailwind
(the shadcn sidebar.tsx stays unused); no new dependency (the dot is a token-colored div, not a
glyph).

### Component decomposition (LeftRail.tsx, full redesign)
- **`LeftRail`** — the 220px column: `TopBand` → `QuickSearch` → one `ProjectBlock` per project.
- **`TopBand`** — "Workspace" small-caps caption + **persistent accent ＋** (D4: add-project,
  promoted from today's hover-only muted ＋; drives `twoProjects` until #421).
- **`QuickSearch`** — the same input restyled quieter (rounded, muted border, focus ring); NO
  keycap hint (no binding exists — spec constraint); #390/#399/#403 filter semantics untouched.
- **`ProjectBlock`** — a selectable project `RailRow` at header indent (label `Marley · main`,
  trailing hover-revealed × and collapse chevron) + its children when expanded: Manager Agent +
  Knowledge Brain as headerless item rows (DD-6), then the four `SectionHeader` groups.
- **`SectionHeader`** — small-caps tracked muted label (`.marley-rail-header`); NEVER fills;
  label click focuses the section's remembered-active child (empty section = no-op, #403);
  trailing affordances hover-revealed: ＋ (the #393 create menu) and collapse chevron — ▸ stays
  persistent while collapsed (collapse state must be visible), ▾ appears on hover (DD-2).
- **`RailRow`** — THE row primitive (every selectable row: project, agent rows, files, terminals,
  arrangements, pane cells, browser tabs): `indent: header|item|paneChild`, a fixed left **dot
  slot** on item/paneChild levels, `selected` (the `--rail-active` rounded fill, geometry
  unchanged per D1), optional prefix badge (agent chips), trailing hover-revealed actions (×).
- **`Dot`** — 5px circle in the slot: **filled** or **ring** (DD-1).
- **Retired:** `PaneMark` ⊞ (D3), `SectionToggle` (leading-chevron grammar), `projectActive`
  ancestry (D1) — the REQ-002 grep demands the NAME vanish entirely, comments included.

### Selection matrix (the REQ-001 exactly-one invariant, all derived)
Two mock-view fields join AppState (**written reason, per the Phase-1 "deviation needs a
reason":** both mock concepts the Rust side ALREADY HAS — the active project and per-cell pane
focus — so they are scaffolding for existing Marley state, never new product concepts; the POC
data-rule of MARLEY-PARITY.md:650 is not disturbed; merge-over-defaults App.tsx:199-208 covers
old persisted sessions):
- `activeProject: 1 | 2` (default 1) — project-2's mock row must be able to hold the selection
  so the REQ-005 sweep can prove cross-project single-fill.
- `paneCellFocus: number | null` (default null) — REQ-001 lists arrangement AND pane cell as
  distinct selectables; no existing field carries which was clicked. Cell click sets it; every
  non-pane selection resets it to null.

Fill rules (exactly one true by construction):
| Row | fills iff |
|---|---|
| project 1 | `activeProject===1 && activeSection==='home' && !activeAgentId` |
| Manager Agent | `activeProject===1 && activeAgentId==='manager'` |
| Knowledge Brain | `activeProject===1 && activeSection==='brain'` |
| file row | `activeProject===1 && activeSection==='editor' && activeFile===file` |
| terminal row | `activeProject===1 && activeSection==='terminal' && originalIndex===activeTerminalTab` |
| arrangement row | `activeProject===1 && activeSection==='panes' && activePane===id && paneCellFocus===null` |
| pane-cell row | `activeProject===1 && activeSection==='panes' && activePane===id && paneCellFocus===idx` |
| browser row | `activeProject===1 && activeSection==='browser' && activeBrowserTab===tab` |
| project 2 | `activeProject===2` |
Headers and file cross-ref rows never fill. Section-header click routes to the remembered child
(Editor→activeFile, Terminal→activeTerminalTab, Panes→activePane arrangement, Browser→
activeBrowserTab if open else first open; empty = no-op).

### Dot matrix (D2)
- **Filled dot = pane-mounted presence** (the #398 ⊞ relocated to the left edge): terminal home
  rows in `panedTerminals`; file cross-ref rows (they exist only because paned). File home rows
  keep no dot (unchanged #398 semantics — the cross-ref row carries the paned signal).
- **Ring dot = the section's remembered-active item while the selection lives elsewhere**: the
  row named by activeFile / activeTerminalTab / activePane / activeBrowserTab gets a ring iff it
  is not the filled row. Cells never ring (the arrangement row carries the section's memory).
- A row earning both (paned terminal that is also the remembered tab) renders **filled** —
  presence outranks memory, one dot per row ever.
- Agent-live chips and dirty state stay OFF the dot (D2 + Prior-art collision note).

### Geometry / token sheet (the numbers #419–#421 copy; final values re-measured at P4)
| Token | Value |
|---|---|
| Rail width | 220px (unchanged) |
| Fill | `--rail-active`, rounded 3px, 2px horizontal inset — unchanged (D1: WHO lights changed, not what lit looks like) |
| Indent ladder | header label x=8 · item dot-slot x=8→20 (12px), label x=20 · pane-child dot-slot x=20→32, label x=32 |
| Dot | 5px ∅, centered in slot (item center ≈14px, pane-child ≈26px); filled `bg --rail-dot`; ring 1px `--rail-dot` border, transparent center |
| `--rail-dot` | light `215 8% 52%` / dark `220 8% 62%` — starting values, tuned live at P3 |
| `.marley-rail-header` | 10px, uppercase, tracking 0.08em, weight 500, muted — recorded beside `.marley-caption`/`.marley-nav` |
| Header spacing | ~10px top margin per section group (grouping carried by space, not indent) |
| × close | hover-revealed (DD-5 — ChatGPT grammar; the re-baseline sanctions replacing the old persistent-×) |
| Empty states | italic muted at item label x (20px) |

### Settled design-time calls
- **DD-1 filled vs ring distinct** — they answer different questions ("on screen in a pane now"
  vs "where you'd land back"); beautifului's own task-rows distinguish filled/ring.
- **DD-2 header affordances trailing + hover-revealed; ▸ persistent when collapsed.** Labels
  never shift (no leading chevron slot).
- **DD-3 add-project = persistent accent ＋ in the top band** — an accent action ROW would
  overweight a rare verb; the ＋ is discoverable without dominating.
- **DD-4 quick-search restyle only** — no keycap, no behavior change.
- **DD-5 × hover-revealed.**
- **DD-6 Manager Agent / Knowledge Brain stay headerless item rows** under the project row —
  inventing an AGENTS header adds taxonomy the ticket didn't ask for.
- **DD-7 project-2 mock: same grammar, collapsed by default;** clicking its row selects it
  (`activeProject: 2`); its sections render as headers + empty states, inert otherwise.

### File manifest
React half (`/Volumes/Offload/Projects/marley-web/artifacts/marley-ide/src/`):
- `components/LeftRail.tsx` — full redesign per the decomposition above.
- `index.css` — `--rail-dot` light+dark; `.marley-rail-header` role.
- `App.tsx` — `activeProject` + `paneCellFocus` fields + defaults + the cell-click/reset
  semantics (~10 lines; everything else read-only).
Docs half (P4): `marley-web/docs/MARLEY-PARITY.md` (the three D5 points) + new captures
`marley-web/docs/captures/34+` (default / pane-cell+dots / tab search / two projects, both themes
where the REQs demand). Marley half: this pipeline pair only (+ BACKLOG/shelf bookkeeping at P5).

### Regression test plan (React+docs ticket — no Rust tests; Marley gates trivially green)
| REQ | Proof |
|---|---|
| REQ-001 | Playwright DOM sweep at localhost:5173: click every selectable (project, agents, file, terminal, arrangement, cell, browser, project-2), assert `document.querySelectorAll` finds EXACTLY ONE fill-class node in the rail per state; screenshots read back |
| REQ-002 | Same sweep asserts ancestor rows lack the fill class; `grep -n projectActive LeftRail.tsx` → empty |
| REQ-003 | Screenshot pair mounted/unmounted (close the pane cell → dot gone); DOM: dot node leads the row; `grep -c '⊞' LeftRail.tsx` → 0 |
| REQ-004 | Both-theme screenshots; DOM: header rows lack fill across the whole sweep; ＋ menu opened live |
| REQ-005 | twoProjects state: REQ-001 sweep re-run including project-2 row |
| REQ-006 | Filter query screenshot vs the #390/#399/#403 rules (sections keep shape; PANE-positional never matches); `pnpm --filter @workspace/marley-ide run typecheck` green |
| REQ-007 | P4 greps on MARLEY-PARITY.md (capture names, 419/420/421, inversion note); `ls docs/captures/` shows the new set |
| REQ-008 | `scripts/gates.sh --diff` exit 0 on the Marley-side docs diff; parity evidence trail in transcript |
Parity-pair row: **N/A by design** — this ticket IS the React side; the React↔Rust pairs are
#419–#421's validate rows. Uncoverable paths: none (all browser-verifiable).

### Risks / decisions
- **R1** — the two mock fields deviate from Phase-1's "no new state fields expected"; reason
  recorded above at design time, not sprung at P3.
- **R2** — ring-dot noise (max ~3 rings + N filled): tuned live; kill-switch = render both
  meanings with the filled treatment (semantics stay locked, treatment collapses) — one line.
- **R3** — old persisted sessions lack the new fields: covered automatically by the
  merge-over-defaults load (App.tsx:199-208).
- **R4** — REQ-002 greps for the NAME `projectActive`: the redesign must not mention it even in
  comments.
- No durable-lesson append: the one candidate ("mock fields must mock existing Rust concepts")
  is already the MARLEY-PARITY.md:650 data rule — nothing new to capture.

## Phase 3 — Implement

**Built (React-first — this ticket IS the React side; no Rust touched):**
- `src/components/LeftRail.tsx` — full redesign to the P2 decomposition: `RailRow` (the one
  selectable-row primitive, dot slot + hover-revealed trailing actions), `SectionHeader`
  (small-caps `.marley-rail-header`, never fills, label-click → remembered child via validated
  focus helpers, trailing hover ＋/chevron with ▸ persistent while collapsed), `Dot`
  (5px filled/ring), the flattened 3-level ladder (header 8 / item 8+12-slot=20 /
  paneChild 20+12=32). `PaneMark`, `SectionToggle` and the ancestry constant are GONE — the
  REQ-002/003 greps verify by name and both pass (0 hits). Cross-ref rows keep their #398 jump,
  now landing on the exact CELL (`jumpToCell` → `focusCell`). Filter semantics byte-compatible
  (#390/#399/#403 comments preserved; "no panes"-under-filter matches the old code's behavior).
- `src/index.css` — `--rail-dot` light `215 8% 46%` / dark `220 10% 62%` + `@theme` wiring;
  `.marley-rail-header` 10px/500/0.08em/uppercase beside the existing roles.
- `src/App.tsx` — the two P2-justified mock fields: `paneCellFocus: number | null`,
  `activeProject: 1 | 2` (+ defaults). Merge-over-defaults covers old sessions (verified live —
  the stored pre-#418 session loaded clean).

**Visually verified at localhost:5173, 1500×680 (screenshots READ back each state):**
dark default (one fill = Browser row, ring on remembered file/arrangement, filled on paned
terminal + cross-ref); terminal-selected (fill moves alone; paned row keeps filled dot);
pane-cell selected (CELL row fills at the deep indent, arrangement rings, ancestors quiet —
the split view opened coherently since `focusCell` syncs the content pointers);
two-projects (project-2 row fills alone with persistent ▸); search "mar" (sections keep shape,
positional PANE hidden, project rows + agent rows unfiltered — all pre-change semantics);
light theme (pale-teal fill, dots legible). DOM assertions each state: fillCount === 1
throughout; Editor-header click landed the fill on the remembered CHILD row.

**Deviations from design:** none. (The P2-anticipated fields were the only AppState change;
typecheck green; `pnpm --filter @workspace/marley-ide run typecheck` exit 0.)

**Not done here (P4):** the DOM click-sweep as a scripted assertion, the ＋ create-menu proof,
the capture set into `marley-web/docs/captures/` (34+), the MARLEY-PARITY.md D5 re-baseline.

## Phase 3.5 — Inspect

Three parallel critics (correctness / data-state integrity / simplification+provenance) over the
POC diff. Ledger:

| # | Finding | Verdict | Fix |
|---|---|---|---|
| 1 | Stale `activeAgentId` → TWO fills (Manager Agent → any content row; also ManagerAgentSetup's terminal buttons) | **REAL (high)** — reproduced | Selection derives from ONE `RailSelection` discriminated union mirroring the center router (agentId outranks section — one value can't name two rows); `select1()` helper owns the reset discipline for all 12 rail handlers; ManagerAgentSetup's two patches clear `activeAgentId` (also un-deads the pre-existing dead button) |
| 2 | `paneCellFocus` never reset by non-rail writers (palette splits, ⌘\/⌘⇧L, addToPane, arrangements apply) → wrong-cell or ZERO fills, persisted | **REAL (high)** | Writer resets added at all 4 sites + render clamp (out-of-range focus → null); selector degrades incoherent panes state to `home` |
| 3 | Palette "Toggle Two Projects" strands `activeProject: 2` → persistent zero-fill | **REAL (high)** | Functional toggle resets `activeProject` on hide + selector requires `twoProjects` for `project2` |
| 4 | Center split-surface wrote cell ORDINAL into `activePane` (rail reads pane id) → zero fills from any center cell click; center border desynced from rail | **REAL (high; pre-existing mismatch surfaced by #418** — old always-lit project masked it) | SplitTerminalView unified on the rail's model: `activePane`=pane id + `paneCellFocus`=idx + content-pointer sync; border reads `paneCellFocus` |
| 5 | Close-last-item in the active section → zero fills; browser-close teleported the selection even when Browser wasn't selected | **REAL (med)** | Close handlers route home when the ACTIVE section empties; browser retarget gated on `activeSection==='browser'`, falls to `home` when no terminals; selector fallback guarantees ≥1 regardless |
| 6 | Agent-detail states (non-manager `activeAgentId`) lit nothing | **REAL (med)** | `kind: 'agent'` → the project row is the selection home for unrepresented workspace states |
| 7 | 12 hand-copied `activeProject/paneCellFocus` patch fragments; 5 duplicated × blocks (one already drifted); 5 empty-state lines; 3 chevron blocks | **REAL (maintenance)** | `select1()` + `CloseX` + `EmptyLabel` + chevron folded into `RailRow` (`expanded`/`onToggleExpand`) |
| 8 | Project-1 × dead affordance (click bubbled to row-select) | **REAL (low)** | `stopPropagation` no-op + comment (#421 wires the real verb) |
| 9 | `rounded-[4px]` odd-one-out on the search input | **REAL (nit)** | → `rounded-[3px]` (house norm) |
| 10 | Terminal-close index shift moves selection to the wrong tab | **REJECTED for #418** — byte-identical pre-existing; #419/#420 resolve against Marley's `adjust_active` truth | documented |
| 11 | Panes empty-state lacks the `!q` guard Editor/Terminal carry | **REJECTED for #418** — pre-existing drift; REQ-006 forbids filter-behavior change here; #419/#420 check against the Rust rail | documented (EmptyLabel keeps byte-compatible guards) |
| 12 | With `activeProject===2`, center writers put project-1 content on screen while the fill stays on project-2 | **REJECTED (known mock limitation)** — project 2 is inert scaffolding; #421 wires real switching | documented |
| 13 | AgentDetail `openTmux` leaves `activeAgentId` set (center/rail desync) | **REJECTED for #418** — pre-existing, non-manager id yields ONE fill (terminal); recorded for a future POC pass | documented |

Provenance: clean (hand-rolled TSX, no copied blocks, shadcn sidebar stays unused, lucide-only
glyphs). Post-fix verification: typecheck green; scripted REQ-001 sweep in the live browser —
**11/11 selectable rows each yield exactly one fill on the clicked row**; the manager→file
double-fill repro now yields 1; visual state re-screenshotted and read (coherent).

## Phase 4 — Validate

React+docs ticket — verification is browser DOM assertions + captures + doc greps + the
Marley static gate (no Rust tests to write; the design's plan row per REQ, all run):

| REQ | Result |
|---|---|
| REQ-001 | **PASS** — scripted sweep at localhost:5173: dark 12/12 selectable rows (project, agents, files, xref, terminal, arrangement, cell, browser, project-2) each yield EXACTLY ONE `bg-rail-active` fill on the clicked row; light theme 11/11 (single-project state), 0 non-single-fill states |
| REQ-002 | **PASS** — 0 ancestor/header fills across both sweeps; `grep projectActive LeftRail.tsx` → 0 hits |
| REQ-003 | **PASS** — mounted state: 2 filled dots (paned terminal home + file cross-ref); cells emptied → 0 filled dots and the cross-ref row collapses (2 rows → 1); dot node leads the row (DOM-verified); `grep -c '⊞'` → 0 |
| REQ-004 | **PASS** — headers small-caps (10px/500/0.8px/uppercase, computed-style-verified) with 0 fill violations across every sweep state in BOTH themes; the Editor ＋ create-verb menu opened live (a11y tree shows menu: "Open File…", "Open in Split") |
| REQ-005 | **PASS** — twoProjects state included in the sweep; project-2 row fills alone; its expanded grammar (small-caps headers + empty states) captured in `37` |
| REQ-006 | **PASS** — "mar" filter: sections keep shape, only `Marley` matches, positional PANE hidden, project/agent rows unfiltered (pre-change semantics, screenshot-verified); `pnpm --filter @workspace/marley-ide run typecheck` exit 0 |
| REQ-007 | **PASS** — captures `34-rail-simple-default` / `35-cell-dots` / `36-tabsearch` / `37-two-projects` / `38-light` landed in marley-web/docs/captures/ (34/35/37 read back and verified state-correct); MARLEY-PARITY.md carries the inversion note ("Marley is the bug until #419–#421"), the re-pointed rail row, the port-target rows (#419/#420/#421 with the selector rule), and the measured geometry sheet |
| REQ-008 | **PASS** — `scripts/gates.sh --fast` on the Marley side: 11 passed, 0 failed, `GATE GREEN [fast]` (docs-only diff — no `.rs` in changeset, so the receipt rule doesn't bind this commit; the real nextest run is in the gate output) |

**Geometry sheet (live-measured, landed in MARLEY-PARITY.md):** labels x=10/22/34 (3-level
ladder + 2px fill inset), dot 5×5 @ x=13.5 (slot 12px), fill x=2 w=215 r=3px rgb(36,64,71)
dark, `--rail-dot` rgb(148,155,168) dark, header 10px/500/0.8px uppercase.

**Live Marley app + parity pair: N/A — explicitly.** This ticket ships ZERO Rust; Marley's
rail is untouched, so there is no Marley-side surface to drive or pair against. The ticket IS
the React-first design pass (spec `## React-first (parity)`); the React↔Rust parity pairs are
#419–#421's validate obligations against captures 34–38. POC-side live verification: the full
sweep + captures above, all screenshots READ back.

**Pre-existing, documented, not in scope:** terminal-close index shift (inspect #10); Panes
empty-state `!q` guard drift (#11); AgentDetail openTmux desync (#13) — all recorded in the
inspect ledger for the ports to resolve against Rust truth.

## Phase 5 — Complete

- **§21 docs:** CHANGELOG.md entry (Unreleased → Changed, the #418 design pass + inversion);
  `docs/marley_architecture/app_shell.md` gained the M31 #418 entry (the port contract for
  #419–#421); the shelf row ticked SHIPPED. Parity sync: the POC IS what shipped —
  MARLEY-PARITY.md re-baselined at P4 (inversion note + captures 34–38 + geometry sheet +
  port-map targets); nothing to back-port.
- **§19 knowledge appended:** F-claude-418-selection-invariant-only-lived-in-one-components-
  handlers-001 + PR-claude-single-selection-is-a-derived-selector-not-scattered-booleans-001
  (at inspect); L-claude-418-scripted-dom-sweep-is-the-poc-test-harness-001 +
  AD-claude-zone-a-inversion-mechanics-001 (at complete).
- **Ticket:** TICKET-418 → closed/ (status: closed). BACKLOG row left at promotion (swept: none
  stale). Pipeline pair → completed/.
- **marley-web side:** committed in its own repo (POC redesign + captures + parity re-baseline).
