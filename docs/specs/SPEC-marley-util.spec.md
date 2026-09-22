---
spec_id: marley-util
component: marley_util
bucket: REIMPLEMENT
milestone: M0
status: draft
title: Workspace value-type vocabulary — file/content identity, host identity, standardized paths
goal: Pin the shared identity and path value types every downstream Marley crate binds to, so file ids, content revisions, host identities, and normalized paths have one defined, mistake-proof contract before any code is written.
reuses: [serde, typed-path, dirs]
spec_source: "behavior-level contract — a leaf value-type crate: a process-unique opaque file identity, a monotonic content-revision counter, a local-or-remote host identity, and a path-standardization function that collapses `.`/`..`, canonicalizes separators, and makes a path absolute, plus a local-vs-remote path sum type built from those. Observable I/O only; no fork module/type/file/static names."
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: N/A
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
  - ./standards/seam-contracts.md
---

## Purpose
`marley_util` is the workspace's foundational **value-type vocabulary**: the small, dependency-light identity and path primitives that the editor, terminal, completer, project model, and asset layers all bind to. It exists so that "a file's identity," "a revision of a file's content," "the host a path lives on," and "a normalized, absolute, separator-canonical path" each have exactly **one** defined type with a defined construction/observation API — instead of being re-declared (with subtly divergent semantics) in every consumer.

Per **seam-contracts §8 (binding)**, this crate is the sole owner of `FileId`, `ContentVersion`, `HostId`, `StandardizedPath`, `LocalOrRemotePath`, and `standardize_path`. It is an M0 leaf crate with **zero internal Marley dependencies**, scheduled first in M0 so the M1/M2 crates that bind to this vocabulary have a fixed contract before code starts. This spec closes review-r1 **Blocker 8** (the zero-coverage M0 value-type crate) and pins the M1-binding subset; the broader utility surface the crate-triage line names (async-git invocation, sync primitives, worktree-name generation) is **explicitly deferred** (see Out of scope) until a real consumer needs it at M1+. Per **seam-contracts §10.2**, the remote-CDN macro path is dropped, so `marley_assets` no longer dangles a `make_absolute_url` / CDN base-URL helper on this crate — `marley_util` owns **no** URL helper at M1.

## Public surface (the contract)
All in `crates/marley_util/src/lib.rs`. Every wrapped field below is **private**; the documented API is the only way to construct or read a value. All identifiers are Marley-original (behavior-level); no Warp-internal type/module/static name appears.

- `pub struct FileId(/* private */ u64);` — a process-unique, opaque file identity. Derives `Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize`.
  - `pub fn new() -> Self` — allocates the next process-unique id (monotonic atomic counter); never returns an id equal to one a prior `new()` returned in this process.
  - `pub fn as_u64(self) -> u64` — the only read accessor for the private field.
- `pub struct ContentVersion(/* private */ u64);` — a monotonic content-revision number for one file's content. Same derives.
  - `pub fn initial() -> Self` — the documented starting revision (wrapped value `0`).
  - `pub fn next(self) -> Self` — the strictly-greater successor revision.
  - `pub fn as_u64(self) -> u64` — the only read accessor for the private field.
- `pub struct HostId(/* private */ String);` — a local-or-remote host identity. Derives `Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize`.
  - `pub fn local() -> Self` — the canonical local-host identity.
  - `pub fn remote(name: impl Into<String>) -> Self` — a named remote host identity.
  - `pub fn is_local(&self) -> bool` — discriminates local from remote.
  - `pub fn as_str(&self) -> &str` — the host name (the canonical local sentinel for `local()`).
- `#[non_exhaustive] pub enum StandardizedPath { Posix(/* private inner */ String), Windows(/* private inner */ String) }` — a normalized, **absolute**, separator-canonical path. The variant records the canonical separator flavor; the inner string is an invariant-bearing canonical form. `#[non_exhaustive]` so no external crate can construct or exhaustively match a variant directly — construction is **only** via `standardize_path`, preserving the canonical invariant. Derives `Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize`.
  - `pub fn as_str(&self) -> &str` — the canonical path string.
  - `pub fn is_absolute(&self) -> bool` — always `true` for a value produced by `standardize_path`.
  - `pub fn flavor(&self) -> PathFlavor` — `Posix` or `Windows`, the canonical separator flavor.
