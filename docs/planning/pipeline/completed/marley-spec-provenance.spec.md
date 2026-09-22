---
pipeline_id: e973adb9-c46f-4505-9b7c-e89bdaabcd34
ticket: forge#8 (e1d6c657-efed-4aa8-b793-715c212b0c8c) · local docs/planning/tickets/open/TICKET-005-spec-provenance.md
aar_id: e247239e-99c3-41a4-af2e-a9951b3ac8d0
status: CANCELLED — gate-16 removed per chad's decision (2026-06-28); crate not built
title: marley_spec_provenance — gate-16 spec-layer clean-room provenance (Rust)
type: feature
milestone: M0
references:
  - ../../../specs/SPEC-gate.spec.md
  - ../../../specs/standards/seam-contracts.md
---

## Title

TICKET-005 — `marley_spec_provenance`. The mutation-tested Rust replacement for the
interim `scripts/spec-provenance.sh` (quality-bar **gate-16**, the spec-layer
clean-room wall). Scans every `docs/specs/SPEC-*.spec.md`, isolates its **Public
surface (the contract)** section + `spec_source` frontmatter, and reports any contract
reusing a fork-private identifier or citing a transcription-level source — fail-closed.

**Contract authoritative in [`SPEC-gate.spec.md`](../../../specs/SPEC-gate.spec.md)
(R1–R15).** Adopted verbatim.

## Scope
### In
- `lib.rs` — `ProvenanceConfig{denylist, allowlist, fork_doc_identifiers,
  transcription_source_patterns}`; `Violation` (4 variants, spec path + 1-based line);
  `SpecProvenanceReport{violations}` + `is_clean()`/`exit_code()`;
  `extract_public_surface` (heading→next-H2, line-numbered); `extract_spec_source`;
  `check_spec`/`check_specs` (walk, `Err` on IO); `harvest_fork_identifiers`.
- `main.rs` — CLI (`--specs`/`--fork-docs`/`--config`), prints `<spec>:<line>: <kind>: <id>`, exits `exit_code()`.
- `scripts/spec-provenance.sh` — rewired to invoke the binary; `gates.sh` `provenance_g` unchanged.
- Deps: `pulldown-cmark`, `regex`, `walkdir`, `serde`, `serde_yaml`, `anyhow`.

### Out (per spec)
Code-layer provenance (the existing inspect check), authoring the `spec_source`
rewrites / IP sign-off, semantic-paraphrase detection, non-`SPEC-*` docs.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — spec is the contract** (R1–R15, 1:1 with the tests).
- **D2 — §11.1 table as DATA (R11).** Parse the seam-contracts §11.1 strip→use markdown
  table: col-1 backtick identifiers → `denylist`, col-2 → `allowlist`. Adding a row
  extends coverage with no scan-logic edit.
- **D3 — THE SELF-TEST (the crux).** The crate over the LIVE `docs/specs/` must report
  **ZERO** violations (a regression fixture). R5's `warp_architecture` harvest (87 docs)
  must therefore be **conservative**: harvest only distinctive fork-private-looking
  identifiers from backtick code spans, filtered by a built-in **stop-set** of std /
  primitive / common tokens (`String`, `Vec`, `Result`, `Option`, `Path`, `PathBuf`,
  `bool`, `u64`, `Range`, `Command`, `Config`, `BTreeSet`, …); the `allowlist` = §11.1
  col-2 + the live specs' own Marley-original surface names + permissive-reuse
  (`gpui::Hsla`, `ropey::Rope`). **Design spikes this against the live tree FIRST and
  iterates to 0.** If unreachable conservatively → present a scope decision (narrow R5
  to the §11.1 strip-column diff + explicit denylist; defer the broad harvest) — never a
  silent weakening (§0/§15).
- **D4 — testable IO seams.** `check_specs(specs_dir)` / `harvest_fork_identifiers(dir)`
  take a dir param (no hardcoded path) → tested over `tests/fixtures/` + the live tree.
  IO failure → `Err` (R12, fail-closed). (PR-claude-absolutize-via-injected-cwd-seam.)
- **D5 — gate wiring.** `scripts/spec-provenance.sh` → `cargo run -q -p marley_spec_provenance -- …`
  (fast on a warm build); `gates.sh provenance_g` unchanged. scripts/*.sh are
  gate-defining (fingerprinted) — in scope.
- **D6 — §21:** CHANGELOG + `docs/marley_architecture/marley_spec_provenance.md`.

## Acceptance Criteria (EARS)
All **R1–R15** in [SPEC-gate.spec.md](../../../specs/SPEC-gate.spec.md), verified by the
spec's named tests (`r1_…`–`r15_…` over `tests/fixtures/`) **plus the golden self-test**
(live `docs/specs/` → 0 violations). Plus the bar:

| # | Bar | Verify |
|---|---|---|
| AC-A | R1–R15 fixture tests pass | `cargo nextest run -p marley_spec_provenance` |
| AC-B | **self-test: live docs/specs/ → 0 violations** | a `#[test]` running `check_specs` over the real tree |
| AC-C | 100% line coverage (lib + main arg-parse) | gate:4 |
| AC-D | mutation MSI 100 (section bounds, membership tests, boundary regex, exit_code, IO-err) | gate:5 |
| AC-E | deny/audit/machete clean (pulldown-cmark/regex/walkdir/serde_yaml/anyhow) | gate:7/8/9 |
| AC-F | gate-16 rewired; `gates.sh` gate:16 green via the binary | gate:16 |
| AC-G | FULL `scripts/gates.sh` → `GATE GREEN [full]` + receipt | /commit |
| AC-H | CHANGELOG + arch doc (§21) | enforce-changelog + inspect |

## Phase Plan
- **P2 Design** — **SPIKE harvest + self-test FIRST** (over the 87 fork docs + live
  specs; tune the stop-set/allowlist to 0); then the module/type design, the markdown
  parsing (pulldown-cmark vs line-scan for the surface slice + line numbers), the
  identifier-boundary regex (R7), the §11.1 table parser, the CLI, the gate wiring; the
  r1–r15 + self-test manifest + mutation map.
- **P3 Implement** — `lib.rs`/`main.rs` + Cargo.toml + the §11.1/config data + `scripts/spec-provenance.sh`.
- **P3.5 Inspect** — critics: section-bound correctness + the harvest false-positive surface, mutation/coverage readiness, clean-room (the denylist-as-data posture), the gate-wiring + fingerprint.
- **P4 Validate** — r1–r15 + the self-test; FULL gate green (cov 100 / MSI 100; gate-16 via the binary).
- **P5 Complete** — CHANGELOG + arch doc; AAR; archive; close #8.
