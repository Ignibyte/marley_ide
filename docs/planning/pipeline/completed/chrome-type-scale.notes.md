# Calibrate sidebar/files text to Warp size via type_scale — Notes

- **Forge ticket:** #230 (7707f0be-d9d6-43d1-a0c3-a765bc8bd79f)
- **AAR:** 11e662ad-a41d-4597-90b7-e18fab7885bf
- **Local ticket doc:** docs/planning/tickets/open/TICKET-230-chrome-type-scale.md
- **Pipeline spec:** chrome-type-scale.spec.md

## Phase 1 — Plan
- **Request:** calibrate the files/sidebar text to Warp size + fold #223 (route the ~20 hardcoded chrome
  `text_size(px)` literals through `type_scale`). chad live feedback #2. 3rd ticket of the /work 228-237 M13 train.
- **Classification / tier:** work pipeline · chore · M13 · one pure-seam-refactor slice (design confirms / may split).
- **Forge recall (§18.3):** the type-scale foundation is #195 (`PR-...` the `no-default-struct-return-needs-full-value-assert`
  M1.E rule applies — `TextStyle` has no Default → assert the FULL value); #223 is the out-of-range consolidation
  this folds. Clean-room §20.
- **Discovery (this turn):**
  - `typography.rs` (PURE, cov/MSI 100): `type_scale(role) -> TextStyle { size, weight }`; Role = { Command
    13/Medium, Output 13/Normal, Caption 11/Normal } (#195 Warp-calibrated). Tests assert FULL values per role
    (`type_scale_by_role`) + a `≥12` legibility floor for Output/Command. `weight_value` maps to CSS numerics.
  - app.rs ~20 hardcoded chrome `text_size(px(N))`: sidebar/rail ~:2270/:2280/:2297/:2315/:2374 (px 13 — chad's
    "too big" files/workspace text) + :2355 (11); rail/multi-project :4006(12)/:4061(11)/:4105(12)/:4180(13)/
    :4280(12); pane :5226(12); status bar :5642(13)/:5809(13)/:5931(12); completion popup :6090(9)/:6134(12)/
    :6177(12)/:6233(13); context menu :6327(13). 5 sites already route through `type_scale` (Caption/Command/Output).
  - `workspace.rs::fallback_cell` — a stale `14.0` in the non-positive-input guard default (dead branch; #223).
- **Decisions:** D1 Role taxonomy + Warp sizes = design's call (keep Command/Output/Caption; new chrome bands may
  be <12); D2 full-value pins per Role; D3 no behavior change beyond the calibrated sizes; D4 clean-room (observe
  Warp, pick our values); D5 env-considerate driven capture → unit-proven values + chad's vibe-check.
- **Env note:** chad is ACTIVELY on a SHARED desktop → a driven capture would hijack his screen; the calibrated
  VALUES are unit-proven (cov/MSI 100) + observe-Warp-referenced (the deskcheck captures show Warp's left session
  list as a size reference); offer the visual vibe-check when his screen's free. A clean-room visual calibration —
  chad's eye is the final vibe check regardless.

## Phase 2 — Design

### Architecture / approach
An Explore pass mapped all 20 chrome literals + found the REAL culprit for chad #2. **chad's "the files IDE
portion text is too big" = the `files_panel` file tree (app.rs ~:2435-2499, rows :2464-2489) sets NO
`text_size` → inherits gpui's ~16px default (the largest chrome text).** The only px(13) SIDEBAR-RAIL row is
the `RailLevel::Tab` session entry (:4180); its rail siblings are already ≤12 (project 12, pane 12, ws-header 11).

**PURE seam (typography.rs):** add ONE new role — `Role::Nav` = **12.0 / Normal** — the left-sidebar/nav band
(rail rows, files tree, pane title, status, top-bar), Warp-calibrated (Warp's session list reads ~11-12; picked
12 as the safe modest value — chad vibe-checks). Keep `Command`(13/Med)/`Output`(13/Norm)/`Caption`(11/Norm)
unchanged. `type_scale` has ZERO viable mutants (`--list`: only `→Default::default()`, UNVIABLE — no `Default`);
the per-role test is for COVERAGE (each arm executed) + regression (full-value pin), per the no-Default rule.

**SHIM (app.rs render):** the 2 VISUAL fixes (chad #2) + a safe same-value consolidation of the bands `Nav`/
`Caption` already cover:
| Site | now | → | Δ |
|---|---|---|---|
| `files_panel` container (~:2491 / the wrapping panel ~:4336) — the file tree | unsized ~16 | `text_size(px(type_scale(Role::Nav).size))` | **16→12 (chad's fix)** |
| :4180 rail `Tab` session row | px(13) | `Role::Nav` | **13→12 (rail outlier)** |
| :4006 rail search, :4105 project, :4280 pane, :5226 pane-title, :5931 status, :6134 top-search input, :6177 top-search results | px(12) | `Role::Nav` | 12→12 (no change, consolidate) |
| :4061 ws-header, :2355 agent tail | px(11) | `Role::Caption` | 11→11 (no change, consolidate) |

(For the files_panel: set `text_size(Nav)` on the panel CONTAINER so the tree rows INHERIT it — gpui text_size
inherits to children; the header keeps its own `caption_header`→Caption(11). Confirm at implement the container
has no competing text_size.)

**MINOR (workspace.rs):** `fallback_cell`'s non-positive guard default `else { 14.0 }` → `else { 13.0 }` (matches
`TERMINAL_FONT_SIZE`); update/extend its test (the `13.0` needs a guard test — `fallback_cell(0.0)` → w=7.8,
h=15.6 — else the literal's `→0/1/-1` mutants survive; if the existing `fallback_cell_scales_and_guards` already
asserts the non-positive guard, just update the expected value from 14-based to 13-based).

### Scope decision — a FOCUSED slice (NOT the full #223 fold)
DEFER the 13-chrome (right-dock Details/Agents/Forge :2270/:2280/:2297/:2315/:2374, git :5642, diff :5809,
completion :6233, context-menu :6327) + the 9-badge (:6090) → routing those needs a `Body`(13/Norm — a value-DUP
of Output) + `Micro`(9) role and a call on whether to calibrate them down, NONE of which is chad's ask, and
routing 10 more sites BLIND (env-blocked pixel check) is low-value churn. **#230 = chad's files/sidebar fix +
the clean 12/11-band consolidation; #223 STAYS OPEN for the 13-chrome + badge (Body/Micro).** Honest: #230
partially folds #223, not fully — update #223's scope at complete; the CHANGELOG/ticket say so.

### File manifest
| File | Kind | Change |
|---|---|---|
| `crates/marley_app/src/typography.rs` | PURE | add `Role::Nav` (12/Normal) + extend `type_scale`; extend `type_scale_by_role` to assert Nav full-value |
| `crates/marley_app/src/app.rs` | SHIM | files_panel container `text_size(Nav)` (16→12) + :4180 →Nav (13→12) + route the px(12) sites →Nav + the px(11) sites →Caption (no-change consolidation) |
| `crates/marley_app/src/workspace.rs` | PURE | `fallback_cell` guard default 14.0→13.0 + its test |

### Regression Test Plan
| REQ | Test | Note |
|---|---|---|
| REQ-001 | typography `type_scale_by_role` asserts `Nav == {12.0, Normal}` (covers the new arm + pins it) | + driven/env-considerate → mechanism + chad vibe-check |
| REQ-002 | code review + `rg 'text_size\(px\((11\|12)\.0\)\)'` over the routed sites → none remain in the routed set (the 13-chrome/9-badge remain, deferred) | consolidation completeness |
| REQ-003 | `type_scale_by_role` full-value per role (all 4 roles' `{size,weight}`) — all arms covered, cov/MSI 100 | zero viable mutants; test = coverage + regression |
| REQ-004 | the existing `type_scale_terminal_text_clears_legibility_floor` (Output/Command ≥12) stays green (Nav is chrome, may be <12 — but it's 12, still ok) | floor preserved |
| REQ-005 | `workspace.rs::fallback_cell` guard test: `fallback_cell(0.0)` → w≈7.8, h≈15.6 (13-based), killing the `13.0` `→0/1/-1` mutants | the #223 fallback refresh |

**Uncoverable path:** the live rendered text SIZE — the pure `type_scale`/`fallback_cell` VALUES are unit-proven
(cov/MSI 100), but "does it match Warp / look right" is a clean-room visual judgment → chad's vibe-check (the
machine's shared-desktop env blocks a non-intrusive driven capture; offer it when free).

### Risks / decisions
- **R1 — the files_panel fix is inheritance-based:** setting `text_size(Nav)` on the container relies on gpui text
  inheritance to the tree rows. CONFIRM at implement no row sets its own `text_size` (the Explore pass says none
  do). If inheritance doesn't apply, set it per-row instead.
- **R2 — Nav=12 is a picked value:** clean-room from Warp's ~11-12 sidebar; chad confirms the vibe (13→12 rail is
  subtle, 16→12 files is the real fix). Easy to tune to 11 if he wants smaller.
- **R3 — scope honesty:** #223 is NOT fully folded (13-chrome/badge deferred) — say so in the CHANGELOG + keep
  #223 open.
- Decisions D1 (Nav only, defer Body/Micro), D2 full-value pins, D3 no-change consolidation, D4 clean-room, D5
  env-considerate validation.

## Phase 3 — Implement
**Built (manifest as designed — the FOCUSED slice):**
- `typography.rs` (PURE): added `Role::Nav` (doc + the `type_scale` arm `{12.0, Normal}`); extended
  `type_scale_by_role` to assert `Nav == {12.0, Normal}` (covers the new arm + pins it).
- `workspace.rs` (PURE): `fallback_cell` guard default `14.0` → `13.0` (matches TERMINAL_FONT_SIZE); updated the
  `fallback_cell_scales_and_guards` guard cases `fallback_cell(0.0)`/`(-5.0)` from 14-based (8.4/16.8) to
  13-based (7.8/15.6) — the positive-`14.0` input case stayed 8.4/16.8.
- `app.rs` (SHIM) — the routing:
  - **chad's fix:** `files_panel` container gained `.text_size(px(type_scale(Role::Nav).size))` (:2491) — the
    file-tree rows (which set no text_size) now inherit 12px instead of gpui's ~16px default. The `caption_header`
    keeps its own Caption(11).
  - **rail outlier:** the `RailLevel::Tab` session row (:4180) `px(13)` → `Nav` (13→12).
  - **consolidation (no visual change):** all 7 `.text_size(px(12.0))` → `Role::Nav` (replace_all); both
    `.text_size(px(11.0))` → `Role::Caption` (replace_all).
  - **deferred to #223 (untouched):** the 9 `px(13)` chrome (right-dock/git/diff/completion/menu) + the `px(9)`
    badge — confirmed by grep: 10 `px(13|9)` text_size literals remain, 0 `px(11|12)` remain.

**Deviations from design:** none — implemented the design's focused slice (Nav only; Body/Micro + the 13-chrome/
badge deferred to #223). The design already made that scope call.

**Checks:** `cargo fmt` clean; `cargo check -p marley --all-targets` ✓; `cargo clippy -p marley --all-targets --
-D warnings` exit 0; the 4 changed pure tests (type_scale_by_role incl Nav, the floor, weight_value,
fallback_cell) all PASS.

## Phase 3.5 — Inspect
2 general-purpose critics (pure seams + routing) + self-review. **Both CLEAN on correctness.** 1 LOW doc finding
fixed.

- **Critic 2 (routing) — the LOAD-BEARING files_panel inheritance CONFIRMED at the gpui 0.2.2 SOURCE level:**
  `Interactivity::paint` wraps child painting in `window.with_text_style(style.text_style(), |w| …paint
  children…)` (div.rs:1823), which pushes the refinement onto `text_style_stack` for the ENTIRE recursive
  child-paint closure (window.rs:2334-2343); a child with no `text_size` doesn't push (transparent); the leaf
  reads the FOLDED stack (text.rs:331 → window.rs:1440). So the files_panel container's `Nav(12)` cascades to the
  row-name leaves (grandchildren) → **12px** (chad's fix WORKS), while `caption_header` (a direct child that sets
  its own Caption(11)) overrides → the "Files" header stays 11. Confirmed by the shipping `line_row` precedent
  (:4871 sets Output size once, its spans inherit). Lenses 2-5 clean: replace_all hit ONLY the correct 9 Nav + 2
  Caption sites (each read + semantically confirmed); 0 `px(11|12)` remain; exactly 10 `px(13|9)` deferred sites
  untouched; the rail Tab row (:4185) is the sole px13 rail outlier → Nav; only 2 sizes changed (files 16→12,
  rail 13→12), all else size-preserving. Clean-room intact.
- **Critic 1 (pure seams) — cov/MSI 100 confirmed by `--list`:** `type_scale` yields only `→Default::default()`
  (UNVIABLE — no `Default` on `TextStyle`) → ZERO viable mutants; Nav adds none; all 4 arms (Command/Output/
  Caption/Nav) asserted by `type_scale_by_role` → cov 100. `weight_value`'s 3 mutants killed. `fallback_cell`:
  the `13.0` LITERAL has NO mutant (cargo-mutants doesn't mutate it) — the real set is the `>`-guard swaps (killed
  by the 0.0/-5.0 cases) + the `*0.6`/`*1.2` arithmetic swaps (killed by the positive 14/10 cases) + the unviable
  `→Default`; all 7 viable killed → MSI 100. The 4 changed tests pass.
- **F1 [LOW — REAL, FIXED] the `fallback_cell` rustdoc still said "falls back to 14.0"** (workspace.rs:104) while
  the diff changed the code + inline comment to 13.0 — a self-contradiction the diff introduced (the ticket's
  whole point is killing the stale 14). **Fixed:** rustdoc :104 "14.0" → "13.0". No behavioral/test/MSI impact.

No `failure-record`/`prevention-rule` — the only finding was a doc-staleness introduced + fixed within this diff
(not a shipped bug). Lenses covered: pure values, coverage, mutation (both pure files), the gpui inheritance
mechanism, replace_all safety, deferred-untouched, rail-row correctness, no-silent-size-change, clean-room §20.

## Phase 4 — Validate
**Tests:** the pure-seam pins were written at implement (the `type_scale_by_role` Nav full-value assert + the
`fallback_cell` 13-based guard cases) — no new tests to add (a routing/calibration change; the shim is
cov-excluded). Re-confirmed against the design plan: REQ-001/003 = type_scale_by_role (Nav={12,Normal});
REQ-004 = the floor test; REQ-005 = the fallback_cell guard; REQ-002 = the grep (0 `px(11|12)` remain).

**RUN:** `cargo nextest run -p marley` → **312 passed, 2 skipped** (incl the 4 changed pure tests; the routing
broke nothing).

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] 15/15** — cov ≥100% lines + MSI ≥100% on typography.rs
(the Nav arm covered by type_scale_by_role; type_scale has 0 viable mutants) + workspace.rs (fallback_cell's
guard/arithmetic mutants killed by the 0.0/-5.0/14/10 cases). Receipt written for `/commit`.

**Driven capture (REQ-001) — DEFERRED (env-considerate), values unit-proven + mechanism-confirmed:** chad is
STILL actively on a SHARED desktop (Warp with a live agent thinking, Teams frontmost, the desktop-share banner
present) — a driven capture would hijack his screen. Verified instead: the calibrated VALUES are unit-proven
(Nav=12 pinned + covered; fallback_cell 13) and the inspect critic confirmed at the gpui 0.2.2 SOURCE level that
`text_size` on the `files_panel` container cascades via `text_style_stack` to the file-tree row leaves (so the
files text goes ~16→12) + the rail Tab row (13→12), with `caption_header` overriding for the "Files" header
(stays 11). **This is a clean-room visual calibration — chad's eye is the final vibe check; OFFER the visual
confirm (files/sidebar text now ~12px, Warp-like) when his screen's free** — no separate ticket. gate-15 headless.

**Pre-existing failures:** none.

## Phase 5 — Complete
**Docs (§21):** CHANGELOG.md — #230 in `[Unreleased] ### Changed` (top). app_shell.md — updated the M1.E
type-scale CAVEAT to reflect #230 (added Role::Nav; files/sidebar calibrated via container inheritance; the 12/11
consolidation; fallback 14→13; #223 remains for 13-chrome + badge).

**Forge capture (§19):**
- `aar-submit` 11e662ad — completed, effectiveness 5. Lessons: (a) chad's "files text too big" root cause was
  NOT a px() literal but an UNSIZED `files_panel` inheriting gpui's ~16px default — the Explore agent found it by
  READING the render; a grep of px() literals would have MISSED it (the fix was to ADD a size, not change one);
  (b) gpui `text_size` CASCADES to descendants via the `text_style_stack` (confirmed at the gpui 0.2.2 source) so
  sizing a container sizes its unsized (grand)children — the idiomatic inheritance fix; (c) a struct-return fn
  with no Default (type_scale→TextStyle) has ZERO viable mutants → the per-Role test is for COVERAGE + regression,
  not mutant-killing; (d) the `13.0` literal in fallback_cell has NO cargo-mutants mutant — the real set is the
  `>` guard + `*0.6`/`*1.2` arithmetic swaps [run --list]; (e) scoped a FOCUSED slice (chad's fix + the SAFE
  same-value consolidation) + deferred the awkward 13-chrome/badge (Body-dup-of-Output + Micro + a calibration
  call) to #223 rather than route 10 sites blind under an env-blocked pixel check.
- `ticket-comment` on #223 (the scope split — #230 did the 12/11-band + files/sidebar; #223 remains for the
  13-chrome + badge). No failure-record/PR — the only finding was the fallback_cell rustdoc 14→13 doc-nit, fixed
  in-diff at inspect.

**Env note:** a driven vibe-check (files/sidebar text now ~12px, Warp-like) when chad's screen is free — no
separate ticket (values unit-proven + gpui-source-confirmed).

**Close + archive:** forge #230 → done; #223 stays OPEN (scope-commented). Local TICKET-230 → closed/. Pipeline
doc pair → completed/. Spec status → Phase 5 — Complete PASS.
