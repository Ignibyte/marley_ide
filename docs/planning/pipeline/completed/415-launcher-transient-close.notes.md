# 415 — launcher transient-close gap — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-415-launcher-transient-close-gap.md
- **Pipeline spec:** 415-launcher-transient-close.spec.md

## Phase 1 — Plan
- **Request:** `/work` queue top — TICKET-415 (bug, M20): `close_transient_overlays`
  omits `agent_launcher`; found writing #318's draw-smoke (fleet drew over a
  lingering launcher). Auto-approved batch run (415, 414, 416, 223, 225, 226, 227).
- **Classification / tier:** bug; single work pipeline, one shippable slice
  (roster add + doc-comment + smoke tightening). No sub-split needed.
- **Recall (§18.3):**
  - `PR-claude-mouse-opened-modal-must-close-all-transient-overlays-001` — ONE
    shared roster called from every mouse-opened modal; never per-site closer
    lists; a new key-armed overlay must join the helper. (The launcher accreted
    the fix across 8 inspect findings #229/#312/#317/#323–#327 before #393
    extracted the helper — FROM the launcher's own closers, which is exactly why
    the launcher itself never made the list.)
  - F-#244 (failures.md:493) — the orphan class: a mouse-opened overlay that
    closes a partial set leaves a keyboard-live overlay stacked underneath;
    grep the precedent, don't re-derive the list.
  - #318 notes (completed archive): the smoke deliberately shipped per-index
    asserts because the launcher survives a transient-close; the strict
    five-flag equality upgrade was routed to THIS ticket. Fleet is toggle-scoped
    by design.
  - L-claude-318-driving-the-live-app-needs-an-active-user-session-001 — not
    triggered here: zero visual delta, headless smoke only.
