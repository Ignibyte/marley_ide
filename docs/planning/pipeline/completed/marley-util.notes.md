# marley_util — Notes

- **Forge ticket:** #4 `0da40c8a-da93-430e-9c04-0c7c86bbbd60` (feature, M0), claimed `dc7df9b5-…`.
- **AAR:** `0b16e47e-09c2-49be-bb60-37ab2971b025`.
- **Local ticket doc:** `docs/planning/tickets/open/TICKET-002-marley-util.md`.
- **Pipeline spec:** `marley-util.spec.md` (pipeline_id `5fe7d6c7-…`).
- **Branch:** `ticket-002-marley-util` (stacked on the doc-phase ticket).

## Phase 1 — Plan

- **Request:** implement `marley_util` per SPEC-marley-util.spec.md (R1–R16). Leaf,
  zero internal deps; data foundation for 003 + the remote seam's HostId/path.
- **Classification / tier:** work pipeline, **feature** (new leaf crate).
- **Forge recall (§18.3):** no util-specific lessons; the 001/doc-phase ADs apply —
  **AD-claude-newtype-macro-mutation-001** (factor mutatable logic into a parameterized
  fn → here, `standardize_with(p, flavor)` so both flavors reach the gate),
  **AD-claude-coverage-model-001** (whole-workspace 100%), **AD-claude-doc-phase-enforcement-001**
  (§21 — a CHANGELOG entry is now hook-required for this `.rs` commit).
- **Authoritative contract:** SPEC-marley-util R1–R16 adopted 1:1 (D1).

### Carry to Design (the non-trivial bits)
- **`standardize_path` flavor split (D2)** — the crux. `typed-path` gives host-independent
  `UnixPath`/`WindowsPath` (+ `Utf8` variants) that parse + normalize regardless of the
  runner OS. Design `standardize_with(p: &Path, flavor: PathFlavor)`:
  - relative → resolve against `std::env::current_dir()` (the `dirs`/cwd absolutize, R9);
  - parse via the flavor's typed-path type; collapse `.` (drop) and `..` (pop the prior
    normal component, clamp at root) (R10); canonicalize to the flavor's separator (R11);
    emit the canonical string into `StandardizedPath::{Posix,Windows}` (private inner).
  - public `standardize_path(p)` = `standardize_with(p, host_flavor())` where
    `host_flavor()` is `cfg!(windows) ? Windows : Posix`. Tests drive `standardize_with`
    with BOTH flavors (R11 both-separator fixtures) → both branches covered + mutated.
  - **Idempotence (R12):** `standardize_with(Path::new(s.as_str()), s.flavor())` == `s`.
- **`FileId::new()`** — `static NEXT: AtomicU64; NEXT.fetch_add(1, Relaxed)` → process-unique
  (R2). The increment is the mutation target (fetch_add(0)/delete → killed by the
  no-duplicates batch test).
- **`ContentVersion::next()`** — `Self(self.0 + 1)` (R5 strictly-monotone); `initial()=Self(0)`.
- **R13 trybuild** — a downstream `match StandardizedPath { Posix(_) => …, Windows(_) => … }`
  (no `_` arm) and a direct `StandardizedPath::Posix(String::new())` construction must NOT
  compile (`#[non_exhaustive]`). `.stderr` captured with `TRYBUILD=overwrite`.
- **Cargo.toml:** `serde` (derive), `typed-path`, `dirs` — verify each is USED (machete);
  `trybuild` + `serde_json` dev-deps.

### §21 reminder
This is a `.rs` ticket → `enforce-changelog` will block the commit unless a staged
`CHANGELOG.md` entry exists. Complete must also add a `docs/marley_architecture/`
note for `marley_util` (the first crate doc; sets the per-crate doc convention).

**Phase 1 status:** PASS (autonomous-through-commit per session goal). → Phase 2 Design.

## Phase 2 — Design

