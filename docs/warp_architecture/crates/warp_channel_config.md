# warp_channel_config

> Per-crate reference (Marley round 2) — crate dir `crates/warp_channel_config`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (round 3):** `[Warp-derived/AGPL]` — Warp's per-channel `ChannelConfig` loader (the embed-at-build-vs-runtime-generate `load_config!` fork). Marley ships one binary with **no channels** → no counterpart. See [subsystem 07 → Marley status @ M15](../subsystems/07-app-entry-build-tooling.md#marley-status--m15).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL — rebrand]` — per-channel config/branding; replaced by Marley's one canonical `marley_core::Config` (`net.ignibyte.marley`; no server/RTC/telemetry fields). See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [platform-settings-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (workspace `AGPL-3.0-only`; no own LICENSE marker) |
| Internal deps | 2 (`command`, `warp_core`) |
| Used by | 2 (`warp`, `warp_tui`) |

## Purpose

Loads the per-channel **[`ChannelConfig`](./warp_core.md)** for Warp's channel binaries (dev/preview/stable GUI + TUI). It centralizes a single decision: in a `release_bundle` build the config JSON is *embedded at compile time*; otherwise an external `warp-channel-config` generator binary is invoked at runtime to produce it. Putting this behind one macro means every channel binary loads config the same way instead of re-implementing the embed-vs-generate fork.

## Key types, modules & public API

- `load_config!($channel)` macro (`src/lib.rs`, `#[macro_export]`) — the entry point consumers call. Its `#[cfg(feature = "release_bundle")]` branch is evaluated in the **consuming** crate, so each binary crate opts into embedding by defining its own `release_bundle` feature and generating `<channel>_config.json` into its `OUT_DIR` via a build script. Embeds via `include_str!` + `load_config_from_embedded`, else calls `load_config_from_generator`.
- `load_config_from_generator(channel: &str) -> ChannelConfig` — runs `command::blocking::Command::new("warp-channel-config")` with `--channel/--target-family/--target-os` args, deserializes stdout JSON. Panics with an actionable message ("run `./script/install_channel_config`") if the binary isn't on `PATH`.
- `load_config_from_embedded(json: &str) -> ChannelConfig` — `serde_json::from_str`, panics on parse failure.
- `path_concat!` macro — OS-correct path separator join for the `include_str!` path (Windows `\` vs `/`).
- Re-exports / returns `warp_core::channel::ChannelConfig`.

## Depends on (internal)

- [command](./command.md) — `command::blocking::Command` to shell out to the `warp-channel-config` generator at runtime.
- [warp_core](./warp_core.md) — the `channel::ChannelConfig` type that is loaded/returned.

## Used by (internal dependents)

- [warp](./warp.md) — GUI app binaries load their channel config at startup.
- [warp_tui](./warp_tui.md) — TUI binaries share the same loading logic (the stated reason the logic lives here).

## Related crates

- [warp_core](./warp_core.md) — defines `ChannelConfig`; the real schema lives there.
- [channel_versions](./channel_versions.md) — sibling "channel" crate, but for *version/update metadata*, not build/runtime config.
- [command](./command.md) — the subprocess primitive used for the generator path.

## Marley relevance

**STUB.** The whole crate is a fork between "embed config at build time" and "call Warp's internal `warp-channel-config` generator (not in this repo) over `PATH`." Marley has no internal generator binary and no multi-channel release pipeline. Simplest path supporting goal (4) de-Warp rebrand and offline boot: replace `load_config!` with a hardcoded single `ChannelConfig` for Marley (one channel), so neither the missing generator panic nor the `release_bundle` build-script dance can fire. Keep the `ChannelConfig` type (it lives in `warp_core`) but **short-circuit the loader**. Renaming the crate to `marley_channel_config` is cheap (only 2 dependents) and worth doing alongside the broader rebrand. Do NOT ship the generator branch — it would fail closed on any machine without Warp's `script/install_channel_config`.

## Notes / gotchas

- **Out-of-repo dependency:** `warp-channel-config` is an internal Warp generator binary expected on `PATH`; it is *not* part of this fork. Any non-`release_bundle` build that hits `load_config_from_generator` will `panic!` on a clean checkout.
- Every failure path is a `panic!`, not a `Result` — config loading is treated as fatal-at-startup by design.
- The `release_bundle` cfg is intentionally evaluated in the *consumer*, so this crate's own feature set looks empty; grep the consuming binary crates' build scripts to see where `<channel>_config.json` is generated.
- `target_family`/`target_os` are computed from `cfg!` at compile time and passed as generator args, so a cross-compiled binary reports the *target's* OS, not the host's.
