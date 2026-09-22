# warp_features

> Per-crate reference (Marley round 2) — crate dir `crates/warp_features`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Marley-original]` — rebuilt as `marley_core::FeatureFlag`: a three-layer tri-state gate (override → user-pref → baseline); four offline flags, **no account/login gating**. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [platform-settings-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (workspace `AGPL-3.0-only`; no own LICENSE marker) |
| Internal deps | 0 |
| Used by | 3 (`input_classifier`, `settings`, `warp_core`) — and transitively almost everything via `warp_core` |

## Purpose

The app-wide **feature-flag registry**. A single `FeatureFlag` enum enumerates every gated feature, and a set of lock-free global atomic arrays hold each flag's effective state. It is a zero-internal-dependency leaf crate (only `enum-iterator`) precisely so it can be checked from anywhere — UI, core, classifiers, settings — without dependency cycles. It distinguishes the *baseline* per-channel state (set at startup) from a *user preference* override and a *test* thread-local override.

## Key types, modules & public API

- `FeatureFlag` enum (`src/lib.rs`) — ~hundreds of variants (e.g. `Changelog`, `CrashReporting`, `Autoupdate`, `KnowledgeSidebar`, `RuntimeFeatureFlags`, `CloudObjects`, `FetchChannelVersionsFromWarpServer`, `LocalClaudeCodexChildHarnesses`, `AgentSharedSessions`, `SettingsFile`, …). Derives `Sequence` (`enum-iterator`) so `cardinality::<FeatureFlag>()` sizes the state arrays at compile time.
- Resolution order in `FeatureFlag::is_enabled(&self) -> bool`: thread-local **override** → **user preference** (`USER_PREFERENCE_MAP`) → **global flag state** (`FLAG_STATES`) → `false`. In debug builds it asserts flags aren't read before `mark_initialized()`.
- Mutators: `set_enabled(self, bool)` (global; panics if called from a plain test), `set_user_preference(self, bool)`, and (feature `test-util`) `override_enabled(self, bool) -> overrides::OverrideGuard` (RAII thread-local override).
- `mark_initialized()` — flips the debug-only `FEATURES_INITIALIZED` guard.
- `flag_description(&self) -> Option<&'static str>` — Preview-changelog-facing descriptions (intentionally sparse; only Preview-exclusive flags).
- Channel flag sets (`pub const`): `DEBUG_FLAGS`, `LOCAL_FLAGS`, `DOGFOOD_FLAGS`, `PREVIEW_FLAGS`, `RELEASE_FLAGS`, `RUNTIME_FEATURE_FLAGS` — the lists a channel's startup code applies.
- Internal state: `static FLAG_STATES: [AtomicBool; N]`, `static USER_PREFERENCE_MAP: [AtomicTriState; N]` (a `TriState`/`AtomicTriState`/`AtomicU8` of unset/true/false), `static FEATURES_INITIALIZED: AtomicBool`. The `overrides` module has two cfg variants (no-op in release, thread-local `HashMap` under `test-util`, with `get_overrides`/`set_overrides` re-exported).

## Depends on (internal)

- *(none — leaf crate; only third-party `enum-iterator`.)*

## Used by (internal dependents)

- [warp_core](./warp_core.md) — gates core behavior; itself depended on by ~most of the workspace, so `warp_features` is transitively ubiquitous.
- [settings](./settings.md) — surfaces user-preference toggles for flags.
- [input_classifier](./input_classifier.md) — gates classifier behaviors behind flags.

## Related crates

- [warp_core](./warp_core.md) — primary consumer and the bridge that spreads flag access across the app.
- [channel_versions](./channel_versions.md) / [warp_channel_config](./warp_channel_config.md) — the "channel" trio; channel identity decides which `*_FLAGS` const gets applied at boot.
- [settings](./settings.md) — persistence/UI for the user-preference layer.

## Marley relevance

**KEEP + EXTEND (defer rename).** This is exactly the mechanism Marley needs to ship new behavior incrementally. For goal (1) **custom UI panel**, add a `FeatureFlag::MarleyPanel` (or similar) and gate the panel on it; for goal (2) **session spawn/write/read**, flags like `LocalClaudeCodexChildHarnesses` are directly relevant prior art for gating child-harness behavior. For goal (3) **de-auth/login stub**, *force-set* the team/cloud-gated flags (`CloudObjects`, `AgentSharedSessions`, anything that today depends on a paying-team check or server experiment) to a fixed value at boot so they no longer consult auth/server state. For goal (4) **de-Warp rebrand**, retire server-coupled flags like `FetchChannelVersionsFromWarpServer` along with the `channel_versions` stubbing. Renaming the crate to `marley_features` is feasible but it is transitively load-bearing (via `warp_core`), so **defer the rename** to a coordinated pass; the *enum and API* are clean to keep as-is.

## Notes / gotchas

- State arrays are sized via `cardinality::<FeatureFlag>()` at compile time — adding/removing a variant resizes every static array automatically; just keep the `Sequence` derive.
- `set_enabled` **panics** if called from a non-`integration_tests` test — tests must use `override_enabled` (the RAII guard). Overrides are thread-local and do **not** propagate to spawned threads.
- Debug builds assert any `is_enabled()` read happens after `mark_initialized()`; forgetting to initialize feature flags surfaces as a panic, not a silent `false`.
- `flag_description` is deliberately incomplete (only Preview-exclusive flags) to keep the Preview changelog from ballooning — don't treat it as a complete catalog.
