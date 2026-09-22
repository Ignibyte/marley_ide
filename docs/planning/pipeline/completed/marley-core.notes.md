# marley_core — Notes

- **Forge ticket:** #5 `644bd932-eb18-4e13-9883-03983c2dbaae` (feature, M0), claimed `dc7df9b5-…`.
- **AAR:** `5bb453d9-7fc1-40d4-ad83-af3ffdf1aeaf`.
- **Local ticket doc:** `docs/planning/tickets/open/TICKET-003-marley-core.md`.
- **Pipeline spec:** `marley-core.spec.md` (pipeline_id `766f41dd-…`).
- **Branch:** `ticket-003-marley-core` (stacked on 002).

## Phase 1 — Plan

- **Request:** implement `marley_core` per SPEC-marley_core R1–R18 — the offline-boot
  core (SessionId, ~/.marley paths, single-channel Config, feature-flag registry).
- **Classification / tier:** work pipeline, **feature** (foundation crate).
- **Forge recall (§18.3):** surfaced the prevention rules that all apply here —
  PR-…-cfg-conditional-as-const (per-OS values), PR-…-absolutize-via-injected-cwd-seam
  (the home seam), PR-…-non-exhaustive-variant (if any closed enum), the 001
  debug-panic/release-defined PR (R17). No flag-registry-specific lesson — novel; the
  global-state-under-threaded-mutation problem is the new hard part (D3).

### Confirmed pre-flight facts
- **`mutation_g` = `cargo mutants --jobs 4 --no-times`, NO `--test-tool`** → cargo-mutants
  default = `cargo test` (multi-threaded, one process). gate:3 + coverage use **nextest**
  (process-per-test, isolated). So global flag-state is only contaminated during the
  MUTATION run — and a racy baseline makes cargo-mutants exit 4 (fail-closed). → **D3:
  serial_test + reset seam** for R12/R13/R16/R17.
- **deny.toml** allow = MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, (BSD family?).
  Must **add `0BSD`** for `enum-iterator` 1.4.1.

