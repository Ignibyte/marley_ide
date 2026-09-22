# marley_visual_harness — the gate-15 visual/accessibility harness

`marley_visual_harness` is Marley's **single normative target for quality-bar gate 15**: the
dev/test-support crate that launches the app headed, asserts the macOS **AXUIElement** element
tree, and diffs screenshots against approved baselines. Every spec with a `visual_acceptance`
clause (foundation-spike, ui-components, app-shell, …) binds to *this* crate's API. It links into
no shipped product binary. Contract: [`docs/pipeline/visual-testing.spec.md`](../pipeline/visual-testing.spec.md)
(R1–R27). INVENT (net-new); macOS-only at M0/M1. **Current @ M15:** the pure/shim seam is intact; the
one API addition since is `send_keystroke` (M1.C #23 — the headed lane's real input driver, below).
Clean-room `[Marley-original]` on `[gpui Apache-2.0]` (see [Provenance](#provenance-clean-room)); a
gpui-native offscreen path (`render_to_image`) is a known [future upgrade](#future-upgrade--gpui-native-offscreen-render-zed-finding).

## The pure / shim seam (the 004 pattern)

The crate has a hard seam between a **pure decision surface** (100% line coverage + mutation
MSI 100, headless, no display) and a **thin display/subprocess shim** (ACCEPTED-UNTESTABLE).

**Pure layer** — every assertion is a pure operation over plain in-memory data:
- `geometry` — `is_above`/`is_leading`/`contains`/`is_centered_in`/`covers` + `CENTER_EPS_PX`/
  `BOUNDS_EPS_PX` (R10–R14). The ordering/containment core the AX `assert_*` methods delegate to.
- `image_diff` — `Image`/`Tolerance`/`PixelDiff`/`ChannelSet`/`RegionMask` + `compare`/`state_delta`/
  `diff` (R17–R19); perceptual distance via `image-compare` (SSIM). `Tolerance::gate15_default()`
  is `per_channel = 4, max_changed_fraction = 0.0, perceptual_mean = 1.0`. **Text-rendering windows**
  (TICKET-008, the M1 enabler) use `Tolerance::gate15_text()` (`8 / 0.02 / 0.05` — absorbs gpui subpixel
  AA, with `max_changed_fraction` the binding gate for real content changes >2% of pixels) +
  `RegionMask::titlebar_band` (exclude the focus-variable macOS titlebar) applied via `mask_out`
  (mask-out-then-compare: fill the band identical in both → excluded from the per-pixel AND the SSIM
  decision) + `Screenshot::assert_matches_baseline_masked`. `gate15_default` stays the strict default
  for static fixtures.
- `baseline` — `evaluate_baseline` + `resolve_approval` (the fail-closed "no silent drift" core,
  R23/R24) + `APPROVE_ENV` + path helpers + the PNG IO + `write_pending`/`diff_image`.
- `ax` — `AxSnapshot`/`AxNode`/`AxQuery`/`AxRole`/`AxValue`/`AxMatch`/`AxFrame` + `find`/`find_all` (by an
  `AxMatch` role/title/value matcher) + the `assert_*` family (`assert_role`/`assert_title`/`assert_value`/
  `assert_size`/`assert_count`/`assert_above`/`assert_leading`/`assert_contains`/`assert_centered_in`/
  `assert_covers`, R7–R15). Assertions panic with expected-vs-actual on failure — the fail-closed
  test-harness idiom.
- `ax_parse` — the pure parse of the osascript dump (`WIN`/`ELEM` tab-separated lines) → `AxSnapshot`.
- `ax_exec` (pure half) — `ax_dump_script` (the AppleScript text generator) + `is_accessibility_denied`.

**Shim layer** — display/subprocess-bound; ACCEPTED-UNTESTABLE (see below):
- `os_shim` — the three literal OS calls: `run_ax_dump` (osascript AX dump) + `capture_window_region`
  (`screencapture -R`) + **`send_keystroke`** (osascript System Events keystroke — the headed lane's input
  driver, added M1.C #23 so a headed test can drive a real cmd-d split; the key is restricted to a single
  ASCII-alphanumeric so a quote can never break out of the generated AppleScript).
- `launch` — `FixtureWindow`/`HarnessError`/`HeadedSession` (`HeadedSession::launch` = the `mount`-equivalent,
  plus `snapshot` / `capture` / `send_keystroke`) + `Drop` (kills + reaps the runner child, idempotent).
- `selftest` + `src/bin/marley_harness_selftest.rs` — the gpui runner that mounts `TwoElementFixture`.

## Key architecture decisions

1. **Subprocess model (not in-process).** gpui's `Application::run()` owns the macOS main thread
   and blocks, so a fixture cannot be mounted *inside* a `#[test]` and then asserted. Instead
   `HeadedSession::launch(runner_bin, title, timeout)` spawns a runner **binary** (via
   `marley_command`, the non-PTY spawn seam — `std::process::Command` is clippy-banned), polls for
   window readiness, and observes it **out-of-process**: `snapshot()` shells out to osascript and
   parses the AX tree; `capture()` runs `screencapture -R<x,y,w,h>` over the window's AX frame.
   `Drop` kills the child (idempotent). Each downstream UI crate ships its own runner bin + fixture.

2. **AX access via osascript / `System Events`, not objc2 FFI.** Keeps the crate **`unsafe`-free**
   (gate:6 miri N/A, gate:13 SAST clean). The AppleScript text is a pure, mutation-tested generator
   (`ax_dump_script`); only the literal `Command` call is the shim. (objc2-application-services /
   accessibility-sys remain a fallback if a future need outgrows osascript.)

3. **gpui exposes only window-level AX.** A spike confirmed gpui 0.2.2 has no `accesskit` dependency:
   a captured window exposes to AX only the **window** (AXRole/AXTitle/AXSize/AXPosition) + its AppKit
   titlebar chrome (traffic-light buttons + the title static-text). Inner gpui-rendered content is
   Metal-painted and **invisible to AX**. Therefore window-level clauses (e.g. foundation-spike R2)
   assert via AX, and **inner-content clauses (R9/R10) ride the screenshot baseline** (which R10
   already mandates). The full AX element-tree API is still built + tested over synthesized snapshots.
   Note: the AX window size includes the ~32px titlebar (a 400×300 inner window reports ~400×332).

4. **ACCEPTED-UNTESTABLE shim.** The display/subprocess code can't be reliably unit-/mutation-/
   coverage-gated (it launches real GUI windows; its error arms need fault injection; the runner
   executes in a child process). So `os_shim`, `launch`, `selftest`, and the runner bin are
   `#[cfg_attr(test, mutants::skip)]` AND excluded from the coverage gate via a single **documented**
   `--ignore-filename-regex` in `scripts/gates.sh rust_cov` (the policy's §0/D1-sanctioned explicit
   exclude). Every *testable* line — the whole pure layer + `ax_exec`/`capture`'s pure fns — stays
   in the 100% denominator. The shim is proven by a **headed self-test** (`tests/headed_selftest.rs`,
   `#[ignore]`): it launches the runner, AX-asserts the window, captures, and checks a committed
   screenshot baseline. It is run in a headed lane, NOT the per-commit gate, so the gate stays
   display-free and mutation never spawns a window per-mutant.

5. **Fail-closed baseline workflow.** `resolve_approval` yields `WriteBaseline` ONLY when an operator
   sets `MARLEY_VISUAL_APPROVE=1` AND the capture doesn't match; every mismatch/missing with approve
   unset (every CI + `cargo nextest` run) is `FailWritePending` — it writes `target/visual/pending/`
   artifacts and never touches the committed `tests/visual/baselines/`. The committed self-test
   baseline (`harness_selftest.png`, 800×664 @2× retina) was generated once with approve=1 and is
   checked into git.

6. **The stdout-report headed lane (#365) — state asserts without AX or pixels.** A second headed
   idiom joined the AX/screenshot family: the app binary carries an env-gated self-report
   (`MARLEY_FONT_POLICY_SELFTEST=1` → one tab-delimited post-boot line — resolved mono family +
   flash text, both tab/newline-stripped — then `cx.quit()`; inert otherwise, the
   `MARLEY_WEBVIEW_PROBE` gate precedent), and `crates/marley_app/tests/headed_fonts.rs` spawns
   the real binary with a tempdir-hermetic `HOME`/`TMPDIR`/cwd, a 20 s deadline-poll, and
   content-pinned asserts on the line. This is the seam for contracts that live in app STATE
   under a REAL text system (the #344/#361 resolvable-font arms — `#[cfg(test)]` oracles don't
   exist in a spawned binary, and a Metal-painted flash never reaches AX): needs only a live
   WindowServer session — no Accessibility, no Screen Recording, no activation (the armed path
   skips `cx.activate`). Headed-lane targets now: `headed_selftest`, `headed_text` (harness),
   `headed_shell`, `headed_panes`, `headed_fonts` (app), `headed_widgets` (ui_components) — all
   `#[ignore]`, run per-target with `cargo test -p <pkg> --test <t> -- --ignored`.

## Future upgrade — gpui-native offscreen render (Zed finding)

> **#424 spike verdict (2026-08-14): WAIT, pre-de-risked.** The current Zed tree splits gpui
> into an unpublished crate family; the offscreen mechanism (measured at pin `a21007b7`) is a
> ~1,300–1,500-line adaptation — deliberately NOT taken early. The environmental unknowns are
> CLOSED: offscreen Metal + CoreText enumeration + glyph rasterization all proven over ssh
> (no WindowServer). Adoption recipe, traps, and the release watch live in
> `docs/planning/tickets/closed/TICKET-424-render-to-image-backport-spike.md`; #271 consumes it.

The whole subprocess/osascript/`screencapture` shim (decisions 1, 2, 4) exists because gpui **0.2.2 owns
the main thread and blocks**, forcing out-of-process observation — which brought the frontmost-window race
and shadow-offset hazards recorded in the self-test PRs (and the sibling live-app driven-capture path). The
Zed gpui round-2 review found gpui **already ships** the deterministic alternative, at **zero license
cost** (`[gpui Apache-2.0]`, `test-support` feature):

- **`Window::render_to_image() -> Result<image::RgbaImage>`** (`window.rs`) — renders the current scene
  **offscreen**: no window compositor, no frontmost race, no shadow offsets. Exposed as
  `capture_screenshot(window)` on `HeadlessAppContext` / `VisualTestContext`.
- **`VisualTestContext`** — `simulate_keystrokes(win, "cmd-d")` / `simulate_input` / `dispatch_action` /
  `simulate_mouse_*`, all injected **straight into the entity tree** (no CGEvents, no "input hits the
  frontmost window" problem) — which would also retire `send_keystroke` and the deferred R27 pointer/clock.

Adopting it would replace this crate's osascript AX + `screencapture` shim with an in-process render+assert,
and is the recommended next move (see [`zed_architecture/crates/gpui.md`](../zed_architecture/crates/gpui.md)
§9 "Platform, Scene & headless render" / §10 "Test support").

**Status after M16 #264 — the INPUT+STATE half is ADOPTED; the PIXEL half is not available on 0.2.2.**
Ground truth from the vendored crates.io `gpui-0.2.2` (correcting the paragraph above, which described the
NEWER Zed tree): `VisualTestContext`/`TestAppContext` + `simulate_keystrokes`/`simulate_input` DO ship
behind `test-support` — but `render_to_image`/`capture_screenshot` do **not** exist there, and the test
platform's `draw(&Scene)` is a no-op (no pixels are ever produced). Marley's adoption:
`crates/marley_app/src/headless_drive.rs` — a `#[gpui::test]` lane that boots the real `RootView` on a
tempdir config (`RootView::new_in`, the §14 seam added for this), injects real keystrokes through the real
key ladder, and asserts state (boot/launcher, ⌘T tab+1, the #265 ⌘D split incl. the selection landing).
Hard-won lane rules: FOCUS the root handle before injecting (a test window starts unfocused — the
driven/pixel lane below has the same requirement in a different guise: **#383** proved a `clickat:<pane>`
mouse-DOWN is what sets gpui keyboard focus there, while a programmatic app raise / the `focus` verb does
NOT, so driven keys land only AFTER a click), and END with
an off-thread session reap (`reap_sessions` — dropping a live `TerminalSession` on the test thread blocks;
the stack-sampled hang is documented in the #264 pipeline notes). Pixel capture stays HERE (the headed
shim) until a gpui upgrade ships `render_to_image` — tracked by the #264 follow-up ticket.

A third lane rule (M21 #334): a test that waits on a REAL subprocess (a spawned shell reporting its
integration `pwd`) must poll EVENT-DRIVEN to a generous FINITE deadline, not a fixed iteration count. The
shared `poll_until` helper loops `sleep(25ms)` + `tick_pump` (the mock-clock pump idiom — `advance_clock` +
`run_until_parked`, the per-iteration body) while the probe is unmet AND `std::time::Instant::now() <
deadline` (`POLL_CEILING = 30s`, independent of gpui's mock clock). A healthy box still exits in ~0.1s; a
loaded box waits longer instead of failing. The old fixed 200-iteration (~5s) budget failed the
`cargo mutants` baseline under load (exit 4 = "not a valid measurement" → the commit gate fails closed).
The Shape-2 worker-thread polls (waiting on a background syntax-tree landing keyed on `(nonce, version)`) get
the same treatment via a probe-FIRST sibling `poll_until_pre` (M22 #364): identical finite `POLL_CEILING`
deadline + mock-clock pump, but it evaluates the probe ONCE BEFORE the first `tick_pump` (not body-then-probe),
because `syntax_async`'s "the cache lags immediately after the keystroke" read observes the pre-tick state.
`poll_until` (body-first, real-PTY spawns) and `poll_until_pre` (probe-first, worker-parse landings) are the
harness's two poll helpers; all seven Shape-2 sites use the sibling, replacing their fixed 100-iteration
(~2.5s) budgets.

## Dependencies

REUSE only: `gpui` (the headed window + framebuffer), `image` (PNG-only, encode/decode of baselines),
`image-compare` (SSIM perceptual diff; MIT — replaces the AGPL `dssim`), `marley_command` (spawning
the runner child). gpui's large tree introduced **MPL-2.0 / CC0-1.0 / NCSA** licenses and 5
gpui-transitive **unmaintained** RUSTSEC advisories — all allowlisted/ignored in `deny.toml` with
per-entry reasons (chad-approved; a private non-OSS app). See
[`docs/planning/pipeline/completed/marley-visual-harness.notes.md`](../planning/pipeline/completed/marley-visual-harness.notes.md)
for the full design + the env-tests-need-`#[serial]`-under-cargo-mutants note.

## Provenance (clean-room)

`[Marley-original]` dev/test-support crate on `[gpui Apache-2.0]`. The whole harness is **INVENT** (net-new
Marley code — the pure geometry/diff/baseline/AX layers, the AppleScript generator, the subprocess model);
it carries **no** Warp code and links into no shipped product binary. Its deps are the REUSE set above —
gpui (Apache-2.0, the Zed-authored renderer Marley already depends on), plus `image` / `image-compare` /
`marley_command`. It is one of Marley's three direct gpui dependents, alongside `marley_app` and
`marley_ui_components` — all of which render gpui-native, not on WarpUI (see
[`warp_architecture/subsystems/01-ui-framework-rendering.md`](../warp_architecture/subsystems/01-ui-framework-rendering.md)
§ "Marley status @ M15"). Workspace license **`MIT OR Apache-2.0`**; carries **no** Warp copyleft.

## Deferred (follow-up tickets)
- **R27 `SyntheticInput`** — PARTIALLY landed: keystroke injection now ships (`send_keystroke`, M1.C #23,
  for the headed cmd-d split). The remaining pieces — synthetic **pointer/press** + a synthetic **clock**
  (only ui-components hover/press/tooltip needs them) — stay deferred.
- **R25 `cargo xtask visual review`/`approve` CLI** — the `resolve_approval` logic ships + is tested;
  the thin xtask wrapper is a follow-up.
- Non-macOS snapshot capture; animation assertions; cross-machine pixel-exactness.