### typed-path spike (scratchpad/tpprobe, v0.12.3) — confirmed
`typed_path::Utf8Path<E>` (generic over `E: Utf8Encoding`; `Utf8UnixEncoding` /
`Utf8WindowsEncoding`) `.normalize()` does **exactly** R10+R11 host-independently:
- `norm::<Unix>("/a/./b/../c")` → `/a/c`; `norm::<Unix>("a/./b")` → `/cwd/a/b`
  (relative absolutized); `norm::<Win>(r"C:\a\.\b\..\c")` → `C:\a\c`;
  `norm::<Win>("C:/a/b\\c")` → `C:\a\b\c` (mixed-separator canon); `norm::<Unix>("/a/../..")`
  → `/` (`..` clamps at root); a normalized relative result `is_absolute()` → true.

### D6 — `standardize_inner<E: Utf8Encoding>` (the generic core)
```rust
fn standardize_inner<E: Utf8Encoding>(s: &str) -> String {
    let p = Utf8Path::<E>::new(s);
    let abs = if p.is_absolute() { p.to_path_buf() }
              else { Utf8Path::<E>::new(&cwd_string()).join(p) };  // R9 absolutize vs cwd
    abs.normalize().to_string()                                    // R10 collapse + R11 canon
}
fn cwd_string() -> String { std::env::current_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default() }
fn standardize_with(p: &Path, flavor: PathFlavor) -> StandardizedPath {
    let s = p.to_string_lossy();
    match flavor {
        PathFlavor::Posix   => StandardizedPath::Posix(standardize_inner::<Utf8UnixEncoding>(&s)),
        PathFlavor::Windows => StandardizedPath::Windows(standardize_inner::<Utf8WindowsEncoding>(&s)),
    }
}
pub fn standardize_path(p: &Path) -> StandardizedPath {
    standardize_with(p, if cfg!(windows) { PathFlavor::Windows } else { PathFlavor::Posix })
}
```
- **Why generic:** the absolutize/normalize logic is ONE source line set → covered via
  the Unix instantiation (the relative branch is exercised by the Posix-relative test);
  the Windows flavor is exercised by absolute Windows fixtures. So **both flavors +
  100% coverage on one (macOS) runner**, and the flavor-selection (Posix↔Windows) is the
  2-arm match killed by `flavor()` assertions. (Two non-generic arms would leave the
  Windows-relative `else` branch dead → coverage gap. Mirrors AD-claude-newtype-macro-mutation-001.)
- `standardize_with` is `pub(crate)` (tests drive both flavors); `standardize_path` is the public host-flavor wrapper.
- `StandardizedPath::is_absolute()` → `true` (the invariant; R13/R9); `as_str()` → inner; `flavor()` → the variant.

### Value types
- `FileId(u64)`: `static NEXT: AtomicU64 = AtomicU64::new(0); pub fn new() -> Self { Self(NEXT.fetch_add(1, Ordering::Relaxed)) }` — process-unique (R2). `as_u64()`.
- `ContentVersion(u64)`: `initial() = Self(0)`; `next() = Self(self.0 + 1)` (R5); `as_u64()`.
- `HostId(String)`: `const LOCAL: &str = "localhost"`; `local() = Self(LOCAL.into())`; `remote(n) = Self(n.into())`; `is_local() = self.0 == LOCAL`; `as_str() = &self.0`. (Edge: `remote("localhost")` collides with local — out of contract; documented.)
- `LocalOrRemotePath::new(host, path)`: `if host.is_local() { Local(path) } else { Remote { host, path } }` (R14); `path()` → `&StandardizedPath` either arm; `host()` → `None`/`Some(&h)` (R15).
- Derives per spec; serde on the 4 value types (R16). `StandardizedPath` `#[non_exhaustive]`.

### Dependencies (revised)
`serde` (derive) + `typed-path` (0.12). **DROP `dirs`** — `std::env::current_dir()`
covers R9; `dirs` is user-dirs (home/config), not cwd → unused → machete-flagged.
dev-deps: `trybuild`, `serde_json`.

