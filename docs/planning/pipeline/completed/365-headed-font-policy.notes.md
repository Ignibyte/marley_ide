# 365 headed font-policy verification — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-365-headed-font-policy-verification.md
- **Pipeline spec:** 365-headed-font-policy.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** `/work` goal batch (Chad /goal 2026-08-14): close #6, then #365 +
  #423 **auto approved** — autonomous through `/commit`, no per-phase pauses.
  Queue-top promotion; #407's pipeline pair was PARKED first (§3 one-active;
  its remaining scope waits on the dev-box sweep).
- **Classification / tier:** chore (test-lane), work pipeline, one shippable
  slice: headed drives + lane wiring only. No app behavior change (spec D3).
- **Recall (§18.3):**
  - #361 L-entry (lessons.md:404): the two headless RESOLVABLE drives shipped
    at #361 Phase 3 CANNOT pass — NoopTextSystem resolves nothing (the
    `!resolves` short-circuit fires before `font_is_monospace`) and fakes
    equal-width advances; inspect Critic 2 (HIGH) caught it; drives deleted.
    The gap this pipeline closes is exactly that deletion's residue.
  - #361 L-entry (lessons.md:464): the two app-side probes are cov-excluded
    AND `#[cfg_attr(test, mutants::skip)]` — an integration-only shim needs
    BOTH; do not disturb those attributes when touching nearby code.
  - #344 notes: the TempDir persist-settings→boot→assert idiom (`zoom_persists`
    lineage), `mono_family_for_test()` + `flash_message_for_test()` observables,
    TERMINAL_FONT built-in = "Menlo", flash ordering not-found BEFORE
    not-monospace.
  - Environment probe (pre-flight): live GUI session EXISTS on this machine
    (console user since Aug 13, WindowServer up, 3440x1440) — the headed lane
    is runnable at P4.
- **Discovery:** Explore agent mapped the headed harness + font wiring (report
  summarized at Phase 2 entry; lane entry point + skip condition are design
  inputs D4 confirms). gpui 0.2.2 registry sweep: NoopTextSystem `pub(crate)`
  + hardwired in TestPlatform; real CoreText only via the mac platform; no
  `render_to_image` in 0.2.2.
- **Decisions:** spec D1–D4 (system fonts only; in-process state asserts; no
  app-code delta beyond a minimal hook if strictly needed; lane conventions
  followed). EARS REQ-001..004 incl. the REQ-003 sabotage smoke and the
  REQ-004 skip-not-false-green contract.

## Phase 2 — Design

### Discovery (Explore agent, harness + wiring map — the design inputs)
- **Headed = subprocess.** gpui's `Application::run()` owns the macOS main
  thread, so the lane spawns the REAL binary (`CARGO_BIN_EXE_marley`,
  `headed_shell.rs:23` idiom); real mac platform ⇒ real CoreText. There is no
  in-process headed entry (`marley_visual_harness/src/launch.rs:1-5`).
- **The `*_for_test` observables DO NOT EXIST in the child** — `#[cfg(test)]`
  (`mono_family_for_test` app.rs:3075, `flash_message_for_test` app.rs:12012).
  The flash toast is Metal-painted → absent from AX; window-level AX only
  (`marley_visual_harness/src/lib.rs:19-21`). Pixels could prove flash
  PRESENCE but never its text. ⇒ plan-D2's in-process assert is impossible;
  AMENDED (below).
- **Settings injection lever = `$HOME`.** Config dir is
  `home::home_dir()/.marley/config` (`marley_core/src/paths.rs:33-39`); home
  0.5.12 reads `$HOME` on unix. No `MARLEY_CONFIG_DIR` exists;
  `RootView::new_in(dir)` is `pub(crate)` (app.rs:1701) and unreachable from a
  spawned bin. Seed `tmp_home/.marley/config/settings.toml`, spawn with
  `HOME=tmp_home`.
- **Runtime env-gate precedent for a shipped-binary test mode:**
  `MARLEY_WEBVIEW_PROBE` (`marley_webview_probe/src/main.rs:20-29`) and
  `MARLEY_WIDGET_FIXTURE` (`marley_widgets_gallery.rs:128-129`).
- **Lane convention:** headed tests are `#[ignore]` with the shared reason
  string, invoked per-target `cargo test -p <pkg> --test <t> -- --ignored`; no
  env-var gating; gate:3/gate:15 never run them (nextest skips ignored;
  visual_g runs only marley_visual_harness headless) — so the new target
  changes NO gate surface.
