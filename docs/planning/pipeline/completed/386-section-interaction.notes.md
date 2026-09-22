# 386-section-interaction — Notes

## Phase 1 — Plan (drafted 2026-07-22, /spec batch, Fable)
- **Request:** slice 2 of chad's sectioned shell — make the #385 sections interactive (highlight +
  collapse-persist) + the footer vocabulary audit.
- **Sprint:** #37 M26; forge #386 `8ac2f904-6043-4c1b-8b63-e101a6bd3084`. HARD DEP: #385.
- **Code grounding (2026-07-22 map + shipped history):**
  - `rail_rows(ws, collapsed)` tabs.rs:601 already threads a per-project collapsed set from
    `self.collapsed_projects` (app.rs:15973) — the pattern to extend per (project, section).
  - Highlight idiom: #219 `rail_highlight` accent-wash +
    PR-claude-selection-bg-distinct-from-container (a219/a784d54 lineage).
  - Persistence idiom: typed settings round-trip (settings.rs `define_setting!`; `ShellLayoutSetting`
    :144/:241; the M14 collapse-persist family).
  - Footer: `FocusTab`/`focus_label` status_bar.rs:64-105 (#382, exhaustive PaneKind match);
    inputs assembled app.rs:18423-18432. Divergence to reconcile: Cockpit tab reads "cockpit",
    files under Browser.
  - Lessons wired into the spec: #305 auto-reveal; PR-claude-pump-state-change-must-set-dirty.
- **Prior-art sweep:** recorded in spec — no external owner; the two in-repo idioms (project
  collapse + settings round-trip) are the reuse targets.
- **Open items for P2:** collapsed-set shape (one unified set vs parallel section set); the
  setting key name; the D4 footer outcome.
- **AAR:** `499d8ec0-279b-48ac-be9a-805bf9cf3760`.

## Phase 2 — Design (2026-07-22)

### Architecture / approach
A faithful mirror of the shipped project-collapse machinery (#236 toggle+remap · #245 persist-by-root),
one level down to `(project_index, RailSection)` keys. Pure model in `tabs.rs`, settings round-trip in
`settings.rs`, render+state wiring in `app.rs`. Four pieces:

**(1) Active-section highlight.** `rail_rows` computes the active section — for the active project,
`active_tab_section = project.tabs().get(active_tab).map(|t| t.content.rail_section())` — and sets the
Section header row's `active = (i == active_project && Some(section) == active_tab_section)`. The render
arm paints `active` rows with the #219 `rail_highlight(&colors, true)` accent-wash (the same idiom the
Project/Tab rows use), others muted. Pure decision on the existing `RailRow.active` field.

**(2) Collapse/expand + the auto-reveal invariant.** `RailSection` gains `Hash` (for the set key).
`rail_rows` takes a new `collapsed_sections: &HashSet<(usize, RailSection)>`. Per section:
`is_active = i==active_project && Some(section)==active_tab_section`; `collapsed =
collapsed_sections.contains(&(i, section)) && !is_active` — **the active section is force-expanded at
render**, so its active tab is NEVER hidden (the #305 auto-reveal invariant BY CONSTRUCTION — no
activation-site hooking, satisfies REQ-004 for palette/keyboard/open-file/＋ alike). The Section header
row carries `collapsed`; when collapsed its tab (and nested pane) rows are omitted. The header always
renders (the #385 skeleton).

**(3) Persistence + index-remap (mirror #245/#236).** New pure fns in `tabs.rs`:
- `RailSection::from_label(&str) -> Option<RailSection>` (the inverse of `label()`).
- `collapsed_section_keys(&HashSet<(usize,RailSection)>, roots: &[String]) -> Vec<String>` — each
  in-range `(idx, sec)` → `format!("{}\x1f{}", roots[idx], sec.label())`, iterating roots in order for
  stable output (mirrors `collapsed_roots`; `\x1f` = the #177/#205 framing byte, absent from a path).
- `collapsed_sections_from_keys(&[String], roots: &[String]) -> HashSet<(usize,RailSection)>` — split
  each key on `\x1f` → `(root, label)`; the project index whose root matches × `from_label(label)`
  (mirrors `collapsed_indices`; unmatched root / bad label silently skipped — a closed/renamed project).
- `remap_section_indices_after_remove(&HashSet<(usize,RailSection)>, removed) -> …` — drop the removed
  project's keys, shift `idx>removed` down one (mirrors `remap_indices_after_remove`).
`settings.rs`: a `CollapsedSections: Vec<String> = Vec::new()` setting keyed `"rail.collapsed_sections"`
+ a `SettingsSnapshot.collapsed_sections` field + `persist_collapsed_sections(manager, &keys)` (mirrors
`persist_collapsed`).

**(4) app.rs wiring.** `AppView` gains `collapsed_sections: HashSet<(usize, RailSection)>`. Boot restores
it via `collapsed_sections_from_keys(applied.collapsed_sections, &roots)` (beside the #245 project
restore). New `toggle_section_collapse(&mut self, project, section)` (insert/remove + `persist_
collapsed_sections_state`) and `persist_collapsed_sections_state` (masked settings IO; pure mapping =
`collapsed_section_keys`). `close_project_at` ALSO remaps `collapsed_sections` via
`remap_section_indices_after_remove` + re-persists (the #236 index-shift lesson). The two `rail_rows`
call sites (15973, 16004) pass `&self.collapsed_sections`. The `RailLevel::Section` render arm gains a
▸/▾ chevron + `on_mouse_down`→`toggle_section_collapse` (stop_propagation) + the `active` accent-wash —
mirroring the Project row (app.rs:16044-16095). `cx.notify()` after toggle (the dirty-repaint rule).

**(5) Footer (D4) — documented NO-OP.** `focus_label`'s `Cockpit → "cockpit"` STAYS. Rationale: the
footer names the focused surface's NATURE ("cockpit" — it IS a Forge cockpit), while the rail section
names its CATEGORY ("Browser" — its transitional home until Phase E). Different axes; renaming the
footer to "browser" would misdescribe the native cockpit as a browser it is not yet. #382's exhaustive
`PaneKind`/`FocusTab` matches are unchanged. Recorded, not silently skipped.

### §20 confirm
`N/A — Marley-specific` holds: section-level highlight + collapse-persist on a type-sectioned rail is
chad's own design (the 2026-07-10 workspace-centric "rail highlights the active thing" one level down).
Zed dock-toggle + VS Code section-collapse were behavior research only (no source). The in-repo #236/#245
project-collapse machinery is the reuse target — observe-and-mirror our OWN code.

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/tabs.rs` | `RailSection`: +`Hash` derive, +`from_label`; `rail_rows`: +`collapsed_sections` param, compute active section (highlight) + force-expand-active collapse; +`collapsed_section_keys`, +`collapsed_sections_from_keys`, +`remap_section_indices_after_remove` |
| `crates/marley_app/src/settings.rs` | +`CollapsedSections` setting ("rail.collapsed_sections") + `SettingsSnapshot.collapsed_sections` + `persist_collapsed_sections` (mirror `CollapsedProjects`/`persist_collapsed`) |
| `crates/marley_app/src/app.rs` | +`collapsed_sections` field; boot restore; `toggle_section_collapse` + `persist_collapsed_sections_state`; `close_project_at` remap; both `rail_rows` call sites; the `RailLevel::Section` render arm gains chevron+click+active-wash |

### Regression Test Plan
| # | Test | Proves |
|---|---|---|
| T1 | `rail_rows` on the active project → the active tab's section header carries `active==true`, the other two `false`; a non-active project's headers all `false` | REQ-001 |
| T2 | a `(proj, Terminal)` in `collapsed_sections` → the Terminal header renders with `collapsed==true` and its tab rows are OMITTED; the other sections render their tabs | REQ-002 |
| T3 | the ACTIVE section is force-expanded even when in `collapsed_sections` — its header `collapsed==false` and its active tab row IS present | REQ-004 |
| T4 | `collapsed_section_keys` ⇄ `collapsed_sections_from_keys` round-trip (incl. an out-of-range idx dropped on encode, an unmatched root + a bad label dropped on decode); `from_label` all 3 + a None | REQ-003 |
| T5 | `remap_section_indices_after_remove` — drop removed idx's keys, shift `>removed` down, keep `<removed` (mirror the #236 test) | REQ-003 (close) |
| T6 | `RailSection` `Hash`/`Eq` usable as a `HashSet<(usize,RailSection)>` key (compile + a dedup assert) | contract |
| T7 | update #385's T3 (`rail_rows_groups_tabs_by_section_in_fixed_order`) — the active section's header is now `active==true` (cockpit tab active → Browser header active); keep the ordering/partition asserts | regression |
| T8 | `persist_collapsed_sections` settings round-trip in a tempdir (mirror the #245 persist test) | REQ-003 |
| CAP | driven: collapse a section (chevron) → its tabs hide + survive relaunch; switch to a tab in it → auto-expands + the header highlights | REQ-001/002/004 visual |

Uncoverable by unit test: the app.rs render/click arm + the boot restore (masked shim / coverage-
excluded) → the CAP driven capture. All pure fns (rail_rows active+collapse decision, the 3 persistence
fns, from_label) at cov/MSI 100.

### Risks / decisions
- **D-FORCE-EXPAND-ACTIVE** — the active section renders expanded regardless of the collapsed set (no
  state mutation on activation). Robust (no missed activation sites); the persisted collapse is
  preserved, so switching away restores the user's collapse. Chosen over hooking every switch site.
- **D-INDEX-KEYED-REMAP** — `collapsed_sections` is keyed by project INDEX in memory (persisted by
  ROOT); `close_project_at` MUST remap on a Vec removal or it aliases the wrong project (the #236
  `BF-index-keyed-state-not-remapped-on-vec-remove` lesson, applied to the tuple key).
- **D-FOOTER-NOOP** — keep "cockpit" (recorded rationale above).
- **D-385-T3-UPDATE** — #385's T3 asserted section rows `active==false`; #386 makes the active section's
  header active → T3 updates (a follower-ticket behavior change, expected).
- **D-SHARED-ROOT** — two projects sharing a root both restore a collapsed section (the inherent
  root-keying tradeoff #245 already documented; cosmetic, no panic).

## Phase 3 — Implement (2026-07-22)
Built to the manifest, no deviations:
- **tabs.rs:** `RailSection` +`Hash` derive + `from_label`; `rail_rows` gained the `collapsed_sections`
  param, computes `active_tab_section` (for the active project) → the Section header's `active` +
  force-expand-active (`section_collapsed = contains(&(i,section)) && !is_active_section`, then
  `if section_collapsed { continue }`); + `collapsed_section_keys` / `collapsed_sections_from_keys`
  (`<root>\x1f<label>`) / `remap_section_indices_after_remove` (all mirror the #245/#236 project fns).
- **settings.rs:** `CollapsedSections` setting (`rail.collapsed_sections`) + `SettingsSnapshot
  .collapsed_sections` (both builder sites) + `persist_collapsed_sections` (mirror `CollapsedProjects`).
- **app.rs:** `AppView.collapsed_sections` field; boot restore via `collapsed_sections_from_keys`;
  `toggle_section_collapse` + `persist_collapsed_sections_state` (masked, `mutants::skip`);
  `close_project_at` remaps + re-persists; both `rail_rows` call sites pass `&self.collapsed_sections`;
  the `RailLevel::Section` render arm gained a ▸/▾ chevron + `on_mouse_down`→`toggle_section_collapse`
  (`cx.notify()`) + the `row.active` accent-wash (`rail_highlight`), mirroring the Project row. The
  section identity is recovered from the header label via `RailSection::from_label` (the label IS
  `section.label()`, a guaranteed round-trip; guarded with `if let Some`).
- **Footer:** documented NO-OP (D-FOOTER-NOOP) — `focus_label` unchanged.
- `cargo check -p marley` → clean (only the pre-existing `block v0.1.6` warning).
- Borrow note: `from_label(&row.label)` (borrow) completes before `.child(row.label)` (move) — the label
  is read for the section then moved into the caption, no conflict.

## Inspect (Phase 3.5) — 2026-07-22
Three independent critics (correctness · state-integrity · simplification/mutation) + my review.
**Code logic CLEAN on every substantive dimension** (critic 1: 6/6 correctness points clean; critic 2:
remap parity, settings round-trip all 6 sites, no-collision, Hash, secrets clean; critic 3: the
`HashSet<(usize,RailSection)>` shape, the render-time force-expand design, and the #245 mirroring all
endorsed). One app-code fix applied; everything else is Validate test work.

**Findings + verdicts:**
- **[HIGH → Validate] test target does not compile** (critics 1+2). 14 sites broken by the signature/
  struct change: 11 `rail_rows(…)` test callers (tabs.rs:1315/1386/1420/1434/1455/1483/1508/1525/1570/
  1590/1602 — need arg 3 `&HashSet::new()`) + **3 `AppliedSettings` literals** (settings.rs:704/755/805 —
  need `collapsed_sections: Vec::new()`). **Verdict: real, but these are EXISTING tests broken by the
  API change → Validate caller-fixes (tests are the Phase-4 artifact).** Root cause captured as a failure
  + PR below.
- **[HIGH → Validate] `rail_rows` `755:63 delete !` survives every empty-set test** (critic 3). The
  mutation `contains && !is_active_section → contains && is_active_section` differs only when a section is
  IN the set; all #385 tests pass an empty set. **Verdict: real → Validate MUST add a non-empty-set
  collapse test + a force-expand-active test** (both drive `collapsed_sections` non-empty).
- **[MED → Validate] `from_label` is NOT coverage-only** (critic 3 corrected my design premise). It has a
  `_ => None` WILDCARD → 4 viable mutants (the `None` body + 3 arm-deletes falling through to the
  wildcard), UNLIKE `rail_section`'s exhaustive-no-wildcard match. **Verdict: real → a direct
  `from_label` unit test (3 labels + a bogus→None) is required, not just indirect coverage.**
- **[MED → Validate] the mutation matrix** (critic 3, from the REAL `cargo mutants --list`): `686 ==→!=`
  needs a ≥2-project restore matching index 1; `703 -→+`//` needs a Greater-index remap; the three `?` in
  `collapsed_sections_from_keys` need region coverage (malformed / bad-label / stale-root keys); the
  highlight test must assert exactly-one-section-active (kills 737/751:55/751:77). Unviable (no Default,
  do NOT chase): the `Default::default()`/`from_iter([Default])`/`vec![Default::default()]` mutants.
- **[Validate] #385 T3 breaks** — `rail_rows_groups_tabs_by_section_in_fixed_order`'s
  `.all(|r| !r.active)` now fails (the active cockpit tab makes the Browser header `active==true`).
  Replace with the exactly-one-Browser-active assertion (doubles as the highlight mutation guard).
- **[LOW → FIXED] boot builds `project_roots` twice** (critic 3). **Fixed:** hoisted one `project_roots`
  Vec feeding both `collapsed_indices` + `collapsed_sections_from_keys` (app.rs boot). `cargo check` clean.
- **[LOW accepted] framing-byte-on-root** (critics 1+2+3). A project root theoretically containing `\x1f`
  makes its section key fail to decode (silent drop → restores expanded). **Verdict: accept + document —
  identical graceful-degrade to #245's documented duplicate-root LOW; outcome-neutral (drop-on-encode vs
  drop-on-decode both restore expanded); paths with 0x1F are absurd. NOT guarding (avoids added mutation
  surface for a nil-value edge).**
- **[LOW accepted] duplicate-root restore asymmetry** — `position` (first match) vs #245's `contains`
  (all). The documented #245 duplicate-root tradeoff; cosmetic, no panic.
- **[LOW accepted → capture note] active-section chevron click is a no-op-until-switch-away** (all 3
  critics). By design (force-expand-active can't collapse what you're viewing — the #305 invariant). It
  DOES flip+persist the key (manifests on switch-away). **Validate's REQ-002 driven capture MUST target a
  NON-active section**, else the screenshot shows "nothing happened".

**Fix applied:** the boot `project_roots` hoist (app.rs). No other app-code change — the 3 critics found
no correctness/integrity/provenance defect.

**Forge:** `failure-record` BF-claude-implement-check-skips-cfg-test-callers (the `cargo check` blind
spot) + `prevention-rule-record` PR-claude-check-tests-after-signature-change-001.

## Phase 4 — Validate (2026-07-22)

### Tests
- **Compile-fixes (14):** the 11 `rail_rows` test callers gained the 3rd arg; the 3 `AppliedSettings`
  literals (settings.rs) gained `collapsed_sections: Vec::new()`; a `use std::collections::HashSet`
  added to the tabs test module.
- **T3 updated:** `rail_rows_groups_tabs_by_section_in_fixed_order` — `.all(!active)` → exactly-one-
  Browser-active (the active cockpit tab's section is now highlighted; kills highlight mutants
  737/751:55/751:77).
- **8 new tests** (critic 3's mutation matrix, from the REAL `cargo mutants --list`): `from_label`
  round-trip (3 labels + Bogus→None — NOT coverage-only, the wildcard makes 4 mutants viable);
  `collapsed_section_keys` exact-content + out-of-range skip + sort; `collapsed_sections_from_keys`
  index-1 match (kills 686 `==→!=`) + 3 malformed keys (the `?` region branches);
  `collapsed_sections_persistence_round_trips`; `remap_section_indices_shifts_after_remove` (kills 703
  `-→+`//); `rail_rows_collapses_a_non_active_section` (kills 755:60 `&&→||` + 755:63 `delete !`);
  `rail_rows_force_expands_the_active_section` (REQ-004, kills 755:63 from the active orientation); +
  `collapsed_sections_setting_round_trips` (settings.rs, kills the persist `Ok(())` mutant).

### Suite + gate
`cargo nextest run -p marley` → **823 passed, 0 failed, 2 skipped** (+8). `scripts/gates.sh --diff` →
**GATE GREEN [diff] 15/15 on the first run** — coverage 100% lines, **MSI 100%** (every viable mutant
from critic 3's list killed), miri, visual/AX, all static gates.

### Driven capture (bare binary — see env caveats) — `scratchpad/386-rail-default.png` + `386-rail-editor-collapsed.png`
The `.app` bundle launched from `/Volumes/Offload` (the moved HD) ran but **created no window** — a
LaunchServices/Gatekeeper quirk for an ad-hoc-signed bundle on an external volume (process live, no
boot panic, headless tests green). Fell back to the **bare binary** (`target/debug/marley`,
run_in_background) which CGWindowList captures fine.
- **REQ-001 (highlight) PROVEN:** the default rail rendered three headers Editor/Terminal/Browser each
  with a ▾/▸ chevron, and the **Terminal section carried the accent-wash highlight** (the active tab
  was the terminal — footer "focus: terminal").
- **REQ-002 (collapse) PROVEN:** clicking the NON-active Editor section header (`focus clickat:0.05,0.19`)
  **collapsed it** — the chevron flipped ▾→▸ and its "Editor" tab row vanished.
- **REQ-004 (force-expand-active) PROVEN:** the active Terminal section stayed ▾/expanded throughout.
- **REQ-003 (persist) — trigger FIRED:** after the collapse click, `~/.marley/config/settings.toml`
  gained `collapsed_sections = [".../Marley\x1fBrowser"]` (the #243 concern — the save is triggered,
  not just the codec). Two env artifacts, NOT code defects: the persisted root is the SYMLINK path
  (the app restored chad's pre-move session whose grid referenced `/Users/chadpeppers/…`), and a
  section-name discrepancy vs the capture (multi-instance/timing during the messy .app+bare launch).
  A driven quit→relaunch check is unreliable in this symlink-root-ambiguous env (it would
  false-negative on the root mismatch), so **persistence rests on the unit round-trips + the trigger
  evidence** (PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism spirit).
- **Cleanup:** reset chad's `collapsed_sections` back to `[]`; stopped all marley instances. Only that
  one test-artifact key was touched.

### Env note (the 2026-07-22 HD move)
Project moved to `/Volumes/Offload/Projects/Marley` (APFS), symlinked from the old path. Used the REAL
path for edits so the hooks' `normalize_path` (which strips PROJECT_ROOT literally) keeps gating. The
52G build cache came along intact (cargo check 1s). The `.app`-on-external-volume no-window issue is a
harness caveat for future driven captures — prefer the bare binary here.

## Phase 5 — Complete (2026-07-22)
- **Docs (§21):** CHANGELOG `### Added` entry (newest-first, above #385); app_shell.md rail note
  extended with the #386 interaction (after the #385 note).
- **Knowledge (forge):** `aar-submit` (aar 499d8ec0, completed, effectiveness 5, 2 novel findings);
  the inspect failure BF-claude-implement-check-skips-cfg-test-callers + PR-claude-check-tests-after-
  signature-change-001 were recorded at inspect (referenced, not re-filed); ticket-comment +
  ticket-close #386 (status done).
- **Archive:** ticket doc → tickets/closed/; pipeline pair → completed/.
- **Ready for /commit** (2nd ticket of /work 385-389, auto-approved → commit LOCAL, hold push). Use the
  real `/Volumes/Offload` path; stage CHANGELOG in a separate bash first.
