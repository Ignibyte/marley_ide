---
pipeline_id: de732c78-a419-4598-982e-0149c8a3ae30
ticket: forge#1 (773e5990-f7c4-40fa-8e16-a02846d27bcd) · local docs/planning/tickets/open/TICKET-000-finalize-pipeline.md
aar_id: 50fc22d0-9c2d-4c92-ac25-7563e0a9b079
status: Phase 5 — Complete PASS
title: Finalize the Marley pipeline — port the gate + hooks + CONSTITUTION to Marley
type: chore (infra; gate-is-test)
milestone: M0
references:
  - ../../../tickets/M0-round-1.md
  - ../../../specs/standards/quality-bar.spec.md
  - ../../../../CONSTITUTION.md
  - ../../../pipeline/visual-testing.spec.md
---

## Title

TICKET-000 — Finalize the Marley pipeline. Port the inherited Ignibyte-IDE
enforcement apparatus (gate script + Stop/PreToolUse hooks + CONSTITUTION +
supply-chain config) to Marley's gpui/crates workspace so **every quality gate
of [quality-bar.spec.md](../../../specs/standards/quality-bar.spec.md) actually
enforces on Marley's real layout**. The apparatus was scaffolded from
ignibyte_ide and still targets a `cockpit` crate under `src-tauri/` that does
not exist here — today the gate cannot go green and the phase-gate watches the
wrong directory.

This is a **"gate-is-test"** change (config + tooling + docs, **zero `.rs`
application code**). Per the reference precedent (oathstar
`WORK-add-tauri-shell-quality-gate`), verification is the gate's own **exit
codes + negative smokes** that prove each gate catches its target drift — not
new unit tests.

## Scope

### In
1. **`scripts/gates.sh` → the 16-gate quality bar** (renumbered to match
   quality-bar.spec.md): drop the JS/prettier gate; tests via `cargo nextest run
   --workspace` + `cargo test --workspace --doc`; coverage via `cargo llvm-cov
   nextest --workspace --fail-under-lines 100` (strip the Ignibyte
   `main|lib|forge|repo|heartbeat` filename excludes); mutation **whole-workspace,
   zero exclusions**, keep DIFF (`--in-diff` over `crates/`) + FULL, `--jobs 4`,
   MSI floor 100; add a `cargo doc -D warnings` gate; add a **miri**
   conditional-on-`unsafe` gate with a spec-declared per-crate exemption; **wire
   gate-15** (visual/AX) conditional + fail-closed; **wire gate-16**
   (`scripts/spec-provenance.sh`).
2. **Hooks port** (the only Marley-incorrect references): `enforce-phase-gate.sh`
   gated path `src-tauri/src/*.rs` → `crates/*/src/**` (+ `examples`);
   `enforce-quality.sh` fmt manifest → workspace root `Cargo.toml`;
   `enforce-tests-ran.sh` help-text manifest; `lib-hook-helpers.sh`
   `gate_state_hash` drop the dead `src-tauri/src` from `ls-files`.
3. **Supply chain:** `deny.toml` retitled; license allowlist = the permissive set
   (MPL-2.0 **dropped**); advisory-ignore list pruned to Marley's actual tree;
   `license.workspace = true` added to each crate `Cargo.toml`.
4. **`.gitleaks.toml`:** allowlist the `.mcp.json.example` placeholder bearer so
   gate-7 stays green.
5. **CONSTITUTION.md:** wording `ignibyte_ide`/`cockpit`/`src-tauri` →
   Marley/gpui/`crates`; §0 gate table → the 16 gates; add a **clean-room
   provenance §**. Fix the stale `clean-build-plan.md:3` "this path is the AGPL
   fork" line.
6. **Bootstrap `docs/planning/`** working tree (`pipeline/{active,completed,_templates}`,
   `tickets/{open,closed}`, `intake`, `_templates`) + minimal templates so the
   inherited apparatus paths resolve.

### Out (explicitly deferred)
- Building **`marley_visual_harness`** (the gate-15 target crate) — the **next
  run**, not this one. 000 wires the gate as conditional + fail-closed only.
- Implementing **TICKET-001** (`marley_text_offsets`) — the immediately-following
  run; it lands the first real `.rs` that proves the FULL gate (coverage +
  mutation + commit receipt) green end-to-end.
