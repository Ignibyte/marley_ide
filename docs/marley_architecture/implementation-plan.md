# Marley (clean build) — Implementation Plan

> **Status (2026-07-12) — EXECUTED; historical as a phased plan.** This plan's core decision was adopted and carried out: Marley was built **academically** (the pipelined `plan → design → implement → inspect → validate → complete → /commit`), not shotgunned. Phase 0 (the throwaway spike), Phase 1 (the spec fan-out → `docs/specs/SPEC-*.spec.md`), and Phase 2 (the quality-gate harness — `scripts/gates.sh` enforces fmt / clippy / nextest / 100%-on-touched coverage / `cargo-mutants` MSI / miri / `cargo-deny` + `cargo-audit` / docs, fail-closed; Marley is a live forge project with the RLM wired — so then; that forge/RLM wiring itself retired at #409, the pipeline is purely file-local since) all landed. Phase 3 has since run **far past** the M1–M5 build order below — the shipped product is at **M15** (see [../../CHANGELOG.md](../../CHANGELOG.md)). The *strategy* still governs; only the phase/milestone framing below is a dated snapshot. For current state see [app_shell.md](app_shell.md), [editor.md](editor.md), and [crate-map.md](crate-map.md).

> How we actually build it. Captures the strategy decision (shotgun vs academic), the phased plan, the Rust quality-gate stack, and the forge-pipeline integration. Companion to [clean-build-plan.md](clean-build-plan.md) (the *what*) and [crate-triage.md](crate-triage.md) (the *crate map*).

## The decision: academic for the product, shotgun for the spike + the spec

**Recommendation: do NOT shotgun the production build. Be academic (pipelined) for the real code — but aim the multi-agent "shotgun" at two things where it's exactly right: a throwaway viability spike, and *spec generation*.**

Four reasons the product build must be pipelined, not shotgunned:

1. **Clean-room provenance is the whole game.** The legal claim to *own* Marley depends on every reimplemented piece being authored **from our spec docs, not from the AGPL source**. A speed-first shotgun invites agents to peek at Warp's code "to get it working" — which silently contaminates the clean room. A spec→design→implement pipeline is not just quality hygiene here; it's the **audit trail that proves the build was clean**. This alone settles the question.
2. **It's a foundation you'll build on for years.** Shotgunning a skeleton and "imposing rigor later" means rewriting the skeleton once the rigor exposes it. For a foundation, throwaway code is the expensive path.
3. **The quality bar you want *requires* stable, spec'd code.** Mutation testing, 100%-on-touched coverage, miri-clean unsafe — you can't meaningfully apply these to a shotgunned sketch. The pipeline is the vehicle that delivers them.
4. **We don't have an *implementation* spec yet.** We have architecture + per-crate reference + triage — a great description of Warp's *behavior*, not a Marley *implementation* spec (module layout, interfaces, EARS requirements, test/mutation plan). "Shotgun to a good spec" presupposes a spec we still have to write.

Where shotgun **is** right: (a) the disposable **M0 viability spike** — don't pipeline a throwaway whose only job is to answer "does the stack compose"; and (b) **spec generation** — fan out agents to *produce the spec*, not the code.

## The phases

### Phase 0 — Viability spike (shotgun, throwaway, ~days)
Prove the REUSE stack composes on this Mac before committing: a `gpui` window + an `alacritty_terminal` PTY rendering **one Block**, plus a bare `cef-rs` offscreen→texture proof. **Disposable** — explicitly not product code, not pipelined, deleted after it answers the question. De-risks the foundation picks (gpui, alacritty, CEF) for real.

### Phase 1 — Spec generation (multi-agent fan-out → the implementation spec)
This is where the "shotgun a good spec" energy goes. Fan out one agent per REIMPLEMENT crate + INVENT component to turn our docs into an **implementation spec**, each producing:
- module/type layout for the Marley crate (`marley_*`), public interfaces, data flow;
- **EARS requirements** (one WHEN/THEN clause per behavior — these become the tests);
- a test plan + **mutation-testing targets** (what must survive `cargo-mutants`);
- a **clean-room provenance line**: "spec'd from `docs/...`, behavior-only, no AGPL source read."
Output: `docs/spec/<component>.md` — reusable documentation, the pipeline's input.

### Phase 2 — Quality-gate harness + the Rust forge pipeline (set up ONCE, before M1)
Stand up the gate stack so it's enforced from crate #1 (the Rust analog of the AIC pipeline):

| Concern | Tool | Bar |
|---|---|---|
| Format | `cargo fmt --check` | clean |
| Lint | `cargo clippy --all-targets -- -D warnings` | zero warnings |
| Tests | `cargo nextest` | zero fail/skip |
| Coverage | `cargo-llvm-cov` | **100% on touched** |
| **Mutation** | **`cargo-mutants`** | MSI threshold (≥ ~90%, AIC-style) |
| Unsafe safety | **`miri`** (+ `loom` for concurrency) | clean — *critical* for the CEF/wgpu `unsafe` |
| Supply chain | `cargo-deny` + `cargo-audit` | no advisories/licence violations |
| SAST | `semgrep` (rust) | clean |
| Docs | `cargo doc` + `#![deny(missing_docs)]` | every public item documented |

Then: adapt the `/work` pipeline (plan → design → implement → inspect → validate → complete → `/commit`) for Rust; **register Marley as a forge project** and wire the **RLM** so failures/lessons/AARs compound across the 29-crate build (crate 5 learns from crate 1). CI = the fail-closed outer loop.

### Phase 3 — Pipelined crate-by-crate build (academic, M1 → M5)
Each REIMPLEMENT crate + INVENT component runs the full pipeline: spec in → designed → implemented-from-spec → adversarially inspected (incl. a **provenance check**: was anything derived from AGPL source?) → validated (tests + mutation + coverage + miri) → documented → committed behind the green gate. Build order = the M0–M5 milestones in [crate-triage.md](crate-triage.md).

## Standards baked in (your stated bar)
- **Architecture + reusable docs**: every component ships its spec (Phase 1) *and* an as-built doc (Phase 3); docs update in the same change as code — they never drift.
- **Highest quality gates**: the Phase-2 stack, fail-closed; mutation + 100%-on-touched + miri-clean are not optional.
- **Clean-room provenance**: a per-component provenance note + a provenance check in every inspect phase; periodic audit.

## The risk to manage
Clean-room discipline *under multi-agent speed*. The mitigation is structural: agents implement from `docs/spec/*` (behavior), never from the fork's source; the inspect phase explicitly checks provenance; the fork stays a separate "reference only" tree the build agents don't read from.

## What this isn't
Not pure shotgun (provenance + rework + quality bar rule it out). Not pure academic-from-line-one (don't pipeline the disposable M0 spike). It's **academic where it ships, shotgun where it's thrown away or where it produces the spec.**

## Next action
Phase 0 spike (days) **in parallel with** the Phase 1 spec fan-out. Phase 2 harness lands before the first M1 crate.