- **Boot wiring (assert targets):** probe order `resolves` →
  `!resolves || font_is_monospace` short-circuit (app.rs:2680-2687; the
  short-circuit keeps panicking `resolve_font` off the non-resolve path);
  `font_family: font_res.applied` (:2802, empty = built-in) and
  `status_flash: font_res.flash.map(Flash::new)` (:2860);
  `TERMINAL_FONT = "Menlo"` (app.rs:677); `mono_family()` empty→builtin
  mapping (app.rs:3065-3071). Flash countdown is pump-ticks (120), set at
  construction — a dump right after boot cannot race it.
- **This machine can run the lane NOW:** live console session (Aug 13),
  WindowServer up, 3440x1440 vscreen. The #318 ghost-window failure mode
  (display asleep / no console user) is the documented risk; our drives need
  NO Accessibility, NO Screen Recording, NO activation (no AX read, no
  screenshot, no keystroke — just boot + stdout), which sidesteps most of the
  #417 class. Preflight at implement: run the M0-proven
  `headed_selftest -- --ignored` once to prove the session before building.

### Architecture (approach)
1. **App-side hook (the ONE permitted app-code delta, spec D3):** an env-gated
   self-report in the window-open closure of `run()` (app.rs:22488-22504),
   directly after `RootView::new` returns: when
   `MARLEY_FONT_POLICY_SELFTEST=1`, print ONE line to stdout —
   `MARLEY_FONT_POLICY_SELFTEST family=<mono_family()> flash=<none|message>`
   — then `cx.quit()`. Gate check first-thing; exact-match `"1"`; inert
   otherwise. Attributes per the shim conventions:
   `#[cfg_attr(test, mutants::skip)]` on a small helper fn (app.rs is already
   cov-excluded), `// shim:` comment stating the DECISION lives in the drive's
   asserts. The hook is a dumb REPORTER — assertions live in the test.
2. **Drives:** new `#[ignore]`-gated integration target
   `crates/marley_app/tests/headed_fonts.rs` (package `marley`), two tests:
   - `headed_font_monaco_applies_silently`: seed `font_family = "Monaco"` →
     expect `family=Monaco flash=none`.
   - `headed_font_helvetica_falls_back_with_flash`: seed `"Helvetica"` →
     expect `family=Menlo` + flash containing `Helvetica` and `not monospace`.
   Each: tempdir HOME, write
   `<home>/.marley/config/settings.toml` (`[appearance] font_family = "…"` —
   the settings file IS the interface per settings.rs:39-51), spawn
   `CARGO_BIN_EXE_marley` with `HOME` + `MARLEY_FONT_POLICY_SELFTEST=1`
   (spawn via `marley_command::blocking::Command` — §14 adapter; dev-dep if
   not present), capture stdout, deadline-poll ~20 s, kill on timeout with a
   "live GUI session?" hint, parse the dump line, assert.
3. **No marley_visual_harness changes** — no AX/pixels needed; `HeadedSession`
   env extension unnecessary. Reference §20 stays N/A — confirmed (test-lane
   chore; no reference-app behavior matched; gpui read was registry ADOPTION).

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/app.rs` | + env-gated `font_policy_selftest_dump` hook called from the window-open closure after `RootView::new` (≈15 lines incl. the shim attrs + why-comment). |
| `crates/marley_app/tests/headed_fonts.rs` | NEW — module doc with the invocation command + the two `#[ignore]` drives + tempdir/seed/spawn/deadline/parse helpers. |
| `crates/marley_app/Cargo.toml` | only if `marley_command` isn't already usable from tests (dev-dep add); no version bumps. |
| `docs/marley_architecture/marley_visual_harness.md` | P5: add the fonts target to the headed-lane target list. |