### File manifest
| File | Change |
|---|---|
| `crates/marley_util/src/lib.rs` | the 6 types + `standardize_inner`/`standardize_with`/`standardize_path`; `#![deny(missing_docs)]` |
| `crates/marley_util/Cargo.toml` | `serde`(derive) + `typed-path`; dev `trybuild` + `serde_json` |
| `crates/marley_util/tests/ui/non_exhaustive_standardized_path_fail.rs` + `.stderr` | R13 — external construct + exhaustive-match must not compile |

### Regression test plan (one row per R; both flavors for the path branches)
| Test | R | kills (mutation) |
|---|---|---|
| `r1_file_id_private_read_only` | R1 | (privacy/trybuild) |
| `r2_file_id_new_is_process_unique` (batch, no dup as_u64) | R2 | **`fetch_add` increment** |
| `r3_file_id_as_u64_and_eq_agree` | R3 | accessor/Eq |
| `r4_content_version_initial_is_zero` | R4 | `initial` non-zero |
| `r5_content_version_next_strictly_monotonic` (+ chain) | R5 | **`next` +1↔-1/+0/self** |
| `r6_content_version_as_u64_identity` | R6 | accessor |
| `r7_host_id_local_is_local` | R7 | `is_local` const/invert |
| `r8_host_id_remote_discrimination` | R8 | sentinel cmp; `as_str` |
| `r9_standardize_is_absolute_and_resolves_cwd` (Posix relative) | R9 | **absolutize branch** |
| `r10_collapses_dot_and_dotdot` (Posix + Windows fixtures) | R10 | (normalize — typed-path) |
| `r11_canonicalizes_separators` (**both flavors**: Unix `/`, Windows `\` from mixed) | R11 | **flavor match arms; `flavor()`** |
| `r12_idempotent` (both flavors) | R12 | re-standardize stability |
| `r13_accessors_and_non_exhaustive` + trybuild | R13 | `as_str`/`flavor`/`is_absolute` |
| `r14_local_or_remote_split` | R14 | **`is_local` split branch** |
| `r15_local_or_remote_accessors` | R15 | **`path()`/`host()` arms** |
| `r16_serde_roundtrip` (4 types) | R16 | serde |
| `monotonic_property` (many new()/next()) | R2/R5 | regression |

### Risks
- **R1 — cwd read in `standardize_inner`** is process-global; the r9 test reads
  `current_dir()` and asserts the prefix (no injectable seam needed — read-only, not a
  config-write race). Noted.
- **R2 — `remote("localhost")` collides with `local()`** — out of contract (R8 = "non-local name"); documented in the rustdoc.
- **R3 — typed-path feature flags** — the Utf8 types need default features (std); confirm at implement (`cargo deny` + machete).

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement

Built `crates/marley_util/{src/lib.rs, Cargo.toml}` per the design. Clean: `cargo
check`, `clippy --all-targets -D warnings` (exit 0), `fmt --check`, `machete` (no
unused — `dirs` correctly dropped), `deny licenses` (typed-path 0.12.3 = Apache/MIT,
clean).

### Two deliberate deviations from the design (both improve coverage/mutation)
1. **`host_flavor()` is `#[cfg(windows)]`/`#[cfg(not(windows))]`-gated, not a runtime
   `cfg!(windows)` ternary.** A `let f = if cfg!(windows) {Windows} else {Posix}` puts
   the non-host arm (Windows on macOS) in the compiled output as a never-taken branch →
   an **uncovered line** → gate:4 fail. Two `#[cfg]`-gated fns compile the host one only,
   so there's no dead branch to cover. (Functionally identical; this is the coverage-safe form.)
2. **`#[allow(clippy::new_without_default)]` on `FileId::new()`** with a justification
   comment. `new()` mints a fresh unique id, so a `Default` (a fixed value) would be
   misleading **and** would add an uncovered `default()` line (no test would call it).
   Specific + justified → passes gate:12 (no-suppressions). Mirrors the §0 "justified
   narrow allow" allowance.

### As-built notes
- Generic `standardize_inner::<E: Utf8Encoding>` + non-generic `process_cwd()` helper;
  `pub(crate) standardize_with` (both flavors, for tests) + public `standardize_path`.
- `StandardizedPath::is_absolute()` → `true` (the invariant); `as_str()` via an
  or-pattern; `flavor()` via an in-crate exhaustive match (allowed inside the defining
  crate despite `#[non_exhaustive]`).
- `process_cwd()` uses `current_dir().map(...).unwrap_or_default()` — no panic on the
  (essentially-never) cwd-unavailable path.
- No `unsafe`. Cargo.toml: `serde`(derive) + `typed-path` 0.12; dev `trybuild` + `serde_json`.

No other crates touched.

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5)