- `pub enum PathFlavor { Posix, Windows }` — separator-flavor discriminant. `Copy, Eq, Debug`.
- `pub fn standardize_path(p: &Path) -> StandardizedPath;` — normalizes `.` / `..` components, canonicalizes separators to the host flavor's canonical separator, and makes the path absolute (resolving a relative input against the process current working directory).
- `pub enum LocalOrRemotePath { Local(StandardizedPath), Remote { host: HostId, path: StandardizedPath } }` — a path that is either on the local host or on a named remote host. Derives `Clone, Debug, PartialEq, Eq, Hash`.
  - `pub fn new(host: HostId, path: StandardizedPath) -> Self` — splits to `Local(path)` when `host.is_local()`, else `Remote { host, path }`.
  - `pub fn path(&self) -> &StandardizedPath` — the path regardless of variant.
  - `pub fn host(&self) -> Option<&HostId>` — `None` for `Local`, `Some(&host)` for `Remote`.

## EARS Requirements
R1. The system shall expose `FileId` wrapping a **private** `u64` that is readable only via `as_u64()` and constructible only via `new()`; the wrapped value is never exposed as a public tuple field.

R2. WHEN `FileId::new()` is called, the system shall return a `FileId` whose `as_u64()` differs from that of every `FileId` previously returned by `new()` in the same process (process-unique identity).

R3. WHEN `as_u64()` is called on a `FileId`, the system shall return its wrapped `u64` unchanged, and two `FileId`s shall compare equal under `Eq` if and only if their `as_u64()` values are equal (identity round-trip).

R4. The system shall expose `ContentVersion` wrapping a **private** `u64` that is readable only via `as_u64()`, with `initial()` as the sole starting constructor whose `as_u64()` equals `0`; the wrapped value is never exposed as a public tuple field.

R5. WHEN `next()` is called on a `ContentVersion` `v`, the system shall return a `ContentVersion` whose `as_u64()` is strictly greater than `v.as_u64()`, such that any chain `initial().next()…next()` is strictly monotonically increasing (monotonic revisions).

R6. WHEN `as_u64()` is called on a `ContentVersion`, the system shall return its wrapped `u64` unchanged (version round-trip).

R7. WHEN `HostId::local()` is called, the system shall return a `HostId` whose `is_local()` is `true`.

R8. WHEN a `HostId` is constructed via `HostId::remote(name)` with a non-local host name, the system shall return a `HostId` whose `is_local()` is `false` and whose `as_str()` equals `name` (local-vs-remote discrimination).

R9. WHEN `standardize_path(p)` is called with any `p`, the system shall return a `StandardizedPath` whose `is_absolute()` is `true`; and WHERE `p` is relative, the system shall resolve it against the process current working directory before normalizing (absolute-ization).

R10. WHEN `standardize_path(p)` is called with a `p` containing `.` components, the system shall remove every `.` component; and WHEN `p` contains `..` components, the system shall collapse each `..` against its immediately preceding non-`..` component (`.`/`..` collapse).

R11. WHEN `standardize_path(p)` is called with a `p` whose separators include the non-canonical separator for the host flavor, the system shall return a `StandardizedPath` whose `as_str()` contains only that flavor's canonical separator, and whose `flavor()` is that host flavor (cross-platform separator canonicalization).

R12. WHEN `standardize_path` is applied to a `Path` built from the `as_str()` of an already-standardized path `s`, the system shall return a `StandardizedPath` equal to `s` (standardization idempotence).

R13. The system shall expose each `StandardizedPath` value's canonical string via `as_str()` and its separator flavor via `flavor()`, while keeping the inner string private and the enum `#[non_exhaustive]` so the canonical invariant cannot be bypassed by external construction or exhaustive matching.

R14. WHEN `LocalOrRemotePath::new(host, path)` is called and `host.is_local()` is `true`, the system shall return `LocalOrRemotePath::Local(path)`; and WHEN `host.is_local()` is `false`, the system shall return `LocalOrRemotePath::Remote { host, path }` (Local/Remote split).

R15. WHEN `path()` is called on a `LocalOrRemotePath`, the system shall return the contained `StandardizedPath` for either variant; and WHEN `host()` is called, the system shall return `None` for `Local` and `Some(&host)` for `Remote`.