- **Hardening the 3 parked §19 forge hooks** (`enforce-mcp-config`,
  `enforce-docs-before-code`, `enforce-completion`) to hard blockers. Forge is
  now wired for Marley, so this is a legitimate §19 ratchet — but deferred until
  the forge integration is proven across the first couple tickets (recorded as a
  follow-up, not silently skipped).
- Any change to `crates/*/src/*.rs` (the stubs stay as-is).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — Coverage = whole-workspace 100% lines.** `RUST_COV_FLOOR=100`, no
  filename excludes. ACCEPTED-UNTESTABLE is an **explicit, documented exclude
  list** (empty for the M0 pure-lib crates; populated only for FFI/GPU/headed
  paths when those crates arrive). A clean build has no legacy-uncovered line, so
  whole-workspace 100% IS the bar's "100%-on-touched", just stricter. No sub-100
  number can buy a green.
- **D2 — License allowlist = permissive set, MPL-2.0 dropped.** Allow exactly
  `{MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-2-Clause, BSD-3-Clause,
  ISC, Unicode-3.0, Zlib}`. MPL-2.0 (weak/file-level copyleft) is removed —
  conflicts with "own it outright".
- **D3 — gate-15 conditional + fail-closed now; harness crate next run.** The gate
  skip-cleans (printed) when no in-scope spec has a non-N/A `visual_acceptance`
  clause, and fails closed when one is declared but unasserted. 001 is
  `visual_acceptance: N/A` → gate-15 skip-cleans this run.
- **D4 — Keep `docs/planning/` as the working-scratch root (do NOT flatten).**
  `docs/planning/` is baked into every hook + command and into the gate-11
  doc-todos exclusion; flattening to `docs/` is high-churn and breaks that
  exclusion. Marley's durable docs (`specs/`, `marley_architecture/`,
  `decisions/`, the milestone ticket lists in `tickets/`) stay flat under `docs/`;
  per-pipeline WORKING docs live under `docs/planning/`. (Supersedes the earlier
  "docs/planning/→docs/" scope note.)
- **D5 — miri is conditional on `unsafe` with a spec-declared per-crate
  exemption.** A crate is miri-checked only if its source contains `unsafe` not
  covered by an exemption its spec declares (e.g. the visual harness's macOS AX
  FFI, which miri provably cannot model). No M0 lib crate has `unsafe` → the gate
  skip-cleans; nightly+miri install is deferred to the first `unsafe` crate.
