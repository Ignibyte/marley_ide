# 260-topbar-workspace-removal — Notes

- **Forge ticket:** #260 7d0d3756-9c85-4fdd-8fee-34e6b69d3566
- **AAR:** 0dcc41ed-4c50-4d1c-b22a-44c86752d953
- **Local ticket doc:** docs/planning/tickets/open/TICKET-260-topbar-workspace-removal.md
- **Pipeline spec:** 260-topbar-workspace-removal.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan
- **Request:** /goal batch, ticket 3 of 10. chad's live-review feedback:
  workspace presence at the top is unwanted duplication — the left rail owns
  it.
- **Classification / tier:** chore (UI removal), single slice.
- **Forge recall (§18.3):** AAR opened. Prior art in-context from the M13
  train (#233 rail-click switch, #235 indicator, #236 highlight, #244
  popover — the last two are what this removes).
- **Discovery (inline grep — 3 files, under the §18.2 threshold):**
  - app.rs: indicator render block 7097-7143 (name+branch read, label,
    absolute at slot-3, occlude, toggle listener closing 9 co-open overlays);
    popover render 7457-7534 (backdrop L/R dismiss + rows via
    `workspace_switcher_rows` + switch_project/sync/persist on click);
    `workspace_switcher_open` field :209 + init :1162; imports :111-112;
    consts :327-328; **the cockpit-tab cluster anchors at
    `topbar_icon_x(3,…) + TOPBAR_INDICATOR_WIDTH` (:7157)** — must re-anchor
    on removal or a 180px ghost gap remains.
  - titlebar.rs: `focused_workspace_indicator` :50 (+ tests :195-216),
    `SwitcherRow` :70 + `workspace_switcher_rows` :78 (+ tests :151-190).
  - `workspace_switcher_open` appears NOWHERE in the key ladder (click-away
    dismiss only) — removal is contained; no esc-handler orphan.