### Carry to Design (the hard parts)
- **Feature-flag state machine (D3) — the crux.** 3 layers: thread-local override
  (R14/R15, isolated), global user-preference (R12), global baseline (R13/R16). Storage
  sized by `enum_iterator::cardinality::<FeatureFlag>()` (R18, no manual length). The
  init guard (R17) is a global `AtomicBool`. **Spike** the threaded determinism: confirm
  `#[serial]` + a `reset_for_test()` (clear baseline+user-pref+init) makes R12/R13/R16/R17
  pass under `cargo test` (and that cargo-mutants' baseline is green). The override
  layer needs NO serial (thread-local).
- **R17 debug-panic** — `is_enabled` in debug asserts `INITIALIZED`; `#[should_panic]`
  test reads before `mark_initialized`. Must be `#[serial]` + reset (so no other test
  set INITIALIZED first). Structure per the debug-panic/release-defined pattern so the
  panic line is covered + no equivalent mutant.
- **home_dir_from(Option<PathBuf>) (D2)** — Some(h)→`h.join(".marley")`, None→`Err`.
  public `marley_home_dir()` passes `directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf())`.
  Tests call `home_dir_from` directly (both arms) — no env.
- **Config once_cell (R10)** — `Config::marley()` returns `&'static` from a
  `once_cell::sync::Lazy` / `OnceCell`; two calls pointer-equal (`std::ptr::eq`).
- **R9/R18 compile-time (D4)** — `const _: () = { ... }` structural assertion (no
  network field; storage len == variant count). No runtime branch → no viable mutant.
- **SessionId** — `next()` (atomic), not `new()` → no `clippy::new_without_default`.
  `From<u64>`/`From<SessionId> for u64` round-trip (R2); `#[repr(transparent)]` (R1).

### §21 reminder
`.rs` ticket → enforce-changelog requires a staged CHANGELOG entry at commit; complete
adds `docs/marley_architecture/marley_core.md`.

**Phase 1 status:** PASS (autonomous-through-commit per session goal). → Phase 2 Design.

## Phase 2 — Design

### Spike (scratchpad/coreprobe) — all confirmed
- **enum-iterator 1.4 (0BSD):** `#[derive(Sequence)]`; `Flag::CARDINALITY` (assoc const)
  sizes `static BASELINE: [AtomicU8; N]` (R18, no manual length); `all::<Flag>()`
  iterates (apply_default_flags); `cardinality::<Flag>()` for the compile assert.
- **Flag state machine works** under threaded `cargo test`: `f as usize` index; two
  `[AtomicU8; N]` tri-state arrays (0 unset / 1 off / 2 on) for baseline + user-pref;
  `thread_local! OVERRIDE: RefCell<[Option<bool>; N]>`; `INITIALIZED: AtomicBool`;
  `is_enabled` = override → user-pref → baseline → false; RAII `OverrideGuard` Drop
  restores prior. Inline-const array init `[const { AtomicU8::new(0) }; N]` compiles.
- **Determinism:** 6 tests green + stable under threaded `cargo test` with
  `serial_test::#[serial]` + `reset_for_test()`. serial_test 3.x = MIT/Apache.
- **once_cell:** `static C: Lazy<Config>` → `&C` is pointer-stable across calls (R10).
- **Compile-time asserts carry NO mutant:** `cargo mutants --list` on the spike showed
  **0 mutants** on the two `const _: () = assert!(...)` lines (confirms R9/R18 are
  ACCEPTED-UNTESTABLE by construction, not via an exclusion file). The flag surface's
  9 mutants (`idx→0/1`, `set_enabled→()`, `apply→()`, `is_enabled→true/false`, the two
  userpref match-arm deletes, `==`→`!=`) are all killable.

### Architecture — 4 modules + `lib.rs`
- **`session_id.rs`** — `#[repr(transparent)] pub struct SessionId(u64)` (private field);
  `next()` = `static NEXT: AtomicU64; Self(NEXT.fetch_add(1, Relaxed))` (R3 monotonic —
  note `next` not `new` ⇒ no `clippy::new_without_default`); `as_u64`; `From<u64>` +
  `From<SessionId> for u64` (R2 round-trip); derive `Copy,Clone,Eq,PartialEq,Hash,Debug`.
- **`paths.rs`** — `enum PathError { HomeDirUnresolved }` (Debug + Display + std::error,
  no panic); private `home_dir_from(opt: Option<PathBuf>) -> Result<PathBuf,PathError>`
  = `opt.map(|h| h.join(".marley")).ok_or(PathError::HomeDirUnresolved)` (D2 testable
  seam); `marley_home_dir()` = `home_dir_from(BaseDirs::new().map(|b| b.home_dir().to_path_buf()))`;
  `marley_{config,data,themes,skills}_dir()` = `marley_home_dir().map(|h| h.join(<child>))`;
  `marley_mcp_config_file_path()` = `marley_config_dir().map(|d| d.join("mcp.json"))`;
  `marley_logfile_path()` = `marley_data_dir().map(|d| d.join(Config::marley().logfile_name()))`.
- **`config.rs`** — `struct AppId { org, app, kind: &'static str }` + `as_str()` (cached);
  `struct Config { app_id: AppId, logfile_name: &'static str }`; `Config::marley()` =
  `static C: Lazy<Config>; &C` (R10 pointer-stable); `app_id()`/`logfile_name()`. **R9**:
  private fields + only those two accessors → no network surface; a **trybuild compile-fail**
  (`Config::marley().server_url()` / `.auth_token()` etc. → E0599) proves no network accessor.
- **`features.rs`** — `#[derive(Sequence, Serialize, Deserialize, Copy,…)] pub enum FeatureFlag { … Marley-original variants … }`; the spike's state machine; `is_enabled`
  (R11, R17 `debug_assert!` init-guard), `set_enabled` (R13), `set_user_preference`
  (R12), `override_enabled`→`OverrideGuard` (R14/R15), `apply_default_flags` (R16),
  `mark_initialized`, `pub const DEFAULT_FLAGS: &[FeatureFlag]`; `#[cfg(any(test,
  feature="test-util"))] pub fn reset_for_test()`. **R18**: `const _: () = assert!(<arrays>.len() == cardinality::<FeatureFlag>())`.

### File manifest
| File | Change |
|---|---|
| `crates/marley_core/src/lib.rs` | module decls + re-exports; crate docs; `#![deny(missing_docs)]` |
| `crates/marley_core/src/session_id.rs` | SessionId |
| `crates/marley_core/src/paths.rs` | paths + PathError + home_dir_from seam |
| `crates/marley_core/src/config.rs` | AppId, Config (once_cell) |
| `crates/marley_core/src/features.rs` | FeatureFlag state machine |
| `crates/marley_core/Cargo.toml` | directories, once_cell, serde(derive), enum-iterator; dev serial_test; `[features] test-util = []` |
| `crates/marley_core/tests/ui.rs` + `tests/ui/config_no_network_accessor.rs` + `.stderr` | R9 trybuild |
| `deny.toml` | add `"0BSD"` to `[licenses] allow` (enum-iterator) + justification comment |

### Regression test plan (one row per R; **[S]** = `#[serial]` + reset_for_test)
| Test | R | serial? | kills |
|---|---|---|---|
| `session_id_as_u64_transparent` | R1 | — | accessor; repr asserted by `size_of`==8 |
| `session_id_u64_roundtrip` | R2 | — | From impls |
| `session_id_next_strictly_increasing` (batch, HashSet) | R3 | — | **fetch_add increment** |
| `session_id_eq_hash_copy` | R4 | — | Eq/Hash |
| `home_dir_resolves_dot_marley` (`home_dir_from(Some)`) | R5 | — | `.marley` join |
| `home_dir_unresolved_is_err` (`home_dir_from(None)`) | R6 | — | `ok_or` |
| `subdirs_are_fixed_children` (== home.join(child)) | R7 | — | each child string |
| `mcp_and_logfile_paths` | R8 | — | mcp.json / logfile joins |
| `config_no_network` (trybuild compile-fail) | R9 | — | compile-time (no runtime mutant) |
| `config_marley_pointer_stable` (`ptr::eq`) | R10 | — | Lazy identity |
| `flag_resolution_order` | R11 | **[S]** | the 3 resolution branches |
| `user_preference_overrides_baseline` | R12 | **[S]** | userpref store + arm |
| `set_enabled_sets_baseline` | R13 | **[S]** | baseline store + `==` |
| `override_forces_and_reverts` | R14 | **[S]** | OverrideGuard set + Drop |
| `override_is_thread_local` (spawn a thread) | R15 | **[S]** | thread-local isolation |
| `apply_default_flags_sets_listed` | R16 | **[S]** | **apply loop** (listed vs not) |
| `read_before_init_panics` (`#[should_panic]`) | R17 | **[S]** | debug init-guard |
| `variant_count_sizes_storage` (compile-time) | R18 | — | compile-time (no runtime mutant) |
| `idx_distinguishes_flags` (set A, read B) | (R11) | **[S]** | **`idx→0/1`** |

Uncoverable: none (home seam injected; flags reset+serial). R9/R18 are compile-time
(no runtime branch) — excluded from MSI by construction.

### Risks
- **R1 — serial scope.** Only the global-state tests need `#[serial]`; over-marking
  serializes needlessly but is safe. The override/session/path/config tests stay parallel.
- **R2 — directories `BaseDirs::new()`** is the one line not driven by an injected value
  (real home on the runner); covered when `marley_home_dir()` is called; the Err-home
  branch is covered via `home_dir_from(None)`.
- **R3 — R9 trybuild name list** is a heuristic (catches the listed network names); paired
  with private fields + the 2-accessor surface. Inspect re-checks the Config surface.

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement

Built `crates/marley_core/{src/{lib,session_id,paths,config,features}.rs, Cargo.toml}`
+ `deny.toml` (0BSD). Clean: `cargo check` (default + `test-util`), `clippy
--all-targets --all-features -D warnings`, `fmt`, `machete` (no unused), `deny
licenses` (clean).

### Two deviations from design/spec (both forced + correct)
1. **`directories` → `home` (license).** The spec's `reuses` lists `directories`, but
   `directories` pulls **`option-ext` (MPL-2.0)** transitively (via `dirs-sys`), and
   Marley's `deny` allowlist **bars MPL** (deliberately dropped). Switched to the
   `home` crate (0.5.12, MIT/Apache — cargo/rustup's own home resolver, no MPL): `home::home_dir() -> Option<PathBuf>` feeds `home_dir_from` directly. `option-ext` is
   now absent from the lockfile. (Spec reuse-list correction; flag at complete.)
2. **`override_enabled`/`OverrideGuard` gated `#[cfg(any(test, feature = "test-util"))]`,
   not just `feature = "test-util"`.** The gate runs tests/coverage/mutation with
   **default features only** (no `--all-features`), so a `feature="test-util"`-only gate
   would leave R14/R15 uncompiled + untested. The `test` cfg makes the override surface
   compile under the gate's `cargo test`/nextest runs while staying absent from a
   production build (faithful to the spec's "test-only" intent, and actually testable).

### As-built
- `session_id`: `#[repr(transparent)] SessionId(u64)`; `next()` (atomic, not `new()` →
  no clippy::new_without_default); `as_u64`; `From<u64>`/`From<SessionId> for u64`.
- `paths`: `home_dir_from(Option<PathBuf>)` seam; `marley_home_dir = home_dir_from(home::home_dir())`; subdirs; `PathError::HomeDirUnresolved` (Display + Error, no panic).
- `config`: `AppId{id}` + `Config{app_id, logfile_name}`, `Config::marley()` via
  `once_cell::Lazy` (ptr-stable); no network/auth field.
- `features`: `FeatureFlag{CommandBlocks, AgentMode, SessionRelay, ThemeStudio}`
  (Sequence + serde); `[AtomicU8; CARDINALITY]` tri-state baseline + user-pref; thread-local
  `OVERRIDE`; `INITIALIZED`; 3-layer `is_enabled` (R17 debug_assert); set/override/apply/
  mark; `DEFAULT_FLAGS=[CommandBlocks]`; `reset_for_test`; two R18 `const _` asserts.
- No `unsafe`.

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5)

3 critics (correctness/thread-safety · mutation/coverage · clean-room/no-network/license),
all verifying empirically (scratch tests; `cargo mutants --list` = 34 listed / 29 viable /
5 unviable; `cargo deny`/`cargo metadata` license sweep).

**No CRITICAL/HIGH bugs.** The flag resolution order (R11), `Ordering::Relaxed`
soundness (independent per-flag atomics, no cross-flag invariant), R17 debug-guard,
`idx = flag as usize`, SessionId, paths, and Config ptr-stability all verified correct;
no reachable panic/unwrap (§14 clean); no network surface (R9); no MPL/copyleft (option-ext
gone); `home` 0.5.12 = MIT/Apache cargo-team crate, correct non-deprecated API.

| # | Sev | Finding | Verdict | Action |
|---|---|---|---|---|
| 1 | LOW | `OverrideGuard` corrupts thread-local state under **non-LIFO** drop (restore-prev idiom) | REAL, test-only blast radius (cfg test-util); idiomatic scoped use is LIFO+correct | **Documented** the LIFO/RAII contract on `override_enabled` rustdoc; validate uses scoped guards |
| 2 | LOW | `INITIALIZED` Relaxed is not a sync edge | REAL but **sound today** (single-thread startup + thread-creation happens-before); latent only if reused as a "baselines published" barrier | Left Relaxed (sound); noted |
| 3 | MED | planned R9 trybuild (accessor denylist) is **non-exhaustive** (misses un-listed names; tests accessors not fields) | REAL | Validate's primary R9 check → **in-crate exhaustive destructure** `let Config { app_id:_, logfile_name:_ } = …` (no `..`) → E0027 on any added field. forge `BF-r9-no-field-denylist…` + `PR-claude-assert-no-field-via-exhaustive-destructure-001` |
| 4 | LOW | deny `0BSD` is actually `enum-iterator-**derive**` (main crate is MIT in 1.5.0) | REAL | **Fixed** the deny.toml comment |
| 5 | MED | `Default`-derive fragility — the 5 unviable mutants (Config/AppId/SessionId/OverrideGuard `Default::default()`) become **viable with no killer** if anyone adds `#[derive(Default)]` | REAL latent | Recorded here as a standing warning; R10 ptr-eq + non-{0,1} round-trip already kill 2 of them if they flip |
| 6 | MED | spec cites clean-room behavior doc `docs/specs/behavior/marley-core.behavior.md` that **does not exist** | REAL provenance gap | **Pre-existing + project-wide** (affects all REIMPLEMENT specs); flag to the user at complete — not a 003 code fix |
| 7 | LOW | spec `reuses:` still lists `directories` (stale post-swap) | REAL | Note in the arch doc + flag at complete |

### Carry to VALIDATE — the mutation-kill map + coverage traps (Critic 2, REQUIRED for MSI 100 / cov 100)
- **29 viable mutants** (5 unviable `Default` ones auto-drop). The R18 `const _` asserts +
  all statics carry **0 mutants** (confirmed absent from `--list`).
- **Tests MUST be IN-CRATE `#[cfg(test)] mod tests`** (per module): `home_dir_from` is
  **private** — its `None`→Err branch is the ONLY coverage of `paths.rs` Err + an integration
  `tests/` file can't reach it. Same for flag internals.
- **Traps a naive test leaves alive:**
  - `OverrideGuard::drop` (BLOCKER) — must assert the **post-drop REVERT** (`base` before, `!base` inside the scope, `base` after), not just that the override takes effect.
  - userpref match **arm-2 (`2=>true`) AND arm-1 (`1=>false`)** — test BOTH directions (pref≠baseline), or one delete survives.
  - `idx→0`/`idx→1` — set **≥2 different flags** to different values + assert each (a single-flag test can't see a constant index).
  - `as_u64→0/1`, `u64::from→0` — round-trip a literal **∉ {0,1}** (e.g. 42); `next()`'s first id is 0.
  - `mark_initialized→()` / `reset_for_test→()` — killed via the debug_assert: `reset` (INITIALIZED=false) → `mark_initialized` → a **normal** flag read (stub leaves it false → panics → test fails). **`#[serial]`+reset** mandatory.
  - per-fn path **suffix asserts** (all 8 path fns called individually; the whole-fn `Ok("")` mutant): home→`.marley`, config→`.marley/config`, …, mcp→`.marley/config/mcp.json`, logfile→`.marley/data/marley.log`.
  - `PathError` Display — a `format!` assert (only test that covers `paths.rs` Display line).
  - is_enabled true-read AND false-read; `==2`→`!=2` both directions; the **unset-pref fall-through** read (covers the `_=>{}` arm + the baseline line).
  - **`#[serial]` + `reset_for_test()`** on every global-state test (R11/R12/R13/R16/R17 + the idx + mark/reset tests); the override/session/path/config/serde tests stay parallel.
- **Add a `FeatureFlag` serde round-trip test** (R16-style; the derive is public surface, otherwise the derive lines may be uncovered).
- Debug-only kill paths (mark_init/reset via debug_assert) hold because the gate builds
  debug; do not add `--release`.

### §21 + spec hygiene (for complete)
- Arch doc records the `directories→home` (MPL) deviation + the exhaustive-destructure R9.
- Surface findings #6 (missing behavior doc) + #7 (stale `reuses`) to the user.

Post-fix: clippy PASS, deny clean, fmt clean.

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate

**Tests:** 22 in-crate `#[cfg(test)]` unit tests across the 4 modules, following the
inspect mutation-kill map exactly. **22 pass** (nextest), 0 doctests.
- `session_id`: r1 (size_of==8 + as_u64), r2 (round-trip 42 — kills as_u64→0/1 + From→0), r3 (1000-id HashSet), r4 (eq/hash/copy).
- `paths`: r5 `home_dir_from(Some)`, r6 `home_dir_from(None)→Err` (private seam, in-crate), r7/r8 (all 8 path fns called with component `ends_with` asserts), PathError Display.
- `config`: **r9 exhaustive destructure** `let Config { app_id:_, logfile_name:_ } = …` + `AppId { id:_ }` (E0027 if any field added — the inspect-upgraded R9 check), r10 ptr-stable + accessors.
- `features` (12 tests, global-state ones `#[serial]` + `reset_for_test`): r11 order, r12 both pref directions, r13 set_enabled both, r14 override **forces + reverts** (kills Drop→()), r15 thread-local (spawned thread), r16 apply_default_flags, r17 `#[should_panic]` read-before-init, idx-distinguishes-flags (≥2 flags), mark_initialized-flips-guard, reset-clears-state, unset-pref fall-through, serde round-trip (all 4 variants).

**FULL gate:** `scripts/gates.sh` → **GATE GREEN [full], 16/16 on the first run.**
- gate:4 coverage **100% lines**; gate:5 mutation **MSI 100%** (all 29 viable mutants
  killed; the 5 `Default::default()` mutants unviable; R18 `const _` + statics carry no
  mutant). gate:8 deny clean (0BSD for enum-iterator-derive; no MPL). gate:6 miri N/A
  (no unsafe); gate:15 visual N/A.
- Receipt written.

**Pre-existing failures:** none. (Spec-hygiene flags for the user, carried to complete:
the missing `docs/specs/behavior/marley-core.behavior.md` clean-room ref, and the stale
`directories` in the spec `reuses:`.)

**Phase 4 status:** PASS — GATE GREEN [full]. → Phase 5 Complete.
