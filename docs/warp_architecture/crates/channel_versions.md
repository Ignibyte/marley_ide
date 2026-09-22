# channel_versions

> Per-crate reference (Marley round 2) — crate dir `crates/channel_versions`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (round 3):** `[Warp-derived/AGPL]` — Warp's dev/preview/stable update-channel version metadata. Marley has **no update channels and no auto-update** (single un-channelled `marley` binary) → `[Marley-original: none]`, no counterpart. See [subsystem 07 → Marley status @ M15](../subsystems/07-app-entry-build-tooling.md#marley-status--m15).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL — auto-update/de-auth]` — channel version feed (tied to `FetchChannelVersionsFromWarpServer`); Marley is single-channel offline (`marley_core::Config`). Obviated. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [platform-settings-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (workspace `AGPL-3.0-only`; no own LICENSE marker) |
| Internal deps | 0 |
| Used by | 2 (`warp`, `warp_terminal`) |

## Purpose

Defines the data model for Warp's **release-channel version metadata** (the `dev` / `preview` / `stable` update channels) and the logic for parsing/comparing Warp version strings and applying per-OS overrides. This is the schema the Warp server publishes and the client deserializes to decide "is there a newer build, am I past the soft cutoff, do I show the update-by warning banner / prominent-update UI." It has **no internal dependencies** (only third-party serde/regex/chrono), making it a leaf data crate shared between the app and the `channel-versions` repo's tooling.

## Key types, modules & public API

- `ChannelVersions` (`src/lib.rs`) — top-level deserialized document: `dev`/`preview`/`stable: ChannelVersion` plus optional `changelogs: Option<ChannelChangelogs>`.
- `ChannelVersion` — wraps a `VersionInfo` plus a `Vec<VersionOverride>`. `version_info()` resolves overrides against `overrides::Context::from_env()`; `version_info_for_execution_context(&Context)` resolves against an explicit context (used by the `apply_overrides` bin).
- `VersionInfo` — the consumer-facing fields: `version`, `version_for_new_users`, `update_by: Option<DateTime<FixedOffset>>`, `soft_cutoff`, `last_prominent_update`, `is_rollback`, `cli_version` (with `cli_version()` falling back to `version`).
- `ParsedVersion` (`src/lib.rs`) — `(major, date: NaiveDateTime, patch)` parsed from the `v{maj}.{YYYY.MM.DD.HH.MM}.._{patch}` format via `VERSION_RE`; implements `Ord` and is memoized in a `MemoMap` cache. `TryFrom<&str>`.
- `overrides` module (`src/overrides.rs`) — `Context { target_os }`, `TargetOS` enum (`MacOS`/`Linux`/`Windows`/`Web`/`Unknown(String)`, with `current()` from build cfg), private `OverridePredicate::TargetOS`, `VersionOverride`, and `VersionInfo::with_overrides_applied(overrides, context)` (applies the **first** matching override only).
- `ChannelChangelogs` / `Changelog` / `Section` / `MarkdownSection` — changelog content model (legacy bullet `Section`s + newer `markdown_sections`).
- Binaries (feature `cli`, default on): `apply_overrides` (reads channel-versions JSON, applies overrides for a `--target-os`, re-emits JSON) and `version_compare` (exits non-zero if a rollout version is older than the current one — guards the rollback workflow).

## Depends on (internal)

- *(none — leaf crate; third-party deps only: `serde`, `serde_json`, `regex`, `chrono`, `lazy_static`, `memo-map`, optional `clap`, `anyhow`.)*

## Used by (internal dependents)

- [warp](./warp.md) — the GUI app reads channel versions to drive autoupdate / cutoff banners.
- [warp_terminal](./warp_terminal.md) — terminal binary consuming the same version metadata.

## Related crates

- [warp_channel_config](./warp_channel_config.md) — sibling "channel" concept, but for *build/runtime config* rather than *version metadata*; both feed the channel/update story.
- [warp_features](./warp_features.md) — channel-gated feature flags (`FetchChannelVersionsFromWarpServer` lives there) decide *where* version info is fetched from.

## Marley relevance

**STUB / REMOVE candidate.** This crate exists to talk to Warp's update infrastructure (GCP / Warp server channel-versions endpoint) which Marley will not have. For an offline, self-built Marley there is no `dev`/`preview`/`stable` ladder and no update-by banner. Concretely: keep the *types* compilable but **stub the fetch path** — wherever `warp`/`warp_terminal` query channel versions, short-circuit to a single static `VersionInfo` (current build, no `update_by`, no `soft_cutoff`) so autoupdate/cutoff UI is inert (supports goal (3) de-auth/offline boot by removing a server round-trip, and goal (4) de-Warp by killing the `*.warp.dev` update dependency). The `apply_overrides`/`version_compare` bins are release-pipeline tooling for *Warp's* repo and can be **REMOVE**d outright. Net: low-risk to neuter; the only callers are two binaries.

## Notes / gotchas

- The version string grammar is rigid: `v(\d+)\.(.+)\.(.+)_(\d+)` with the middle group parsed as `%Y.%m.%d.%H.%M` — Marley's own versioning will not match it, so don't route Marley build IDs through `ParsedVersion`.
- `PARSED_VERSIONS_CACHE` is a process-global `MemoMap` that never evicts — fine for the handful of channel versions, not a general version parser.
- `TargetOS::Unknown(String)` is a `#[serde(untagged)]` catch-all so unknown server enum variants don't fail deserialization; `clap` skips it via `value(skip)`.
- Only the **first** matching override is applied (`with_overrides_applied` breaks on first hit) — order in the JSON matters.
