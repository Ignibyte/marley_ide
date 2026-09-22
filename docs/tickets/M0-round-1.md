# Marley — First Round of Tickets (M0)

> The M0 foundation, ready to hand to an agent-in-folder. Each ticket implements one spec **through the work pipeline** (`/work` → plan → design → implement → inspect → validate → complete → `/commit`) to a green gate. The spec's EARS clauses ARE the acceptance criteria and the test list. **No code merges below the quality bar** ([../specs/standards/quality-bar.spec.md](../specs/standards/quality-bar.spec.md)): clippy `-D warnings`, 100%-on-touched coverage, **mutation MSI 100%**, miri on any `unsafe`, gate-16 provenance green. Once forge is wired, each ticket below becomes a forge ticket (RLM-tracked).

**Status going in:** specs at GO_WITH_FIXES (gate-16 green); the Cargo workspace + 5 M0 crate stubs build; the pipeline runs (`scripts/gates.sh`). Build order per [../marley_architecture/crate-triage.md](../marley_architecture/crate-triage.md).

## TICKET-000 — Finalize the Marley pipeline *(infra; do first)*
> ✅ **DELIVERED** (forge #1) — apparatus ported to Marley's gpui/crates workspace; `gates.sh --fast` green 12/12 + 5/5 gate negative-smokes; hooks bite on `crates/`; CONSTITUTION + deny/gitleaks adapted; gate-15/16 wired. FULL gate (coverage+mutation+receipt) proven by TICKET-001 (first `.rs`). Archived: `docs/planning/pipeline/completed/finalize-marley-pipeline.spec.md`.

**Goal:** the full `scripts/gates.sh` goes green on Marley's workspace and the Stop hooks enforce it.
**Acceptance:** `deny.toml` license-allowlist (MIT/Apache/BSD) with crate `license.workspace = true` inheritance → gate-5 green; `.gitleaks.toml` tuned (the `forge_pk` placeholder is not a leak) → gate-7 green; drop the inherited JS/`prettier` gate (Marley has no JS); **wire gate-15** (the visual/AX harness, [../pipeline/visual-testing.spec.md](../pipeline/visual-testing.spec.md)) and **gate-16** (`scripts/spec-provenance.sh`) into `gates.sh` + the Stop hooks; adapt `CONSTITUTION.md` wording from `ignibyte_ide`→Marley (gpui workspace, the clean-room provenance §). Verify a full `scripts/gates.sh` is green.

## TICKET-001 — `marley_text_offsets` *(leaf; no deps)*
> ✅ **DELIVERED** (forge #2) — R1–R22; **FULL gate `GATE GREEN [full]`: coverage 100% + mutation MSI 100% (10/10)** + trybuild R2 + receipt. The first crate to prove the full pipeline end-to-end. Archived: `docs/planning/pipeline/completed/marley-text-offsets.spec.md`.
**Spec:** [../specs/SPEC-text-offsets.spec.md](../specs/SPEC-text-offsets.spec.md) · **Acceptance:** all EARS R1–R20 satisfied + the bar. The single shared `CharOffset`/`ByteOffset` vocabulary (private fields) + the streaming byte→char converter. Every consumer (editor/completions/search) imports from here — get it right first.

## TICKET-002 — `marley_util` *(leaf; no deps)*
**Spec:** [../specs/SPEC-marley-util.spec.md](../specs/SPEC-marley-util.spec.md) · **Acceptance:** the value-type vocabulary 16 downstream crates bind to — `FileId`, `ContentVersion`, `HostId`, `StandardizedPath` (`.`/`..` collapse + canonicalize + absolutize), the local-vs-remote path sum type. EARS-complete + bar.

## TICKET-003 — `marley_core` *(depends on: 001, 002)*
**Spec:** [../specs/SPEC-marley_core.spec.md](../specs/SPEC-marley_core.spec.md) · **Acceptance:** the process-unique monotonic `SessionId`, resolved `~/.marley` config/data/cache dirs, the single release-channel config, the feature-flag registry (defaults + runtime override). EARS-complete + bar. (`SessionId` is the canonical one terminal-blocks/app-shell import — seam-contracts §2.)

## TICKET-004 — `marley_command` *(leaf)*
**Spec:** [../specs/SPEC-process-command.spec.md](../specs/SPEC-process-command.spec.md) · **Acceptance:** cross-OS non-PTY child spawn (program/args/cwd/env/stdio, exit status, captured streams), Windows console-flash suppression + lifetime binding, WSL detection. The PTY shell builder seam (seam-contracts §4.2/§5). EARS-complete + bar.

## TICKET-005 — `marley_spec_provenance` *(the gate crate)* — ❌ CANCELLED (→ TICKET-006)
**Cancelled (chad, 2026-06-28):** gate-16 removed entirely. Marley is a private, non-OSS rebuild, and a raw identifier diff of each spec's public surface against the `warp_architecture` docs can't distinguish Marley's own names from the fork's shared Rust/terminal vocabulary (a design spike flagged 194 legitimate Marley/std/English names, making the mandated zero-violation self-test unreachable). `scripts/spec-provenance.sh`, `SPEC-gate.spec.md`, and the crate stub were removed; the quality bar is **15 gates**; clean-room is enforced by review.

## TICKET-006 — `foundation-spike` *(depends on the REUSE stack; throwaway)*
**Spec:** [../specs/SPEC-foundation-spike.spec.md](../specs/SPEC-foundation-spike.spec.md) · **Acceptance:** a **gpui** window + an **alacritty_terminal** PTY rendering **one command Block** on this Mac — proving the REUSE foundation composes before the real build. **Exercises the visual/AX harness** (gate-15) for the first time (window present, one Block visible). Disposable — not product code; delete after it answers "does the stack compose."

---

## After M0
M1 (terminal MVP — editor, terminal-blocks, syntax, settings, ui-components, palette, completions, classifier, app-shell, assets) specs are already written + reviewed; they become round 2. The INVENT cockpit layer (project model, panels, agent orchestration, brain-MCP, Forge panes, k8s/Acquia ops, Chromium pane) gets its spec round at M2. **Hand-off:** open TICKET-000, then TICKET-001 in an agent-in-folder and run `/work`.