### Regression test plan (final — asserts via the selftest dump line; the
### `*_for_test` accessors don't exist in the spawned binary, see Discovery)
| # | Proves | Test |
|---|---|---|
| T1 | REQ-001 (mono applies silently — #344 RESOLVABLE-APPLY + #361 no-false-warn) | headed drive `headed_font_monaco_applies_silently`: tempdir `HOME` seeded with `appearance.font_family = "Monaco"`, spawn the real binary with `MARLEY_FONT_POLICY_SELFTEST=1`, parse the dump line, assert `family=Monaco` (NON-default: built-in is "Menlo", so a do-nothing impl and a fallback both read wrong) AND `flash=none`. |
| T2 | REQ-002 (#361 warn arm end-to-end) | headed drive `headed_font_helvetica_falls_back_with_flash`: seed `"Helvetica"`, spawn, assert `family=Menlo` (the built-in) AND flash containing `"Helvetica"` + `"not monospace"` (content-pinned: the not-FOUND message would fail it, proving the right arm fired). |
| T3 | REQ-003 (sabotage smoke, one-off at P4) | flip `font_is_monospace` to always-false (working-tree only), run T1 → must go RED (Monaco falls back + flashes); revert; re-run T1 → GREEN. Receipt (commands + outcomes) recorded in the P4 notes entry. |
| T4 | REQ-004 (skip-not-false-green) | (a) default runs skip: the drives list as ignored (reason string states the headed need) in `cargo nextest run -p marley` / plain `cargo test -p marley --test headed_fonts` — observed at P4; (b) no-hang: the drive's deadline-poll kills the child and fails with the "live GUI session?" hint if boot ghosts (reviewed; exercised only if the session drops). |
- **Uncoverable/conditional by nature:** the drives NEED a live GUI session +
  real CoreText — they are lane-conditional exactly like the existing headed
  tests (gate:15's documented conditionality), never part of the headless
  `cargo nextest` bar. The pure policy + headless garbage-drive coverage from
  #344/#361 is untouched and stays the gate:4/5 surface.
- **Font ground truth on this machine (checked):** `/System/Library/Fonts/`
  has `Monaco.ttf`, `Helvetica.ttc`, `Menlo.ttc`, `Courier.ttc`.

### Risks
- **Activation/focus (the #417 class):** asserts here read app STATE
  (`*_for_test`), not AX focus or after-capture pixels — boot + read should not
  need activation; if the lane's boot path demands frontmost status, the drive
  inherits the lane's existing conditionality (skip, not hang) — confirm at
  implement.
- **Parallel-drive settings races:** each drive seeds its own TempDir config
  dir (the #337/#344 idiom); headed drives additionally serialize (one window
  server) — follow the lane's existing serialization convention.
- **Runtime cost:** each headed boot is seconds, not ms; the lane is
  conditional + local, so suite time is unaffected.
- **Menlo-as-built-in ambiguity:** T2 asserts fallback==Menlo; if a future
  built-in change lands, the assert reads the constant (`TERMINAL_FONT`), not a
  literal, so the drive tracks it.

## Phase 3 — Implement
- **React-first: N/A** (spec: no UI delta — verification of shipped behavior).
- **Built:**
  - `crates/marley_app/src/app.rs` — the #365 self-report, INLINE in `run()`'s
    boot closure right after `open_window` (binds the previously-discarded
    `WindowHandle`): env exact-match `MARLEY_FONT_POLICY_SELFTEST=1` →
    `window.update` reads `root.mono_family()` + `root.status_flash` → one
    tab-delimited stdout line → `cx.quit()`; `return` skips `cx.activate` so a
    selftest boot never steals frontmost. Inert otherwise.
  - `crates/marley_app/tests/headed_fonts.rs` — NEW: `boot_font_selftest`
    (tempdir `$HOME` → seed literal TOML → adapter spawn with piped stdout →
    20 s deadline-poll with kill + "live GUI session?" panic → parse) + the two
    `#[serial_test::serial]` `#[ignore]` drives (Monaco silent / Helvetica
    fallback+flash, content-pinned).
- **Deviations from design (with reason):**
  - The hook is INLINE in `run()` rather than the manifest's named
    `font_policy_selftest_dump` fn: `run()` already carries
    `#[cfg_attr(test, mutants::skip)]`, so inlining adds ZERO new mutation
    surface (a separate fn would need its own skip + justification). Same
    ~20 lines, one fewer shim.
  - `Cargo.toml` untouched: `marley_command` is a MAIN dep (test targets link
    main deps too) and `tempfile`/`serial_test` were already dev-deps.
  - Added `#[serial_test::serial]` to both drives (not in the design table):
    two simultaneous real-app boots on one WindowServer are avoidable
    nondeterminism at zero cost.
- **Checks:** `cargo check -p marley --all-targets` clean (6.3 s);
  `cargo clippy -p marley --all-targets` exit 0; `cargo fmt --all --check`
  clean. (The `block v0.1.6` future-incompat note is pre-existing/transitive.)

## Phase 3.5 — Inspect
- **Critics:** 2 spawned in parallel over the diff (correctness/AC lens;
  security+state+simplification lens) — findings table below (appended as the
  reports land).
- **Lead self-review (empirical, ahead of the critics):**
  | Check | Result |
  |---|---|
  | Both drives RUN on this machine | GREEN first run, 0.74 s total — Monaco applied+silent, Helvetica fell back+flashed; `cx.quit()` during the init closure exits 0 (~0.4 s/boot) — the quit-during-init soundness question is answered by reality. |
  | Flash-tick race (pump could count the flash down before the read) | Not observed — the report reads the flash synchronously in the same init closure, before pump ticks; Helvetica's message was present. |
  | stdout collision on the report line | Only ONE `println!` exists in app.rs (the hook); an app boot emits no other stdout line (verified live — the report is the sole line). |
  | Malformed seeded TOML (settings load error → defaults path) | Boots defaults: `family=Menlo flash=none` (verified live with garbage TOML) — distinguishable from BOTH drives' expectations, so a seeding regression fails loudly, never false-passes. |
  | Env leak routes into config resolution | Single route: `home::home_dir()` → `$HOME/.marley/config` (paths.rs:33-39); no XDG, no other `MARLEY_*` read at boot. The tempdir `$HOME` isolates fully. |
  | Kill/reap ordering in the timeout arm | `kill()` then `wait()` (reap) then panic — no zombie; success arm's `try_wait() == Some` is already reaped. |
  | Default-run skip (T4a, pre-verified) | `cargo test -p marley --test headed_fonts` → `0 passed; 2 ignored` with the headed reason string, 0.00 s. |
  | Regression sweep (early gate:3 signal) | `cargo nextest run -p marley`: 1069 passed, 4 skipped (the headed ignores); `cargo test --doc -p marley`: 0 (none in package). |
  | New mutation surface from the hook | `cargo mutants --list -p marley` → 0 mutants in the run() hook region (2965 package-wide, unchanged shape) — inlining under run()'s existing skip verified. |
  | **REQ-003 sabotage smoke (T3) — run EARLY, during inspect** | `font_is_monospace` Ok-arm flipped to `false` (the always-false regression) → `headed_font_monaco_applies_silently` FAILED (`family=Menlo`, "a resolvable mono family must APPLY") while Helvetica stayed green (its expectation IS fallback); sabotage reverted (diff shows 0 SABOTAGE markers) → 2/2 GREEN, 0.63 s. The end-to-end false-warn detection the pure units cannot provide is PROVEN. |

- **Critic findings ledger (both reports in; NO HIGH, NO MED):**
  | Sev | Finding (critic) | Verdict | Fix |
  |---|---|---|---|
  | LOW | Settings-controlled flash text can embed tab/newline and break the report framing (security) | REAL (bounded: same-principal, worst case a confusing test failure) | Printer now strips `\t`/`\n` from BOTH fields — the one-line frame is a printer-local invariant. |
  | LOW | `Err(open_window)` arm exited 0 with no line (security) | REAL (diagnosability) | `Err` arm now prints `open_window failed` to stderr; drive still fails loudly on the missing line. |
  | LOW | Test seeder TOML quote-injection on a non-literal family (security) | REAL-but-latent (both callers literal; malformed seed fails loudly — verified) | Doc contract on `boot_font_selftest` ("quote/backslash-free — literal TOML"). |
  | LOW | Tempdir-HOME has pre-existing bypasses: `$TMPDIR/marley-zsh-*` zdotdir litter + gpui's uid-keyed NSUserDefaults write-once (security) | REAL, pre-existing, benign | `TMPDIR` now also points at the tempdir (contains the zdotdir); NSUserDefaults bypass documented in the drive doc (cfprefsd keys by uid, not `$HOME`). |
  | LOW | Drive child inherited cwd → boot's `Project::discover_in(cwd)` walked the real repo (correctness) | REAL (hermeticity + speed; report content unaffected) | `.current_dir(home.path())` — discovery lands in the tempdir. |
  | LOW | Exit-status assert fired before stdout was read — a crashed boot reported no context (correctness) | REAL (diagnosability) | Loop now breaks with `status`; stdout is read FIRST and included in the assert message. |
  | LOW | Latent pipe-buffer deadlock if a future writer exceeds ~64 KB stdout (correctness) | REAL-but-latent (today: the report is the SOLE stdout line — verified by grep across all 15 path-dep crates + no logger; future case degrades to the deadline-kill, loud not hung) | Documented in the drive doc with the "drain incrementally" upgrade path. |
  | INFO | `run()`'s `ExitCode::SUCCESS` unreachable on the quit path (AppKit `exit(0)` wins) | Noted | None needed. |
  - **Deep hunts settled at the gpui-0.2.2 source level (correctness critic):**
    flash-tick race IMPOSSIBLE (`open_window` does one synchronous draw; no
    runloop turn — hence no pump tick — can precede the hook's synchronous
    `window.update`); quit-during-init is the full ⌘Q teardown chain
    (`dispatch_async` → `terminate:` → `App::shutdown` → `windows.clear()` →
    PTY reap → `exit(0)`)). Clean-room confirmed (env-gate + spawn patterns are
    Marley's own precedents; gpui consumed via public API only).
  - **No `F-`/`PR-` ledger appends:** no critic finding was a BUG (nothing
    failed; both drives green before and after) — all eight were hardenings or
    documentation of latent/pre-existing conditions, recorded here.
- **Post-fix verification:** `cargo fmt --all` applied; clippy exit 0 (only the
  pre-existing `block v0.1.6` future-incompat note); drives re-run 2/2 GREEN
  (0.42 s) with the sanitizing printer + hermetic spawn.

## Phase 4 — Validate
- **Tests are the deliverable here** (test-lane chore): the two headed drives
  were WRITTEN at implement (per the design manifest) and needed no additions —
  every plan row was already exercised by inspect; this phase re-ran them
  formally.
- **T1+T2 (REQ-001/002) formal run** — `cargo test -p marley --test
  headed_fonts -- --ignored`: `2 passed; 0 failed` in 0.42 s (4th consecutive
  green incl. the post-hardening rerun). Monaco applied+silent; Helvetica →
  Menlo + "not monospace" flash, content-pinned.
- **T3 (REQ-003) receipt** — run EARLY during inspect (ledger above):
  always-false `font_is_monospace` sabotage → Monaco drive RED
  (`family=Menlo` ≠ `Monaco`), Helvetica stayed green (its expectation IS
  fallback); reverted (0 sabotage markers in diff) → 2/2 GREEN 0.63 s.
- **T4 (REQ-004) formal evidence** — default `cargo test -p marley --test
  headed_fonts`: `0 passed; 2 ignored` (reason string names the headed need),
  0.00 s. No-hang half: deadline-poll kill + "live GUI session?" panic,
  review-verified (exercised only if the session drops).
- **Live-app capture: N/A — no UI delta.** The shipped change is an env-gated
  boot hook (inert without `MARLEY_FONT_POLICY_SELFTEST=1`; skips
  `cx.activate` when armed, so it never steals frontmost) + an `#[ignore]`
  test target. No render/input/layout path touched; the spec's React-first
  section is `N/A — no UI delta`. The drives themselves ARE live-app
  verification: each boots the real binary on the live WindowServer session.
- **Pre-existing, not in scope:** the `block v0.1.6` future-incompat cargo
  note (transitive via gpui's objc stack).
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15/15 PASS
  (rustfmt, clippy -D warnings, nextest+doctests, audit, deny, machete,
  gitleaks, shellcheck, no-suppressions, source-bans, docs, coverage ≥100%,
  mutation MSI ≥100%, miri, visual/AX — harness suite 158/158 + 2 skipped
  headed). Exit 0; receipt written for `/commit`.

## Phase 5 — Complete
- **§21 docs:** CHANGELOG `[Unreleased] Added` entry (TICKET-365) + fixed the
  407 entry's stale `active/` notes path (→ `parked/`);
  `docs/marley_architecture/marley_visual_harness.md` gained point 6 (the
  stdout-report headed idiom + the six-target lane list). React parity: N/A.
- **Ledger appends:**
  `AD-claude-365-env-gated-stdout-self-report-is-the-headed-state-assert-seam-001`
  (architecture-decisions.md);
  `L-claude-365-headed-drives-cannot-see-cfg-test-observables-001` was appended
  at Phase 2 (design). No `F-`/`PR-`: no critic finding was a failure —
  rationale in the inspect ledger.
- **Ticket:** TICKET-365 closed (shipped; close note cites the AD + the
  no-AX/no-Screen-Recording discovery) and moved to `tickets/closed/`.
  BACKLOG swept — no stale row (left at promotion).
- **Archive:** pair moved to `pipeline/completed/`.
