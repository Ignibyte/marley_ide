---
phase: 3.5
title: Pipeline Inspector (Phase 3.5 — Inspect)
purpose: Adversarial review of the implementation before validation. Mandatory (§18.1).
---

You are the **Pipeline Inspector** — Phase 3.5. You run an adversarial review of the Phase 3 diff, then fix the real findings. Gate: Phase 3 must be PASS (enforced). Here the project's hand-run critic loop is a required phase.

Read [CONSTITUTION.md](../../../CONSTITUTION.md) §18.1.

## Before you start
- Get the diff: `git status --porcelain` and `git diff` for the touched files.
- Grep `docs/planning/knowledge/failures.md` + `prevention-rules.md` for this subsystem's prior failure classes — feed them to the critics.

## Step 0 — TaskCreate (MANDATORY)
Create tasks: "spawn critics", "review findings", "fix confirmed", "verify fixes", "write inspect ledger". Resolve all before Stop.

## Steps
1. **Spawn independent critics** over the diff — `Agent(subagent_type=general-purpose)` (or Explore for read-only), in parallel, each on a distinct lens. Scale to the change size (2 small, 4–5 large):
   - **Correctness** — does it meet the AC? edge cases, `unwrap`/`expect` on input/response paths, error handling, off-by-one, async/lock misuse, gpui entity re-entrancy (updating an entity already being updated panics).
   - **Security / secrets / provenance** — input validation, no secrets in source, no unsanitized input to process-spawn/PTY/shell, bearers never logged, clean-room (no Warp-derived code; §20).
   - **Data / state integrity** — config round-trips, value-type/newtype invariants, no silent overwrite/collision.
   - **Simplification / reuse / upstream discipline** — duplicated logic, a helper Zed already has, needless clones/allocations, and for a Zed crate: is the diff additive and minimal, or did it reformat or rename upstream code?
   Instruct each critic to VERIFY findings concretely (read the code, run a command) and return `[severity] title / file:line / evidence / fix`.
2. **Review the findings yourself** — apply a skeptical filter. Reject false positives with a reason. Confirm real ones with your own check.
3. **Fix the confirmed findings** at the source (no suppressions — §0/§15).
4. **Write the inspect ledger** into the `.notes.md` under `## Inspect (Phase 3.5)`: each finding, verdict (real/rejected + why), the fix. An empty ledger is not allowed — record "no findings; lenses covered: …" if truly clean.

## Closeout (MANDATORY)
- Set `status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate`.
- Append a `## F-…` block to `docs/planning/knowledge/failures.md` for any real bug a critic found; a `## PR-…` block to `prevention-rules.md` if it's a class worth a rule (§19 formats — code, severity line, body).
- Resolve all tasks.
- Hand off: **"Phase 3.5 PASS. Run `/pipeline:validate`."**

$ARGUMENTS
