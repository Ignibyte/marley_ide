# `marley_core` — app/runtime core

> Per-crate architecture note (CONSTITUTION §21). Authoritative behavior contract:
> [`docs/specs/SPEC-marley_core.spec.md`](../specs/SPEC-marley_core.spec.md) (EARS
> R1–R18). Delivered by TICKET-003 (forge #5). GATE GREEN [full], coverage 100 / MSI 100.
> **Verified current @ M15** (re-read against the code 2026-07-12 — the four modules are
> unchanged since delivery; API matches). **Provenance (§20):** `[Marley-original]` — the own
> `SessionId` + the **no-cloud/no-auth** `Config` are the de-authed answer to Warp's
> `[Warp-derived: AGPL]` `warp_core` (which carries `ChannelState` server URLs + telemetry).

## Purpose & ownership

The lowest load-bearing crate — the **offline-boot foundation**. Canonical owner
(seam-contracts §2) of `SessionId`; provides the `~/.marley` filesystem layout, the
single-channel `Config`, and the process feature-flag registry. **No network, cloud,
or auth coupling** — Marley boots with no login. `marley_terminal` is a downstream
`SessionId` consumer (its shell-hook id is the distinct `ShellSessionId`, owned there).

## Modules

| Module | Surface |
|---|---|
| `session_id` | `#[repr(transparent)] SessionId(u64)`; `next()` (atomic, process-unique monotonic), `as_u64()`, `From<u64>` / `From<SessionId> for u64`. |
| `paths` | `marley_home_dir()`=`~/.marley`; `config/data/themes/skills` children; `mcp_config_file_path`, `logfile_path`; `PathError::HomeDirUnresolved` (Display + Error, **no panic**). |
| `config` | `AppId` + `Config` (`marley()` lazy pointer-stable via `once_cell`); `app_id()`/`logfile_name()`. No server/cloud/auth field. |
| `features` | `FeatureFlag { CommandBlocks, AgentMode, SessionRelay, ThemeStudio }`; `is_enabled`, `set_enabled`, `set_user_preference`, `override_enabled`→`OverrideGuard`, `apply_default_flags`, `mark_initialized`, `DEFAULT_FLAGS=[CommandBlocks]`. |

`SessionRelay` is the retained, currently **non-functional** remote-connection seam
(off by default) — see `docs/planning/intake/remote-connection-seam.md`.

## Key design decisions (and why)

- **Feature-flag state machine.** Three layers resolved in order: thread-local
  test override → global user-preference → global baseline → `false`. Each layer is a
  per-flag tri-state `[AtomicU8; FeatureFlag::CARDINALITY]` (0 unset / 1 off / 2 on);
  the override layer is a `thread_local! [Option<bool>; N]`. **Storage is sized from the
  variant count** (`enum_iterator::Sequence::CARDINALITY`), so adding/removing a flag
  needs no manual length edit (R18); two `const _: () = assert!(len == cardinality())`
  fail compilation if they ever drift. `Ordering::Relaxed` is sound — each flag is an
  independent atomic with no cross-flag invariant; startup is single-threaded and
  thread-creation supplies the happens-before for later readers.
- **Debug init-guard (R17).** `is_enabled` `debug_assert!`s `INITIALIZED` — a debug-build
  read before `mark_initialized` panics with a named message; release is a no-op (per spec).
- **`home` not `directories` (license).** `directories` pulls `option-ext` (**MPL-2.0**)
  transitively, and Marley's `deny` allowlist bars MPL. `home` (0.5.12, MIT/Apache —
  cargo/rustup's resolver) gives `home::home_dir() -> Option<PathBuf>` directly. The
  spec's `reuses:` list is stale on this point.
- **Testable home seam.** `home_dir_from(Option<PathBuf>)` (`Some`→`.marley`, `None`→`Err`)
  is the testable core; tests drive it directly — no `$HOME` env manipulation (which
  would race the threaded test runner). (PR-claude-absolutize-via-injected-cwd-seam-001.)
- **R9 — no network field, proven by exhaustive destructure.** An in-crate
  `let Config { app_id: _, logfile_name: _ } = …` (no `..`) fails to compile (E0027) if
  any field is ever added — a positive, exhaustive compile-time guard (better than an
  accessor-name denylist). (PR-claude-assert-no-field-via-exhaustive-destructure-001.)

## Testing strategy (stateful crate)

The flag layers are process-global. The coverage/test gates use **nextest**
(process-per-test, isolated), but the **mutation gate uses `cargo test`** (threaded,
one process) — so the global-state tests are marked `serial_test::#[serial]` and call
a `#[cfg(any(test, feature = "test-util"))] reset_for_test()` first, keeping them
deterministic (a racy baseline would make `cargo-mutants` fail closed). `override_enabled` /
`OverrideGuard` / `reset_for_test` are gated `#[cfg(any(test, feature = "test-util"))]`
— **not** just `feature = "test-util"` — because the gate runs with default features, so
the `test` cfg is what makes that surface compile, run, and be mutated.

## Dependencies

`home`, `once_cell`, `serde` (derive), `enum-iterator` (the derive sub-crate is 0BSD —
a documented `deny.toml` allowance); dev `serial_test` + `serde_json`. **No `unsafe`**
(gate-6 miri N/A). No UI surface (gate-15 N/A).

## Out of scope (deferred)

Multi-channel matrix, server/RTC/IAP/telemetry/MCP-OAuth, autoupdate, OS-info, flag
preference disk-persistence (M1 settings), `ShellSessionId` (owned by `marley_terminal`).

## See also

- [terminal_blocks.md](terminal_blocks.md) — the downstream `SessionId` consumer; its shell-hook id is the distinct `ShellSessionId`.
- [`warp_architecture/subsystems/03-terminal-session-core.md`](../warp_architecture/subsystems/03-terminal-session-core.md) — its *Provenance & licensing* section: `warp_core::ChannelState` server URLs + branding are `[Warp-derived]` and **dropped** here in `marley_core` `[Marley-original]` (the de-auth touchpoint).
