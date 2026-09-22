# integration

> Per-crate reference (Marley round 2) — crate dir `crates/integration`. Marley is forked from Warp ([warpdotdev/warp](https://github.com/warpdotdev/warp)).
>
> **Provenance (round 3):** `[Warp-derived/AGPL]` — Warp's headless end-to-end GUI-drive harness. Marley's counterpart is `[Marley-original]`: `crates/marley_visual_harness` + `scripts/gates.sh` **gate-15** (headed gpui + AXUIElement + screenshot baseline). See [subsystem 07 → Marley status @ M15](../subsystems/07-app-entry-build-tooling.md#marley-status--m15).

| Field | Value |
|-------|-------|
| Subsystem | [07 — App Entry & Build Tooling](../subsystems/07-app-entry-build-tooling.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`); no own LICENSE marker. |
| Internal deps | **10** |
| Used by | **0** |

## Purpose

`integration` is the **end-to-end / integration-test harness** for the `warp` app. It defines a
`Builder` for scripting UI-driven test scenarios (steps, setup/cleanup callbacks, timeouts, seeded
user-defaults and persisted data, real-or-headless display), a large catalogue of concrete test
scenarios in `test.rs`, and a binary (`warp-integration-test`) that boots the real app via
`warp::run_integration_test` and runs a named scenario. It exists so Warp can drive the actual GUI
through realistic flows (open settings, run a command, apply a theme, completions, text selection,
waterfall input, etc.) rather than only unit-testing internals.

## Key types, modules & public API

`crates/integration/src/lib.rs` re-exports:

- **`pub use builder::Builder`** (`builder.rs`) — `pub struct Builder` with a fluent API:
  `Builder::new()`, `with_timeout`, `set_should_run_test`, `with_real_display`, `with_step` /
  `with_steps`, `with_step_group_name`, `with_setup` / `with_cleanup` / `with_on_finish`,
  `with_user_defaults`, `with_static_persisted_data`, `use_tmp_filesystem_for_test_root_directory`,
  and `build(test_name, create_temp_dir_for_test) -> TestDriver`. The produced `TestDriver` is what
  `warp::run_integration_test(driver)` consumes.
- **`pub mod test`** (`test.rs`) — the scenario library: `pub fn test_single_command()`,
  `test_open_and_close_settings()`, `test_add_theme_to_warp_config()`,
  `test_palette_opens_when_theme_chooser_is_open()`, `test_completions_with_autocd()`,
  `test_clear()`, `test_waterfall_input*()`, `test_with_24_bit_color()`, … each returning a `Builder`.
  Also `pub struct TestOnlyAssets` (a `rust-embed` asset bundle for fixtures).
- **`pub mod user_defaults`** and **`pub mod util`** — seeding helpers / shared test utilities.
- **`pub use warp::integration_testing::view_getters`** — re-exports the app's sanctioned view
  accessors so scenarios can assert against the live UI without exposing `warp`-internal types.
- **`pub use warpui_core::integration::TestStep`** — the step primitive scenarios are built from
  (`step.rs` adapts these into the builder).
- Binary `crates/integration/src/bin/integration.rs` — `warp-integration-test`: a `clap` `Args`
  (`integration_test_name` + optional `WorkerCommand`), sets a `Channel::Integration` `ChannelState`
  pointed at IANA-blackhole IPs (`192.0.2.0:9`) so no real server traffic occurs, then dispatches the
  named test through `integration::test::*`.

## Depends on (internal)

- [`warp`](./warp.md) (feature `integration_tests`) — the app under test; provides
  `run_integration_test`, `integration_testing::view_getters`, and `TestDriver`.
- [`warp_cli`](./warp_cli.md) (feature `integration_tests`) — `WorkerCommand` subcommands the runner
  forwards.
- [`warp_core`](./warp_core.md) — `Channel`/`ChannelState`/`ChannelConfig`/`AppId` used to configure
  the `Integration` channel in the bin.
- [`warpui`](./warpui.md), [`warpui_core`](./warpui_core.md), [`warpui_extras`](./warpui_extras.md) —
  the GUI framework + `integration::TestStep` (MIT).
- [`settings`](./settings.md) — seeding/asserting user settings in scenarios.
- [`command`](./command.md) — process spawning in test setup.
- [`sum_tree`](./sum_tree.md) — shared data structure used by some scenarios/utilities.
- [`app-installation-detection`](./app-installation-detection.md) — install-state used by certain flows.

(Also pulls non-graph deps: `warp-command-signatures`, `warp_multi_agent_api`, `mockito`, etc.)

## Used by (internal dependents)

- **None (0).** This is a leaf: a test harness + binary, imported by nothing in the workspace. It is
  the consumer at the top, not a library others build on. (Note: distinct from the *app's own*
  `integration` binary target in `app/src/bin/integration.rs`.)

## Related crates

- [`warp`](./warp.md) — the system under test; read `warp::integration_testing` alongside this.
- [`warp_tui`](./warp_tui.md) — the other headless front-end of the app; conceptually similar boot path.
- [`warp_cli`](./warp_cli.md) — shares the `WorkerCommand`/channel-config bootstrapping.

## Marley relevance

**Classify: KEEP, then EXTEND (later) / candidate STUB.** Value vs. cost:
- **KEEP** the harness — it is exactly the regression net for goals (1)–(3): once Marley adds a
  custom panel (goal 1), exposes session spawn/write/read (goal 2), or stubs auth (goal 3), new
  scenarios here (`Builder` + `TestStep`) are how we prove those flows still work headlessly without a
  real backend (the bin already black-holes server traffic to `192.0.2.0:9`).
- **EXTEND** with Marley-specific scenarios (e.g. "boots straight to terminal with no login modal"
  validates goal 3; "custom panel opens from palette" validates goal 1).
- **Rebrand (goal 4):** low-touch — strings like `Channel::Integration`, `AppId::new("dev","warp",…)`,
  `warp_integration.log`, and `test_add_theme_to_warp_config` carry the Warp name but are
  test-internal; rename opportunistically, not urgently.
- It could be **STUB/dropped from the shipped build entirely** — it's dev-only and never linked into
  the product binary — so it never blocks an offline boot.

## Notes / gotchas

- **Two different "integration" things:** this crate's `warp-integration-test` binary, *and* the
  `app/src/bin/integration.rs` target inside the `warp` package. Don't conflate them.
- **No real network:** the runner deliberately configures `server_root_url`/`rtc_server_url` to the
  IANA test-net (`192.0.2.0`) discard port so tests never hit Warp's backend — handy precedent for
  Marley's offline/de-auth work.
- **Embedded fixtures:** `TestOnlyAssets` uses `rust-embed`; test data lives under
  `crates/integration/tests/data` and `crates/integration/assets`. See
  `crates/integration/tests/INTEGRATION_TESTING.md`.
- Depends on `warp`/`warp_cli` **with the `integration_tests` feature** — building this crate pulls
  that feature into the app, which exposes `integration_testing` (otherwise private).
- Heavy non-workspace deps (`mockito`, `simplelog`, `version-compare`, `pathfinder_geometry`) are
  test-scaffolding only.
