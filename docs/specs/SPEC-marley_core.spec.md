---
spec_id: marley-core
component: marley_core
bucket: REIMPLEMENT
milestone: M0
status: draft
title: App/runtime core — SessionId, ~/.marley paths, single-channel config, feature flags
goal: Provide the shared identity, path, single-channel configuration, and feature-flag layer that every other Marley crate imports, with no cloud/auth coupling.
reuses: [directories, once_cell, serde, enum-iterator]
spec_source: "behavior-only — observable I/O of an app-runtime core: a process-unique monotonic session identity, the resolved per-user app config/data/cache directories, a single release-channel identity + its config, and a feature-flag registry with defaults and runtime overrides. No fork module/type/file/static names. Seams per standards/seam-contracts.md."
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: N/A
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
  - ./standards/seam-contracts.md
---

## Purpose
`marley_core` is the app/runtime core: the lowest load-bearing crate that defines the `SessionId` threading through every shell/block, the `~/.marley` filesystem layout, a single hardcoded `Config` (one channel, no cloud/server/auth surface), and the process-wide feature-flag registry. It exists so that low-level crates can share identity, paths, and gated behavior without depending on the app layer and without any network or login dependency. It is the offline-boot foundation for the rest of Marley.

## Public surface (the contract)
`marley_core` is the canonical owner of `SessionId` (seam-contracts §2). `marley_terminal` is a **downstream consumer**: it depends on `marley_core` for `SessionId` and does not redefine it (the shell-hook-reported id is the distinct `marley_terminal::ShellSessionId`, out of scope here).

The feature-flag layering (global baseline vs user preference vs test override) and any per-flag state storage are **private implementation details** — no internal state container, map, or atomic tri-state type appears in this surface, and the choice of iteration/sizing backend is the implementer's.

```rust
// session_id.rs — canonical owner (seam-contracts §2); marley_terminal depends on this type
pub struct SessionId(u64);          // process-unique, monotonic; PRIVATE field
impl SessionId {
    pub fn next() -> SessionId;      // allocate the next process-unique id
    pub fn as_u64(self) -> u64;      // read the wrapped value unchanged
}
impl From<u64> for SessionId {}
impl From<SessionId> for u64 {}     // SessionId: Copy + Clone + Eq + Hash + Debug

// paths.rs — all return PathBuf rooted at the resolved Marley home
pub fn marley_home_dir() -> Result<PathBuf, PathError>;   // $HOME/.marley
pub fn marley_config_dir() -> Result<PathBuf, PathError>; // <home>/config
pub fn marley_data_dir() -> Result<PathBuf, PathError>;   // <home>/data
pub fn marley_themes_dir() -> Result<PathBuf, PathError>; // <home>/themes
pub fn marley_skills_dir() -> Result<PathBuf, PathError>; // <home>/skills
pub fn marley_mcp_config_file_path() -> Result<PathBuf, PathError>; // <config>/mcp.json
pub fn marley_logfile_path() -> Result<PathBuf, PathError>;         // <data>/<logfile_name>
pub enum PathError { HomeDirUnresolved }

// config.rs — single channel, no server/cloud/auth fields
pub struct AppId { /* org, app, kind */ }
impl AppId { pub fn as_str(&self) -> &str; }
pub struct Config { /* app_id, logfile_name */ }
impl Config {
    pub fn marley() -> &'static Config;   // the one canonical, lazily-built config
    pub fn app_id(&self) -> &AppId;
    pub fn logfile_name(&self) -> &str;
}

// features.rs — flag state storage is a private impl detail (not in this surface)
#[derive(serde::Serialize, serde::Deserialize)]
pub enum FeatureFlag { /* explicit Marley-original variants */ }
impl FeatureFlag {
    pub fn is_enabled(self) -> bool;
    pub fn set_enabled(self, value: bool);          // global baseline layer
    pub fn set_user_preference(self, value: bool);  // user override layer
    #[cfg(feature = "test-util")]
    pub fn override_enabled(self, value: bool) -> OverrideGuard; // RAII thread-local
}
pub fn apply_default_flags();   // enable exactly the DEFAULT_FLAGS baseline (startup seam)
pub fn mark_initialized();      // flip the init guard after defaults are applied
pub const DEFAULT_FLAGS: &[FeatureFlag];
pub struct OverrideGuard; // reverts on Drop
```

## EARS Requirements

R1. The system shall represent `SessionId` as a transparent `u64` newtype whose `as_u64` returns the wrapped value unchanged.

R2. The system shall round-trip a value losslessly through `From<u64>` then `From<SessionId> for u64` such that the output equals the original `u64`.

R3. WHEN `SessionId::next` is called, the system shall return a `SessionId` whose `u64` is strictly greater than every `SessionId` previously returned by `next` in the same process.

R4. The system shall make `SessionId` derive `Copy`, `Clone`, `Eq`, `Hash`, and `Debug` so two `SessionId`s with equal `u64` compare equal and hash identically.

R5. WHEN `marley_home_dir` is called and a home directory is resolvable, the system shall return the path `<home>/.marley`.

