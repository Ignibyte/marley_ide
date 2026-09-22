# Rail single-selection highlight + dot indicators (#419) — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-419-rail-single-selection-dots.md
- **Pipeline spec:** 419-rail-single-selection-dots.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** the M31 "simple rail" shelf, slice 2 of 418→419→420→421 (+ #422)
  (docs/planning/design-notes/simple-rail-shelf.md — Chad's call 2026-08-12, order confirmed "as
  listed"): port #418's selection model to the Rust rail. Kill ancestor lighting — today one click
  lights up to six rows with the identical accent fill — and replace it with exactly ONE selected
  row (the focused content itself) plus small left-edge dot indicators on rows whose content is
  open/mounted elsewhere. The shelf was authored on `main` @ a23dd0d; this Plan RE-verified every
  cited line against today's tree (2026-08-12) — drift + corrections recorded under Discovery.
- **Classification / tier:** feature, M31, standard work pipeline, one shippable slice (pure model
  reshape + six render arms + test re-pin + parity pair). **HARD-ordered after #418** — the design
  ticket must land the new POC rail AND the MARLEY-PARITY rail re-baseline before this port has a
  source of truth (/work must not promote #419 while #418 is unshipped). Independent of #420/#421.
  React-first policy (2026-08-04) applies: UI-AFFECTING, Zone A (rail).
- **Recall (§18.3):** local ledger (grep docs/planning/knowledge/) + completed-pipeline archive:
  - **F-#386** (failures.md:400): at #386 implement, `cargo check -p marley` reported clean while
    the crate did NOT compile under `--tests` — widening `rail_rows` by one param + one struct
    field broke 14 existing test sites (11 `rail_rows` callers + 3 `AppliedSettings` literals)
    that plain `cargo check` never compiles. The fan-out is BIGGER today: **26 `rail_rows`
    `#[cfg(test)]` call sites** (tabs.rs:1990-3060) + 2 runtime callers (app.rs:18502, :18540).
    Locked as **D5**: `cargo check --tests` is a required exit step of P3 AND P4.
  - **F-#236** (failures.md:410) + the index-remap prevention rule (prevention-rules.md:1183):
    index-keyed view state (`collapsed_projects: HashSet<usize>`) aliased a live project on close
    because the write side never remapped. Binding shape here: the new selection/indicator must
    stay **derived per render from workspace focus** — no stored selected-index anywhere — which
    `rail_rows` already does for `active`; D1 pins that property through the reshape.
  - **#386 history, 5653c72** ("interactive rail sections — highlight, collapse-persist,
    force-expand-active", M26): the active-section flag drives TWO things — the header FILL (dies
    with this ticket) and the force-expand gate (`section_collapsed = contains && !is_active_section`,
    tabs.rs:1103-1107 — MUST survive, or the active tab can be collapse-hidden, the #305 auto-reveal
    invariant). Locked into D2's second clause + REQ-002 + a P3.5 lens.
  - **#398 archive** (completed/398-add-to-pane-cross-link.spec.md + notes): the `pane_mounts`
    pre-pass semantics were inspect-CORRECTED there (a tab row is the home entry for ALL its
    cells; the terminal cross-link signal is foreign hosting ≥2 distinct tabs, not any-mount) —
    D3's state (i) adopts those signals unchanged; only the presentation moves from ⊞/fill to dot.
- **Discovery (the precise edit/file surface for Design — every line re-verified 2026-08-12):**
  - `crates/marley_app/src/tabs.rs`
    - `RailLevel` :757-781 — six variants: Project, Section, Tab, Arrangement, Pane, CrossRef.
    - `RailRow` :787-813 — `active` :793 (the field D1 renames/splits), `collapsed` :802,
      `pane_marked` :807 (the ⊞ input D3 absorbs), `content` :812.
    - `rail_rows` :1032-1254 — the per-level independent `active` computations: Project :1047
      (`i == active_project`); Section :1103 (`is_active_project && active_tab_section ==
      Some(section)`) → :1111; Arrangement :1146 (`tab_active`, computed :1139); Pane :1164
      (`tab_active && *pid == focused`); Tab :1180 (`tab_active`) → :1213; CrossRef :1241
      (`focused_cell_coord == Some((*t, *k))`). Worst case six rows true at once (Project +
      Section + Tab + Arrangement + Pane [+ CrossRef when the focused cell is a CodeView]).
    - `pane_mounts` :989-1024 + `PaneMounts` :978-987 — `editor: Vec<(cid, basename, p, t, k)>`
      (first mounting cell per file) + `terminal_hosts: HashMap<cid, distinct-hosting-tabs>`
      (≥2 ⇒ today's ⊞) — D3 state (i)'s unchanged signal source. Foreign-host read at :1194-1201.
    - Tests: mod :1279; the 26 `rail_rows` callers :1990-3060; the active-flag pins to re-pin sit
      at :2000, :2013-2016, :2115-2118 (section active), :2263-2267 (cross-project), :2347-2349
      (exactly-one-pane-active — the closest existing single-selection pin), :2386, :2508-2514
      (arrangement/Panes header), :2665 (cross-ref active).
    - **Correction to the shelf note:** its "pinned rail tests at tabs.rs:1464-1505" is actually
      `open_or_switch_code_cases` (#237/#240 — #420's pin, no `rail_rows` call, no active
      assertions). The re-pin surface for THIS ticket is the :2000-2665 set above. Verified by
      reading the range + grepping the callers.
  - `crates/marley_app/src/app.rs`
    - `rail_highlight` :1425-1427 — `accent.opacity(0.22 active / 0.10 hover)`; D4 freezes the
      hover arm. (The shelf cited :1421-1423 — drifted by 4; doc comment now :1422-1424.)
    - Runtime callers: :18502 (the per-frame row list, registry labels) and :18540 (the wheel
      scroll clamp, `ContentLabels::default()` — label-independent count).
    - The six `entry = if row.active { bg(rail_highlight(true)) … } else { hover }` arms —
      Project :18618-18627 (also flips text foreground/muted), Tab :18748-18755, CrossRef
      :18843-18850, Arrangement :18924-18931, Pane :18972-18979, Section :19046-19053 (also the
      caption emphasis). ⊞ renders at :18740-18742 (Tab, `pane_marked`-gated) and :18841
      (CrossRef, unconditional) — both retire under D3.
  - `marley-web` (the React side, for the P3 visual-verify + the port map)
    - `artifacts/marley-ide/src/components/LeftRail.tsx` — `projectActive = true` :169 with the
      comment (:166-168) naming the ancestry model as deliberate ("Marley lights the whole path")
      — #418 replaces it; `RailFill { active, indent }` :54 (the fill primitive whose #418
      replacement this port mirrors into the `rail_highlight` routing); `PaneMark` ⊞ :11-14, used
      :286 (cross-list) / :318 (paned terminal) — absorbed by the dot under D3.
    - `docs/MARLEY-PARITY.md` — port-map row :572 (`components/LeftRail.tsx` ↔ app.rs (rail) /
      tabs.rs / context_menu.rs); the rail capture row :26 (shots 02/27/32/35) is what #418
      re-baselines; this pipeline consumes the NEW baseline.
- **Prior-art sweep (§20), three legs — run BEFORE locking D1-D5:**
  - **Behavior maps:** Warp — nothing; docs/warp_architecture/subsystems/ carries zero sidebar/rail
    material (grep-verified), so N/A-Marley-specific stands. Zed —
    docs/zed_architecture/subsystems/07-workspace-panes-palette.md maps one active item per pane
    (`active_item_index` :132), one active panel per dock (:204), "highlight the active workspace"
    (:238): the one-lit-thing invariant at behavior level, never ancestor lighting. Research docs
    only; no Warp/Zed source.
  - **Published material:** the WAI-ARIA single-select tree/listbox grammar — `aria-selected` on
    exactly one node, selection distinct from expansion — is D1+D2 in standards form. Observed
    convention (VS Code explorer, ChatGPT sidebar): one selected row; "open elsewhere" is a
    decoration (dot/badge), never a second fill; ancestors don't light.
  - **Permissive deps:** **none — checked gpui, the seam is our own.** gpui 0.2.2
    (~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-0.2.2) has no `Selectable`, no
    selected state in styled.rs/interactive.rs; elements/{list,uniform_list}.rs are
    virtualization-only. Nothing to adopt (reading it was sanctioned adoption); the invariant
    lands in our pure `rail_rows`. Sweep changed the draft in one place: it CONFIRMED the pure-fn
    placement (D1) rather than any element-level affordance.
- **Decisions:** **D1** single-selection invariant in the pure model (`rail_rows` emits ≤1
  selected; `active` → `selected` + `indicator` or equivalent, exact shape Phase 2; derived per
  render, no stored index — the F-#236 class); **D2** Project/Section/Arrangement never selected;
  #386 force-expand survives the fill's death; **D3** the left-edge dot replaces duplicate
  lighting AND absorbs the rail-row ⊞ — states: (i) mounted-elsewhere (`pane_mounts` signals
  unchanged), (ii) container-of-focus (split Tab row dots while its focused cell's row selects),
  (iii) background-active (non-active project's active tab); ⊞ retires from the Tab/CrossRef arms;
  **D4** hover 0.10 wash unchanged, only the 0.22 fill's routing moves; **D5** F-#386 —
  `cargo check --tests` required at P3 + P4 (26 test callers fan out invisibly). D-OPENs to
  Phase 2 with the spec's framing: the exact `RailRow` shape (make two-selected unrepresentable),
  the per-state truth table over all six levels, the dot's visual spec + active-project text
  emphasis (both bound to #418's as-built design), the `pane_marked` field's fate as indicator
  input, and the re-pin map over the :2000-2665 assertions.
- **Promotion (2026-08-12, the `/work 418-422` batch):** #418 SHIPPED first (Marley `2d244f6` +
  the marley-web twin) — the ordering precondition holds: the POC rail is the new grammar, and
  MARLEY-PARITY.md carries the inversion note, captures 34–38, the geometry/token sheet, and this
  ticket's port-target row. Rust cites re-verified at promotion (tabs.rs :793/:989/:1032/:1047/
  :1103-1107/:1111/:1146/:1164/:1180/:1213/:1241; app.rs :1425-1427/:18502/:18540/the six
  `row.active` arms/⊞ ×2; 27 `rail_rows` occurrences = def + the 26 test callers): **zero drift**.
  The spec's LeftRail.tsx cites (projectActive :169, RailFill :54, PaneMark :11-14) now describe
  the SUPERSEDED pre-#418 POC — correct as written ("whose #418 replacements this port mirrors");
  the as-built source is the shipped LeftRail (RailSelection selector, `Dot` filled/ring,
  `select1`) + the parity sheet. **P2 must bind to the as-built #418 grammar** — notably: the
  as-built dot vocabulary is FILLED (pane-mounted) vs RING (remembered-active/container-of-focus/
  background-active family), and the selection derives from ONE selector value
  (PR-claude-single-selection-is-a-derived-selector-not-scattered-booleans-001, appended at #418
  inspect — binding on this port). BACKLOG row removed (§19).

## Phase 2 — Design

### Approach + §20 confirm
Reference (§20) confirmed as speced: **N/A — Marley-specific selection model**; the design source
is #418's AS-BUILT POC (shipped: RailSelection selector / `Dot` filled-ring / the parity sheet) +
the observed beautifului captures. No Warp/Zed source; gpui owns nothing on this seam (re-confirmed
at Plan). This design BINDS to the as-built grammar per the promotion note.

### The shape (D1 settled): a selection COORDINATE, then per-row flags
The PR-…-derived-selector-001 rule ports structurally. `rail_rows` computes ONE value up front:

```rust
enum RailSelection {            // internal to rail_rows (not on RailRow)
    None,                       // empty workspace — nothing selects ("at most one" includes zero)
    Tab  { project, tab },      // the active project's active tab (single-cell / no grid)
    Cell { project, tab, cell } // the focused cell of the active multi-cell tab
}
```

derived from `active_project_index` + `active_tab_index` + the existing `active_focused_cell`
logic (hoisted; grid present && panes.len() > 1 → Cell, else Tab; no tabs → None). Rows compare
coordinates — unique per row, and Tab/Cell are exclusive enum variants, so **two selected rows are
unrepresentable by construction**. `RailRow.active` → **`selected: bool`**; `pane_marked` is
**REMOVED** (absorbed); new field **`dot: RailDot { None, Filled, Ring }`**. Selection stays
derived per render — no stored index (F-#236 class).

### The truth table (the D3 deliverable; the POC-as-built vocabulary: Filled=mounted, Ring=context)
| Row state | selected | dot |
|---|---|---|
| Project (any) | never | None |
| Section (any) | never | None — `is_active_section` SURVIVES as an internal for the #386 force-expand gate (D2) |
| Tab: active proj + active tab, single-cell | **YES** | Filled iff foreign-hosted (≥2 hosting tabs, unchanged #398 signal) else None |
| Tab: active proj + active tab, multi-cell | no | Filled iff foreign-hosted else **Ring** (container-of-focus, D3 ii) |
| Tab: active proj, non-active tab | no | Filled iff foreign-hosted else None |
| Tab: NON-active proj, its active tab | no | Filled iff foreign-hosted else **Ring** (background-active, D3 iii / REQ-005) |
| Tab: non-active proj, other tabs | no | Filled iff foreign-hosted else None |
| Arrangement hosting the selection (`selection == Cell{i, its tab, _}`) | never (D2) | **Ring** (container-of-focus — POC capture 35) |
| Arrangement otherwise | never | None |
| Pane: `selection == Cell{i,j,k}` | **YES** | None (cells never dot — POC) |
| Pane otherwise | no | None |
| CrossRef (any) | never (the POC cross-ref never selects; clicking it jumps → the Pane row selects) | **Filled** always (it exists because mounted — the ⊞ absorbed) |
Dot precedence on Tab rows: **Filled outranks Ring** (one dot per row — POC rule). REQ-004's
"(Pane/CrossRef)" resolves to the **Pane row** selecting for an editor-cell focus; the CrossRef
carries the filled dot (its old focused-active state dies with the multi-light).

### DD-419-COLLAPSE (flagged for inspect)
A collapsed section SKIPS its rows (`continue` at tabs.rs:1119), so a selection whose host section
is collapsed (e.g. Panes collapsed while a terminal cell of a split is focused — the active section
is Terminal, only IT force-expands) is a legal **zero-VISIBLE-selection** state: the model still
emits ≤1 selected (zero here), the container Tab row's ring still shows where focus lives, and
collapse is the user's own gesture (VS Code behaves identically; the POC as-built does too —
collapsing the section unmounts the filled row). #386 force-expand is PRESERVED EXACTLY, not
extended (D2/REQ-002 pin "unchanged").

### Render-arm manifest (app.rs; rail_highlight itself untouched — D4)
| Arm | Change |
|---|---|
| Project :18601-18637 | fill + foreground-flip branch DIES (the #236 active-project background retires under the #418 grammar — no substitute emphasis, D1/D2): always muted text + hover wash. No slot. |
| Tab :18691-18755 | `pl(28)` → `pl(16)` + a **12px leading dot slot** (before the agent glyph; label stays at x=28); dot renders Filled/Ring from `row.dot` (5px circle: `.size(px(5.)).rounded_full()` + `.bg(colors.muted)` filled / `.border_1().border_color(colors.muted)` ring); the ⊞ block :18740-18743 DIES; fill routes from `row.selected`. |
| CrossRef :18830-18860 | same slot treatment (pl 28 → 16 + slot; dot always Filled); the unconditional ⊞ child DIES; fill routes from `row.selected` (constant-false by the model — honest routing, the model tests own the constant). |
| Arrangement :18913-18931 | same slot treatment (pl 28 → 16 + slot; Ring when hosting); fill routes from `row.selected`. |
| Pane :18962-18985 | rename only (`row.active` → `row.selected`); NO slot (cells never dot); pl 40 untouched. |
| Section :19033-19053 | fill + emphasis branch DIES: always muted + hover wash. Type/chevron/＋ untouched (#419 ports selection/dots only; the small-caps header type is not this slice). |
Dot ink: `colors.muted` (the ⊞'s ink — same signal, new form); geometry converges with the POC
sheet at the P4 parity pair; if the pair reads wrong, tune then (R2). Indent ladder deliberately
NOT ported here (labels stay byte-positioned via slot-inside-existing-padding) — the ladder is
#420+'s surface.

### File manifest
- `crates/marley_app/src/tabs.rs` — `RailDot` enum; `RailRow` reshape (selected / dot; drop
  pane_marked; doc comments :783-812 updated); `rail_rows` hoists the selection coordinate +
  emits per the truth table; `pane_mounts` UNCHANGED; the 26 `#[cfg(test)]` callers get the
  mechanical field-rename compile fixes at P3 (assertion re-pins land at P4 — §7 split).
- `crates/marley_app/src/app.rs` — the six arms per the manifest; `rail_highlight` untouched;
  the two runtime callers untouched (signature identical).
- `marley-web` — expected ZERO changes (#418 landed the design); P3 re-verifies the POC baseline
  live at :5173 before porting (React-first step), any gap found becomes a written deviation.

### Test re-pin map (P4 writes; P3 only compiles)
| Existing pin | Re-pin |
|---|---|
| :2000 `rows[0].active` (active project) | Project rows NEVER selected (structural sweep) |
| :2013-2016 project/tab actives | active tab's Tab row selected (single-cell fixture) |
| :2115-2118 section actives | Sections never selected + #386 force-expand unchanged (REQ-002) |
| :2263-2267 cross-project actives | REQ-005: non-active project — zero selected rows in its block; its active tab Rings |
| :2347-2349 exactly-one-pane-active | the ≤1 sweep ancestor → `selected` (REQ-001's direct re-pin) |
| :2386 / :2508-2514 / :2665 | arrangement-never-selected + Ring-when-hosting; CrossRef never-selected + always-Filled |
NEW (one per REQ): REQ-001 the every-RailLevel click-target sweep over multi-project/sectioned/
split fixtures asserting `count(selected) ≤ 1`; REQ-002 structural sweep + force-expand; REQ-003
mount fixtures (CrossRef Filled; foreign-hosted Tab Filled; Filled-outranks-Ring); REQ-004 split
fixture (Pane selected, Tab Rings, Arrangement Rings); REQ-005 two-project fixture; REQ-006 is
§18.1 inspect + the app smoke; REQ-007 parity pair; REQ-008 negative smoke + `cargo check --tests`.
DD-COLLAPSE unit: Panes collapsed + cell focused → zero selected emitted, Tab row still Rings.

### Risks
- **R1** two rows per editor cell (Pane + CrossRef) — the table pins exactly one selects; sweep
  fixture includes the #246 pane-only editor file.
- **R2** dot ink `colors.muted` may not read like the POC's `--rail-dot` — settle at the P4 pair
  (a dedicated theme color is the fallback, added only if the pair demands it).
- **R3** DD-419-COLLAPSE zero-visible state — inspect lens.
- **R4** the F-#386 fan-out — `cargo check --tests` REQUIRED at P3 exit and P4 (D5).
- **R5** the active-project/active-section emphasis removal is a visible delta beyond the fill —
  locked by the #418 grammar (D1 "no substitute ancestry signal"), called out for Chad's eyeball
  at the parity pair.

## Phase 3 — Implement

- **React-first step:** the as-built #418 POC re-verified live at localhost:5173 (screenshot READ:
  terminal selected + filled/ring dots exactly per the truth table's source grammar). ZERO React
  changes needed — #418 left no gap.
- **tabs.rs:** `RailDot { None, Filled, Ring }` + the internal `RailSelection { None, Tab, Cell }`
  coordinate (computed once; every branch fixture-reachable — zero-project workspaces exist in the
  suite, tabs.rs:1381); `RailRow.active` → `selected` (docs rewritten), `pane_marked` REMOVED,
  `dot: RailDot` added; the six literals emit per the truth table (Project/Section/Arrangement/
  CrossRef constant-unselected; Tab = coordinate match vs `Tab{i,j}`; Pane = match vs `Cell{i,j,k}`;
  dots: Filled foreign-hosted/cross-ref, Ring container-of-focus + background-active, Filled
  outranks Ring); `focused_cell_coord` deleted (its consumer died); `is_active_section` +
  force-expand PRESERVED byte-identically; `pane_mounts` untouched.
- **app.rs:** `rail_dot_slot(dot, colors)` helper beside `rail_highlight` (12px slot, 5px circle,
  `colors.muted` ink — the ⊞'s ink); all six arms route the fill from `row.selected` uniformly
  (Project keeps no substitute emphasis — the #236 active-project background retired, comment
  updated); slot added at Tab/CrossRef/Arrangement with pl 28 → 16 (labels stay at x=28); the two
  ⊞ render sites died; Pane/Section arms are pure renames; `rail_highlight` + both runtime callers
  untouched.
- **D5 (F-#386):** `cargo check --workspace` green; `cargo check --workspace --tests` surfaced
  exactly the predicted fan-out — 30 E0609s in tabs.rs tests — patched mechanically on the
  erroring lines only (`.active` → `.selected`; `.pane_marked` → `(.dot == RailDot::Filled)`, the
  boolean-equivalent mapping for the legacy semantics) → green. `cargo fmt` clean. NOTE for P4:
  the mechanical rename keeps OLD assertions compiling; the ones asserting ancestor lighting
  (project/section/arrangement/cross-ref actives) now FAIL at runtime by design — the P4 re-pin
  map rewrites them; the suite is expected RED between P3 and P4.
- **Deviations from design:** none.

## Phase 3.5 — Inspect

Three critics (two-fills hunt / arms+regression / simplification+provenance). Ledger:

| # | Finding | Verdict | Fix |
|---|---|---|---|
| 1 | **1-cell CodeView-grid active tab**: the #398 Editor override force-expanded Editor while the SELECTED Tab row sat in a still-collapsible Terminal section — collapse hides the selection with NO ring anywhere (the pre-#419 breadcrumb was the CrossRef's active fill, which died) | **REAL (med)** — the one genuine model bug | `focused_cell_is_editor` gained the **multi-cell gate** (`.filter(pane_ids().len() > 1)`): a 1-cell grid's force-expand protection stays on its OWN Tab row's section (#305 restored); multi-cell keeps the Editor breadcrumb rationale. Comment documents the load-bearing gate |
| 2 | `active_focused_cell` carried a dead tuple element + a duplicate focused-position scan (the deleted consumer's residue) | **REAL (low)** | dissolved into the gated `focused_cell_is_editor` chain (the F1 fix) |
| 3 | duplicated container-of-focus `matches!` at 2 sites, pattern-shadowing loop vars | **REAL (low)** | `RailSelection::is_cell_in(p, t)` method; both sites use it |
| 4 | stale docs: `rail_rows` fn contract (old active semantics), `RailLevel::CrossRef` + `pane_mounts`/`terminal_hosts` ⊞ vocabulary, app.rs Project #156 foreground note, Section-arm "accent-wash highlight" claim, CrossRef-arm "Editor/Terminal ⊞" note | **REAL (doc)** | all seven sites rewritten to the selected/dot vocabulary with #419 tombstones |
| 5 | **P4 re-pin hazard**: `rail_rows_two_projects`' cross-project asserts are now vacuously green, and the old `&&`-mutant kill moved to the UNCOVERED background-active Ring arm (tabs.rs `is_cell_in`-OR-background arm) | **REAL — deferred to P4 by design** | P4 checklist: re-pin y → `dot == Ring`, x → `dot == None`; cover the Ring arm explicitly; reword banners :2575/:2836/:2949/:3045; clean the mechanical `(x.dot == Filled)` forms (clippy nonminimal_bool) |
| 6 | 6 tests red (`:2095/:2210/:2359/:2604/:2760/:3131`) — exactly the declared ancestor-lighting pins; 1015 others pass incl. every mount/dot/collapse pin; #386 force-expand verified LIVE (the :2759 collapse assert passes before the dead fill pin fails) | **expected P3→P4 state** | P4 re-pins |
| 7 | provenance | **clean** — in-house refactor of Marley's own #398 model; no Warp/Zed |
Verified clean by the critics (concrete traces): two-selected unrepresentable (one coordinate, unique
per-row equality, exclusive variants, four constant-false levels); 1-cell grids select their Tab row;
empty/zero-tab/collapsed degradations all ≤1; background-active uses the per-project active tab (no
cross-project aliasing); Filled-outranks-Ring single-dot; slot geometry (dot first child, labels
byte-positioned at 28, Pane 40 / Project 8 / Section 14 untouched); gpui `items_center` semantics
center the dot both ways; `pane_mounts` + #386 gate byte-identical; no stale field readers
(compile-impossible); `colors.muted` ΔL≈0.48 vs the sidebar in both themes. Post-fix:
`cargo check --workspace --tests` green, fmt clean.

## Phase 4 — Validate

**Tests written + RUN (the re-pin map executed + the new #419 suite):**
- Re-pins: `rail_rows_one_project` (project never selects + exactly-one), the sections test
  (headers never select + the ACTIVE TAB row carries it — fixture's real active tab is idx 3, the
  #403 browser tab; the old "cock idx 2" comment was stale), `rail_rows_two_projects` (the REQ-005
  rewrite: background-active Ring on the non-active project's active tab, None on its sibling and
  on the selected row — kills the `!=`/`&&` mutants at the Ring arm, closing inspect #5's vacuous-
  assert hazard), the arrangement test (never selects; Ring when hosting; the selection is the
  focused CELL row), force-expand (collapse assert survives; header never selects; the active tab's
  row selected), focused-codeview (header/cross-ref never select; xref Filled; the CELL row is the
  one selection; Terminal-focus arm re-pinned too).
- New: `rail_rows_at_most_one_selected_everywhere` (REQ-001 sweep over the adversarial two-project/
  all-sections/split/pane-only workspace + the REQ-002 structural sweep),
  `rail_rows_container_of_focus_rings` (REQ-004), `rail_rows_filled_outranks_ring_on_foreign_
  hosted_container` (REQ-003 precedence over a real 2-tab twin fixture),
  `rail_rows_one_cell_codeview_grid_protects_its_own_section` (the inspect-F1 regression pin —
  kills the `>1`→`>=1` gate mutant), `rail_rows_collapsed_host_section_hides_selection_but_keeps_
  ring` (DD-COLLAPSE), `rail_rows_empty_states_select_nothing` (zero-project + zero-tab edges).
- **Run:** `cargo nextest run --workspace` → **2131/2131 passed** (5 skipped — the documented
  stubs); `cargo test --workspace --doc` clean; `cargo check --workspace --tests` green (D5).
  Two of my own test drafts were fixed during the run (stale idx-2 target; a cross-ref assert
  that ran while Editor was collapsed — rows are SKIPPED with their section, so the assert moved
  to an expanded-state call).

**Live app driven (the self-test harness; direct-exec after the documented TCC stall):**
boot (restored workspace, collapsed project → ZERO fills — legal and correct), project expanded →
**exactly one fill on the active Editor tab row** (project row + headers quiet — the ancestor
lighting is dead in the pixels), new terminal via the top-bar ＋ → the fill MOVED alone to the
terminal row (Editor section re-collapsed per its stored key, #386 correct), ⌘⇧L split → **the
truth table live**: focused CELL row filled (the one selection), its Tab row RING, PANE 1
arrangement RING, everything else quiet. Captures read at every step (/tmp/marley-419-*.png).
No frame ever showed two fills (REQ-008's negative smoke, driven).

**Parity pair (REQ-007):** POC driven to the same split-cell-focused state
(scratchpad/419-poc-pair-cell.png) vs the live capture (/tmp/marley-419-split.png). Verdict:
**behavior 1:1** — one fill on the focused cell row, arrangement Ring, headers/project quiet,
left-edge 5px dots in the muted ink at the slot position (pixel-sampled: Marley ring ink
rgb(106-134,…) = the muted family; POC dot rgb(148,155,168); Marley fill rgb(32,56,61) vs POC
rgb(36,64,71) — the PRE-EXISTING opaque-token vs accent-wash-compositing pair, unchanged by
design D1/D4). **One recorded semantic difference, Marley-authoritative:** the POC shows a
FILLED dot on the self-split terminal's home row (its flat mock cannot distinguish self-hosting
from foreign-hosting); Marley Rings it (the #398 inspect-corrected semantics, locked unchanged by
D3(i)) — a known POC mock-data limitation (MARLEY-PARITY.md "the POC never ports its data"), not
a Rust defect and not a POC code change.

**Geometry note (scoped):** the dot slot sits inside Marley's EXISTING indent ladder (labels
byte-positioned; dot center ≈ x22) — the #418 sheet's flattened ladder is deliberately NOT this
slice (the spec scopes selection+dots only).

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff], 15 passed, 0 failed** — static set +
coverage (≥100% lines) + mutation on the touched lines (MSI ≥ 100 — the new selection/dot branches
all killed) + miri + visual/AX. Receipt written for /commit. Pre-existing exclusions: none new;
the `block v0.1.6` future-incompat note is the documented upstream dependency state.

## Phase 5 — Complete

- **§21 docs:** CHANGELOG.md entry (the #419 model + arms + the force-expand correction);
  `docs/marley_architecture/app_shell.md` gained the M31 #419 as-shipped entry above the #418 one.
- **Parity sync:** MARLEY-PARITY.md port-map row ticked "#419 ✅ LANDED" with the recorded POC
  mock-data limitation (self-split Filled-vs-Ring — Marley-authoritative); no POC code change
  (its flat mock cannot express self-vs-foreign hosting; the grammar itself matches 1:1).
- **§19 knowledge:** F-claude-419-force-expand-protected-a-breadcrumb-not-the-selection-001 +
  PR-claude-invariant-rewrites-must-reaudit-redundancy-consumers-001 (at inspect);
  L-claude-419-port-the-selector-not-the-booleans-proved-out-001 (at complete). No new AD (the
  selector decision is #418's PR rule, proved not re-decided).
- **Ticket:** TICKET-419 → closed/. Shelf row ticked SHIPPED. BACKLOG clean (row left at
  promotion). Pipeline pair → completed/.