R16. WHEN any of `FileId`, `ContentVersion`, `HostId`, or `StandardizedPath` is serialized and then deserialized through `serde`, the system shall produce a value equal to the original (value round-trip).

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | `FileId` wraps a private `u64`; read only via `as_u64()`, construct only via `new()`; no public tuple field (R1) | planned |
| 2 | Repeated `FileId::new()` returns ids no two of which share an `as_u64()` value in one process (R2) | planned |
| 3 | `as_u64()` returns the wrapped value; `Eq` agrees with `as_u64()` equality (R3) | planned |
| 4 | `ContentVersion` wraps a private `u64`; `initial().as_u64() == 0`; no public field (R4) | planned |
| 5 | `next()` is strictly greater than its receiver; chained `next()` is monotonic (R5) | planned |
| 6 | `ContentVersion::as_u64()` returns the wrapped value unchanged (R6) | planned |
| 7 | `HostId::local().is_local()` is `true` (R7) | planned |
| 8 | `HostId::remote(name)`: `is_local()` is `false`, `as_str() == name` (R8) | planned |
| 9 | `standardize_path` result `is_absolute()` is `true`; relative input resolves against cwd (R9) | planned |
| 10 | `.` components removed; `..` collapses against the preceding component (R10) | planned |
| 11 | Non-canonical separators replaced by the flavor's canonical separator; `flavor()` is the host flavor (R11) | planned |
| 12 | Re-standardizing a standardized path's `as_str()` yields an equal value (idempotence) (R12) | planned |
| 13 | `StandardizedPath` exposes `as_str()`/`flavor()`; inner private; enum `#[non_exhaustive]` (R13) | planned |
| 14 | `LocalOrRemotePath::new` yields `Local` for a local host, `Remote{host,path}` for a remote host (R14) | planned |
| 15 | `path()` returns the path for both variants; `host()` is `None`/`Some` by variant (R15) | planned |
| 16 | serde serialize→deserialize round-trips each value type to an equal value (R16) | planned |

## Visual / Behavioral Acceptance
N/A — pure non-UI value-type vocabulary; no window, pane, or AXUIElement surface. `browser_testable: no`; no `visual_acceptance` clause, so quality-bar gate 15 does not apply to this spec.

