# Finalize the Marley pipeline — Notes

- **Forge ticket:** #1 `773e5990-f7c4-40fa-8e16-a02846d27bcd` (chore), claimed by `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`.
- **AAR:** `50fc22d0-9c2d-4c92-ac25-7563e0a9b079`.
- **Local ticket doc:** `docs/planning/tickets/open/TICKET-000-finalize-pipeline.md`.
- **Pipeline spec:** `finalize-marley-pipeline.spec.md` (pipeline_id `de732c78-…`).

## Phase 1 — Plan

- **Request:** TICKET-000 (M0-round-1.md) — finalize the Marley pipeline so the gates enforce. First of a two-ticket run (000 then 001).
- **Classification / tier:** work pipeline, single shippable slice, **chore (gate-is-test infra)**. Not application code.
- **Forge recall (§18.3):** no bulletins; no Marley tickets/lessons yet (fresh project). `docs-search` surfaced the reference-project (oathstar) precedents that map 1:1:
  - **`WORK-add-tauri-shell-quality-gate`** — the **gate-is-test** pattern: a gate/tooling change adds no unit tests; verification = gate exit codes + **negative smokes** (append fmt drift → `rc=1` → revert; clippy lint → `rc=101` → revert), then the existing suites.
  - **`WORK-ratchet-coverage-floors`** — floors clamp up; inspect ran 2 parallel critics (gate-boundary correctness + doc-consistency). Adopting that inspect shape.
  - **`WORK-raise-coverage-baseline`** — uncovered code yields no viable mutants, so MSI stays 100%; the coverage/mutation interplay behind D1.

### Discovery — the precise port surface (for Design)

**Root cause:** the `.claude/` apparatus + `scripts/gates.sh` were scaffolded from
ignibyte_ide and target a `cockpit` crate under `src-tauri/`. Marley has 5
`marley_*` crates under `crates/` and no `src-tauri/`. So the gate can't go green
and the hooks watch the wrong tree.

**`scripts/gates.sh`** — the big rewrite (Ignibyte → 16-gate quality bar):
- gate:12 prettier → DELETE (no JS).
- gate:3 `cargo test` → `cargo nextest run --workspace` + `cargo test --workspace --doc`.
- gate:13 coverage → `cargo llvm-cov nextest --workspace --fail-under-lines 100`; drop `--ignore-filename-regex '(main|lib|forge|repo|heartbeat)\.rs$'`; `RUST_COV_FLOOR=100`.
- gate:14 mutation → drop `--package cockpit`, `--exclude '**/lib.rs'|forge.rs|repo.rs`, and the entire cockpit `--exclude-re`; mutate the whole workspace; `--in-diff` source `git diff HEAD -- crates/cockpit` → `crates/`; keep `--jobs 4`, MSI 100, the 27.x exit-code handling, the FULL "no viable mutants → fail closed".
- ADD gate (docs): `cargo doc --workspace --no-deps` under `RUSTDOCFLAGS=-D warnings`.
- ADD gate (miri): conditional-on-`unsafe`, spec-declared per-crate exemption; skip-clean when no unguarded `unsafe`.
- gate:15 browser/`scripts/browser-check.sh` → REPLACE with the visual/AX **conditional + fail-closed** gate (scan `docs/specs/*.spec.md` for non-N/A `visual_acceptance`; none in scope → printed SKIP; declared-but-unasserted → FAIL).
- ADD gate:16: `scripts/spec-provenance.sh docs/specs` (exit code = verdict).
- Renumber to quality-bar.spec.md's 1–16; the receipt logic (`gate_state_hash`) is unchanged (already hashes `crates/`).

