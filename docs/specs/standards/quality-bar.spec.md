# Quality Bar — Binding Standard (Marley)

> **2026-09-18, the Zed fork:** this standard is the gpui-era bar, kept as the design record. The
> binding bar for the fork is [`CONSTITUTION.md` §0](../../../CONSTITUTION.md): the same gates,
> scoped to the Marley crates plus the crates a change touches, with gate:15 (the macOS AX
> harness) retired in favor of gpui driven tests (§7), gate:9 on cargo-shear, and gates 7, 8, 10
> and 14 re-scoped to what a fork of a 250-crate workspace can honestly gate. Where this file and
> §0 disagree, §0 wins.

> The single source of truth for "is this shippable?" Adapted from `ignibyte_ide`'s `scripts/gates.sh` + CONSTITUTION §0/§7 for Marley's gpui/Rust workspace. **Strict by charter: no baselines, no suppressions, source-fix only.** Floors clamp UP (ratchet), never down. Every gate's verdict is the tool's **exit code**, never a grep of output. Enforced fail-closed by Stop hooks + `/commit`.
>
> **Cross-spec types are binding via [`seam-contracts.md`](./seam-contracts.md).** Where a spec disagrees with that doc on a shared type's name, owner, or field shape, seam-contracts wins and the spec is edited to match; the spec-layer clean-room wall (former gate 16) is now enforced by review (TICKET-006).

## The gates (`scripts/gates.sh`)

| # | Gate | Command | Bar |
|---|---|---|---|
| 1 | rustfmt | `cargo fmt --all --check` | clean |
| 2 | clippy | `cargo clippy --workspace --all-targets -- -D warnings` | zero warnings (pedantic/nursery = ratchet roadmap) |
| 3 | tests | `cargo nextest run --workspace` (+ `cargo test --doc`) | zero fail / zero skip |
| 4 | coverage | `cargo llvm-cov` | **100% on touched** (uncovered remainder must be an explicit ACCEPTED-UNTESTABLE decision, not a pending gap) |
| 5 | **mutation** | `cargo mutants` (`--in-diff` per-commit; full = periodic ratchet) | **MSI 100%** — every viable, non-excluded mutant killed |
| 6 | unsafe safety | `cargo +nightly miri test` on `unsafe`-bearing crates (conditional; per-crate spec-declared exemption for FFI miri can't model); `loom` for concurrency = **ratchet** (added when a concurrency crate lands) | clean on the `unsafe` surface — the wgpu / objc2 / macOS-FFI interop |
| 7 | advisories | `cargo audit` | no RUSTSEC advisories |
| 8 | supply chain | `cargo deny check` | advisories/licenses/bans/sources clean (license allowlist = MIT/Apache/BSD only — no copyleft) |
| 9 | unused deps | `cargo machete` | none |
| 10 | secrets | `gitleaks detect` (history) + `gitleaks dir` (working tree) | none |
| 11 | shell | `shellcheck` (hooks + scripts) | clean |
| 12 | no-suppressions | every `#[allow]`/`#[expect]` carries a `// justification`; blanket group suppressions BANNED | clean |
| 13 | SAST / source-bans | no `mem::transmute`; every `unsafe` carries a `// SAFETY:` note | clean |
| 14 | docs | `cargo doc` + `#![deny(missing_docs)]`; no TODO/FIXME in committed docs; no whole-word Warp/Zed brand mention in `crates/**/*.rs` (#262 — the `docs/*_architecture/` reference transcriptions are exempt by scope) | clean |
| 15 | **visual / accessibility** | the app launches headed, and every `visual_acceptance` spec clause is asserted via the macOS accessibility (AXUIElement) + screenshot harness | pass (see [../../pipeline/visual-testing.spec.md](../../pipeline/visual-testing.spec.md)) |

## Modes
- `scripts/gates.sh` — FULL: all gates + whole-workspace mutation. The periodic audit + MSI
  ratchet. **gate:5's FULL half is LANE-SPLIT (#407, 2026-08-14):** on this machine the sweep
  itself runs on the dev box (`git push dev main && ssh dev marley-mutants start main`) and the
  gate IMPORTS the measurement (`MUT_OUTCOMES=<merged pull> MUT_OUTCOMES_SHA=<measured sha>`)
  behind artifact-derived provenance belts — hex-literal sha == HEAD, `MEASURED_SHA` sidecar,
  freshness mtime, foreign-repo probe, clean measured surface, and SET-EQUALITY between the
  imported outcomes and the current tree's own full enumeration (an `--iterate` pull merges its
  generations to satisfy it — recipe in the gates.sh header). A local full sweep needs an
  explicit `MUT_FULL_LOCAL=1` (capable machines only; per-test threads capped). Scoped
  verification (`cargo mutants -f <file> --re <fn>`) stays the ad-hoc local lane.
- `scripts/gates.sh --diff` — per-commit: mutation only on touched lines (`--in-diff`). Commit-valid.
- `scripts/gates.sh --fast` — static gates only (no coverage/mutation). Inner-loop.

## Testing standard (CONSTITUTION §7)
- **Unit** — every public behavior; each **EARS clause maps to at least one test** (the clause IS the acceptance assertion).
- **Integration** — cross-crate seams (session spawn/write/read, panel↔agent, brain-MCP).
- **Visual / behavioral** — headed launch + AXUIElement assertions + screenshot regression for any `visual_acceptance` clause.
- **Regression** — the full suite is green before any merge; a passing-then-failing test is a regression, fix-or-revert.
- **Mutation** — `cargo-mutants` proves the tests actually fail when behavior breaks (MSI 100% on the testable surface).
- No `#[ignore]` without a `// justification`. No baselines. Source-fix only.

## Clean-room provenance gate (Marley-specific)
Every REIMPLEMENT change carries a provenance line — *behavior-derived from a fork-reference doc; IP-counsel sign-off pending*. The inspect phase includes a **code provenance check**: no code structurally derived from the fork/AGPL source. REUSE (MIT/Apache) is unrestricted.

### Spec-layer provenance (former gate 16) — REMOVED (TICKET-006)

The automated spec-layer provenance gate (`scripts/spec-provenance.sh` / the planned
`marley_spec_provenance` crate) was **removed by owner decision**: Marley is a private,
non-OSS rebuild, and a raw identifier diff of each spec's public surface against the
`warp_architecture` docs cannot distinguish Marley's own names from the fork's shared
Rust/terminal vocabulary (a spike flagged **194 legitimate** Marley/std/English names,
making the mandated zero-violation self-test unreachable without an allowlist that
defeats the gate). The clean-room posture is now enforced by **review, not an automated
spec gate**: the behavior-level `spec_source` discipline + Marley-original naming per
[`seam-contracts.md` §11.1](./seam-contracts.md) (still the authoritative strip→use map)
+ the inspect-phase **code-layer** provenance check + manual review before merge.

## Enforcement
Stop hooks (`.claude/hooks/enforce-*.sh`) block the turn until the active phase's gates pass; `/commit` runs the full `scripts/gates.sh` + opens the PR behind a fail-closed CI check. Forge RLM records every failure/lesson so the bar compounds across the build.