R6. IF the home directory cannot be resolved, THEN `marley_home_dir` shall return `Err(PathError::HomeDirUnresolved)` and shall not panic.

R7. The system shall resolve `marley_config_dir`, `marley_data_dir`, `marley_themes_dir`, and `marley_skills_dir` as the fixed children `config`, `data`, `themes`, and `skills` of `marley_home_dir`, respectively.

R8. The system shall resolve `marley_mcp_config_file_path` as `marley_config_dir` joined with `mcp.json`, and `marley_logfile_path` as `marley_data_dir` joined with `Config::marley().logfile_name()`.

R9. The system shall expose exactly one channel: `Config::marley` returns the single canonical configuration and the public `Config` surface contains no server URL, RTC URL, cloud, telemetry-endpoint, or auth field. *(Verified at compile time — see Test Plan; ACCEPTED-UNTESTABLE for runtime mutation, mechanism stated in Mutation Targets.)*

R10. The system shall make `Config::marley` idempotent: every call returns a reference to the same lazily-initialized `Config` instance (pointer-equal), constructed at most once per process.

R11. The system shall resolve `FeatureFlag::is_enabled` in the order test thread-local override, then user preference, then global baseline state, then `false` when no layer has set a value.

R12. WHEN `FeatureFlag::set_user_preference(flag, value)` is called, the system shall make `flag.is_enabled` return `value` unless a test override for that flag is active on the calling thread.

R13. WHEN `FeatureFlag::set_enabled(flag, value)` is called, the system shall set the global baseline so that `flag.is_enabled` returns `value` whenever no user-preference and no test override apply.

R14. WHERE the `test-util` feature is present, WHEN `FeatureFlag::override_enabled(flag, value)` is called, the system shall force `flag.is_enabled` to `value` on the calling thread and restore the prior effective resolution when the returned `OverrideGuard` is dropped.

R15. The system shall confine a test override to the thread that created the `OverrideGuard` and shall not let it affect `is_enabled` reads on other threads.

R16. WHEN `apply_default_flags` is called, the system shall set the global baseline state of every flag listed in `DEFAULT_FLAGS` to enabled and leave every unlisted flag's baseline disabled.

R17. IF the build is a debug build AND `FeatureFlag::is_enabled` is read before `mark_initialized` has been called, THEN the system shall panic with a message naming the uninitialized-features condition.