**Hooks** (the only `src-tauri` references in the WIRED hooks — everything else is layout-agnostic):
- `enforce-phase-gate.sh:25` — `src-tauri/src/*.rs|src-tauri/examples/*.rs` → `crates/*/src/**/*.rs|crates/*/examples/**/*.rs` (handle nested module dirs; `case` glob `*` doesn't cross `/`, so add the `crates/*/src/*/*.rs` levels or restructure the test). Tests under `crates/*/tests/` stay exempt (validate-phase artifact; §3 names only `src`+`examples`).
- `enforce-quality.sh:26,29` — `cargo fmt --manifest-path src-tauri/Cargo.toml` → workspace root `Cargo.toml`.
- `enforce-tests-ran.sh:33` — help-text manifest path → workspace (the **detection** regex line 29 is already manifest-agnostic).
- `lib-hook-helpers.sh:45` — `ls-files -- crates src-tauri/src` → `ls-files -- crates` (drop dead path); tidy the comment.
- Already correct (no edit): `enforce-commit-gate.sh` (triggers on any `.rs` in changeset → 000 with no `.rs` exits 0), `enforce-pipeline-completion.sh`, `enforce-phase-tasks.sh`, `get_active_pipeline_doc` (→ `docs/planning/pipeline/active`, now created).

**`deny.toml`** — retitle "Ignibyte IDE" → Marley; `[licenses].allow` = the permissive set, **remove `MPL-2.0`**; prune `[advisories].ignore` (all the tauri/gtk/syntect/unic/bincode ids are Ignibyte-tree, not Marley's — keep only what Marley's actual dep tree pulls, re-derived by running `cargo deny check`). Add `license.workspace = true` to each `crates/*/Cargo.toml` (they currently inherit nothing → deny would flag them once checked).

**`.gitleaks.toml`** — already `useDefault=true` + the Firebase-key allowlist; ADD an allowlist entry for the `.mcp.json.example` placeholder bearer if `gitleaks dir` flags it (verify by running gate-7).

**`CONSTITUTION.md`** — §0 gate table → the 16 quality-bar gates; §3 "application code = src-tauri/src/**" → "crates/*/src/**"; §7 testing (tower::oneshot Axum is Ignibyte-web — Marley libs are `#[cfg(test)]` unit + mutation; visual/AX harness for UI crates); naming ignibyte_ide/cockpit → Marley/gpui workspace; **add a clean-room provenance §** (behavior-derived-from-fork-docs, no AGPL source; spec-layer gate-16 + code-layer provenance — the crown jewel). `docs/planning/` references in §3 STAY (D4). Fix `clean-build-plan.md:3` stale "AGPL fork at ~/Projects/ignibyte/Marley" line.

**Bootstrap** — `docs/planning/` tree created this phase. Templates (pipeline spec/notes, ticket, intake) to be authored in Implement.

### Sequencing reality (D6)
- 000 changes **no `.rs`** → `enforce-commit-gate.sh` exits 0 → 000 commits without a FULL receipt. Validate = `gates.sh --fast` green + negative smokes.
- 001 changes `crates/marley_text_offsets/src/lib.rs` (+ tests) → commit-gate requires a FULL/DIFF receipt → the gate is green there (real testable code) → receipt written → commit. The FULL coverage+mutation green is **proven at 001**, not faked at 000.

### Open decision deferred to follow-up
- Wiring the 3 parked §19 forge hooks (`enforce-mcp-config`, `enforce-docs-before-code`, `enforce-completion`) to hard blockers now that forge is wired. Deferred until forge proves stable across the first tickets — recorded, not hidden.

**Phase 1 status:** PASS (autonomous-through-commit per session goal). → Phase 2 Design.

## Phase 2 — Design

**Approach: gate-is-test.** No `.rs` application code changes. Verification =
each gate's exit code on a real run + one negative smoke per new/changed gate
(inject drift → red → revert → green). Coverage/mutation/cargo-doc smokes that
need real `.rs` are deferred to TICKET-001 (D6 sequencing); the config/doc-level
gates are smoked here.

### Grounded supply-chain state (measured this phase)
- `cargo deny check` → **`licenses FAILED`** (5 crates declare no license); advisories/bans/sources OK; **every** `[advisories].ignore` entry warns *advisory-not-detected* (all Ignibyte-tree, none on Marley's). → Fix: `license.workspace = true` ×5; **empty** the advisory-ignore list (re-add justified entries only when a real dep pulls an advisory, §0).
- `gitleaks detect -s .` → **`no leaks found`** (exit 0). The `.mcp.json.example` placeholder `Bearer REPLACE_WITH_MARLEY_FORGE_PK_TOKEN` is low-entropy → not flagged. REQ-011 already satisfied; the live `.mcp.json` is gitignored + outside the gate's `dir` scan. No `.gitleaks.toml` change needed; verify-green only.

### Gate map — `scripts/gates.sh` → quality-bar.spec.md numbering (1–16)
FAST runs the **static set**; FULL adds the **heavy set**. Receipt logic unchanged.

| # | Gate | Command (verdict = exit code) | Set |
|---|---|---|---|
| 1 | rustfmt | `cargo fmt --manifest-path Cargo.toml --all --check` | static |
| 2 | clippy | `cargo clippy --workspace --all-targets -- -D warnings` | static |
| 3 | tests | `cargo nextest run --workspace` **+** `cargo test --workspace --doc` | static |
| 4 | coverage | `cargo llvm-cov nextest --workspace --fail-under-lines 100` (no excludes) | **FULL** |
| 5 | mutation | `cargo mutants --workspace --jobs 4` (DIFF `--in-diff` over `crates/` / FULL), MSI ≥100, "no viable mutants → fail closed" | **FULL** |
| 6 | miri | conditional-on-`unsafe` (algorithm below); no non-exempt unsafe → printed SKIP | **FULL** |
| 7 | advisories | `cargo audit -f Cargo.lock` | static |
| 8 | supply chain | `cargo deny --manifest-path Cargo.toml check` | static |
| 9 | unused deps | `cargo machete` | static |
| 10 | secrets | `gitleaks detect -s . -c .gitleaks.toml` + `gitleaks dir crates scripts .claude/hooks` | static |
| 11 | shell | `shellcheck -S info -e SC1091 .claude/hooks/*.sh scripts/*.sh` | static |
| 12 | no-suppressions | grep meta-gate (unjustified/blanket `allow`/`expect` in `crates`) | static |
| 13 | SAST | grep meta-gate (`mem::transmute`; `unsafe` without `// SAFETY:`) in `crates` | static |
| 14 | docs | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | static |
| 15 | visual / AX | conditional + fail-closed (algorithm below) | **FULL** |
| 16 | spec provenance | `scripts/spec-provenance.sh docs/specs` | static |

FAST = {1,2,3,7,8,9,10,11,12,13,14,16} → `GATE GREEN [fast]`. FULL adds {4,5,6,15} + writes the receipt. (Prettier deleted; `browser-check.sh` replaced by gate 15.)

### gate 15 — visual/AX conditional + fail-closed (algorithm)
```
for f in docs/specs/*.spec.md:
    v = frontmatter visual_acceptance (first match)
    if v nonempty AND v !~ ^N/A : declared += "$f: $v"
if declared empty: echo "SKIP (clean) — no in-scope visual_acceptance"; return 0
if ! -d crates/marley_visual_harness: echo "FAIL — visual_acceptance declared but harness absent"; print declared; return 1
else: run the harness assertions (next-run wiring); exit code = verdict
```
This run: all specs are `visual_acceptance: N/A` (incl. visual-testing.spec.md itself) → **SKIP clean**. Negative smoke proves the fail-closed arm.

### gate 6 — miri conditional + per-crate exemption (mechanism)
- A crate is miri-checked iff `crates/<c>/src` contains `unsafe` AND its `Cargo.toml` does **not** declare `[package.metadata.marley] miri-exempt = "<reason>"` (greppable token `miri-exempt`; the reason is also in the crate spec's ACCEPTED-UNTESTABLE §, e.g. the visual harness's macOS-AX FFI which miri cannot model).
- No non-exempt `unsafe` across `crates/*/src` → printed `SKIP (clean)`; nightly+miri install deferred to the first `unsafe` crate. If a crate needs miri and nightly/miri is absent → fail with the install hint (can't leave unsafe unverified).
- M0 lib crates have zero `unsafe` → SKIP this run.

### Hooks — exact edits
- `enforce-phase-gate.sh:24-27` gated case →
  ```
  case "$NORMALIZED" in *.rs) ;; *) exit 0 ;; esac          # only .rs gated
  case "$NORMALIZED" in
      crates/*/src/*|crates/*/examples/*) : ;;             # gated (case-glob * crosses '/', so any depth)
      *) exit 0 ;;                                          # crates/*/tests/*, build.rs, etc. exempt (§3 = src+examples)
  esac
  ```
  (Verified: in `case`, `*` matches `/`, so `crates/*/src/*` catches nested modules; `/src/` is literal so `src_gen/` won't false-match.)
- `enforce-quality.sh:26,29` `--manifest-path src-tauri/Cargo.toml` → `--manifest-path Cargo.toml` (workspace root).
- `enforce-tests-ran.sh:33` help text `src-tauri/Cargo.toml` → `Cargo.toml` (detection regex line 29 already manifest-agnostic — no logic change).
- `lib-hook-helpers.sh:45-46` `ls-files -- crates src-tauri/src` → `ls-files -- crates`; refresh the comment (gated source = `crates/**/*.rs`).

### Supply chain — exact edits
- `crates/*/Cargo.toml` (×5): add `license.workspace = true` under `[package]`.
- `deny.toml`: retitle → Marley; `[advisories].ignore = []` (empty); `[licenses].allow` = `{MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, Zlib}` (**remove MPL-2.0**); keep `private.ignore`, `bans`, `sources`. Re-run `cargo deny check` → expect green.
- `.gitleaks.toml`: no change (already green); add a commented note that the `.mcp.json.example` placeholder is intentionally low-entropy.

### CONSTITUTION.md + docs — edit plan (stable §-anchors kept; hooks don't parse the file)
- **Header/product description** (lines 9-14): rewrite ignibyte_ide (Tauri/Axum/Datastar/agent-bridge) → **Marley** (clean-room, gpui, Rust Warp-reimplementation + the cockpit layer: project model, agents, brain-MCP, Forge panes, ops). The fork is reference-only; clean code never copies AGPL source.
- **§0**: replace the gate table with the 16-gate map above; prose `cockpit`/`src-tauri` → Marley workspace/`crates`; the deny `tauri/glib/unic` examples → "transitive advisories, justified in `deny.toml`"; keep "process-spawn permitted" (relevant to `marley_command`, TICKET-004); coverage floor 100 / MSI 100.
- **§3**: application code `src-tauri/src/**` + `src-tauri/examples/**` → `crates/*/src/**` + `crates/*/examples/**`; keep all `docs/planning/` paths (**D4**).
- **§7**: reframe — M0 crates are pure libs: `#[cfg(test)]` unit (one per EARS clause) + mutation (MSI 100) + `trybuild` compile-fail contracts; UI crates assert via the visual/AX harness (gate 15). Drop the Ignibyte `tower::oneshot` framing (keep as "the pattern for any future server surface").
- **§14**: `src-tauri/` → `crates/`; process-spawn/PTY confined to `marley_command` (not repo.rs/bridge.rs).
- **§15/§18**: keep; §18.1 inspect, §18.2 Explore, §18.3 forge-first.
- **§19**: Marley **is** forge-wired (own bearer) — recall/capture active; note the 3 forge hooks (`enforce-mcp-config`/`-docs-before-code`/`-completion`) are the remaining ratchet (follow-up, not this ticket).
- **NEW §20 — Clean-room provenance (binding)**: behavior-derived from the `warp_architecture` behavior docs, never the AGPL fork source; no line-by-line translation; spec-layer gate 16 + code-layer provenance check at inspect; REUSE (MIT/Apache) unrestricted; IP-counsel sign-off pending. The crown jewel.
- `clean-build-plan.md:3`: fix the stale "AGPL Warp fork at `~/Projects/ignibyte/Marley`" line — this path is the **clean build**; the fork reference lives elsewhere (per `docs/README.md`).

### Templates to author (Implement) — under `docs/planning/`
`pipeline/_templates/pipeline.spec.md`, `pipeline/_templates/pipeline.notes.md`, `_templates/ticket.md`, `_templates/intake.md` — lean, with placeholder frontmatter (`status: IN PROGRESS`, `pipeline_id: <uuid>`) so the phase-gate's "real UUID" + pipeline-completion's "not a placeholder" checks correctly reject an un-filled template.

### File manifest (gate-is-test — zero `.rs`)
| File | Change |
|---|---|
| `scripts/gates.sh` | rewrite to the 16-gate map (Marley paths, +cargo-doc/miri/visual/provenance, −prettier/browser) |
| `.claude/hooks/enforce-phase-gate.sh` | gated case → `crates/*/src/**`+examples |
| `.claude/hooks/enforce-quality.sh` | fmt manifest → workspace root |
| `.claude/hooks/enforce-tests-ran.sh` | help-text manifest |
| `.claude/hooks/lib-hook-helpers.sh` | `gate_state_hash` drop dead `src-tauri/src` |
| `deny.toml` | retitle; empty advisory ignores; drop MPL-2.0 |
| `crates/{text_offsets,util,core,command,spec_provenance}/Cargo.toml` | `license.workspace = true` |
| `CONSTITUTION.md` | product reframe + §0 table + §3/§7/§14/§19 + new §20 |
| `docs/marley_architecture/clean-build-plan.md` | fix line 3 |
| `docs/planning/**/_templates/*` | 4 new templates |

### Regression test plan — negative smokes (one per new/changed gate)
| Smoke | Gate | Method (red → revert → green) | REQ | Where |
|---|---|---|---|---|
| `gates.sh --fast` GREEN | 1-3,7-14,16 | run it; exit 0 | REQ-001 | 000 |
| fork-private id in a spec public surface | 16 | inject `ParsedToken` into a `docs/specs/*` Public surface → `spec-provenance.sh` exit 1 → revert | REQ-009 | 000 |
| fake `visual_acceptance` spec | 15 | add temp `SPEC-zzz.spec.md` w/ `visual_acceptance: window present` → visual gate FAIL (declared, no harness) → delete | REQ-008 | 000 |
| visual default | 15 | with all specs N/A → visual gate SKIP clean | REQ-008 | 000 |
| MPL not allowed / license bites | 8 | remove a `license.workspace=true` → `cargo deny` licenses FAILED → restore | REQ-010 | 000 |
| secret detected | 10 | drop a fake AWS key into `scripts/` → `gitleaks dir` red → remove | REQ-011 | 000 |
| miri default | 6 | no unsafe → miri SKIP clean | REQ-007 | 000 |
| coverage <100 fails | 4 | (001) delete a test line → coverage <100 → restore | REQ-003 | **001** |
| surviving mutant fails | 5 | (001) the real `cargo mutants` run; MSI must be 100 | REQ-004 | **001** |
| rustdoc warning fails | 14 | (001) break an intra-doc link → `cargo doc` red → fix | REQ-006 | **001** |
| fmt/clippy drift | 1,2 | (001) real `.rs` drift → red → fix | REQ-001 | **001** |
| phase-gate blocks pre-Design `lib.rs` write | 12/REQ-012 | (001) attempt `Edit` to `crates/.../src/lib.rs` before Design-PASS → hook exit 2 | REQ-012 | **001** |

### Risks / reversible-but-load-bearing
- **R1** `case` glob `*` crossing `/`: relied on for the nested-module gate. Verified semantics; the inspect critic re-checks it (a wrong glob = ungated code, the exact bug 000 fixes).
- **R2** Emptying `deny.toml` advisory ignores: correct now (none encountered), but means the FIRST real advisory (e.g. via 001's `serde`/`get-size2`/`num-traits`) fails the gate until a justified ignore is added — intended fail-closed, flagged for 001.
- **R3** Renumbering gates changes the §0 table; the doc-consistency critic must confirm CONSTITUTION ↔ gates.sh ↔ quality-bar all agree on 1-16.
- **R4** gate 15/6 are new shell functions — the fail-closed arms are the security-relevant ones; the gate-boundary critic re-derives each `return 1`.

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement

Applied the 10-file manifest (zero `.rs`). **`scripts/gates.sh --fast` → `GATE
GREEN [fast]`, 12/12 static gates pass** (1 fmt · 2 clippy · 3 tests · 7 audit ·
8 deny · 9 machete · 10 gitleaks · 11 shellcheck · 12 no-suppress · 13 SAST · 14
docs · 16 spec-provenance CLEAN). gate:4,5,6,15 correctly SKIP under `--fast`.
`cargo deny check` → all four sections OK. All hooks + scripts shellcheck-clean.

### Built
- `scripts/gates.sh` — full rewrite to the 16-gate quality-bar map; static-then-heavy grouping; receipt logic intact.
- 4 hooks ported (phase-gate case → `crates/*/src/**`+examples; quality/tests-ran manifest; helpers `gate_state_hash` → `crates`).
- `deny.toml` retitled; advisory ignores emptied; MPL-2.0 dropped. `license.workspace = true` ×5 crates.
- `CONSTITUTION.md` — full Marley rewrite (16-gate §0 table, §3 crates app-code, §7 testing reframe, §14, §19 forge-wired, **new §20 clean-room**). `clean-build-plan.md` lines 3 + "Two Marleys" fixed.
- 4 templates under `docs/planning/`.

### Deviations from design (with reason)
1. **gate:3 `--no-tests=warn`** — empirically `cargo nextest run` exits **4** on a no-test workspace (would red `--fast` on the M0 stubs). `warn` exits 0 with a VISIBLE warning; the binding test-existence enforcement is gate:4 (cov 100) + gate:5 (MSI 100) at FULL. Documented in the gate + §0 known-scope.
2. **gate:14 doc-todos refined** — actionable pattern `(TODO|FIXME|XXX)[:(!]` + exclude `docs/(planning|warp_architecture)/`. Found 6 false-positives: the quality-bar doc *describing* "no TODO/FIXME", and 4 `warp_architecture/` reference docs quoting **Warp's** `TODO(kevin)`/`TODO(vorporeal)` markers (not Marley unfinished work). Both classes correctly pass now; real Marley `TODO:`/`FIXME(` still caught.
3. **gate:15 in-scope = component crate EXISTS** — the 3 specs with real `visual_acceptance` (app-shell/foundation-spike/ui-components) name components (`marley_app`/`marley_spike`/`marley_ui_components`) that aren't built → out of scope → skip-clean. Becomes fail-closed the moment one of those crates lands without the harness.
4. **CONSTITUTION full rewrite** (not targeted edits) — most reliable way to guarantee zero stale `ignibyte_ide`/`cockpit`-crate/`src-tauri` tokens in binding sections. ("cockpit" survives only as the product *concept* — "agentic dev cockpit"; `ignibyte-gate-receipt` is the factory receipt filename, intentional.)
5. **deny.toml advisory ignores → `[]`** — every inherited ignore reported *advisory-not-detected* on Marley's tree. The full permissive allowlist is kept (forward-looking; unused entries warn harmlessly until deps land).
6. **`.gitleaks.toml` unchanged** — `gitleaks detect` already clean; the `.mcp.json.example` placeholder is low-entropy and not flagged. REQ-011 satisfied as-is.

### Carry-forward to Validate (Phase 4)
- Negative smokes (config/doc-level, no `.rs`): gate-16 fork-id, gate-15 fake-visual fail-closed, gate-8 license, gate-10 secret, gate-14 doc-todo. The `.rs`-dependent smokes (coverage/mutation/cargo-doc/fmt/clippy) are exercised at TICKET-001's FULL gate (D6).

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5)

4 independent critics (general-purpose subagents) over the diff, each verifying by
RUNNING commands (case-glob replicas, sed-extracted gate functions, the real
hooks, `cargo deny`/`mutants`/`llvm-cov`). Lenses: gate-boundary/fail-closed ·
anti-circumvention/not-weakening · doc-consistency · simplification/shell.

**Verified CORRECT (no action):** the `case`-glob `*` crosses `/` (nested
`crates/x/src/foo/bar.rs` GATED, `tests/` + `build.rs` EXEMPT, `src_gen/` no
false-match); `visual_g` fail-closed arm (empirically: built-component +
non-N/A visual + no harness → rc=1); `mutation_g` FULL "no viable mutants → fail
closed" vs DIFF "→ pass" not swapped; `miri_g` skip-clean; the 16-gate numbering
agrees across CONSTITUTION §0 ↔ gates.sh labels ↔ quality-bar.spec.md; no stale
`ignibyte_ide`/`cockpit`-crate/`src-tauri`/`Axum` token in a binding CONSTITUTION
section; no broken intra-doc links; bash-3.2/BSD portability clean; `--no-tests=warn`
is genuinely backstopped (cov gate exits 1 + mutation returns 1 on stubs — both
fail closed); no floor lowered (cov 93→100 RAISED); no blanket `#[allow]`; no
`|| true` swallows a gate verdict.

| # | Sev | Finding | Verdict | Fix |
|---|---|---|---|---|
| 1 | HIGH | Commit receipt fingerprints only `crates/**/*.rs`; gate-defining files (gates.sh, hooks, deny.toml) committable ungated — a gate-weakening could be pre-staged. | **REAL** (verified with the live hook) | `gate_state_hash` now fingerprints the gate-defining files too (scripts/*.sh, .claude/hooks/**, deny.toml, .gitleaks.toml, Cargo manifests + lockfile). Full closure (commit hook also gating no-`.rs` edits to those paths) deferred — needs a FULL-greenable workspace; disclosed in §15. forge `BF-commit-receipt-scope-001` + `PR-…receipt-binds-gate-definition-001`. |
| 2 | MED | `enforce-tests-ran.sh:29` detection regex `cargo (test\|llvm-cov)` doesn't match `cargo nextest` — would spuriously block a validator running the now-prescribed runner. (I introduced it.) | **REAL** | regex → `cargo (test\|nextest\|llvm-cov)`. forge `BF-tests-ran-runner-detection-001` + `PR-…detection-tracks-runner-001`. |
| 3 | MED | `enforce-docs-before-code.sh:19` (parked hook) still keyed to `src-tauri` — fail-open no-op if wired verbatim later. | **REAL** | case → `crates/*/src/*.rs\|crates/*/examples/*.rs`; comment updated (Marley has a bearer; §19 ratchet). |
| 4 | MED | 5 pipeline command docs (validate/implement/design/inspect/work) prescribe `src-tauri`/`tower::oneshot`/`browser-check.sh`/"15 gates" — mis-instruct future phases. | **REAL** (adjacent but in-mandate) | All 6 de-staled to Marley (crates, nextest+doctests, 16 gates, visual/AX harness, marley_command). |
| 5 | LOW | `miri_g` exemption `grep -q 'miri-exempt'` is a loose whole-file substring (a comment would exempt). | **REAL** | anchored: `grep -qE '^[[:space:]]*miri-exempt[[:space:]]*='`. |
| 6 | LOW | `unsafe[[:space:]]` (miri_g + source_bans_g gate 13) misses `unsafe{` (no space). | **REAL** (fmt mitigates, but SAST should stand alone) | → `unsafe[^_[:alnum:]]` in both. |
| 7 | LOW | `enforce-quality.sh` `\.rs$` lacks the C-quote tolerance the commit-gate has. | **REAL** (trivial) | → `\.rs"?$`. |
| 8 | LOW | quality-bar gate 6 mandates `loom` (unimplemented) + a stale `CEF` interop ref. | **REAL** (doc drift) | reworded: miri conditional, loom = ratchet, dropped CEF (kept wgpu/objc2/FFI). |
| 9 | LOW | `no_suppr_g` requires the justification on the SAME line as the allow/expect. | **REJECTED** — intentional policy (inline justification), inherited + documented; not a defect. | none. |
| 10 | LOW | `source_bans_g` transmute check is `mem::transmute` substring — a `use std::mem::transmute; transmute(…)` bypasses. | **NOTED/deferred** — exotic; the qualified `std::mem::transmute` idiom is caught; revisit with the Rust SAST crate. | none this ticket. |
| 11 | LOW | gate-16 interim `spec-provenance.sh` uses a hardcoded denylist, not the harvest-and-diff quality-bar describes; omits `enum_iterator::cardinality` + the Component/Params/Options triad. | **DEFERRED to TICKET-005** — explicitly interim (flagged in 3 places); the mutation-tested `marley_spec_provenance` does the real harvest-and-diff. | none this ticket. |

Post-fix: `bash -n` + `shellcheck` clean across all hooks+scripts; `gate_state_hash`
→ `fdd94a37…` (now spans 20+ files incl. the gate definition); nextest detection
confirmed; **`scripts/gates.sh --fast` → GATE GREEN [fast], 12/12** (unchanged).

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate

Gate-is-test change (zero `.rs`) → no unit tests; verification = the gate's exit
codes + **negative smokes** (inject drift → RED → revert → GREEN), run via
`scratchpad/smokes.sh` (self-reverting; touches only `*.md`/`*.toml`/temp files).

**Negative smokes — 5/5 GREEN (0 failures):**

| Gate | Drift injected | Result |
|---|---|---|
| 16 spec-provenance | `ParsedToken` (fork-private) into a spec Public surface | `spec-provenance.sh` rc=1 RED → revert → rc=0 GREEN ✓ |
| 15 visual (fail-closed) | temp spec: `component: marley_text_offsets` (BUILT) + non-N/A `visual_acceptance` | `visual_g` rc=1 FAIL-CLOSED → delete → rc=0 SKIP-CLEAN ✓ |
| 8 cargo-deny | removed a crate's `license.workspace = true` | `cargo deny check licenses` rc=4 RED → restore → rc=0 GREEN ✓ |
| 10 gitleaks | fake GitHub PAT `ghp_…` in `scripts/` | `gitleaks dir` rc=1 LEAK DETECTED → remove → rc=0 CLEAN ✓ |
| 14 doc-todos | `<!-- TODO: … -->` in a `docs/specs/*.md` | grep caught 1 → revert → 0 markers CLEAN ✓ |

- **Smoke-design note (not a gate bug):** the gitleaks smoke first used a bare AWS
  access-key *id* (`AKIA…`), which gitleaks 8.30 does **not** flag; switched to a
  GitHub PAT (`github-pat` rule) which it reliably detects. The gate (unchanged by
  this ticket) works; the *probe* needed a detectable shape.

**Gate run (transcript-of-record):** `scripts/gates.sh --fast` → **`GATE GREEN
[fast]`, 12 passed / 0 failed** (gate:4,5,6,15 correctly SKIP under `--fast`).
Re-confirmed clean after the smokes; no leftover smoke artifacts in the worktree.

**FULL gate (coverage + mutation + miri + visual + receipt) — DEFERRED to
TICKET-001 (D6), not fabricated.** TICKET-000 touches **no `.rs`**, so
`enforce-commit-gate.sh` exits 0 (no Rust source to fingerprint) and 000 commits
without a FULL receipt. A FULL run on the all-stub workspace would (correctly)
fail closed at gate:4 (`no coverage data`) and gate:5 (`no viable mutants`) — that
is the intended fail-closed behavior, not a green to manufacture. The FULL gate's
first real green is proven by TICKET-001, the first crate with testable `.rs`.

**Pre-existing failures:** none. **Skips:** the 4 heavy gates under `--fast` (by
design); no `#[ignore]`, no lowered floor, no suppression.

**Phase 4 status:** PASS (static green + 5/5 smokes; FULL deferred to 001 per D6). → Phase 5 Complete.