## Test Plan
- **Unit:** one `#[test]` per EARS clause, named for the requirement:
  - `r1_file_id_private_field_read_only_via_as_u64`, `r2_file_id_new_is_process_unique` (allocates a batch via `new()`, asserts the `as_u64()` set has no duplicates), `r3_file_id_as_u64_and_eq_agree`, `r4_content_version_initial_is_zero_private`, `r5_content_version_next_strictly_monotonic` (asserts `next()` > receiver and a chain is strictly increasing), `r6_content_version_as_u64_identity`, `r7_host_id_local_is_local`, `r8_host_id_remote_discrimination`, `r9_standardize_path_is_absolute_and_resolves_cwd` (captures `std::env::current_dir()` and asserts a relative input's result is absolute and prefixed by cwd), `r10_standardize_path_collapses_dot_and_dotdot`, `r11_standardize_path_canonicalizes_separators` (input mixes the foreign separator; assert `as_str()` holds only the canonical one and `flavor()` matches host), `r12_standardize_path_idempotent`, `r13_standardized_path_accessors_and_non_exhaustive` (asserts `as_str()`/`flavor()`; the no-external-construction guarantee is additionally backed by the R13 `trybuild` compile-fail below), `r14_local_or_remote_path_split` (local→`Local`, remote→`Remote{host,path}`), `r15_local_or_remote_path_accessors`, `r16_serde_roundtrip` (parameterized over all four serde-deriving value types).
  - R13's "cannot be constructed or exhaustively matched externally" half is enforced by a `trybuild` compile-fail case (`tests/ui/non_exhaustive_standardized_path_fail.rs` + `.stderr`) asserting that direct variant construction / exhaustive `match` of `StandardizedPath` from a downstream crate fails to compile (mirrors the `#[non_exhaustive]` contract).
  - R10/R11 use **deterministic, host-flavor-explicit fixtures** (foreign-separator and `.`/`..`-laden inputs whose canonical result is fixed regardless of the runner's cwd, except the absolute-prefix portion which the test derives from cwd), so the separator/normalization branches are testable on any platform.
  - 100% line coverage on every touched line of `lib.rs`.
- **Integration:** none internal (zero internal Marley deps). A downstream seam test in the first consumer (M1) confirms the consumer **imports** `FileId`/`ContentVersion`/`HostId`/`StandardizedPath`/`LocalOrRemotePath` from `marley_util` rather than re-declaring them (seam-contracts §8); out of scope here, named for traceability.
- **Visual:** none (`browser_testable: no`).
- **Regression:** the full suite plus the `trybuild` snapshot must stay green; the `FileId`/`ContentVersion` counters must remain strictly monotonic (a property test over many `new()`/`next()` calls runs as a regression guard).

## Mutation Targets
`cargo mutants` must kill every viable mutant on the value types and the path-normalization function:
- the **monotonic counters**: mutating `FileId::new()`'s atomic increment (deleting the increment, swapping `fetch_add(1)` to `fetch_add(0)`, or returning a constant) — killed by R2; mutating `ContentVersion::next()` (`+1`→`-1`/`+0`, or returning `self`) — killed by R5; `initial()` returning a non-zero constant — killed by R4.
- the **path-normalization branches** in `standardize_path`: dropping the `.`-skip branch, dropping or inverting the `..`-pop branch, off-by-one in the `..` pop, skipping the separator-canonicalization step, skipping the absolute-ization (relative-resolve) branch, and the host-flavor selection (`Posix`↔`Windows`) — killed by R9/R10/R11/R12.
- `HostId::is_local()` returning a constant or inverting the local-sentinel comparison — killed by R7/R8.
- `LocalOrRemotePath::new`'s split predicate (`is_local()` branch inverted, or always one variant) — killed by R14; `host()`/`path()` returning the wrong variant arm — killed by R15.
- `as_u64()`/`as_str()`/`flavor()` accessors returning a constant or the wrong field — killed by R3/R6/R8/R13.
- comparison-operator mutants in any derived/explicit `Eq`/`Ord` path — killed by R3/R5.
- MSI target: **100%** on the testable surface. **No unsafe** in this crate, so quality-bar gate 6 (`miri`) is N/A. No ACCEPTED-UNTESTABLE lines are anticipated; the entire surface is pure, in-process, and deterministically testable.

## Dependencies
- REUSE (permissive, MIT/Apache): `serde` (derive + round-trip for the value types), `typed-path` (cross-platform Unix/Windows path parsing + separator canonicalization that underpins `standardize_path`/`PathFlavor`, host-independent so the normalization branches test deterministically on any runner), `dirs` (resolving the process base directory for absolute-ization where needed). All MIT/Apache and pass `cargo deny`'s allowlist.
- Marley components: none (leaf crate; 0 internal deps). This crate is the **upstream** value-type vocabulary the M1/M2 crates import (seam-contracts §8).

## Out of scope / deferred
- **Async git invocation, sync primitives, and worktree-name generation** — the rest of the utility surface the crate-triage `marley_util` line names. These are **deferred** with a crate-triage note: the unused subset is re-scoped out of M0 and picked up at **M1+ only when a concrete consumer needs it** (e.g. worktree-name generation lands with M3 agent-orchestration; async-git with the first crate that shells out to git). Pinning only the M1-binding value-type subset now keeps the M0 contract small and lets the deferred reuses (`content_inspector`, `mime_guess`, `event-listener`, `rand`) arrive with their consumer rather than as speculative M0 surface.
- **No CDN / URL base helper** — per seam-contracts §10.2 the remote-CDN macro path (`remote_asset!`/`bundled_or_fetched_asset!`/`make_absolute_url`) is dropped at M1, so `marley_assets` no longer dangles `make_absolute_url` on this crate; `marley_util` owns no URL helper. If a local-first absolute-URL need ever arises it gets its own spec.
- **File content-type sniffing** (`content_inspector`/`mime_guess`) — deferred to the consumer that needs MIME classification, not part of the M0 value-type vocabulary.

## Clean-room provenance
Behavior-derived from a fork-reference doc; **IP-counsel sign-off pending** (open item in `clean-build-plan.md`). The `spec_source` above is a behavior-level statement (observable I/O only — no private module/type/static names, no fork file paths). The public surface uses Marley-original identifiers and seam-contracts §8-blessed names only (`FileId`, `ContentVersion`, `HostId`, `StandardizedPath`, `LocalOrRemotePath`, `standardize_path`); no Warp-internal taxonomy is reproduced. REUSE crates (`serde`, `typed-path`, `dirs`) are MIT/Apache. The package is named `marley_util`; no AGPL/fork source was read or transcribed for this spec.
</content>
</invoke>
