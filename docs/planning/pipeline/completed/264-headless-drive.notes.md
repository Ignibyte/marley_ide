# 264-headless-drive — Notes

- **Forge ticket:** #264 bdae95c9-2b24-448d-a42d-8296fd230c4d
- **AAR:** 7c42b598-3032-442a-bb76-48ec050727ba
- **Local ticket doc:** docs/planning/tickets/open/TICKET-264-headless-drive.md
- **Pipeline spec:** 264-headless-drive.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan
- **Request:** /goal batch, ticket 6 of 10. Adopt gpui offscreen render +
  VisualTestContext per the roadmap Phase A.
- **LOAD-BEARING CORRECTION (verified in the vendored crates.io source):**
  gpui 0.2.2 has `VisualTestContext` (test_context.rs:669) with
  simulate_keystrokes/input (:715/:721), mouse events, dispatch_action,
  TestAppContext::draw (:814) — behind `test-support` — but NO
  `render_to_image`/`capture_screenshot`, and the TestWindow platform's
  `draw(&scene)` is `{}` (test/window.rs:269 — a no-op; no pixels exist).
  The ticket's render_to_image citation (window.rs:2272) is from the NEWER
  Zed tree the docs effort cloned, not the crates.io dep. → the slice is
  INPUT+STATE now, PIXELS on a gpui upgrade (follow-up ticket at P5).
- **Classification / tier:** chore (testing infra), one slice with an
  explicit deferred half.
- **Forge recall (§18.3):** AAR opened; the memory's screencapture hazards
  (locked-screen, shadowing, offsets) are the retirement targets; the
  `*_in(dir)` config-isolation convention (§14) binds the tests.
- **Discovery:** marley_app has gpui = "0.2.2" plain (no features);
  tests/ has headed_panes/headed_shell/integration; RootView::new(_window,
  cx) at app.rs:569 (dependency surface = design's probe).
- **Decisions:** D1–D4 in the spec.
- **Autonomy note:** /goal run — no human pause.

## Phase 2 — Design

### Architecture / approach (§20 stays N/A — gpui's own public harness)
1. **Config isolation via the §14 `*_in` convention at the ctor** (probes:
   `SettingsManager::load(settings_file_in(dir))` carries its file path; ALL
   10 persist sites go through `self.settings` → injecting at boot covers
   every write; paths.rs deliberately has NO env override):
   `RootView::new_in(config_dir: Option<PathBuf>, window, cx)` holds today's
   body with the one `marley_config_dir()` call replaced by the param;
   `pub fn new` delegates with `marley_config_dir().ok()`. run()'s separate
   geometry read (app.rs:7492) is outside the view — untouched.
2. **The test lane**: `#[cfg(test)] mod headless_drive;` (new file
   src/headless_drive.rs — in-crate for private-field access; test-only code
   is outside coverage/mutation denominators). Dev-dep
   `gpui = { version = "0.2.2", features = ["test-support"] }` (the normal
   dep stays featureless; feature unification applies to test builds only).
   `#[gpui::test]` is re-exported (gpui.rs:81); TestAppContext::add_window
   (:215); VisualTestContext::from_window + simulate_keystrokes (:715).
3. **Test mechanics**: each test — tempdir config; `cx.add_window(|w, cx|
   RootView::new_in(Some(tmp), w, cx))`; explicitly focus the root handle
   (`window.focus(&view.focus_handle)` — a test window starts unfocused, and
   the on_key_down listener hangs off track_focus); run_until_parked (the
   pump loop PARKS at its timer await under the test dispatcher — the
   infinite loop is timer-gated, so parking terminates); simulate_keystrokes
   ("cmd-t" etc.); assert via in-crate reads (tab_count, editor().is_some())
   through window.read_with. RootView::new spawns a REAL zsh per test —
   fine headless (PTYs need no window server; nextest = process-per-test;
   Drop reaps).
4. **Fresh-boot state verification item**: a virgin config dir may boot to
   the #247 LAUNCHER (0 projects — keystrokes would hit the launcher, not
   the keymap). Implement probes this; if launcher, each test pre-writes a
   minimal settings.toml fixture (one terminal project via the #163 grid
   codec) into the tempdir so boot lands in a workspace deterministically.
