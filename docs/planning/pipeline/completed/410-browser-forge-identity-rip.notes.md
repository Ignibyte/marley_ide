# 410 — Browser forge-identity rip — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-410-browser-forge-identity-rip.md
- **Pipeline spec:** 410-browser-forge-identity-rip.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** `/work 410` (auto-approved through commit per session goal). TICKET-410 —
  product-rip slice 1: the browser loses its forge identity; the generic embedded-browser
  infrastructure (wry child, z-order shim, #406 lifecycle machine) survives. MUST precede
  TICKET-411 (the browser chain holds three independent edges into `marley_forge_client`;
  deleting the crate first would take the pane down as collateral).
- **Classification / tier:** chore, M30, work pipeline, one shippable slice.
- **Recall (§18.3):**
  - `AD-claude-embedded-browser-substrate-001` — wry-as-child WKWebView is the generic
    substrate; hide-shim + focus/IME handoff proven by #402. The rip must not disturb it.
  - `BF-claude-wall-admit-vs-parsed-host-divergence-001` (#404) + the paired PR: a hand-rolled
    admit parse vs the consumer's real parse diverges on hostile authorities; any surviving
    origin predicate must assert on the value the consumer actually uses.
  - `PR-claude-callback-stored-inside-owned-resource-captures-weak-001` — binds the surviving
    #405/#406 wry wiring; the rip must not convert weak captures to strong while re-plumbing.
  - Failure `M29 #403` (React POC) — changing browser chrome fixtures leaves type-correct
    callers behind; sweep every caller of the removed identity, not just the definition.
- **Discovery (Explore sweep vs HEAD e729128):** every ticket-claimed line ref verified exact;
  two missed sites found (struct-literal init app.rs:2735; field decl :539). Full seam:
  - `forge_web_base` chain: app.rs 537-539, 2489-2491, 2735, 2928-2933, 10913-10914,
    11066-11071, 11085-11096, 11111-11126, 19077-19081; oracle reads
    headless_drive.rs:11697/:11710/:12043; prose browser.rs:193.
  - **Ticket correction 1:** `mcp_config.rs` (197 lines, sole pub fn `mcp_json_path`) has TWO
    call sites; the boot one (app.rs:2476) also feeds the sprint `forge_client` (:2480-2483)
    and fleet-brain `endpoint_for_brain` (:2513) — 411-scope. Module deletion RE-SCOPED to
    411; 410 cuts only the forge_web_base legs. Its 9 units (7 pure-path + 2 forge-coupled
    :147/:185) go to 411 with it.
  - **Ticket correction 2:** `same_web_origin` is DEFINED in `marley_forge_client/src/lib.rs:185-187`
    (webview_shim.rs:74 is its sole caller, inside the wry nav handler); it leans on private
    `web_origin_of` (lib.rs:198-211) which carries the #404 parsed-host loopback re-check and
    the `url` crate dep. `marley_app` has NO direct `url` dep — F2 must pick promotion vs
    pin-drop.
  - Copy: browser.rs:29-31 `placeholder_detail()` + exact-copy test :207-215; rendered
    app.rs:19095-19096; headline "No page loaded" survives. POC twin
    `BrowserPlaceholder.tsx:23` (same literal), `BrowserPane.tsx:33` ORIGIN stand-in.
  - CommandId(31): app.rs:10432-10442, "forge" keyword at :10439 only; palette.rs:157/:320
    assert action only — keyword removal breaks no test.
  - `orchestration.web_url` DEAD confirmed: definition orchestration.rs:20; every reader is a
    test/fixture (orchestration.rs, settings.rs, fleet_live.rs:195, headless_drive.rs:41,
    tests/fleet_livewire.rs:200); production reads `brain_endpoint` only. Serde-forward-
    compatible to drop.
  - Test ledger: RETIRE 1 drive (`browser_boot_derivation_feeds_pane_headless` :11680-11715);
    REPOINT 2 (`placeholder_copy_is_exact` browser.rs:209;
    `browser_retry_with_absent_config_unconfigures_headless` :12038-12077); SURVIVE 10 generic
    drives + 20 browser_state units (2 semantically at risk: retry_plan/effective_retry_base
    arms :641/:661 — F1-dependent) + 6 browser.rs units. Probe edge (browser_state.rs:29,
    app.rs:11049, drives :11929/:11951/:12136-12172) stays WIRED — 411 re-homes.
  - Neither-ticket vocabulary flagged: Browser section's ＋-default `OpenCockpit(Forge)`
    (context_menu.rs:160-166, tabs.rs:111-115) → recorded into TICKET-411's doc.
  - `marley_webview_probe`: zero forge hits. False-positive "forge/forgery" English verbs
    listed (grid_layout, fleet_rail:99, app.rs misc, keymap:985) — do not touch.
  - Phase-5 arch-doc targets: embedded-browser-model.md (§Q3 :90-127 primary, Q4 :146-147),
    orchestration-shell.md §8 :165-190, app_shell.md :822-958, pane-composition-model.md
    :22/:284/:298-312, roadmap.md :219-228/:252/:277; gates.sh:227-231 comment cites
    same_web_origin (F2-dependent). settings.md needs nothing.
- **Decisions:** D1-D6 locked in spec; forks F1 (origin source / web_url fate), F2 (predicate
  re-home vs pin-drop), F3 (replacement copy) → Phase 2.

## Phase 2 — Design

### Fork resolutions
- **F1 — origin source: NONE (none-until-a-future-URL-feature), `web_url` DELETED.**
  The rip does not grow a settings-driven browser feature mid-chore; a future URL
  feature designs its own source. Concretely:
  - `RootView.forge_web_base` field deleted outright (decl :539, init :2735, boot
    derivation :2489-2491, test-hook getter :2931-2933). No replacement register:
    the mount gate calls `mount_plan(None)` and the Retry arm calls
    `retry_plan(None, mounted)`, each with a comment naming the literal `None` as
    THE seam a future URL feature fills. This keeps every pure decision seam
    CALLED (a test-only-referenced `pub(crate)` fn would trip dead_code at
    `-D warnings`), keeps the machine compiled + total, and adds zero dead state.
  - `derive_forge_web_base_now` (:11111-11126) deleted. `effective_retry_base`
    deleted from `browser_state.rs` (+ its arms test :661): its transient-vs-
    absent `.mcp.json` disambiguation is derivation-specific semantics with no
    input left. `retry_plan` SURVIVES untouched (+ its arms test :641): generic
    desired-vs-mounted planning. The app Retry match keeps all three arms total;
    `ReAttach` arm performs the same teardown as `Unconfigure` with a contract
    comment (unreachable with a `None` source per retry_plan's tested arms — kept
    total instead of panicking, §14).
  - Caption (:19077-19081): holder-only (`browser_view.origin()`); the
    AttachFailed-no-holder case shows an empty caption — production-unreachable
    post-rip (nothing attaches without a source).
  - `orchestration.web_url` deleted (field + both doc phrases). Safe: zero
    production readers (P1 discovery), no `deny_unknown_fields` on
    `ProjectOrchestration` → old settings files with the key still parse.
    Fixture repoints: orchestration.rs tests (assert via `brain_endpoint`),
    settings.rs round-trips :1227/:1232/:1262, fleet_live.rs:195,
    headless_drive.rs:41, tests/fleet_livewire.rs:200.
- **F2 — predicate re-home: MOVE `same_web_origin` + private `web_origin_of` into
  `browser.rs` (verbatim; doc adapted), promote `url = "2"` to a direct
  `marley_app` dep** (already in-lock via forge_client — zero new supply chain).
  Rationale: browser.rs is covered+mutated (webview_shim.rs is the documented
  §0 exclude — a pure predicate must not live there); the #404 parsed-host
  loopback re-check (BF-claude-wall-admit-vs-parsed-host-divergence-001) stays
  enforced by moved tests. `webview_shim.rs:74` repoints to
  `crate::browser::same_web_origin`; module doc :9 updated. forge_client keeps
  its own copy until 411 deletes the crate — TEMPORARY duplication, one ticket
  long, recorded here. gates.sh:230's exclude prose ("decisions live in the
  tested browser.rs + same_web_origin pure seams") becomes MORE true — no edit.
  Moved tests: `web_origin_of_arms_exact` (lib.rs:858) + the 4 same_web_origin
  units (:1117/:1131/:1154/:1173) → browser.rs test mod.
- **F3 — replacement detail copy (draft, settled visually in React first):**
  "The embedded browser awaits a page source." — states the standing fact,
  promises no arrival, no Forge. Headline "No page loaded" survives. If the
  visual pass adjusts phrasing, both sides move together (the literal is pinned
  by Rust `placeholder_copy_is_exact` + the POC component; no POC-side test
  exists — component + screenshot only).

### Reference (§20) — confirmed
Still N/A — Marley-specific product removal; no reference-app behavior matched.
Prior-art stance executed as adoption-move: the `url` crate origin semantics
(already adopted at #404) relocate with their tests; nothing reinvented.

### File manifest
**React half (built + visually approved FIRST — pnpm dev @ 5173, screenshot READ):**
- `artifacts/marley-ide/src/components/views/BrowserPlaceholder.tsx` — :23 new
  detail copy (F3); :8 doc paragraph drops the `.mcp.json`/#404-arrival story.
- `artifacts/marley-ide/src/components/views/BrowserPane.tsx` — :33 ORIGIN
  stand-in recaptioned (post-forge: a mounted origin awaiting a future URL
  feature); :13/:17 doc forge mentions reworded.
  (`BrowserView.tsx`'s `<ForgePane/>` tab = design-ahead non-porting chrome —
  untouched, 411/cockpit scope.)

**Rust half (crates/marley_app unless noted):**
- `Cargo.toml` — + `url = "2"` (match existing dep style).
- `src/app.rs` — delete field decl :537-539 + init :2735 + boot derivation
  :2485-2491 (comment block included) + test hook :2928-2933 +
  `derive_forge_web_base_now` :11105-11126; mount gate :10913-10914 →
  `mount_plan(None)` + future-seam comment; Retry arm :11062-11099 →
  `retry_plan(None, mounted)` (no re-derive; ReAttach arm = total teardown +
  contract comment); caption :19075-19081 → holder-only; CommandId(31) keywords
  :10439 drop `"forge"`; placeholder render-arm comment :19091-19092 reword
  (drop "`.mcp.json` derivation" phrase).
- `src/browser.rs` — `placeholder_detail()` new copy + doc reword (:26-31);
  module-doc/idiom prose :9-14, :21, :36 drop forge_view/`forge_empty_hint`/
  "Forge web view" references; ADD `same_web_origin` + `web_origin_of` (moved,
  docs adapted: "mounted origin" vocabulary, keep the #404 divergence story) +
  their 5 moved tests; repoint `placeholder_copy_is_exact` :213.
- `src/browser_state.rs` — delete `effective_retry_base` + arms test :661;
  doc prose :344 (`forge_empty_hint` register) reword. `retry_plan` + :641
  untouched.
- `src/webview_shim.rs` — :74 call repoints to `crate::browser::same_web_origin`;
  module-doc :9 updated.
- `src/orchestration.rs` — delete `web_url` field :18-20; module/struct docs
  :2, :10 reword; tests repoint to `brain_endpoint`.
- `src/settings.rs` — round-trip fixtures :1227/:1232/:1262 drop `web_url`.
- `src/fleet_live.rs` — fixture :195 drops `web_url`.
- `src/headless_drive.rs` — RETIRE `browser_boot_derivation_feeds_pane_headless`
  :11680-11715 → REPLACE with `browser_boot_ignores_mcp_json_headless` (fixture
  `.mcp.json` present → phase `None` after opening B; REQ-001); REPOINT
  `browser_retry_with_absent_config_unconfigures_headless` :12038-12077 →
  `browser_retry_without_source_unconfigures_headless` (no fixture semantics;
  same assertions: phase → None + gen bump); fixture :41 + comment :12043.
- `crates/marley_app/tests/fleet_livewire.rs` — fixture :200 drops `web_url`.
- `src/grid_layout.rs` — codec doc :232 reword (prose only, no codec change).
- `src/content.rs` — `resolve_browser` doc :367 reword.
- `src/lib.rs` — :72 `mod mcp_config;` comment reword ("boot-time `.mcp.json`
  resolver for the forge client + brain bearer — module retires with 411").

### Regression test plan
| REQ | Test(s) | Kind |
|---|---|---|
| REQ-001 | NEW `browser_boot_ignores_mcp_json_headless` (fixture present → phase None, no attach) | gpui headless drive |
| REQ-001/002 | `grep -rn 'forge_web_base' crates/marley_app/src/` = 0; `grep -rn 'marley_forge_client::(forge_web_base\|same_web_origin)'` = 0 | validate-step grep + compile |
| REQ-003 | repointed `placeholder_copy_is_exact`; repointed `browser_retry_without_source_unconfigures_headless` | unit + drive |
| REQ-004 | palette.rs :320 action test green + validate grep: :10439 keyword list has no "forge" | unit + grep |
| REQ-005 | orchestration.rs units repointed to `brain_endpoint`; settings.rs round-trips green post-field-drop; compile proves no reader | units |
| REQ-006 | 10 generic drives + 19 browser_state units (post :661 delete) + 6 browser.rs survivors + probe livewire block — full `cargo nextest run -p marley_app` | suites RUN |
| REQ-007 | parity pair: React capture (5173) ↔ live Marley B-tab capture, SAME placeholder state; pixel-sample per MARLEY-PARITY.md (÷2 retina rule) | headed capture |
| REQ-008 | `scripts/gates.sh --diff` exit 0 | gate |
| F2 move | 5 moved units (`web_origin_of_arms_exact` + 4 `same_web_origin`) in browser.rs; mutation `--diff` covers the moved bodies at MSI 100 | units + mutants |
| trybuild | none — pure deletions + a fn move; no new type contract | n/a (recorded) |
| uncoverable | no NEW uncoverable paths; webview_shim.rs stays the documented §0 exclude | n/a |

### Risks / decisions
1. ReAttach-arm totality (teardown, not panic) leans on retry_plan's tested
   contract — inspect critic bait; comment carries the reasoning.
2. Temporary `web_origin_of`/`same_web_origin` duplication (browser.rs +
   forge_client) until 411 — deliberate, one ticket long.
3. `web_url` deletion touches persisted-settings fixtures only; unknown-field
   tolerance verified (no `deny_unknown_fields`).
4. Production post-410: the B tab NEVER attaches (placeholder is the standing
   state) — the parity pair captures exactly that state on both sides.
5. The attach Ok-arm becomes production-unreachable (was already headless-
   unreachable; app.rs is the documented §0 exclude; headed T-lane validated it
   in #405/#406 and re-validates when a future URL feature lands).

## Phase 3 — Implement
- **React-first (executed):** dev server @5173; BEFORE capture read
  (`.playwright-mcp/410-placeholder-before.png` — the forge sentence); edits to
  `BrowserPlaceholder.tsx` (copy + doc) + `BrowserPane.tsx` (ORIGIN recaption, doc
  de-forge); `tsc` green; AFTER capture read (`410-placeholder-after.png`): the
  two-tier card reads "No page loaded / The embedded browser awaits a page
  source." — hierarchy unchanged, F3 settled visually.
- **Rust rip (per manifest):** app.rs field/init/boot-derivation/test-hook/
  `derive_forge_web_base_now` deleted; mount gate → `mount_plan(None)` + Retry →
  `retry_plan(None, mounted)` (both future-seam commented; ReAttach arm total
  teardown + contract comment); caption holder-only; CommandId(31) keyword list
  de-forged; placeholder arm comment re-voiced. browser.rs: new copy + doc
  re-voice, `same_web_origin` + `web_origin_of` moved in (docs adapted, PR/BF
  codes cited), module doc updated. browser_state.rs: `effective_retry_base` +
  arms test deleted; `retry_plan` doc generalized (param renamed `desired`);
  fault_reason prose de-forged. webview_shim.rs: nav handler →
  `crate::browser::same_web_origin`; module doc + :106 justification updated.
  orchestration.rs: `web_url` deleted, docs re-worded, tests repointed to
  `brain_endpoint`. settings.rs/fleet_live.rs/headless_drive.rs:41/
  tests/fleet_livewire.rs fixtures dropped the field. headless_drive.rs: the
  boot-derivation drive retired; the retry drive repointed +
  renamed `browser_retry_without_source_unconfigures_headless`. grid_layout.rs /
  content.rs / lib.rs prose re-worded. Cargo.toml: `url = "2"` edge (in-lock,
  justified comment).
- **Deviations from design (4):**
  1. Only 4 predicate tests moved, not 5 — T8 `nav_inputs_never_carry_bearer` is
     derivation-coupled (`forge_web_base(&cfg)` composes the whole chain); it
     stays in forge_client and dies with the crate in 411.
  2. mcp_config.rs's 2 forge-coupled tests + their `forge_config` helper DELETED
     NOW (design deferred all 9 tests to 411): they pinned the exact composition
     410 removed, and deleting them reaches REQ-001/002's zero-grep with no
     carve-out. The 7 pure path tests + the module remain for 411. A tombstone
     comment marks the retirement.
  3. webview_shim.rs:106 `mutants::skip` justification updated (stale
     `effective_retry_base` mention) — forced by the deletion, not in manifest.
  4. The orchestration serde unit now decodes a payload carrying the DELETED
     `web_url` key — the design's forward-compat claim made executable.
- **Compile state:** `cargo check --workspace --all-targets` 0 errors;
  `cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets
  -D warnings` exit 0. Greps: zero `forge_web_base` / `effective_retry_base` in
  `marley_app/src`; zero `marley_forge_client::same_web_origin` call sites.
  (Full test RUN is Phase 4.)

## Phase 3.5 — Inspect
- **Critics run (3, parallel, general-purpose):** correctness/AC · security-secrets-
  provenance · state-integrity+simplification. All three verified concretely
  (byte-compares — sha256-identical moved fns; targeted test RUNS — 58 lib + 4
  livewire + 11 browser + 90 targeted, all green; repo-wide greps).
- **Findings ledger:**

| # | sev | finding | verdict | action |
|---|---|---|---|---|
| I1 | nit | Forward-compat pin ran serde_json while production decodes TOML (orchestration.rs:70 vs marley_settings value.rs try_into) | REAL (test-strength gap, not a shipped bug) | FIXED — the settings.rs TOML harness fixture now carries the stale `web_url = "http://old"` key through the REAL codec; test re-run green |
| I2 | nit | Caption alloc shape (app.rs:19031 `.map(to_string)` before the 4-arm match) | REJECTED — production-dead post-410, `card()` requires `String`; churn with no benefit | none |
| I3 | info | Phase-5 arch-doc line ranges miss orchestration-shell.md:28/:163 ("web URL" mentions outside §8) | REAL (doc-sweep method) | CARRIED to Phase 5: sweep by grep, not recorded ranges |
| I4 | info | Planned `browser_boot_ignores_mcp_json_headless` drive not yet written | EXPECTED (Phase-3/4 split) | CARRIED to Phase 4 (already a test-plan row) |
| I5 | low | Predicate duplication drift window until 411 | ACCEPTED — recorded design risk 2; 411 is queue-top | none |
| I6 | info | T8 bearer property retires with the crate at 411 (its boot-half drive deleted here) | ACCEPTED — recorded in Phase-3 deviation 1; flag for 411's review | carried into TICKET-411 awareness |

- **Provenance check:** moved code hash-matches Marley's own forge_client; zero
  warp-shaped identifiers in the diff; POC edits are Marley's own files (§20 clean).
- **Secrets check:** entropy + keyword scans over added lines — zero hits; the only
  bearer literals in the diff are on DELETED fixture lines.
- **Supply chain:** Cargo.lock diff = one dependents-edge line; url 2.5.8 single
  pre-existing entry, checksum unchanged (gate:8 posture unchanged).
- **No F-/PR- appends:** no shipped-code bug found (I1 is a test-quality gap fixed
  in-phase); no new failure class worth a rule beyond the already-appended design
  lesson.

## Phase 4 — Validate
- **New test:** `browser_boot_ignores_mcp_json_headless` (headless_drive.rs) — the
  exact fixture the retired boot-derivation drive derived FROM now yields phase
  `None` after opening B + pumps (REQ-001's behavioral half). PASS on first run.
- **Suites RUN (real output):** `cargo nextest run --workspace` →
  **2152 tests run: 2152 passed, 5 skipped** (the 5 skips are the pre-existing
  documented lane skips). `cargo test --workspace --doc` → ok. The inspect-phase
  critics additionally ran targeted lanes (58 lib browser/orchestration/
  mcp_config; 4 fleet_livewire; 11 browser::) — all green.
- **Acceptance greps (REQ-001/002/004):** zero `forge_web_base` files in
  `marley_app/src`; zero `marley_forge_client::{forge_web_base,same_web_origin}`
  references; CommandId(31) keyword list carries no "forge".
- **Live drive (self-test harness) + parity pair (REQ-007):** captures in
  `.playwright-mcp/` (gitignored), all READ:
  - `410-live-boot2.png` — restored workspace (direct-exec relaunch after the
    documented windowless-stall gotcha hit the `open` launch).
  - `410-live-menu.png` — the Browser section ＋ menu: Forge/Agents/Details/
    Browser (the #403 rows; Forge row is 411's).
  - **`410-live-placeholder2.png` — REQ-003 LIVE:** ＋ menu → keyboard (3×down,
    the wrapping menu opens with row 0 selected) → Enter → the B tab opened and
    rendered the two-tier card **"No page loaded" / "The embedded browser awaits
    a page source."** — no webview mounted, `focus: browser`.
  - `410-live-closed1.png` / `410-live-final2.png` — ⌘W closed the B tab
    cleanly (no orphan webview), then the drive-opened cockpit tabs; workspace
    shape left as found.
  - **Parity pair (React `410-placeholder-after.png` ↔ live
    `410-live-placeholder2.png`), pixel-sampled via magick:** headline strip
    maxima 142,146,159 ↔ 134,138,150; caption 90,93,101 ↔ 81,84,92 —
    caption/headline luminance ratio 0.637 ↔ 0.609 (both ≈ the /60 tier);
    bg (11,12,15) ↔ (14,15,17). The absolute Δ≈8/255 is the documented
    capture-space gamma (Chrome sRGB vs screencapture) — the SAME numeric pairs
    the #406 parity capture recorded. Copy/casing/geometry identical.
    **PARITY: PASS — no Rust fix, no marley-web fix.**
- **Harness casualties (recorded honestly):** the pre-run Marley instance was
  killed by the harness protocol (`pkill` before a fresh bundle-launch); in-app
  terminal PTY sessions from before the run — including an in-app `claude`
  session tab — do not survive an app kill (unrecoverable by design; the
  workspace SHAPE restored intact). Two misclicks during rail-coordinate hunting
  transiently opened Agents/Details/Forge cockpit tabs — all closed; one ⌘W
  landed on the relaunch-created replacement terminal (Marley 4) — net shape as
  found. One unexplained windowless stall after a ＋-affordance click on the
  first instance (process alive, zero AX windows — the README's documented
  symptom class); not reproducible on the relaunch; nothing in stderr.
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15 passed / 0
  failed, exit 0. Highlights: gate:4 lines 47182/47182 = 100.00% (floor held with
  the moved predicate + new drive in the covered set); gate:5 mutation on the
  diff — **10 caught / 0 missed → MSI 100.0%**; gate:15 visual/AX 158/158.
  Receipt written (`.git/ignibyte-gate-receipt`).
- **Pre-existing notes:** the `block v0.1.6` future-incompat warning is upstream
  (objc ecosystem), pre-existing, not in scope. No other pre-existing failures.

## Phase 5 — Complete
- **CHANGELOG:** Unreleased/Changed entry added (product-rip slice 1, above the 409
  process-half entry).
- **Architecture docs (grep-swept per inspect I3, not line-ranges):**
  `embedded-browser-model.md` (title + status banner rewritten to the post-410 truth;
  Q3 marked RETIRED with a rip note; Q4's codec claim re-tensed; shipped-slice 5 added
  for #410; CDP pillar renumbered 6), `orchestration-shell.md` (2026-08-09 amendment
  banner; §8 header marked RETIRED; settings-row web_url note), `roadmap.md` (Phase E
  narrative brought current: 404/405/406 shipped + the #410 rip line; :252 parenthetical),
  `app_shell.md` (:958 derivation-ripped note), `fleet-control-plane.md` (:20
  browser-half-retired note). Left for 411 by scope: `marley_agent.md:97` (sprint pane),
  `crate-map`/`00-overview`/`marley_forge_client.md`/`clean-build-plan`/`crate-triage`,
  `pane-composition-model.md` cockpit-Forge vocabulary.
- **Parity sync:** marley-web matches shipped (live-verified at P4); `MARLEY-PARITY.md`
  BrowserPlaceholder row updated (#410 standing-state + re-voiced copy + the parity-pair
  numbers). No port-time deviation to back-port.
- **Ledger appends (codes):** `AD-claude-browser-no-origin-source-until-url-feature-001`
  (architecture-decisions.md); `L-claude-drive-gpui-menus-by-keyboard-not-coordinates-001`
  (lessons.md); plus the P2-appended
  `L-claude-rip-keeps-pure-seams-called-constant-input-001`. No F-/PR- (inspect found no
  shipped-code bug).
- **Ticket:** TICKET-410 → `tickets/closed/`, status closed. BACKLOG sweep: no stale row
  (410's left at promotion; 411 now tops the Queue).
- **Archive:** spec+notes pair → `docs/planning/pipeline/completed/`.