2 critics (correctness/path-normalization · mutation/coverage), both VERIFYING by
running probes (typed-path scratch crate; `cargo mutants --list` + a full run).

| # | Sev | Finding | Verdict | Fix |
|---|---|---|---|---|
| 1 | **BLOCKER** | `#[cfg(windows)] fn host_flavor()` → an **unkillable mutant** on the macOS runner (cargo-mutants mutates the cfg'd-out fn; it never compiles → build unchanged → survives → MSI ~95% → gate:5 RED, unfixable by tests). | **REAL** (empirically: `cargo mutants --list` showed the MISSED `host_flavor:178` mutant) | Replaced the two cfg-gated **fns** with two `#[cfg]`-gated **`const HOST_FLAVOR`**. Re-verified: `cargo mutants --list` → **0 host_flavor mutants**. forge `BF-cfg-gated-fn-unkillable-mutant-001` + `PR-claude-cfg-conditional-as-const-001` (recurs in 004's Windows code). |
| 2 | **HIGH** | `process_cwd()=="" ` on cwd-failure → a relative input stays relative while `is_absolute()` hardcodes `true` → lies + R9 fails (reachable: terminal cwd deleted). | **REAL** (probed: cwd="" → `"a/b"`→`"a/b"` not absolute) | Threaded `cwd` as a parameter to `standardize_under<E>(input, cwd)` (§14 testable seam) with empty-cwd→`/` root fallback → always absolute, and the relative/empty-cwd branches are now testable by injecting cwd. forge `BF-standardize-cwd-failure-non-absolute-001` + `PR-claude-absolutize-via-injected-cwd-seam-001`. |
| 3 | MED | `serde` derive lets an external crate deserialize a non-canonical `StandardizedPath` (R13-vs-R16 tension). | **REAL but SCOPED** | The spec defines R13's check as the **trybuild** (type-system: private inner + `#[non_exhaustive]`), which holds; serde is R16's trusted round-trip. Re-standardizing on deser would add surprising cwd-coupling + a syscall per deser. Added a **rustdoc trust-boundary note** ("deserialize only data this crate serialized"). Recorded as a decision, not a code change. |
| 4 | LOW | counter overflow at `u64::MAX` (ContentVersion `+1` debug-panic; FileId wrap re-mints 0). | **ACCEPTED** | Unreachable (2^64 allocations); §14 needs no guard. Noted. |

**Clean-room self-check (§20):** all identifiers are seam-contracts §8-blessed Marley-original
names (`FileId`, `ContentVersion`, `HostId`, `StandardizedPath`, `PathFlavor`,
`LocalOrRemotePath`, `standardize_path`); no Warp-internal taxonomy. Deps `serde`
(MIT/Apache) + `typed-path` (Apache-2.0/MIT) — permissive; written from the spec's
behavior, no AGPL source read.

Post-fix: clippy `-D warnings` PASS, fmt clean, machete no-unused, `cargo mutants --list`
= 28 listed / ~19 viable (the 9 `Default::default()` mutants are **unviable** — no type
impls `Default`), **0 on `HOST_FLAVOR`**.

### Carry to VALIDATE (Critic B's per-mutant assertion map — required to hit MSI 100 / cov 100)
- **r2/r3 FileId — RELATIONAL only.** The mutation runner is `cargo test` (one process,
  shared `static NEXT`, thread-interleaved) → a specific `new().as_u64()` value is
  non-deterministic. r2 = allocate a batch (e.g. 1000) and assert **no duplicate** `as_u64`
  (kills `as_u64 -> 0/1`). r3 = compare **two distinct** ids: `(a==b) == (a.as_u64()==b.as_u64())`.