5. `.mcp.json` cwd read at boot builds a ForgeClient but fetches NOTHING
   (boot never touches the network) — safe headless.

### File manifest
| file | change |
|---|---|
| crates/marley_app/Cargo.toml | [dev-dependencies] gpui with test-support |
| crates/marley_app/src/app.rs | RootView::new → thin delegate; new_in(config_dir, …) carries the body (one call-site swap) |
| crates/marley_app/src/lib.rs | `#[cfg(test)] mod headless_drive;` |
| crates/marley_app/src/headless_drive.rs (NEW) | the 4 #[gpui::test]s + the fixture helper |
| scripts/selftest/README.md + docs/marley_architecture/marley_visual_harness.md | the headless-first lane doc (P5-adjacent, same slice) |

### Regression test plan
| REQ | Test |
|---|---|
| REQ-001 | the lane compiles + runs under plain `cargo nextest run` (in-transcript) |
| REQ-002 | `boot_headless_parks_clean` — add_window, run_until_parked, ≥1 draw, no panic; asserts ≥1 project/tab |
| REQ-003 | `cmd_t_adds_a_tab` — focus root, count, simulate "cmd-t", count+1 |
| REQ-004 | `cmd_d_splits_by_surface` — terminal tab: "cmd-d" → tab+1; then open the editor tab (in-crate call or ⌘P flow — implement picks the cheapest deterministic route, likely a direct open_file_in_viewer call), "cmd-d" → tab count UNCHANGED |
| REQ-005 | every test boots from a TempDir; a post-test assert that the user's real `~/.marley` mtime is untouched is NOT reliably assertable — instead the ctor-injection is the guarantee (design review) + the fixture asserts writes land in the tempdir |
| REQ-006 | docs updated + the pixel follow-up ticket id recorded in P5 notes |
- Uncoverable: none new (tests are tests; new_in is the same shim body).

### Risks / decisions
- R1 focus not reaching the root listener headless → explicit focus() call;
  if keystrokes still don't land, fall back to dispatching KeyDownEvent via
  simulate_event (same entry point).
- R2 fresh-boot = launcher → the settings fixture (deterministic boot).
- R3 the pump loop under the test executor — parks at the timer (reasoned
  above); a hang would show as a nextest timeout on first run.
- R4 zsh spawn per test — macOS ships zsh; ~4 short-lived shells.
- R5 flake — validate runs the lane 3×.