R18. The system shall require no manual array-length (or other manual sizing) edit when a `FeatureFlag` variant is added or removed: the flag-state storage shall remain consistent with the variant set automatically. *(How the storage is sized — e.g. compile-time variant count — is the implementer's choice; verified at compile time, see Test Plan; ACCEPTED-UNTESTABLE for runtime mutation, mechanism stated in Mutation Targets.)*

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | `SessionId(7).as_u64() == 7`; newtype is `#[repr(transparent)]` u64 (R1) | planned |
| 2 | `u64::from(SessionId::from(42u64)) == 42` (R2) | planned |
| 3 | A sequence of `next()` calls yields strictly increasing, never-repeating ids (R3) | planned |
| 4 | Equal-valued `SessionId`s are `==` and hash equal; type is `Copy` (R4) | planned |
| 5 | `marley_home_dir()` returns `<home>/.marley` for a stubbed home (R5) | planned |
| 6 | With home unresolved, `marley_home_dir()` is `Err(HomeDirUnresolved)`, no panic (R6) | planned |
| 7 | The four subdir functions equal `home.join("config"|"data"|"themes"|"skills")` (R7) | planned |
| 8 | `mcp_config_file_path` ends `config/mcp.json`; `logfile_path` ends `data/<logfile_name>` (R8) | planned |
| 9 | `Config` has no server/cloud/auth fields; only one channel exists — compile-time structural assertion (R9, ACCEPTED-UNTESTABLE for mutation) | planned |
| 10 | Two `Config::marley()` calls return pointer-equal references (R10) | planned |
| 11 | Resolution order override→preference→baseline→false verified per layer (R11) | planned |
| 12 | `set_user_preference(f,true)` flips `is_enabled` absent an override (R12) | planned |
| 13 | `set_enabled(f,true)` flips baseline when no higher layer applies (R13) | planned |
| 14 | `override_enabled` forces value and reverts on guard drop (R14) | planned |
| 15 | Override on thread A does not change `is_enabled` on thread B (R15) | planned |
| 16 | After `apply_default_flags()`, listed flags enabled, others disabled (R16) | planned |
| 17 | Debug read before `mark_initialized` panics with the named message (R17) | planned |
| 18 | Adding/removing a variant compiles with no manual length edit — compile-time (R18, ACCEPTED-UNTESTABLE for mutation) | planned |

## Visual / Behavioral Acceptance
N/A — `marley_core` is a non-UI foundation crate with no window, pane, or accessibility surface.

## Test Plan
- **Unit:** one test per requirement —
  `session_id_as_u64_is_transparent` (R1), `session_id_u64_roundtrip` (R2), `session_id_next_strictly_increasing` (R3), `session_id_eq_hash_copy` (R4), `home_dir_resolves_dot_marley` (R5), `home_dir_unresolved_is_err` (R6), `subdirs_are_fixed_children` (R7), `mcp_and_logfile_paths` (R8), `config_has_no_network_fields` (R9, **compile-time structural assertion** — a `static_assertions`/trybuild check that the `Config` field set contains no network/auth field; there is no runtime branch to exercise), `config_marley_is_pointer_stable` (R10), `flag_resolution_order` (R11), `user_preference_overrides_baseline` (R12), `set_enabled_sets_baseline` (R13), `override_guard_forces_and_reverts` (R14, `test-util`), `override_is_thread_local` (R15), `apply_default_flags_sets_listed` (R16), `read_before_init_panics_debug` (R17, `#[should_panic]`, debug-cfg), `adding_variant_needs_no_length_edit` (R18, **compile-time** — a fixture enum plus a `static_assert!` that the storage length equals the variant count, so it fails to compile if a manual length is hardcoded; no runtime branch to exercise). 100% coverage on touched lines.
- **Integration:** `paths` consumed via a temp `$HOME` to assert the on-disk layout a sibling crate would create; `Config::marley()` + `apply_default_flags()` + `mark_initialized()` driven through a single cold-boot `init` seam to confirm a reader sees the expected baseline (the same `SessionId` + init seam `marley_terminal` binds to as a downstream consumer).
- **Visual:** none (`browser_testable: no`).
- **Regression:** the full `cargo nextest` suite stays green; `SessionId` numeric semantics and path strings are frozen contracts other M0 crates (and `marley_terminal`) bind to.

## Mutation Targets
`cargo mutants` must kill mutants on: the `next()` increment (off-by-one / non-increasing), each path `join` segment string (swap/empty), the `is_enabled` resolution branches (reordering layers, returning the wrong default), the `set_user_preference`/`set_enabled` store, the `apply_default_flags` apply loop (enable-vs-skip / listed-vs-unlisted), and the `OverrideGuard` Drop revert. MSI target 100% on the testable (runtime) surface. No `unsafe` is expected in this crate; gate 6 (miri) is N/A.

ACCEPTED-UNTESTABLE (closed decisions, with mechanism):
- **R9** — "no network/auth field" is a **structural, compile-time** property of the `Config` type, not a runtime branch; it carries no viable mutant. Verified by the `config_has_no_network_fields` compile-time field-set assertion, which fails to compile if such a field is reintroduced. Excluded from MSI by design, not as a gap.
- **R18** — "no manual length edit on variant add/remove" is a **compile-time** sizing property (variant-count-derived storage); it carries no viable runtime mutant. Verified by `adding_variant_needs_no_length_edit` (a `static_assert!` that storage length tracks the variant count). Excluded from MSI by design, not as a gap.
- The debug-only `mark_initialized` assertion path (R17) is exercised by the `#[should_panic]` test, so it is not untestable.

## Dependencies
- REUSE (permissive): `directories` (home-dir resolution, MIT/Apache), `once_cell` (lazy process-global `Config`, MIT/Apache), `serde` (flag/config (de)serialization, MIT/Apache), `enum-iterator` (variant iteration / compile-time variant count, **SPDX `0BSD`** — a permissive BSD-family zero-clause license). Gate 8's base allowlist is MIT/Apache/BSD; `0BSD` is a distinct SPDX id, so `deny.toml` carries an explicit, documented allowance `[licenses] allow = ["0BSD"]` (justification: permissive, no attribution/copyleft obligation). Pin `enum-iterator = "1.4"` (1.4.1 verified `0BSD`). The variant-sizing backend is the implementer's choice — R18 mandates only the observable "no manual length edit," not a specific crate or sizing call.
- Marley components: none upstream (leaf crate). **Downstream consumers:** `marley_terminal` depends on this crate for `SessionId` (seam-contracts §2), as do the feature-flag consumers and the app layer.

## Out of scope / deferred
- Multi-channel matrix (Stable/Preview/Dev/Oz), server/RTC/IAP/RudderStack/MCP-OAuth config, telemetry, crash reporting, and autoupdate — intentionally removed (single-channel, offline). Not revisited; Marley ships one channel.
- The fork's embed-vs-generate channel-config loader is dropped entirely (replaced by the hardcoded `Config::marley`).
- OS-info, semantic-selection, and execution-mode helpers land in later crates (M1+), not here.
- Persisting feature-flag user preferences to disk is deferred to the settings crate (M1).
- The shell-hook-reported session id (`marley_terminal::ShellSessionId`) and its `ShellSessionId → SessionId` correlation are owned by `marley_terminal` (seam-contracts §2), not here.

## Clean-room provenance
Behavior-derived from a fork-reference doc (`docs/specs/behavior/marley-core.behavior.md`) describing observable I/O only — no AGPL/fork source read, no private module/type/static names, no fork file paths. The public surface uses Marley-original identifiers; no fork-internal flag-state container, preference map, or atomic tri-state name appears in the contract (those remain private implementation details). IP-counsel sign-off pending (open item in `docs/marley_architecture/clean-build-plan.md`). REUSE crates (`directories`, `once_cell`, `serde`, `enum-iterator`) are MIT/Apache/`0BSD`.
