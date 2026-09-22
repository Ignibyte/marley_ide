---
pipeline_id: 5fe7d6c7-e960-4f17-8d60-50a0629d6ead
ticket: forge#4 (0da40c8a-da93-430e-9c04-0c7c86bbbd60) · local docs/planning/tickets/open/TICKET-002-marley-util.md
aar_id: 0b16e47e-09c2-49be-bb60-37ab2971b025
status: Phase 5 — Complete PASS
title: marley_util — value-type vocabulary (ids, host, standardized paths)
type: feature
milestone: M0
references:
  - ../../../specs/SPEC-marley-util.spec.md
  - ../../../specs/standards/seam-contracts.md
  - ../../../specs/standards/quality-bar.spec.md
---

## Title

TICKET-002 — `marley_util`. The workspace's foundational **value-type vocabulary**
(seam-contracts §8 sole owner): `FileId`, `ContentVersion`, `HostId`,
`StandardizedPath` + `PathFlavor` + `standardize_path`, `LocalOrRemotePath`. A leaf
crate (zero internal deps) — the data foundation TICKET-003 binds to, and the
`HostId` / local-vs-remote-path the retained remote seam
([[remote-connection-seam]]) will use.

**Contract authoritative in [`SPEC-marley-util.spec.md`](../../../specs/SPEC-marley-util.spec.md)
(R1–R16).** This pipeline doc adopts it verbatim.

## Scope
### In
- `FileId(u64)` private — process-unique `new()` (atomic), `as_u64()`.
- `ContentVersion(u64)` private — `initial()` (=0), `next()` (strictly monotone), `as_u64()`.
- `HostId(String)` private — `local()`, `remote(name)`, `is_local()`, `as_str()`.
- `StandardizedPath` `#[non_exhaustive]` enum `{Posix, Windows}` (private inner) —
  `as_str()`, `is_absolute()`, `flavor()`; constructed ONLY via `standardize_path`.
- `PathFlavor{Posix,Windows}`; `standardize_path(&Path) -> StandardizedPath`
  (collapse `.`/`..`, canonicalize separators to host flavor, absolutize vs cwd).
- `LocalOrRemotePath{Local(StandardizedPath), Remote{host,path}}` — `new`, `path()`, `host()`.
- Deps (REUSE, MIT/Apache): `serde`, `typed-path`, `dirs`.

### Out (per spec "Out of scope")
- Async-git invocation, sync primitives, worktree-name generation (deferred to M1+
  when a consumer needs them). No CDN/URL helper (seam-contracts §10.2). No MIME sniffing.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — The spec is the contract.** R1–R16 are the AC + test list, 1:1.
- **D2 — Flavor-parameterized core for both-branch coverage/mutation.** `standardize_path`
  yields the **host** flavor (Posix on the macOS runner), so a naive impl leaves the
  Windows branch + the `Posix↔Windows` flavor-selection mutant DEAD/unkillable. Factor a
  private `standardize_with(p, flavor: PathFlavor) -> StandardizedPath` (using
  `typed-path`'s **host-independent** Unix/Windows path types) that BOTH flavors exercise
  on one runner; the public `standardize_path` calls it with the host flavor. (Mirrors
  AD-claude-newtype-macro-mutation-001 — put the mutatable logic where the gate can reach it.)
- **D3 — 100% coverage on touched + MSI 100, no ACCEPTED-UNTESTABLE** (pure, deterministic
  surface; the flavor split is made testable by D2). No `unsafe` → gate-6 miri N/A.
- **D4 — R13 `#[non_exhaustive]` via trybuild** — external direct construction / exhaustive
  match must fail to compile.
- **D5 — §21 doc phase (now live):** stage a `CHANGELOG.md` entry + add a
  `docs/marley_architecture/` component note for `marley_util` at complete.

## Acceptance Criteria (EARS)
All of **R1–R16** in [SPEC-marley-util.spec.md](../../../specs/SPEC-marley-util.spec.md),
verified by the spec's named tests (`r1_…`–`r16_…`, the R13 trybuild case, the
monotonic-counter property test). Plus the bar:

| # | Bar | Verify |
|---|---|---|
| AC-A | All R1–R16 tests pass | `cargo nextest run -p marley_util` |
| AC-B | R13 external construct/match fails to compile | trybuild `tests/ui/non_exhaustive_standardized_path_fail.rs` |
| AC-C | 100% line coverage (both path flavors) | gate:4 |
| AC-D | Mutation MSI 100 (counters, normalization branches, accessors, splits) | gate:5 |
| AC-E | deny/audit/machete clean with serde+typed-path+dirs | gate:8/7/9 |
| AC-F | FULL `scripts/gates.sh` → `GATE GREEN [full]` + receipt | /commit |
| AC-G | CHANGELOG entry + docs/marley_architecture note (§21) | gate enforce-changelog + inspect |

## Phase Plan
- **P2 Design** — `standardize_with(p, flavor)` core (typed-path Unix/Windows
  normalization: `.`/`..` collapse, separator canon, absolutize vs `dirs`/cwd); the
  atomic `FileId` counter; the `LocalOrRemotePath` split; the test manifest (one row
  per R; both flavors for R11; the trybuild); deps in Cargo.toml.
- **P3 Implement** — `lib.rs` + `Cargo.toml`.
- **P3.5 Inspect** — critics: contract fidelity, path-normalization correctness
  (`..` past root, mixed separators, idempotence, absolutize), mutation-readiness
  (flavor branch coverage), clean-room.
- **P4 Validate** — R1–R16 tests + trybuild; FULL gate green (cov 100 / MSI 100).
- **P5 Complete** — CHANGELOG entry + docs/marley_architecture note; AAR; archive; close #4.
