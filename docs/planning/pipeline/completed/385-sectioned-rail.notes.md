# 385-sectioned-rail — Notes

## Phase 1 — Plan (drafted 2026-07-22, /spec batch, Fable)
- **Request:** chad (2026-07-22): fixed-order type sections in the rail — Editor top, Terminal,
  Browser last; opening a thing files it under its section. Refinement same conversation: third
  section is Browser only; Forge opens IN the browser (→ cockpit tabs are Browser's transitional
  residents; #389 plans the webview).
- **Sprint:** #37 M26 `848db7fb-1781-46d9-a574-b566e97cf005`; forge #385
  `09006da6-d499-4035-9f47-046accb4fbcd`.
- **Code grounding (2026-07-22 Explore map):**
  - `TabContent<S>` tabs.rs:23-31 — `Terminal(PaneGrid<S>)` / `Cockpit(RightSection)` /
    `CodeView(EditorSurface)`; kind observable via `grid()` / `cockpit_section()` / `editor()`.
  - `Project<S>` tabs.rs:164-171 — private `tabs: Vec<Tab<S>>` + private `active: usize`
    (order-keyed!); `add_tab` :191 appends; `close_tab` :200-215 refuses LastTab/LastTerminal.
  - Rail model: `rail_rows` tabs.rs:601-655 (flat tree: Project header → Tab rows :623-633 →
    nested Pane rows :635-651); `RailRow` :527-544; `RailLevel` :513-522 (Project|Tab|Pane).
  - Render: app.rs:15973 calls rail_rows; row loop app.rs:16011 matches `row.level`.
  - **Hazard (map flag a):** the shell codec (`grid_layout.rs` TabLayout, order-based) + `active:
    usize` key off positional order → this slice is locked display-only (D1).
- **Prior-art sweep:** recorded in the spec. Key: no crate owns the seam; `rail_rows` is the single
  pure producer; Zed 07-workspace doc + Warp 03-terminal doc confirm no reference-app analog → §20
  N/A (Marley-specific, chad's design).
- **Open items for P2:** RailRow field vs RailLevel::Section variant; header styling role; whether
  the existing per-project `collapsed` set interacts (collapse belongs to #386 — P2 here only
  ensures the section rows don't break the project-collapse path).
- **AAR:** `ff1d257e-9cd9-4d91-859d-28f75e693b73`.

## Phase 2 — Design (2026-07-22)

### Architecture / approach
Entirely the pure model (`tabs.rs`) + the render shim (`app.rs`) — no PTY / codec / settings touch.
The rail's producer `rail_rows` is the single seam both the render loop and its unit tests consume.
- **`RailLevel::Section` variant** (chose a level variant over a new `RailRow` field — the render
  loop already `match`es `row.level`, so a Section arm is the natural home; the section name rides
  the existing `label` field; no struct-field churn, no constructor sprawl).
- **`RailSection { Editor, Terminal, Browser }`** + `const ALL: [_;3]` (the fixed order) + `label()`.
- **`TabContent::<S>::rail_section()`** — the total 3-arm mapping (CodeView→Editor, Terminal→Terminal,
  Cockpit→Browser); a method on `TabContent` so it's trivially unit-tested via the existing
  `cockpit()`/`term_tab()`/`Tab::code` test helpers.
- **`rail_rows` rewrite:** after the `if is_collapsed { continue }` guard, replace the single tab
  loop with `for section in RailSection::ALL { push Section header; for (j,tab) in tabs().enumerate()
  { if tab.content.rail_section() != section continue; push Tab row (tab index j PRESERVED as the
  switch target); nested pane rows unchanged } }`. O(3n), trivial. **Display-only:** `Project.tabs`
  order, `active: usize`, `switch_tab`, and `grid_layout.rs` are byte-identical — the map's flagged
  hazard is avoided by construction (j is the original index; sections exist only in the row stream).
- **app.rs render arm** `RailLevel::Section =>` a muted `Role::Caption` (11px) row at `pl(px(18.0))`
  (between project 8 and tab 28), no click handler (collapse/highlight = #386).

### §20 confirm
`N/A — Marley-specific` still holds: the three-fixed-type-sections rail is chad's own design; Warp's
sidebar is a flat session list, Zed groups by panel not open-item type (both from our research maps —
docs/{warp,zed}_architecture/subsystems/*). Observe-and-reimplement; no copyleft source read. gpui
(Apache-2.0) owns no section-list primitive — the rail is our own flex column.

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/tabs.rs` | + `RailSection` enum + `impl` (`ALL`, `label`); + `impl<S> TabContent<S>::rail_section`; + `RailLevel::Section` variant; rewrite `rail_rows` tab loop → sectioned emission (j preserved) |
| `crates/marley_app/src/app.rs` | + `RailLevel::Section` render arm (muted Caption @ pl 18); the only exhaustive `match row.level` gains the arm |

### Regression Test Plan
| # | Test | Proves |
|---|---|---|
| T1 | `rail_section` maps all 3 variants (CodeView→Editor, Terminal→Terminal, Cockpit→Browser) | REQ-001; cov/MSI on the mapping |
| T2 | `RailSection::ALL` order == [Editor,Terminal,Browser]; `label()` per variant | REQ-001; fixed order |
| T3 | `rail_rows` on a project with one tab of EACH kind → rows are Section(Editor),Tab(code),Section(Terminal),Tab(term),Section(Browser),Tab(cockpit) in that order; each Tab under its kind's header | REQ-001/002 |
| T4 | each Tab row's `tab` field == its ORIGINAL `project.tabs()` index (a code tab stored at index 2 keeps `tab: Some(2)` even though it renders first) | REQ-003 (display-only, switch target intact) |
| T5 | a project with only terminal tabs → the Editor and Browser section headers are still present (empty) | REQ-004 |
| T6 | a split terminal tab → its Pane rows nest under the Tab row, which is under the Terminal Section header (section_idx < tab_idx < pane_idx) | REQ-005 |
| T7 | a COLLAPSED project emits NO section headers (the collapse-continue precedes the section loop) — only its Project row | REQ-003/004 edge |
| T8 | update `rail_rows_one_project` (was len==3 + index-based) to the sectioned shape (2 cockpit tabs → both under Browser; Editor/Terminal headers empty) | regression |
| — | grid_layout codec + tab-model (close_tab/switch_tab) tests run UNCHANGED | REQ-003 |
| CAP | driven: a workspace with an editor + terminal + cockpit tab renders 3 headers in fixed order, tabs grouped | REQ-001/004 visual |

Uncoverable by unit test: the app.rs render arm (coverage-excluded masked shim) → the CAP driven
capture. Everything else is pure `rail_rows`/`rail_section` at cov/MSI 100.

### Risks / decisions
- **D-EMPTY-HEADERS** — all 3 headers always render per non-collapsed project (spec D2/REQ-004). Risk:
  clutter with many projects; accepted (chad's "force … at the top" = fixed skeleton). A #386 refinement
  if the capture reads noisy.
- **D-SEARCH-FILTER** — a Section header stays visible during a "Search tabs" filter even if all its
  tabs are filtered out (cosmetic; #386 owns filter/collapse interaction). Noted for inspect.
- **D-SECTION-INERT** — Section rows carry `active=false`, no click, no collapse in #385. #386 adds the
  active-section highlight + click-collapse.
- **D-LEVEL-VARIANT-NOT-FIELD** — a `RailLevel::Section` variant (not a `RailRow.section` field) — the
  render already switches on level; keeps the struct + its Eq derive + ~4 constructors unchanged.

## Phase 3 — Implement (2026-07-22)
Built exactly to the manifest, no deviations:
- `tabs.rs`: `RailSection{Editor,Terminal,Browser}` + `ALL` (fixed order) + `label()`; `impl<S>
  TabContent<S>::rail_section()` (the total 3-arm mapping); `RailLevel::Section` variant; `rail_rows`
  tab loop rewritten to the sectioned emission (per non-collapsed project: `for section in
  RailSection::ALL` → push header → inner `for (j,tab)` filtered by `rail_section() != section`,
  `j` preserved as the switch target; nested pane rows unchanged).
- `app.rs`: the `match row.level` gained a `RailLevel::Section` arm — a muted `Role::Caption` row at
  `pl(px(18.0))`, no click (collapse/highlight = #386).
- `cargo check -p marley` → clean (only the pre-existing `block v0.1.6` future-incompat warning).
- The exhaustive `match row.level` in app.rs is the ONLY exhaustive match on `RailLevel` (the #241
  pre-pass + #112 filter use `== RailLevel::Tab`), so the new variant needed exactly one render arm —
  the compiler confirmed exhaustiveness.

## Inspect (Phase 3.5) — 2026-07-22
Two independent critics (general-purpose) over the diff + my own Step-2 review. **No code defects
found on any substantive dimension.** Lenses: correctness (critic 1), data/state integrity +
simplification + mutation-surface + provenance (critic 2).

**Findings + verdicts:**
- **[test — deferred to Validate] `rail_rows_one_project` is RED** (both critics; C1 HIGH / C2 MED).
  Empirically reproduced: the 2 cockpit tabs now emit 6 rows (Project + 3 section headers + a + b),
  the test still asserts the old flat `len==3` + `rows[1]` positions. **Verdict: not a code defect —
  a rail-SHAPE test legitimately changes when the rail shape changes (REQ-003 scopes "unchanged" to
  persistence/codec tests, which the full-suite run confirmed all pass). Tests are a Phase-4 artifact
  (§3) → carried to Validate as T8** (rewrite by `level`-filter, not fixed index). The other 3 rail
  tests pass (label/level-based `.find`) — confirmed by `cargo nextest -p marley --no-fail-fast`:
  **807/808 pass, only this one fails** (the empirical proof of the display-only claim).
- **[LOW — deferred to #386] Section headers ignore the "Search tabs" filter** (both critics + my
  D-SEARCH-FILTER). The `RailLevel::Section` arm has no `session_filter` gate, so a search shows all 3
  headers above the matching tabs. **Verdict: acceptable for a display-only slice** (D2 makes the
  skeleton invariant); noted for #386 (interaction) to optionally hide zero-match headers during a
  filter. Not a blocker.
- **Correctness (C1) — CLEAN:** tab-index `j` preserved as the switch target (total partition, no
  drop/dup); #241 pre-pass filters `== Tab` (Section rows excluded before any `unwrap_or`); scroll
  math safe (`saturating_sub` + `scroll_code` clamp, same total both sides); only-exhaustive-match
  covered; empty-project → 3 headers, collapsed → none, split panes nest correctly; `active:false`
  inert; no new panic/unwrap/borrow.
- **Integrity (C2) — CLEAN:** display-only verified by construction (no `.sort`/reorder, `active_tab`
  expr byte-identical, grid_layout.rs diff empty, Section arm has NO `on_mouse_down`). Partition is
  total (a future 4th `TabContent` variant is a COMPILE ERROR until classified — robustness positive).
- **Simplification (C2) — CLEAN:** minimal shape; correctly did NOT reuse `Tab::key_context()`
  (focus-dependent — would misfile a terminal tab whose focused pane is an editor under Editor);
  `label().to_string()` required (`RailRow.label: String`); O(3n) fine at rail scale.
- **Provenance/secrets (C2) — CLEAN:** Marley-original design; no copied structure; secret-scan empty.

**Mutation-surface (from `cargo mutants --list -f tabs.rs`, cross-checked by C2) — carried to Validate:**
- `RailSection::label` → `""`/`"xyzzy"` (2 VIABLE) → **T2 asserts each variant's exact literal.**
- `TabContent::rail_section` → `Default::default()` UNVIABLE (no Default derive — the #203/#204 rule)
  → **T1 is a zero-mutant regression GUARD** (kept per the "keep the guard test" rule; nothing else
  protects the CodeView→Editor/Terminal→Terminal/Cockpit→Browser mapping).
- `RailSection::ALL` — not mutated (const array) → **T2 order assert is a guard.**
- **`rail_section() != section` (tabs.rs:685) → `==` (1 VIABLE, the one a naive test MISSES)** →
  **T3 must assert exact section MEMBERSHIP/placement + Tab-row count == n** (build a mixed
  CodeView+Terminal+Cockpit project, assert the full row sequence), not merely "3 headers exist".
- Check whether `active:false` (tabs.rs:678) yields a bool-replacement mutant → **T3 also asserts
  Section rows carry `active == false`.**

**No forge failure-record** (no real bug; the stale test is expected phase-ordering). No new PR
(critic 2's "assert membership not count" reprises the existing distinguishing-tests lesson —
[[PR-claude-clamp-adjust-tests-must-distinguish-each-branch-001]] applied to the section filter).

## Phase 4 — Validate (2026-07-22)

### Tests added / updated (tabs.rs `#[cfg(test)]`)
- **T1** `rail_section_maps_each_content_kind` — CodeView→Editor / Terminal→Terminal / Cockpit→Browser
  (the zero-mutant regression GUARD; the only body mutant is the unviable `Default::default()`).
- **T2** `rail_section_order_and_labels` — `RailSection::ALL == [Editor,Terminal,Browser]` + each
  `label()` literal (kills the 2 `label` `""`/`"xyzzy"` mutants; pins the const-array order).
- **T3** `rail_rows_groups_tabs_by_section_in_fixed_order` — a mixed code+term+cockpit project asserts
  the EXACT 7-row sequence + Tab-count==3 + Section rows `active==false` (kills the `685: !=→==`
  section-filter mutant + any mis-ordering — critic 2's sharpened design).
- **T4** `rail_rows_preserves_original_tab_index` — a code tab stored at idx 1 renders first (Editor)
  but keeps `tab: Some(1)` — the display-only switch-target guarantee.
- **T5** `rail_rows_empty_sections_still_render_headers` — a terminals-only project still shows the
  Editor + Browser headers (REQ-004).
- **T6** `rail_rows_split_panes_nest_under_terminal_section` — Section(Terminal) < Tab < Pane order.
- **T7** `rail_rows_collapsed_project_has_no_section_headers` — a collapsed project emits only its
  Project row; the expanded sibling keeps its 3 headers.
- **T8** rewrote `rail_rows_one_project` to the sectioned shape (level-filter, not fixed index).

### Suite result
`cargo nextest run -p marley` → **815 passed, 0 failed, 2 skipped** (+7 new section tests; the prior
807 all still green — the empirical proof that the display-only change regressed nothing:
codec/model/integration/headless/fleet suites unchanged).

### Driven capture — `scratchpad/385-rail-default.png` (VISUAL ACCEPTANCE PROVEN)
Bundled the debug `.app`, `open`ed it, activated, captured window 4215, read the pixels. A restored
workspace (an Editor tab + a split terminal) rendered the left rail EXACTLY as designed:
- **Marley · main** (project header, active-highlighted, ▾ + ×)
- **Editor** (muted section header) → the "Editor" tab under it
- **Terminal** (muted section header) → the "Marley" terminal tab, with **pane 1** + **pane 2** nested
- **Browser** (muted section header) → **EMPTY** (no cockpit tab open)

Proves **REQ-001** (three headers in the fixed order Editor→Terminal→Browser), **REQ-002** (the editor
files under Editor, the terminal under Terminal), **REQ-004** (the empty Browser header renders), and
**REQ-005** (the split panes nest under the terminal tab, inside the Terminal section). Headers render
as muted captions indented between the project and tab rows. Footer "focus: editor" consistent (#382).
(Cockpit→Browser population is unit-proven by T1/T3; the empty-Browser-header is the REQ-004 capture.)

### Gate — `scripts/gates.sh --diff` → **GATE GREEN [diff]** (receipt written)
15/15 pass: gate:1 rustfmt · gate:2 clippy(-D warnings) · gate:3 tests · **gate:4 coverage ≥100%
lines** · **gate:5 mutation MSI ≥100%** · gate:6 miri · gate:7 audit · gate:8 deny · gate:9 machete ·
gate:10 gitleaks · gate:11 shellcheck · gate:12 no-suppressions · gate:13 source-bans · gate:14 docs ·
gate:15 visual/AX. Diff TOTAL = 100.00% lines (0 missed).

**One coverage catch fixed (gate:4 RED on the first run):** T7's guard used `!rows[p0_idx+1..p1_idx]
.iter().any(|r| …)` — but a collapsed project sits immediately before its expanded sibling, so that
slice is EMPTY and `.any` never runs the closure → 1 uncovered line/function (found via the llvm-cov
JSON: line 1412). Rewrote the guard to `rows.iter().filter(level==Section).all(|r| r.project == 1)`,
which runs the closure over p1's real section rows AND still proves "collapsed p0 emits no section
headers". Re-ran the gate → GREEN. LESSON: an `.any`/`.all` over a slice that can be empty in the
tested scenario leaves its closure uncovered — assert over a non-empty projection instead
([[PR-claude-empty-iterator-closure-uncovered]] class).

Pre-existing sub-line uncovered REGIONS in tabs.rs (the `matches!`/`assert_eq!(…Err…)` false-arms at
1148/1175/1180) are on lines with other covered regions → they do NOT reduce LINE coverage; untouched.

## Phase 5 — Complete (2026-07-22)
- **Docs (§21):** CHANGELOG.md `### Added` entry (newest-first); app_shell.md rail note added after the
  #239 entry (the rail_rows lineage: #152 → #236 → #239 → **#385 sections**).
- **Knowledge (forge wired):** `aar-submit` (aar ff1d257e, outcome completed, effectiveness 5, 13
  verdicts, 1 novel finding); `prevention-rule-record` **PR-claude-empty-iterator-closure-uncovered-001**
  (the empty-between-slice `.any` coverage trap); ticket-comment + ticket-close #385 (status done). No
  failure-record — no real bug (both critics clean; the stale test + the coverage miss were expected
  Validate-phase artifacts).
- **Archive:** ticket doc → tickets/closed/; pipeline pair → completed/.
- **Ready for /commit** (first ticket of /work 385-389, auto-approved → commit LOCAL, hold push).