- **r9 — relative input + `assert!(got.as_str().starts_with(&captured_cwd))`** (and
  `== format!("{cwd}/foo/bar")`), NOT `is_absolute()` (hardcoded true proves nothing).
  Kills `process_cwd -> ""/"xyzzy"` and the absolutize branch. A relative input is the ONLY
  thing that covers `process_cwd` + the `else` branch.
- **Empty-cwd branch** — add an in-crate test calling `standardize_under::<Utf8UnixEncoding>("foo","")`
  asserting `"/foo"` (covers + mutates the `if cwd.is_empty(){"/"}` branch).
- **r10/r11/r12 flavor cases — IN-CRATE `#[cfg(test)] mod tests`** calling `pub(crate)
  standardize_with(Path::new(r"C:\a\.\b\..\c"), PathFlavor::Windows)` with **absolute Windows
  fixtures**, asserting `as_str()==r"C:\a\c"` + `flavor()==Windows`. (Integration `tests/` can't
  see `pub(crate)`; absolute fixtures avoid the driveless-Windows `is_absolute` edge.) Exact-string
  asserts kill `as_str -> ""/"xyzzy"` and `standardize_under -> ""/"xyzzy"`.
- **≥1 test through public `standardize_path`** asserting `flavor()==Posix` (host) — covers
  `standardize_path` + the active `HOST_FLAVOR`.
- **r6** assert a value **≥ 2** (`initial().next().next().as_u64()==2`) — r4 (`==0`) alone lets
  `as_u64 -> 0` survive.
- **r15** test **both** variants (`host()` `Some` for Remote kills `-> None`; `None` for Local).
- **R13 trybuild** (`tests/ui/`) — external direct construction + exhaustive match must fail.

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate

**Tests:** 16 in-crate `#[cfg(test)]` unit tests (r1–r16, per the carried assertion
map) + the trybuild R13 case → **17 tests, all pass**; 0 doctests.
- `crates/marley_util/tests/ui.rs` (harness) + `tests/ui/non_exhaustive_standardized_path_fail.rs` + `.stderr`.

**Two issues found + fixed in-phase (while writing the R13 test):**
1. **R13 needed variant-level `#[non_exhaustive]`.** Enum-level blocks external
   *exhaustive match* but NOT *construction* of a known variant (tuple-variant fields
   are public). The trybuild proved it: with `#[non_exhaustive]` on each variant,
   `StandardizedPath::Posix(String::new())` from another crate → **E0603 "cannot be
   constructed because it is `#[non_exhaustive]`"** + the exhaustive match → **E0004**.
   (Critic A had marked R13 "structurally ✓" — the trybuild caught the gap.)
2. **Uncoverable `assert!` message.** `assert!(cond, "got={} cwd={}", …)` — the format
   args only evaluate on failure → 1 uncovered line (99.47%) once fmt split it out.
   Replaced with the stronger `assert_eq!(got.as_str(), format!("{cwd}/foo/bar"))`
   (still kills the `process_cwd -> ""/"xyzzy"` mutants).

**FULL gate:** `scripts/gates.sh` → **GATE GREEN [full], 16/16**.
- gate:4 coverage **100% lines** (both path flavors via the generic core + the
  injected-cwd seam).
- gate:5 mutation **MSI 100%** (29 caught / 0 missed) — the `HOST_FLAVOR` const fix
  eliminated the unkillable cfg-fn mutant; the 9 `Default::default()` mutants are
  unviable (no `Default` impls).
- gate:6 miri N/A (no unsafe); gate:15 visual N/A (no UI surface) — both skip-clean.
- Receipt written (`.git/ignibyte-gate-receipt`).

**Pre-existing failures:** none.

**Phase 4 status:** PASS — GATE GREEN [full]. → Phase 5 Complete.
