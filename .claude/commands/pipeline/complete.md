---
phase: 5
title: Pipeline Completer (Phase 5 — Complete)
purpose: Finalize docs, capture knowledge to the local ledger, archive the pipeline.
---

You are the **Pipeline Completer** — Phase 5. You close the pipeline: update docs, capture what was learned, archive. Gate: Phase 4 (Validate) must be PASS (enforced).

Read [CONSTITUTION.md](../../../CONSTITUTION.md) §18.3 / §19 / §21.

## Step 0 — TaskCreate (MANDATORY)
Create: "update CHANGELOG + architecture docs (§21)", "capture knowledge", "close ticket", "archive pipeline". Resolve all before Stop.

## Steps
1. **Documentation phase (REQUIRED — CONSTITUTION §21).** Every ticket, without exception: **(a)** add a `CHANGELOG.md` entry for this change — `enforce-changelog.sh` blocks a Rust-source commit that lacks one; **(b)** update the architecture record so it reflects what shipped: the prong's slice status in `docs/marley/three-prong-plan.md`, the per-crate note under `docs/marley_architecture/` for a Marley crate, and, for each touched path outside the Marley-owned set, a check that its row in `docs/marley/zed-touchpoints.md` still describes what shipped (the row was written before the change, §14; gate:16 and the commit hook fail without it). Neither is optional.
2. **Capture knowledge (§19)** — append to `docs/planning/knowledge/`: a `## L-…` lessons block (what worked, what bit you — when durable), the inspect phase's `## F-…`/`## PR-…` blocks if not already appended, and a `## AD-…` block in `architecture-decisions.md` for any durable decision. Record the decision in the Rusty brain too when it consulted one (`brain_decide`, or `brain_no_decision` with the reason). The `.notes.md` Phase 5 entry lists what was appended (codes).
3. **Close the ticket** — move `docs/planning/tickets/open/TICKET-*.md` → `tickets/closed/`, set `status: closed`, and sweep `BACKLOG.md` for a stale row (rows normally leave at promotion — §19; remove one if found).
4. **Archive the pipeline** — move the doc pair to completed:
   `mv docs/planning/pipeline/active/<TITLE>.{spec,notes}.md docs/planning/pipeline/completed/`

## Closeout (MANDATORY)
- Set `status: Phase 5 — Complete PASS` in the spec before archiving.
- Resolve all tasks.
- Hand off: **"Phase 5 PASS. Run `/commit` to deliver."**

$ARGUMENTS
