# `marley_util` — value-type vocabulary

> Per-crate architecture note — **round 4 refresh · 2026-07-12 · current to M15** (API verified unchanged
> against `crates/marley_util/src/lib.rs`). First of the `docs/marley_architecture/<crate>.md` series (one
> per crate, written at the pipeline's Phase 5 per CONSTITUTION §21). Provenance: **`[Marley-original]`**
> (clean-room REIMPLEMENT of the `warp_util` value-type *subset* — the concept, never the AGPL source)
> over **`[permissive/public]`** `serde` + `typed-path` (both MIT/Apache). Authoritative behavior contract:
> [`docs/specs/SPEC-marley-util.spec.md`](../specs/SPEC-marley-util.spec.md) (EARS R1–R16). Delivered by
> TICKET-002 (forge #4).

## Purpose & ownership

The workspace's foundational value-type vocabulary. Per **seam-contracts §8 (binding)**
`marley_util` is the **sole owner** of `FileId`, `ContentVersion`, `HostId`,
`StandardizedPath`, `PathFlavor`, `LocalOrRemotePath`, and `standardize_path` — every
downstream crate imports these rather than re-declaring them, so file identity, content
revisions, host identity, and normalized paths each have exactly one defined contract.

M0 **leaf** crate: **zero internal Marley dependencies**. It is the upstream the M1/M2
crates bind to, and the `HostId` / `LocalOrRemotePath` pair is the seam the retained
(non-functional) remote-connection work will bind to (see
`docs/planning/intake/remote-connection-seam.md`).

## Public surface

| Type / fn | Contract |
|---|---|
| `FileId(u64)` priv | `new()` mints a process-unique id (atomic counter); `as_u64()`. No `Default` (a constant would break uniqueness). |
| `ContentVersion(u64)` priv | `initial()` = 0; `next()` strictly-greater successor; `as_u64()`. |
| `HostId(String)` priv | `local()` (sentinel `"localhost"`) / `remote(name)` / `is_local()` / `as_str()`. |
| `PathFlavor` | `Posix` \| `Windows` — separator-flavor discriminant. |
| `StandardizedPath` | `#[non_exhaustive]` enum (`Posix`/`Windows`), private canonical inner; `as_str()` / `is_absolute()` (always true) / `flavor()`. Constructed **only** via `standardize_path`. |
| `standardize_path(&Path) -> StandardizedPath` | Collapse `.`/`..`, canonicalize separators to the host flavor, make absolute (resolve a relative input against the process cwd). |
| `LocalOrRemotePath` | `Local(StandardizedPath)` \| `Remote { host, path }`; `new()` splits on `host.is_local()`; `path()` / `host()`. |

`FileId`/`ContentVersion`/`HostId`/`StandardizedPath` derive `serde` (value round-trip, R16).

## Key design decisions (and why)

- **Generic-over-encoding `standardize_under::<E: Utf8Encoding>(input, cwd)`** — the
  normalization core is written once, generic over `typed_path`'s `Utf8UnixEncoding` /
  `Utf8WindowsEncoding`, and `standardize_with(p, flavor)` picks the variant. Because
  `typed-path` parses + normalizes **host-independently**, both flavors are exercised on
  a single (macOS) CI runner → **100% coverage + mutation MSI 100% without a Windows
  runner**. (A non-generic two-arm impl would leave the Windows branch dead.) Mirrors
  `AD-claude-newtype-macro-mutation-001` — put mutatable logic where the gate can reach it.
- **`HOST_FLAVOR` is a `#[cfg]`-gated `const`, not a `#[cfg]`-gated `fn`.** A cfg-gated
  *function* produces a mutant that cargo-mutants can't kill on the inactive platform
  (the cfg'd-out body never compiles → the mutant survives → MSI < 100, and exclusions
  are banned). A `const` is never mutated and is not an executable coverage line.
  (`PR-claude-cfg-conditional-as-const-001` — applies to every per-OS constant.)
- **Injected-cwd seam + root fallback** — `standardize_under` takes `cwd` as a parameter
  (not an internal `current_dir()` read), so the relative-input and unavailable-cwd
  branches are deterministically testable, and an empty cwd falls back to the flavor
  root so the result is **always absolute** — `is_absolute()`'s hardcoded `true` never
  lies. (`PR-claude-absolutize-via-injected-cwd-seam-001`.)
- **R13 closed by variant-level `#[non_exhaustive]`.** `#[non_exhaustive]` on the *enum*
  blocks external exhaustive-match but **not** construction of a known variant (tuple
  fields are public); `#[non_exhaustive]` on each *variant* blocks both. A `trybuild`
  compile-fail (`tests/ui/`) proves external construction + exhaustive match don't compile.

## Dependencies

`serde` (derive) + `typed-path` (cross-platform path parsing/normalization). Both
MIT/Apache. **No `unsafe`** (gate-6 miri N/A). No UI surface (gate-15 visual N/A). No
`dirs` — `std::env::current_dir()` covers cwd-absolutization.

## Out of scope (deferred to M1+)

Async-git invocation, sync primitives, worktree-name generation, MIME sniffing, any
CDN/URL helper (seam-contracts §10.2) — picked up when a concrete consumer needs them.