## Phase 3 — Implement
- **Built to manifest:** Cargo dev-dep `gpui { test-support }` (normal dep
  untouched); `RootView::new` → 3-line delegate; `new_in(config_dir:
  Option<PathBuf>, …)` carries the body (the ONE `marley_config_dir()` call
  became the param — everything else byte-identical); lib.rs wires
  `#[cfg(test)] mod headless_drive;`; headless_drive.rs = the seed helper
  (via the crate's own pub `persist_shell` — no codec duplication), the
  boot helper (add_window → VisualTestContext → run_until_parked → explicit
  `window.focus(&view.focus_handle)` → park), and 4 tests
  (virgin-dir→launcher, seeded-dir→workspace, ⌘T tab+1, the #265 ⌘D split
  with the editor half opened via the in-crate `open_file_in_viewer`).
- **Visibility deltas (design's "in-crate access" realized):**
  `RootView.shell` + `.focus_handle` → pub(crate) (commented #264);
  `open_file_in_viewer` → pub(crate). No new PUBLIC surface.
- **Deviations:** seed uses `persist_shell` instead of a raw
  `set::<ShellLayoutSetting>` (the setting struct is module-private; the
  pub helper is the cleaner seam anyway).
- **Verification:** `cargo check -p marley --all-targets` green; smoke run →
  **all 4 PASS in 0.066s** (after the teardown fix below).
- **The teardown hang (found + fixed in-phase):** the first smoke run hung
  two tests ~16min. `sample <pid>` gave the smoking stack:
  `TestAppContext::quit → App::shutdown → release_dropped_entities →
  drop_in_place<RootView> → Workspace → Vec<Project> → Vec<Tab>` — the test
  BODY had already passed; dropping a live `TerminalSession` BLOCKS. The app
  itself never drops sessions on the UI thread (the boot path reaps the
  unused boot PTY off-thread) — so the lane now ends every test with
  `reap_sessions` (mem::replace the workspace with an empty one, drop the
  old on a detached thread), mirroring the app idiom. Boot tests had passed
  only because their zsh wasn't fully up yet — a latent flake, now closed
  uniformly.
- **Process note:** the phase-gate hook briefly blocked the fix because the
  spec status had been flipped to Implement-PASS BEFORE the smoke run
  finished — reverted the status (the phase genuinely wasn't done), applied
  the fix, re-ran. Lesson: don't flip a phase PASS with a verification still
  in flight.

## Phase 3.5 — Inspect
- **Critic run:** 1 focused (test honesty + isolation + shipping-path
  safety + reap hygiene), heavy on measurement (cargo tree feature
  unification, mutants --list + --in-diff, 3 lane executions, zsh/orphan
  counts, the alacritty Pty::drop source).
- **Findings ledger:**

| # | sev | finding | verdict | action |
|---|---|---|---|---|
| 1 | HIGH | detach trap 4th strike, NEW SHAPE: the ctor SPLIT left the skip on the 1-line `new` while the ~600-line body moved to unskipped `new_in` → 57 live mutants; the --diff gate BLIND (its only in-diff mutant is unviable Default::default() → "0 viable" PASS) — the FULL gate would collapse later | REAL (block) | FIXED: skip moved onto new_in (comment documents the strike); --list re-run = 0 new_in mutants; BF recorded (rule refined: --list after any body-MOVE, and a green --diff on a shim refactor can be vacuous) |
| 2 | MED | cmd_d's editor half was swallow-blind (tab-count-unchanged also passes if the key never dispatched) | REAL | FIXED: + sel_before==None + sel_after==Some((0,5)) "alpha" — the positive half proves the scoped binding FIRED (the critic traced the whole chain to guarantee determinism) |
| 3 | LOW | isolation: sound — all persists ride the injected manager; zdotdir is per-pid tmp (nextest process-per-test); the spawned zsh's rc never sources the user's ~/.zshrc; .mcp.json unread (test cwd is the crate dir); PATH-scan + repo walk are read-only. Residual: plain-`cargo test` threading could tear the shared zdotdir write — gates use nextest; noted | accepted + noted | none |
| 4 | LOW | reap: Pty::drop = SIGHUP + blocking wait; detached thread absorbs it; measured 3 runs — zsh 11→11, 0 orphans, 0 zombies. Nuance: the VIRGIN boot path drops its unused boot session ON-THREAD at end of new_in (pre-existing app behavior; 23ms observed) — recorded in the pixel follow-up ticket | accepted + recorded | follow-up note |
| 5 | VERIFIED | shipping path byte-equivalent (both Ok/Err arms traced); dev-dep contained (cargo tree: normal graph has NO test-support; git2/backtrace additions are dev-only); widenings consumed only by the test module | clean | — |

- **Runs:** full -p marley 373/373 (the new lane included); the lane 3×
  total, 40-67ms, zero flake.

## Phase 4 — Validate
- **Tests:** written at implement (the 4-lane) + hardened at inspect (the
  positive selection assert). RUN: the lane 5× total across phases
  (40-68ms, zero flake); full `cargo nextest run --workspace` →
  **899/899 passed, 5 skipped**.
- **Driven UI capture: N/A** — the shipping render/input path is
  byte-equivalent (critic-traced); the LANE ITSELF is the driven
  verification this ticket creates (keystrokes through the real ladder,
  state asserted; the ⌘D selection assert is the positive proof).
- **Docs:** selftest README headless-first banner + the harness arch doc's
  "Future upgrade" section corrected to 0.2.2 ground truth + adoption
  status.
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] 15/15**.
  Receipt written.
- **Pre-existing:** `block v0.1.6` note only.

## Phase 5 — Complete
- CHANGELOG entry; harness docs (above); the PIXEL follow-up ticket filed
  (gpui upgrade → render_to_image; incl. the virgin-path on-thread drop
  nuance from inspect F4).
- AAR submitted. Captured: BF-claude-mutants-skip-body-moved-out-fourth-
  strike (the ctor-split shape + the vacuous-green --diff insight).
- TICKET-264 → closed/; forge #264 → done; docs → completed/.
