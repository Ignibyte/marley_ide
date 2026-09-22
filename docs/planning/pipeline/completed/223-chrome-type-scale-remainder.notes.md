# 223 — chrome type scale remainder — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-223-chrome-text-type-scale.md
- **Pipeline spec:** 223-chrome-type-scale-remainder.spec.md

## Phase 1 — Plan
- **Request:** batch position 4: route the remaining hardcoded chrome text
  sizes through type_scale. Auto-approved batch run.
- **Classification / tier:** chore, single slice (mechanical routing + a
  5-arm table extension).
- **Recall (§18.3):**
  - PR (prevention-rules.md:927) — struct-return-no-Default trap: type_scale
    yields no viable mutants; FULL-value asserts per arm are the only guard.
    typography.rs's own test comments carry the same rule in-file.
  - #337 (content zooms, chrome does not) — binding for all new roles; its
    test is the decision and extends to the new arms.
  - #230 slice (chrome-type-scale.spec.md, completed) — the precedent that
    minted Nav + routed ~20; the three call-shape idioms (inline / hoisted /
    caption_header FONT_SIZE_DEFAULT free-fn pattern with the recorded
    honesty argument).
- **Discovery (Explore, very thorough — the ticket's July anchors were
  stale):** 28 hardcoded sites, ALL in app.rs, ALL chrome (no content
  straggler; #337 already fixed the completion popup). By value: 13.0×10
  (panel bodies/containers: 1220, 1276, 6159, 6169, 6186, 6204, 19453, 21216,
  21383, 22057), 10.0×6 (fleet chips: 1273, 1280, 1293, 1316, 1323, 1344),
  11.0×4 → Caption (1209, 10800, 19530, 19573), 9.0×3 (badges: 1396, 1413,
  21797), 12.0×2 → Nav (1380, 10771), 15.0×2 (headlines: 19473, 19524),
  22.0×1 (launcher title: 10765). Routed set today: 41 calls (14 inline Nav,
  2 inline Caption, 1 Output, 8 hoisted, 15 px(self.font_size) content, 1
  param). The 12 fleet-rail literal sites live in &self-less free fns →
  caption_header pattern. `fallback_cell`: the ticket's stale-14.0 item is
  ALREADY DONE (#337 sources FONT_SIZE_DEFAULT); one doc line still says
  "13.0" bare. NO tests pin any literal (zero test churn; the only related
  pins are typography.rs's own + fallback_cell_scales_and_guards).
- **Decisions:** D1 reuse-before-mint; D2 fixed chrome roles; D3 full-value
  pins; D4 zero-delta + Normal weights; D5 fallback_cell code change dead
  (superseded — recorded as a win). See spec.

## Phase 2 — Design

**Approach.** Pure-table extension + mechanical routing; §20 N/A holds (values
preserved, model unchanged). Role names locked: **`Panel` 13.0** (panel/body
chrome band), **`Chip` 10.0** (chip/meta band), **`Badge` 9.0** (smallest
badge band), **`Headline` 15.0** (empty-state/card headlines), **`Display`
22.0** (launcher title). All fixed-size, `TextWeight::Normal` (D4 — no routed
site carries its weight through the literal; existing font_weight calls stay
caller-side). No name collisions with Command/Output/Caption/Nav.

**Call-shape rule (matches the three house idioms):**
- `&self` sites → inline `px(type_scale(Role::X, self.font_size).size)` (the
  existing inline Nav/Caption idiom; the arg is ignored for chrome — same
  honesty as today's sites).
- `&self`-less free fns (the 12 fleet-rail sites in `fleet_rail_body` /
  `fleet_seat_card`) → the `caption_header` pattern: per-fn hoisted binding
  `let chip = px(type_scale(Role::Chip, crate::font_zoom::FONT_SIZE_DEFAULT).size);`
  when a role repeats in the fn (Chip ×6, Badge ×2 in seat_card), inline
  otherwise. The pattern's recorded honesty argument (caption_header doc)
  carries over.

**Route table (28 rows — line anchors from the Phase 1 inventory; verify at
edit time, the file has drifted +~50 lines since #414/#415):**
- → `Caption` (existing): 1209 (fleet misconfig reason), 10800 (launcher
  recent path), 19530 (browser card caption), 19573 (browser retry line).
- → `Nav` (existing): 1380 (fleet question prompt), 10771 (RECENT WORKSPACES
  label).
- → `Panel` 13.0: 1220 (fleet empty hint), 1276 (seat title), 6159/6169/6186
  (Details empty/rows/placeholder), 6204 (Agents list), 19453 (empty-workspace
  hint rows), 21216 (git panel), 21383 (diff overlay), 22057 (context-menu
  box).
- → `Chip` 10.0: 1273, 1280, 1293, 1316, 1323, 1344 (fleet chips/meta).
- → `Badge` 9.0: 1396, 1413 (fleet phase/dispatch lines), 21797 (agents-tab
  count badge).
- → `Headline` 15.0: 19473 (empty-workspace headline), 19524 (browser card
  headline).
- → `Display` 22.0: 10765 (launcher title).

**Manifest.**
- `crates/marley_app/src/typography.rs` — 5 new Role variants + arms + docs
  (the #337 content/chrome paragraph gains the new-band sentence); extend
  `type_scale_by_role` (full TextStyle per new arm) +
  `t337_type_scale_content_zooms_chrome_does_not` (fixed at 8/20/32 + one
  NaN/0 probe per new role).
- `crates/marley_app/src/app.rs` — the 28 routes per the table.
- `crates/marley_app/src/workspace.rs` — `fallback_cell` doc line names
  `FONT_SIZE_DEFAULT`.

**Test plan.**

| REQ | Check (RUN at validate) | Assert |
|---|---|---|
| REQ-001 | negative grep `text_size\(px\([0-9]` over marley_app/src | 28 BEFORE → **0 AFTER** (in-transcript assert) |
| REQ-002 | extended `type_scale_by_role` | full TextStyle {size, weight} per new arm: (13,Normal)(10,Normal)(9,Normal)(15,Normal)(22,Normal) |
| REQ-003 | extended `t337_…` | new roles fixed across 8/20/32 + NaN/0 probes |
| REQ-004 | `cargo nextest run --workspace` + the 28-row inspect ledger | suite green, zero test edits; 1:1 value mapping reviewed |
| REQ-005 | rustdoc gate + review | fallback_cell doc names the const |

**Notes for inspect:** the legibility-floor test
(`type_scale_terminal_text_clears_legibility_floor`) guards CONTENT only —
Chip 10 / Badge 9 are deliberately sub-12 chrome bands; do not "fix" the
floor to cover them.

**Risks.** None load-bearing. Role naming is judgment (recorded, reversible);
line anchors drift (edit by content, not line).

## Phase 3 — Implement
- **React-first: N/A** (value-identical routing; no visible delta).
- typography.rs: five new Role variants (Panel/Chip/Badge/Headline/Display)
  with per-variant docs + the #223 zero-delta comment block at the arms.
- app.rs: all 28 sites routed. Fleet cluster (12): hoisted `caption_size` /
  `panel_size` (rail body) and `chip_size` / `badge_size` (seat card) via the
  caption_header FONT_SIZE_DEFAULT pattern — the hoisted names take the house
  `nav_size` shape because the seat card already had a Div named `chip`
  (shadowing hazard avoided); Nav prompt inline. The 16 `&self` sites: inline
  `px(type_scale(Role::X, self.font_size).size)` (the existing inline idiom).
- workspace.rs: fallback_cell doc names `FONT_SIZE_DEFAULT` (+ the went-stale-
  twice note); code untouched (already sourced — #337, D5).
- **Negative smoke:** BEFORE 28 → **AFTER 0** literal sites (in-transcript).
- `cargo check --all-targets` 0 errors; `cargo fmt` clean.
- Deviations: none beyond the recorded binding-name choice.

## Phase 3.5 — Inspect

Two parallel critics (value-mapping auditor; idiom+simplification+provenance).

**The 28-row mapping ledger (REQ-004 evidence): ZERO mismatches** — every
site's old literal equals its new role's table size exactly (Panel×10 @13,
Chip×6 @10, Caption×4 @11, Badge×3 @9, Nav×2 @12, Headline×2 @15, Display×1
@22; full row-by-row table in the auditor's report, verified against the diff
pairs and the typography arms). No missed literal (multi-line/`as f32`/
`gpui::px` variants swept: zero); no double-route; no wrong grouping; the
`chip` Div vs `chip_size` Pixels bindings distinct; all 16 `&self` sites in
genuine methods; the free-fn FONT_SIZE_DEFAULT pattern matches caption_header
including its honesty rationale; clippy 0 warnings.

| # | Sev | Finding | Verdict | Action |
|---|---|---|---|---|
| F1 | MAJOR | Five new arms have zero test coverage yet; the module doc's "cov/MSI 100" is transiently false | REAL, PHASE-4-OWNED by design (the pipeline writes tests at validate; REQ-002/003 specify exactly these) | Validate MUST deliver; must-not-drop. |
| F2 | MODERATE | The design-promised #337-paragraph sentence was missed at implement; module header + t337 banner under-enumerated | REAL (doc gap) | Fixed — module doc line, the #337 paragraph invariant sentence ("everything except Command/Output"), the t337 banner. |
| F3 | MINOR | cockpit_body had 4× inline Panel where the house `nav_size` precedent hoists at ×2 in `&self` fns (the design's blanket inline rule contradicted house style) | REAL | Fixed — `panel_size` hoisted in cockpit_body; the four OTHER Panel sites (single-use per fn) stay inline, correctly. |
| F4 | MINOR | fleet_seat_card inlined its ×1 roles producing two 6-line rustfmt blobs while its sibling fn hoisted ×1 roles | REAL (inconsistent rule application) | Fixed — `panel_size`/`nav_size` hoisted in seat_card; blobs gone. rail_body's ×1 hoists recorded as the accepted deviation (reads better). |
| F5 | FLAG→P5 | Stale role enumerations: crate-map.md:114, app_shell.md:166 ("Caption/Nav stay FIXED"), the AD-#337 block | REAL, Phase-5 remit | Picked up at complete. |
| F6 | INFO | marley_webview_probe literals (12/14) out of scope (probe bin); app.rs:1134 svg().size is geometry not text | ACCEPTED | Recorded. |

Post-fix: compile 0 errors, fmt clean, Panel-inline count 8→4 (exactly the
cockpit_body four converted). Provenance clean (values 1:1 Marley's own).

## Phase 4 — Validate
- **Tests written:** `type_scale_by_role` extended with FULL TextStyle pins ×5
  new arms (each carrying its #-provenance comment);
  `t337_…_content_zooms_chrome_does_not` extended with a
  [8/20/32/0/NaN]×5-role fixed-size loop (backs the fleet free-fns'
  FONT_SIZE_DEFAULT honesty).
- **Runs:** `cargo nextest run --workspace` → **2148 passed, 5 skipped, 0
  failed**; doctests 0 ok; typography solo 4/4.
- **Negative smoke (REQ-001):** literal-site grep = **0** across
  marley_app/src (was 28).
- **Gate:** first `--diff` run **RED on clippy** — `panel_size`/`nav_size`
  unused in fleet_seat_card: the implement-phase python replace for the two
  inline blobs missed because rustfmt had reshaped them, and my "blobs
  remaining: 0" counter counted the PRE-fmt string — a vacuous check (the
  L-420 class, caught by the gate as designed). Wired both sites to their
  bindings; re-run → **GATE GREEN [diff], 15 passed 0 failed** (coverage
  100%, MSI 100% incl. the new arms), receipt written.
- **Live drive:** value-identical routing — there is NO pixel delta to
  observe even live; and the no-active-session block stands (0×0 windows
  evidenced twice today in-transcript, #415/#414 validates). Not skipped
  silently: the zero-delta proof is the 28-row exact-value ledger + the
  full-value pins, which is the strongest available oracle for this class.
- Pre-existing: `block v0.1.6` note (transitive) — not in scope.

## Phase 5 — Complete
- **CHANGELOG:** entry (roles, zero-delta proof chain, the vacuous-counter
  gate catch).
- **Architecture docs:** crate-map.md #337 sentence extended (nine roles,
  #223 closed); app_shell.md typography bullets — the "REMAINING #223" line
  became the CLOSED record and the #337 line enumerates the new bands (the
  inspect F5 staleness fixed).
- **Parity sync:** N/A (no visible delta; POC untouched).
- **Ledger appends:**
  AD-claude-223-nine-role-type-scale-chrome-is-everything-but-command-output-001;
  L-claude-223-count-the-final-state-not-your-guess-of-its-shape-001.
- **Ticket:** TICKET-223 → tickets/closed/; backlog clean.
- Archived to docs/planning/pipeline/completed/.