- **D6 — Gate-is-test; no FULL receipt needed for 000.** 000 touches zero `.rs`,
  so `enforce-commit-gate.sh` (which triggers only on `.rs` in the changeset)
  exits 0 — 000 commits without a FULL receipt. The FULL gate is first proven
  green by TICKET-001. The FULL gate is **not** weakened to swallow an all-stub
  workspace (gate-14's "no viable mutants" stays fail-closed).

## Acceptance Criteria (EARS)
Verification column: **G** = gate exit code on a real run; **S** = negative smoke
(inject the drift → gate red → revert → gate green); **R** = source/structural
review.

| # | EARS requirement | Verify |
|---|---|---|
| REQ-001 | WHEN `scripts/gates.sh --fast` runs on Marley's workspace, the system shall execute the quality-bar's static gates (fmt, clippy, nextest+doctests, audit, deny, machete, gitleaks, shellcheck, no-suppressions, source-bans, doc-todos, cargo-doc, spec-provenance) and **shall print `GATE GREEN [fast]` with zero failures**. | G |
| REQ-002 | The gate script **shall contain no prettier / npx / JS gate**. | R (`grep -c prettier scripts/gates.sh` = 0) |
| REQ-003 | WHEN the FULL gate runs coverage, the system shall invoke `cargo llvm-cov nextest --workspace --fail-under-lines 100` with **no Ignibyte filename excludes**, and the baked `RUST_COV_FLOOR` shall be `100` (env may only ratchet up). | R + G (proven on real code at 001) |
| REQ-004 | WHEN the FULL gate runs mutation, the system shall run `cargo mutants` over the **whole workspace** with **no `--package cockpit` and none of the cockpit `--exclude`/`--exclude-re` entries**, `--jobs 4`, MSI floor 100, retaining both DIFF (`--in-diff` over `crates/`) and FULL modes; and on an all-stub workspace FULL mutation shall **fail closed** ("no viable mutants"), never silently pass. | R + G |
| REQ-005 | WHEN the test gate runs, the system shall run **`cargo nextest run --workspace`** AND **`cargo test --workspace --doc`** (doctests, which nextest does not run). | R + G |
| REQ-006 | WHEN the gate runs the docs gate, the system shall run `cargo doc --workspace --no-deps` under `RUSTDOCFLAGS=-D warnings` and **fail on any rustdoc warning**. | G + S (break an intra-doc link → red) |
| REQ-007 | WHERE a crate's source contains `unsafe` not covered by a spec-declared exemption, the system shall run miri on that crate; WHERE no in-scope crate has unguarded `unsafe`, the miri step shall **print `SKIP` (clean)**, never silently absent. | G + R |
| REQ-008 | WHEN the gate's visual step runs and **no** in-scope spec under `docs/specs/` declares a non-N/A `visual_acceptance` clause, the system shall print `SKIP` (clean); WHEN an in-scope spec declares a `visual_acceptance` clause but no `marley_visual_harness` assertion satisfies it, the system shall **FAIL closed**. | G + S (add a fake `visual_acceptance` spec → red → revert) |
| REQ-009 | WHEN the gate runs the spec-provenance gate, the system shall execute `scripts/spec-provenance.sh docs/specs` and take its **exit code** as the verdict (non-zero = fail), never a grep of its output. | G + S (inject a fork-private identifier into a spec public surface → red → revert) |
| REQ-010 | The `deny.toml` license allowlist **shall be exactly the permissive set and shall NOT include `MPL-2.0`**; each crate `Cargo.toml` shall set `license.workspace = true`; and `cargo deny check` shall pass. | G + R (no `MPL` in allow list) |
| REQ-011 | WHEN `gitleaks` scans the working tree, the system **shall not flag** the `.mcp.json.example` placeholder bearer, and gate-7 shall pass. | G |
| REQ-012 | The `enforce-phase-gate.sh` hook shall gate `Write`/`Edit` to `crates/*/src/**/*.rs` (+ `crates/*/examples/**/*.rs`) and exempt `docs/**`, `.claude/**`, `scripts/**`, and config; `enforce-quality.sh` shall fmt-check the **workspace** manifest. | R + behavioral (proven at 001: writing `lib.rs` before Design-PASS is blocked) |
| REQ-013 | `CONSTITUTION.md` normative text shall reference Marley/gpui/`crates` with **no `ignibyte_ide`/`cockpit`/`src-tauri` token** in a binding section, its §0 gate table shall enumerate the 16 quality-bar gates, and it shall contain a **clean-room provenance §**; `clean-build-plan.md` shall not describe this repo as "the AGPL fork". | R |
| REQ-014 | The system shall provide the `docs/planning/` working tree and minimal templates such that `get_active_pipeline_doc` resolves this pipeline's active doc (real `pipeline_id` UUID) and `/work` + `/pipeline:plan` find their templates. | R + G (`get_active_pipeline_doc` returns the doc) |
| REQ-015 | The system shall **not** weaken any gate to pass: no baseline, no blanket `#[allow]`, no floor below the §0 minimum, and the FULL `scripts/gates.sh` shall print `GATE GREEN [full]` + write the receipt **once real testable code exists** (proven at TICKET-001). | G (at 001) + R |

## Phase Plan
- **P2 Design** — exact edit list per file (gate script diff, hook diffs,
  deny/gitleaks/CONSTITUTION edits, template contents), the gate-15 conditional
  algorithm, the miri-exemption mechanism, and the negative-smoke test plan
  (one smoke per new/changed gate).
- **P3 Implement** — apply the edits; create the templates.
- **P3.5 Inspect** — 2 parallel critics (gate-boundary correctness/fail-closed;
  doc/CONSTITUTION consistency), per the oathstar `WORK-ratchet-coverage-floors`
  precedent.
- **P4 Validate** — run `scripts/gates.sh --fast` green; run each negative smoke
  (red→revert→green); confirm gate-13/14/15/16 wiring; document the FULL-green
  deferral to 001.
- **P5 Complete** — archive doc, AAR capture (this is where Marley's knowledge
  corpus gets its first lessons/ADs), close the forge ticket.
- **/commit** — local commit (no remote); no `.rs` → commit-gate exits 0.