- **Discovery:** (precise edit surface for Design)
  - `crates/marley_app/src/app.rs:7402-7429` — the helper + doc comment; roster
    lines carry per-entry `// #NNN:` provenance comments (idiom to match).
  - `crates/marley_app/src/app.rs:162` (`agent_launcher: Option<AgentLauncherState>`),
    `10068-10069` (launcher's own open already calls the helper first),
    `3677` (Esc arm), `21088-21108` (backdrop click-away dismiss) — the launcher
    is a transient centered modal, confirming D1.
  - `crates/marley_app/src/headless_drive.rs:14745-14794` — the #318 smoke
    (`recipe_b_overlays_open_and_draw_headless`): per-index assert + the
    in-code comment documenting this gap; cleanup calls the helper then
    toggle-closes fleet.
  - `crates/marley_app/src/headless_drive.rs:12576-12592` — the naming-draft
    choke-point test also exercises the helper (naming_pane arm); unaffected.
  - POC: `marley-web .../App.tsx:62` single `activeOverlay` enum;
    `Workspace.tsx:288,363` launcher-as-fleet-expansion. Model divergence
    recorded in the spec's Prior art.
- **Decisions:** D1 launcher joins; D2 fleet exempt + recorded; D3 one-hot
  equality. See spec.

## Phase 2 — Design

**Architecture.** Pure app-state membership fix inside `marley_app` — no new
types, no IO, no seams crossed. The roster (`close_transient_overlays`,
app.rs:7408, a `mutants::skip` shim) gains `self.agent_launcher = None;` with a
`// #415:` provenance comment matching the per-line idiom, and its doc comment
gains the membership rule + fleet's recorded exemption. §20 confirmed as
planned: behavior class = Warp's transient-overlay exclusivity (family
precedent #181/#393); launcher chrome itself is Marley-specific; no fork source
read; `N/A` React-first holds (no visible chrome delta; the POC can't represent
the stacking state).

**Call-site audit (the phase-plan question).** Five production callers — all
close-then-open a DIFFERENT modal (`open_add_project_menu` 7436,
`open_section_menu` 7453, add-to-pane menu 7768, `begin_naming_pane` 7802) or
the launcher itself (`new-agent` arm 10068, clear-then-set: no-op interaction).
Two test callers (choke-points 12591, smoke cleanup 14784) want the clear.
**Nothing depends on the launcher surviving a transient-close.**

**File manifest.**
- `crates/marley_app/src/app.rs` — (1) roster line `self.agent_launcher = None;
  // #415: …` after the `open_problems` line; (2) doc comment gains: membership
  rule (every transient modal/picker with a key-arm joins the roster; #415 added
  the launcher — extracted FROM its closers at #393, it never closed itself) +
  the exemption line (persistent docks — fleet/files/forge — stay; fleet is
  toggle-scoped, #318).
- `crates/marley_app/src/headless_drive.rs` — `recipe_b_overlays_open_and_draw_headless`
  (14746): (1) loop-head comment: per-index → one-hot rationale + #415; (2) the
  open assert becomes `assert_eq!(flags, [i==0, i==1, i==2, i==3, i==4], …)`;
  (3) cleanup gains direct asserts — after `close_transient_overlays()`:
  `assert_eq!([palette, finder, history, agent_launcher], [false;4])` (uniform,
  REQ-001 direct), and at the fleet iteration `assert!(fleet_open)` BEFORE the
  toggle (REQ-003's exemption pin — without it, a helper that wrongly cleared
  fleet would pass silently since the `if fleet_open {toggle}` guard reads the
  already-cleared flag); (4) post-loop all-five-false tail assert.

**Regression test plan.**

| REQ | Test (RUN at validate) | Assert |
|---|---|---|
| REQ-001 | `recipe_b_overlays_open_and_draw_headless` cleanup | after the helper: `[palette, finder, history, launcher] == [false;4]` every iteration (launcher's flip is the #415 line); plus iteration 4's one-hot shows `launcher == false` while fleet is up — the exact #318 observed stacking, asserted extinct |
| REQ-002 | same test, per-iteration | `assert_eq!(flags, one-hot(i))` for all five verbs |
| REQ-003 | same test, fleet iteration | `fleet_open == true` AFTER the helper ran, BEFORE the toggle (direct exemption pin); doc-comment review at inspect |
| tail | same test, post-loop | all five flags false (the last iteration's cleanup is asserted, not inferred) |

Mutation posture: the roster is a `mutants::skip` shim (existing, justified —
state clears, no logic); the smoke's equality asserts are the behavioral pin
(L-claude-318-extraction-testing-posture). Coverage: the new roster line is
exercised by both headless tests. No uncoverable paths. No visual/AX rows —
zero pixel delta.

**Risks.** None load-bearing. Worst case is a hidden flow preferring a
surviving launcher — the audit found none, and Esc/click-away already close it
freely.

## Phase 3 — Implement
- **React-first: N/A** (per spec — no UI delta; the POC cannot represent the
  stacking state).
- Built exactly to the manifest:
  - `app.rs`: `self.agent_launcher = None; // #415: the #181 launch picker — the
    roster's own ancestor` appended to the roster; doc comment gained the
    Membership (#415) paragraph (rule + fleet/docks exemption).
  - `headless_drive.rs`: the smoke now asserts one-hot equality per iteration,
    cleanup asserts the four transients all-false after the helper (REQ-001),
    pins `fleet_open` still up at the fleet iteration BEFORE the toggle
    (REQ-003 — the guard-masking rationale in-code), and a post-loop all-five-
    false tail assert; loop-head + outer doc comments updated (#415 provenance,
    per-index assert retired).
- Deviations: none. `cargo check --workspace --all-targets` green (the
  `block v0.1.6` future-incompat note is pre-existing transitive, not ours).

## Phase 3.5 — Inspect

Two parallel critics (correctness; state-integrity + simplification + clean-room),
both instructed to verify concretely. Both ran the smoke + adjacent tests live
(pass). Full suite run by critic 1: 1043+13+3 pass.

| # | Sev | Finding | Verdict | Fix |
|---|---|---|---|---|
| F1 | LOW | `open_file_ref_menu` (app.rs:5513) + the terminal right-click listener (:19940) still hand-close the fossil trio {completion, renaming_tab, naming_pane} instead of the roster — omitting launcher AND every picker. Launcher-under-menu is unreachable live (its backdrop occludes + dismisses both buttons), but picker-under-menu IS reachable (centered cards have no scrim; the terminal/link stays exposed) — the #393 orphan class, live at both sites. | **REAL** (verified both sites myself; no test pins the partial set — both are masked shims) | Both sites now call `close_transient_overlays()`; the helper's stale #398 "both do [hand-clear]" parenthetical updated. F-block appended to failures.md. |
| F2 | LOW | The retired `assert!(flags[i])` was an accidental OOB tripwire for an appended 6th verb; the new fixed-size one-hot equality passes VACUOUSLY for it (both arrays all-false). | **REAL** (traced: from_fn alone does not restore it) | `expected` now `std::array::from_fn(\|j\| j == i)` (kills the literal-4 vs `fleet_i` inconsistency) + explicit `assert!(expected[i], "verb table outgrew the flags mirror")` tripwire with rationale comment. |
| F3 | LOW (info) | Post-loop tail could reuse `browser::overlay_is_up` (all-26-state check, strictly stronger). | **REJECTED** — deliberate: the smoke's stated inventory is the five Recipe-B overlays; widening couples it to unrelated overlay regressions and muddies failure attribution. Recorded here instead. |

Lenses clean: launcher-survival dependence (25 sites swept), one-hot trace vs
real dispatch arms (all five verified, incl. `fleet_open`'s three writers),
snapshot freshness, new-agent clear-then-set ordering, typed-draft drop
semantics (launcher is discard-on-any-dismissal since #229; the protected tier
— fleet_dispatch_draft — is a different contract with refusal arms, not a
roster exemption alone), clean-room provenance.

Ledger appends: F-claude-415-roster-extraction-left-preexisting-hand-closers-unswept-001
(failures.md). No new PR- rule — PR-claude-mouse-opened-modal-must-close-all-
transient-overlays-001 already owns the class; the F-block records the
extraction-without-sweep nuance.

## Phase 4 — Validate
- **Tests RUN:** `cargo nextest run --workspace` → **2147 passed, 5 skipped, 0
  failed** (14.2s; includes the tightened
  `recipe_b_overlays_open_and_draw_headless` carrying REQ-001/002/003 + the
  tripwire + tail). `cargo test --workspace --doc` → 0 (no doctests; ok). The
  smoke was additionally run solo at inspect: PASS.
- **Live drive (mandatory attempt, environment-blocked):** bundled fresh
  (`bundle-app.sh debug` → target/Marley.app, 15:36), launched, `AX_TRUSTED`
  confirmed. The app came up ALIVE with **five 0×0 off-screen windows** (CG
  window list evidence in-transcript) — the detached-session condition of
  L-claude-318-driving-the-live-app-needs-an-active-user-session-001: without
  an active desk/CRD session the window server grants no geometry, so
  `drive.swift find` → NONE and no pixel capture is obtainable. NOT silently
  skipped: the changed behavior is fully asserted in the HEADLESS lane (the
  README's prescribed lane for behavioral state changes — the smoke drives all
  five verbs + the roster); the two converted mouse listeners are masked shims
  whose roster call is the same helper those tests pin (the #398 posture).
  The live right-click-over-picker confirmation rides the #417 after-capture
  battery (Deliberate; needs a live session) — noted for complete.
- **Gate:** `scripts/gates.sh --diff` → first run RED on gate:1 rustfmt only
  (14/15 green incl. coverage 100% + MSI 100%); `cargo fmt --all` applied
  (whitespace-only reflow of the new comments/asserts); re-run → **GATE GREEN
  [diff], 15 passed 0 failed**, receipt written. (Process note: the first run's
  Bash exit code was masked by a `| tail` pipe — read the verdict from the
  output, and don't pipe the gate next time.)
- Pre-existing: the `block v0.1.6` future-incompat cargo note (transitive dep) —
  not in scope.

## Phase 5 — Complete
- **CHANGELOG:** entry added under Unreleased/Fixed (roster membership + fossil
  sweep + smoke tightening; suite + gate results recorded).
- **Architecture docs:** `docs/marley_architecture/app_shell.md` — the
  one-modal-discipline block gained the #415 membership rule + docks exemption +
  fossil-sweep note; the #318 smoke paragraph updated (gap closed, one-hot +
  tripwire + exemption pin + tail).
- **Parity sync:** N/A ticket (no UI delta; POC unaffected — single-enum model).
- **Ledger appends:** F-claude-415-roster-extraction-left-preexisting-hand-closers-unswept-001
  (at inspect); AD-claude-415-transient-roster-membership-rule-001;
  L-claude-415-fixed-size-equality-asserts-need-a-growth-tripwire-001.
- **Ticket:** TICKET-415 → `tickets/closed/`, status closed; backlog clean.
- **Live-capture ride-along:** the right-click-over-picker pixel confirmation
  rides the #417 after-capture battery (Deliberate — needs a live session).
- Archived to `docs/planning/pipeline/completed/`.