- **Decisions:** D1-D4 in the spec (pure removal; cockpit tabs take the
  vacated anchor; delete-don't-deprecate; #142 untouched).
- **Autonomy note:** /goal Stop-hook run — proceeding without a human pause.

## Phase 2 — Design

### Architecture / approach
Pure removal inside the render shim + the titlebar pure module. No new types,
no IO, no §14 surface. `branch_from_git_head` SURVIVES (callers at app.rs:2069
[#142 OS titlebar] + :4956 [footer cwd·branch]) — only the two dead fns leave
the import. Reference (§20) confirmed N/A — removal of Marley-specific chrome.

Removal hunks:
1. app.rs :7097-7143 — the whole `{ … }` indicator block → replaced by a
   one-line breadcrumb comment (#260).
2. app.rs :7457-7534 — the `if self.workspace_switcher_open { … }` popover
   block → deleted (breadcrumb).
3. app.rs :209 field + :1162 init — deleted.
4. app.rs :111-112 import — drop `focused_workspace_indicator` +
   `workspace_switcher_rows`.
5. app.rs :327-328 consts — deleted; :7157 cockpit-tab anchor →
   `topbar_icon_x(3, TRAFFIC_LIGHT_INSET, ICON_GAP)` (D2 — the tabs take the
   vacated slot; comment updated to name #260).
6. titlebar.rs — delete `focused_workspace_indicator` (+ test
   `focused_workspace_indicator_cases`), `SwitcherRow` +
   `workspace_switcher_rows` (+ test `workspace_switcher_rows_cases`).

### File manifest
| file | change |
|---|---|
| crates/marley_app/src/app.rs | 6 removal/edit hunks above |
| crates/marley_app/src/titlebar.rs | 2 fns + 1 type + 2 test blocks deleted |

### Regression test plan (removal — the proof is absence + unchanged survivors)
| REQ | Proof | How |
|---|---|---|
| REQ-001 | indicator/popover gone | driven capture of the top bar (no label at slot 3; click there opens nothing) + grep empty |
| REQ-002 | rail still switches | existing shell/rail tests green (nextest) + driven capture (click rail row → highlight moves) |
| REQ-003 | cockpit tabs at vacated anchor, functional | driven capture + click Details/Agents/Forge |
| REQ-004 | no dead code | `grep -rn "focused_workspace_indicator\|workspace_switcher_rows\|SwitcherRow\|workspace_switcher_open\|TOPBAR_INDICATOR"` empty; clippy -D warnings green |
| REQ-005 | #142/footer intact | `branch_from_git_head` callers :2069/:4956 unchanged; capture shows OS-titlebar label |
- No new unit tests (nothing new to test — deletions); the remaining titlebar
  tests (display_title, rail_tab_title, disambiguate, titlebar_label,
  abbreviate) keep the module's cov/MSI 100 on its surviving surface.
- Uncoverable: none.

### Risks / decisions
- R1 The #244 toggle listener was also the "close 9 co-open overlays" site —
  deleting it removes that closing behavior ONLY for indicator-clicks (which
  no longer exist); every overlay's own open path still manages exclusivity.
  No orphan.
- R2 Cockpit-tab anchor shift is a deliberate visual change (D2) — capture
  documents it.
- R3 gate:15 visual: the headed lane is #[ignore]/conditional (skip-clean in
  --diff, as in #262/#265) — no baseline churn blocks this.

## Phase 3 — Implement
- **Built exactly to manifest:** app.rs — indicator block → #260 breadcrumb
  comment; popover block (3511 chars) → breadcrumb (anchored python splice —
  asserts pinned the block's identity before cutting); field + init deleted;
  import slimmed to `{branch_from_git_head, disambiguate_labels,
  display_title, titlebar_label}`; consts deleted; cockpit-tab anchor →
  `topbar_icon_x(3, …)` with the #235 comment extended by the #260 note.
  titlebar.rs — the 2 fns + SwitcherRow (1574 chars) + the 2 test blocks
  (2675 chars) deleted.
- **Deviations:** none.
- **Verification:** fmt applied; `cargo check --workspace` green; REQ-004
  grep (`focused_workspace_indicator|workspace_switcher_rows|SwitcherRow|
  workspace_switcher_open|TOPBAR_INDICATOR`) already returns ZERO hits
  crate-wide.

## Phase 3.5 — Inspect
- **Critics run:** 2 parallel — A (Explore, removal-completeness/orphans),
  B (general-purpose, layout/overlay/persistence regression + tests).
- **Findings ledger:**

| # | sev | finding | verdict | action |
|---|---|---|---|---|
| A1 | MED | app_shell.md #235 bullet (613-620) + #244 bullet (697-708) describe the removed UI as CURRENT and name 4 deleted symbols; no #260 bullet exists | REAL doc drift | Phase 5 doc pass (planned §21 work) |
| — | — | code side | CLEAN — no orphans (all greps zero), workspace_switcher_open had 8 refs ALL inside deleted blocks, rail_highlight/TOP_BAR_H/MENU_ROW_H retain live callers, branch_from_git_head's 2 survivors untouched | none |

- **Verified (B):** nextest 368/368 (no roster pinned the deleted tests);
  clippy -D warnings exit 0; layout geometry: tabs 362→182px with no other
  element anchored after slot 3 — and the move FIXES a live-at-HEAD defect
  (at the 1024px default width the tabs sat fully under the centered
  occluding search bar; now clear by 8px — clearance strictly improves at
  every width); overlay exclusivity intact (the deleted #229 close-all only
  protected the switcher's own backdrop; keyboard openers are serialized by
  the key-router guard chain — and a latent HEAD asymmetry [no keyboard
  opener closed the switcher → stacked backdrops possible] is now moot);
  persistence: runtime-only, no settings key; provenance: purely subtractive
  (10 added lines = comments + the shrunk import + the minus-one-term
  anchor).
- **Fixes:** none required (the doc drift lands in Phase 5).

## Phase 4 — Validate
- **Tests added:** none (deletion change — per the Phase 2 plan; proof is
  absence + survivors). The deleted titlebar tests left with their subjects;
  no roster pinned them (critic-verified).
- **RUN:** `cargo nextest run --workspace` → **894/894 passed, 5 skipped**
  (the #[ignore] headed lane).
- **Driven live captures (PNGs read + asserted; scratchpad/260-*.png):**
  1. Top bar: NO "Marley · main" text anywhere — the left cluster is
     folder/+/✦ then the three cockpit tabs at the vacated slot-3 anchor
     (REQ-001 render half + REQ-003 position).
  2. Click at the old indicator x (0.14, 0.021): NO popover opens; the click
     lands on the relocated Agents cockpit tab, which opens its section
     (rail "Agents" row highlights, "no agents — ⌘⇧A to launch" renders) —
     REQ-001 interaction half + REQ-003 functionality.
  3. Rail workspace row click: row keeps the #236 active accent wash, app
     stable — REQ-002 (switch_project code untouched by the diff;
     unit-covered).
  4. Both captures show the footer `~/…/Marley · main` (#142 label) intact —
     REQ-005.
- **REQ-004 (RUN):** the 5-symbol grep returns zero crate-wide; clippy
  -D warnings exit 0 (critic-run + the gate's gate:2).
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] — 15/15** (incl.
  coverage ≥100%, MSI ≥100%). Receipt written.
- **Pre-existing:** upstream `block v0.1.6` note only.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG [Unreleased]/Changed entry; app_shell.md — the
  #235 bullet now records the indicator as removed-by-#260, the #244 bullet
  records the popover as removed-by-#260 (both keep their historical shipped
  text with a "removed" coda), + a #260 bullet added to the M-log.
- **Knowledge (§19):** AAR submitted (completed). No failures (both critics
  PASS clean); no new PR/AD — the removal exercised existing rules
  (delete-don't-deprecate, doc-drift caught by inspect per
  PR-claude-gate-change… family).
- **Bonus recorded:** the re-anchor FIXED a live-at-HEAD defect — at the
  1024px default width the cockpit tabs sat fully under the centered
  occluding search bar; clearance now strictly improves at every width.
- **Ticket:** TICKET-260 → tickets/closed/, forge #260 → done.
- **Archive:** spec+notes → docs/planning/pipeline/completed/.

## Phase 5 — Complete
- Docs updated; AAR capture; archive.
