# 390-panes-rail-section — Notes

## Phase 1 — Plan (drafted 2026-07-22, /spec batch, Fable)
- **Request:** chad's 4-section sketch (Workspace → Editor/Terminals/Panes/Browser; "panes should be
  anything that opens a split view and they dynamcally allocate there"). Unit locked via
  AskUserQuestion: **arrangement** (one row per split view). Sprint #38 M27; forge #390
  `b15dd53b-3df2-4f00-a2ae-515afed6eaa6`.
- **Ground truth (this session's shipped state):** the 3-section rail (#385 `29b4218`), interactive
  sections (#386 `5653c72`), per-section ＋ (#387 `7531d05`). `rail_rows` is the one pure row producer;
  `RailLevel::Pane` rows currently nest under their Terminal tab (#155) with a cross-project click
  (#174 idiom, app.rs ~16286-16330). `section_action` is TOTAL over `RailSection` — adding Panes
  FORCES an arm (locked: `SplitFocused` → `split_focused_pane`, reuse).
- **Key design freedom for P2:** how an arrangement row rides in `RailRow` (a new `RailLevel::
  Arrangement` vs reusing Tab-level with a marker) — pick in design; the numbering rule ("PANE n" =
  1-based per project in tab order, renumbers on close — derived, no stored ids) needs a stability
  note in the doc.
- **Deliberate train re-order (recorded):** #388's train put the registry before the Panes section;
  this ships the DISPLAY half early on (tab,pane) coordinates (no identity needed). #394 adds
  ContentId to these rows later.
- **Prior-art sweep:** in-repo owners dominate (rail_rows, #174 click, split_focused_pane, #386
  machinery); no external owner. Recorded in spec.

## Phase 1 — Promote to active (Opus, 2026-07-23)
Promoted queued → `active/` (first of `/work 390-394`, M27 sprint #38 `fc972431`; queue-chore
`06d73e6`). §3 re-confirmed — 390 is the sole active spec. **AAR opened:**
`d347c1e0-0b0b-477e-a373-f27ae2ccd41c`.

**Anchors RE-VERIFIED against the CURRENT tree (post-#387; design must use these):**
- `rail_rows` **tabs.rs:764** (the one pure producer); `RailLevel` enum **:619**; `RailRow` struct
  **:635** (fields `level`/`label`/`active`/`project` + `tab`/`pane`).
- **THE un-nest target — tabs.rs:832-843:** the #155 comment "a split terminal tab (>1 pane) shows its
  panes NESTED; the focused pane is active" + `let panes = grid.pane_ids(); … for (k, pid) in
  panes.iter().enumerate() { push RailLevel::Pane, label "pane {k+1}" }` UNDER the Tab row (:824). This
  loop MOVES to the Panes section (keyed to the arrangement row) in #390.
- `SectionAction` enum **tabs.rs:79**; `section_action` **:91** (3 arms — the Panes arm is added, the
  #387 tests at :1061 will need a 4th assert). `RailLevel::Pane` render arm **app.rs:16328**;
  `RailLevel::Tab` **:16186**; `RailLevel::Section` **~:16361** (#387). `dispatch_section_action`
  **app.rs:6653**; `split_focused_pane(axis)` **app.rs:6684** (the Panes＋ verb, `PaneAxis::Horizontal`).
- The `#[cfg(test)]` rail_rows tests (tabs.rs:~1441/1516) filter by `RailLevel` + assert
  `(level, label, project)` tuples — the design's new arrangement rows extend that matrix.
- **Reference §20 unchanged** (N/A — Marley-specific; chad's sectioned shell, no Warp/Zed analog). D1–D5
  + EARS REQ-001..006 stand as drafted.

## Phase 2 — Design (2026-07-23)

### Architecture / approach
Lands entirely in the gpui-free `tabs.rs` model (rail_rows + RailSection/RailLevel/section_action) plus
the `app.rs` render arms. Reference §20 = **N/A — Marley-specific** (confirmed — the 4-section rail is
chad's own; gpui hover/click is adoption; no copyleft source).

- **RailSection gains `Panes` (tabs.rs:37):** `enum RailSection { Editor, Terminal, Panes, Browser }`;
  `ALL: [RailSection; 4] = [Editor, Terminal, Panes, Browser]` (the rail order); `label` `Panes=>"Panes"`;
  `from_label` `"Panes"=>Some(Panes)`. **`rail_section()` (tabs.rs:78) is UNCHANGED — it never yields
  Panes** (a doc note: "Panes is a DERIVED section — no `TabContent` files under it; `rail_rows` lists
  split ARRANGEMENTS there"). Adding a variant is additive to the #386 `HashSet<(usize, RailSection)>`
  collapse set; old settings have no "Panes" key → restore unaffected (`from_label` tolerant).
- **THE CENTRAL DECISION — a new `RailLevel::Arrangement`** (Option A, chosen over reusing `Tab` with a
  marker): a distinct variant keeps the render arm, the #386 active/collapse logic, and the rail_rows test
  tuples (which filter by `RailLevel`) explicit and clean. `RailRow` needs NO new field — an Arrangement
  row is `{ level: Arrangement, label: "PANE n", active: tab_active, project: i, tab: Some(j), pane: None,
  collapsed: false }`.
- **rail_rows re-plumb (tabs.rs:764) — the un-nest + the derived Panes section.** Inside the
  `for section in RailSection::ALL` loop, after the `push Section row` / `if section_collapsed continue`:
  - **`if section == RailSection::Panes`** → a DERIVED block (not the `rail_section()` filter): a per-
    project `pane_n` counter starts at 0; `for (j, tab) in project.tabs().enumerate()` where
    `tab.grid().is_some_and(|g| g.pane_ids().len() > 1)` → `pane_n += 1`, push an **Arrangement** row
    `"PANE {pane_n}"` (`active = i==active_project && j==active_tab`), then its **nested `Pane` cell rows**
    (the block MOVED verbatim from tabs.rs:832-848 — `for (k, pid) in g.pane_ids().enumerate()` → `Pane`
    row `"pane {k+1}"`, `active = tab_active && *pid == focused`, `tab: Some(j)`, `pane: Some(k)`).
  - **`else`** (Editor/Terminal/Browser) → the existing `for (j, tab)` `if rail_section() != section`
    Tab-row loop **with the #155 nested-Pane block DELETED** (a split terminal now shows only its Tab row
    here; its cells live under Panes). Only Terminal tabs ever had a grid>1, so only they are affected.
- **Active-wash (tabs.rs:801):** `is_active_section = is_active_project && active_tab_section ==
  Some(section)`. Since `active_tab_section` is `Some(rail_section(active_tab))` ∈ {E,T,B}, it is NEVER
  `Some(Panes)` → the Panes header is **never the active section** (no header highlight) — CONFIRMED the
  #386 computation degrades to `false` for a section holding no active tab. The **Arrangement row** carries
  the row-level active highlight (`active: tab_active`) instead. Panes is also never force-expanded (only
  the active section is), so a collapsed Panes stays collapsed even when a split's tab is active — an
  acceptable asymmetry (the split's TAB is still visible+active under Terminal; noted for inspect).
- **section_action gains Panes (tabs.rs:79/91):** `SectionAction::SplitFocused` variant;
  `section_action(Panes) => SplitFocused`. `dispatch_section_action` (app.rs:6653) maps `SplitFocused =>
  self.split_focused_pane(PaneAxis::Horizontal)` (app.rs:6684, reuse-only). `section_action` stays total
  over the now-4 sections.
- **app.rs render (masked shim):** a **`RailLevel::Arrangement` arm** mirroring the Tab arm (app.rs:16186)
  — same #174 cross-project click (`project_changed` → `switch_project(p)` → sync-if-changed →
  `switch_tab(j)` → `persist_grid()` [the #387-F1 rule] → notify), Tab-level indent, `rail_highlight` when
  active, the "PANE n" label. The **`RailLevel::Pane` arm (app.rs:16328) is UNCHANGED** — it keys on
  `(project, tab, pane)` and focuses `pane_ids()[k]`, agnostic to whether it renders under a Tab or an
  Arrangement (the moved rows Just Work). The **Tab arm is UNCHANGED** (rail_rows simply stops emitting
  Pane children under it). The **Section arm (#387) is UNCHANGED** — the Panes ＋ dispatches
  `section_action(Panes)=SplitFocused` through the existing path.

### File manifest
| File | Change |
|------|--------|
| `crates/marley_app/src/tabs.rs` | `RailSection::Panes` (enum + `ALL[4]` + `label`/`from_label`); `rail_section()` doc note (unchanged code); `RailLevel::Arrangement`; rail_rows re-plumb (derived Panes block + the un-nest deletion); `SectionAction::SplitFocused` + `section_action` Panes arm; extend the #385/#386/#387 tests + new rail_rows tests. |
| `crates/marley_app/src/app.rs` | a `RailLevel::Arrangement` render arm (mirror the Tab arm + #174 click + persist); `dispatch_section_action` `SplitFocused` arm → `split_focused_pane(Horizontal)`. (Pane arm, Tab arm, Section arm all UNCHANGED.) |

### Regression Test Plan
| # | Test (`tabs.rs` unit unless noted) | REQ | Coverage |
|---|---|---|---|
| T1 | `RailSection::ALL == [Editor, Terminal, Panes, Browser]`; rail_rows emits exactly those 4 Section headers per project, in order. | REQ-001 | pure |
| T2 | A single-cell terminal project → the Panes section header renders but holds ZERO Arrangement rows. | REQ-002/006 | pure |
| T3 | A 2-cell terminal tab → under **Terminal**: the Tab row with NO nested Pane rows (the un-nest); under **Panes**: one Arrangement "PANE 1" + its 2 nested Pane cell rows. | REQ-002/003 | pure |
| T4 | Two split tabs → "PANE 1" and "PANE 2" (1-based, tab order); unsplit/close one → the survivor renumbers. | REQ-002 | pure |
| T5 | The origin (split) tab keeps its normal Tab row under Terminal while its arrangement lists under Panes. | REQ-004 | pure |
| T6 | `section_action(RailSection::Panes) == SectionAction::SplitFocused` (the #387 routing test gains this 4th assert). | REQ-006 | pure MSI 100 |
| T7 | Arrangement row carries `active: true` iff its tab is the active tab; the Panes Section header is never `active` (never the active section). | REQ-006 | pure |
| T8 | `from_label("Panes")==Some(Panes)`; a `collapsed_sections` containing `(i, Panes)` collapses the Panes header (its arrangement rows omitted); an old settings blob without "Panes" restores unaffected. | REQ-006 | pure |
| D1 | driven: 4 section headers render (capture). | REQ-001 | capture |
| D2 | driven: split the focused terminal → "PANE 1" appears under Panes and its cells no longer nest under the Terminal tab (capture). | REQ-002/003 | capture |

**Test-construction note:** a 2-cell `PaneGrid` in a unit needs a split — `PaneGrid::new(())` then
`.split_focused(axis, SplitDirection::After, || ())` (the `S=()` stub), or whatever the existing
split-grid tests use; validate confirms the exact constructor. **Uncoverable by unit:** the render arms +
clicks (masked shim, driven — attempt via an ISOLATED `HOME` instance; synthetic input may be env-blocked
→ units + the byte-identical #174/#155 mechanisms per the standing PR). chad's app (pid 35894) stays
untouched.

### Risks / decisions
- **D-ARRANGEMENT-LEVEL:** a new `RailLevel::Arrangement` (over reusing `Tab`+marker) — cleaner render/
  test/active-wash; `RailRow` unchanged.
- **D-PANES-NEVER-ACTIVE-SECTION:** `rail_section()` never yields Panes → the Panes header never
  highlights/force-expands; the arrangement row carries row-level active. A collapsed Panes stays
  collapsed even when a split's tab is active (the tab itself is still visible under Terminal) — inspect
  lens, accepted.
- **D-UNNEST-IS-A-MOVE:** the #155 nested-Pane block MOVES from the Tab loop to the Panes block; the Pane
  RENDER arm is untouched (keys on project/tab/pane). Zero behavior change to a cell click.
- **D-NUMBERING-DERIVED:** "PANE n" = a per-render 1-based counter over multi-cell tabs in tab order;
  renumbers on close (no stored identity — #394 adds ContentId to these rows later).
- **D-PANES＋=SPLIT (reuse-only):** `section_action(Panes)=SplitFocused` → `split_focused_pane(Horizontal)`;
  behaves exactly as ⌘D / context-menu Split would when the row's project has no splittable focused
  terminal (not #390's concern — reuse-only).
- **D-EXHAUSTIVE-MATCHES:** adding the `Panes` + `SplitFocused` + `Arrangement` variants will surface any
  non-exhaustive `match` at compile — implement runs `cargo check --workspace --tests` to catch every
  consumer (the #386 `cargo check` skips-cfg-test lesson).

### Test-construction confirmed + existing tests to UPDATE
- Split constructor (proven in-repo): `let mut grid = PaneGrid::new(()); let _ =
  grid.split_focused(PaneAxis::Horizontal, SplitDirection::After, || …);` (tabs.rs:1585/1702).
- **Existing tests the un-nest changes (NOT a spec change — the rows move, the count is preserved):**
  `rail_rows_nested_panes` (tabs.rs:1700 — asserts `panes.len()==2`; the 2 Pane rows now sit under the
  Panes-section Arrangement, not under the Tab) and the "tab precedes its nested panes" assert
  (tabs.rs:~1585-1611 — the ordering changes: the panes now follow the Arrangement row in the Panes
  section, after the Terminal section). Validate rewrites these to assert the NEW placement (still 2 Pane
  rows; now under an `Arrangement`).

## Phase 3 — Implement (2026-07-23)
Built to the manifest; `cargo check --workspace` AND `--workspace --tests` both green, `cargo fmt` clean.
No non-exhaustive `match` broke anywhere in the app code (the variant additions compiled cleanly).

**tabs.rs:** `RailSection::Panes` (variant + `ALL[4]` + `label`/`from_label`); `rail_section()` doc note
(code unchanged); `RailLevel::Arrangement` (RailRow unchanged); rail_rows re-plumb — a
`if section == RailSection::Panes { … continue }` DERIVED block (per-project `pane_n`; `let Some(grid) =
tab.grid() else { continue }` + `panes.len() <= 1` guard → `Arrangement` row + the MOVED #155 cell rows)
before the Tab loop, and the Tab loop's #155 nested-pane block DELETED (the un-nest); `SectionAction::
SplitFocused` + `section_action(Panes)=>SplitFocused`. Panic-free throughout (`let-else`, no bare index).

**app.rs:** `dispatch_section_action` `SplitFocused => split_focused_pane(PaneAxis::Horizontal)` (persists
internally); a `RailLevel::Arrangement` render arm mirroring the Tab arm's #174 click (switch_project +
sync-if-changed + switch_tab + persist + notify), `pl(28)`, `rail_highlight` when active. Pane + Tab +
Section render arms UNCHANGED.

**Compile-fix (needed to compile — not a test expansion):** `rail_section_order_and_labels` asserted
`ALL == [_; 3]` → updated to the 4-element order + the "Panes" label assert.

**RUNTIME failures for VALIDATE to rewrite (4 — the un-nest / 4-section restructure, NOT a spec
regression):** `rail_rows_one_project`, `rail_rows_groups_tabs_by_section_in_fixed_order`,
`rail_rows_empty_sections_still_render_headers`, `rail_rows_collapsed_project_has_no_section_headers` (all
pin the OLD 3-section count/order or the under-Tab pane placement). `rail_rows_nested_panes` still PASSES
(counts Pane rows by level, agnostic to parent). No deviations from the design.

## Inspect (Phase 3.5 — 2026-07-23)
**Method:** 2 independent critics over the diff (A: rail_rows correctness via full hand-traces; B:
state-integrity + collapse round-trip + gpui idiom) PLUS my own reads of `split_focused_pane`/
`workspace_mut`/`grid.focused`/the header-push order.

**Cross-confirmed CORRECT (both critics + my checks — no fix):**
- rail_rows correctness (critic A hand-traced `[Term(2), Editor, Term(1), Term(3)]`): PANE 1/PANE 2
  numbering in tab order; the un-nest complete (split tabs emit ONLY their Tab row under Terminal, cells
  ONLY under the Panes Arrangement — no double-list, no orphan); `active`/`tab_active` correct; empty
  Panes still renders its header (pushed before the derived block's `continue`); panic-free (`let-else`,
  `grid.focused()` returns a field — total). The Arrangement click is byte-identical to the Tab arm's #174
  path. `match row.level` has NO catch-all (the new variant is handled everywhere); tab_labels/active-count
  filters gate on `==Tab` (Arrangement/Pane don't leak in).
- **D-PANES-NEVER-ACTIVE-SECTION sound** (critic B): `rail_section()` never yields Panes → the Panes header
  is never the active section; a collapsed Panes stays collapsed but NEVER hides a tab (the split's Tab row
  lives under Terminal, force-expanded there) → not stranded (the header always renders + is clickable to
  un-collapse). The #386 collapse round-trip holds for `(i, Panes)` (from_label/label round-trip; back-compat
  safe — old blobs lack "Panes"; remap is section-agnostic). stop_propagation on the Panes ＋ intact.
- **Panes＋ = ⌘D** (both): `split_focused_pane(Horizontal)` — `workspace_mut()` resolves the active tab's
  grid else the FIRST terminal's grid via `terminal_grid_index().expect(≥1 terminal)`; the invariant is
  actively enforced (restore force-seeds a terminal into any terminal-less project, app.rs:2097-2115; the
  close guard refuses LastTerminal) → NO new panic surface in #390.

**Findings:**
| # | sev | finding | verdict | fix |
|---|-----|---------|---------|-----|
| F1 | MED | The new `RailLevel::Arrangement` render arm lacked the "Search tabs" filter guard that the sibling `Pane` arm has (`if !self.session_filter.is_empty() { continue; }`) — during a tab-search the Panes section rendered dangling "PANE n" headers with no (hidden) cells beneath. A cosmetic regression from the un-nest. | REAL — found INDEPENDENTLY by BOTH critics; mechanism confirmed (Pane arm has the guard, Arrangement didn't) | Added the identical guard at the top of the Arrangement arm → a filter leaves only the Panes header (consistent with the E/T/B headers persisting when their lists filter empty). |
| F2 | LOW | Panes＋ on a project whose active tab is an editor/cockpit splits the (off-screen) first terminal — the only visible effect is a new "PANE n" row, not a visible split. | REJECTED as a #390 issue — byte-identical to pressing ⌘D there (reuse-only, D-PANES＋=SPLIT); documented, not a defect. | none (noted). FORWARD: `split_focused_pane`'s `workspace_mut().expect(≥1 terminal)` is a **#391-audit site** — Panes＋ would panic on a zero-terminal project post-#392, which #391's totality pass must make safe. |

**Provenance:** all research in-repo (rail_rows + gpui adoption); §20 wall intact (no Warp/Zed source). F1
is a real bug → failure-record + a prevention rule (a relocated/new render arm must mirror the sibling
row's render-time guards).

## Phase 4 — Validate (2026-07-23)

### Tests — 57 tabs tests pass (54 → 57)
- **Rewritten to the 4-section + un-nest reality** (NOT a spec change — rows moved / a header was added):
  `rail_section_order_and_labels` (ALL[4] + "Panes" label), `rail_rows_one_project` (6→7 rows, sections
  +Panes), `rail_rows_groups_tabs_by_section_in_fixed_order` (the seq gains the Panes header + a
  `!sec("Panes").active` assert), `rail_rows_empty_sections_still_render_headers` (+Panes),
  `rail_rows_collapsed_project_has_no_section_headers` (3→4 headers). `rail_rows_split_panes_nest_under_
  terminal_section` → **rewritten** as `rail_rows_split_panes_unnest_into_panes_section` (asserts the
  cells are all after the Panes Arrangement, never under Terminal — a non-vacuous `all` to dodge the
  empty-slice coverage trap PR-claude-empty-iterator-closure-uncovered-001). `rail_rows_nested_panes`
  comment updated (its cell-level asserts still hold under the new placement).
- **New (design T4/T6/T7/T8):** `section_action_routing` +Panes⇒SplitFocused assert;
  `rail_rows_panes_numbering_and_origin_tabs` (two split tabs → PANE 1/PANE 2 in tab order, a single-cell
  tab skipped, all origin Tab rows stay under Terminal, 4 cells all after the Panes header);
  `rail_rows_arrangement_active_but_panes_header_never`; `rail_rows_panes_collapse_round_trips`
  (`(i,Panes)` key round-trips + old-blob-without-Panes back-compat).

### Mutation — real list traced, gate:5 MSI 100
`cargo mutants --list -f tabs.rs` for the changed seams: `section_action` body→Default UNVIABLE (no Default
derive) — arm coverage via the 4-assert routing test; the derived Panes block's viable mutants — `from_label`
"Panes" arm-delete (killed by the ALL round-trip loop + T8), `<=`→`>` (the split predicate — killed by the
un-nest test + the single-pane sub-case), `pane_n +=`→`-=`/`*=` (killed by T4's PANE 1/PANE 2 numbering —
`-=` underflow-panics, `*=` gives "PANE 0"), the arrangement/cell `&&`/`==` active-flag mutants (killed by
T7 + nested_panes) — all killed.

### Gate — GATE GREEN [diff], 15/15 (the .rs commit receipt is written)
coverage **100% lines**, mutation **MSI 100%** on the changed seams, + rustfmt/clippy/nextest+doctests/
audit/deny/machete/gitleaks/shellcheck/no-suppressions/source-bans/docs/miri/visual — all green.

### Driven live-app verification — SUCCEEDED via a pre-seeded restore (no synthetic input needed)
Synthetic input is env-blocked (proven #387), so instead of driving clicks I **pre-seeded an isolated
`HOME`** with a `workspace.shell` blob = `0\n<root>\t0\tT=H:t,t` (a single untitled split-terminal tab,
H:t,t = a 2-terminal horizontal split), launched `HOME=<iso> target/debug/marley`, and the app **restored
that session** — rendering the feature live. The capture (`scratchpad/390-panes-rail.png`, READ) confirms
**all of REQ-001..004 visually**:
- the rail's **4 section headers** render in order — Editor · Terminal · **Panes** · Browser (REQ-001);
- the split lists as a **"PANE 1"** Arrangement row **under the Panes section** (REQ-002);
- its **"pane 1"/"pane 2" cells nest under PANE 1 in Panes** — and the Terminal tab has **no** nested
  panes beneath it (REQ-003, the un-nest);
- the split's **Tab row stays under Terminal** while dual-listing under Panes (REQ-004, model A);
- active-wash correct (the Terminal tab + PANE 1 + the focused cell all washed); the center shows the
  real 2-pane horizontal split.
chad's running app (pid 35894) was **untouched** (I launched a SECOND, isolated instance, captured its
own window 5469, and killed only my pid); the iso scratch dir was cleaned.

### Pre-existing failures
None in scope (the `block v0.1.6` future-incompat is a pre-existing upstream-dep warning).
