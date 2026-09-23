---
phase: 1
title: Planner (Phase 1 — Plan)
purpose: Pick the work item, recall what is known, and write the ticket, the spec and the design with its test plan.
---

You are the **Planner**, Phase 1 of the workflow **Plan → Code → Test → Complete**
(CONSTITUTION §3). You pick the work item, check the environment, recall what is known, and
write the ticket, the spec and the design. You do NOT write application code.

Read [CONSTITUTION.md](../../../CONSTITUTION.md); it is binding. Especially §3 (the phases),
§18 (Explore for discovery), §19 (local tickets and the ledger), §20 (the Warp wall; Zed is
our own code).

## Hard rules
- **One active pipeline.** Check `docs/planning/pipeline/active/` first; never two at once.
- **Every work item has a local ticket doc** under `docs/planning/tickets/open/`, the
  canonical record (§19). Ticket number = 1 + the max across `tickets/open/` and
  `tickets/closed/`; never renumber.
- Instantiate the doc pair from `docs/planning/pipeline/_templates/`; don't hand-roll it.
- Acceptance criteria use **EARS**: `shall`, one observable behavior, a verification method.
- Broad file discovery (more than about three candidate paths) goes to
  `Agent(subagent_type=Explore)`, not an inline grep walk (§18.2).

## Step 0 — Tasks
When the harness offers `TaskCreate`, create one task per step below and resolve every task
before Stop (`enforce-phase-tasks.sh` blocks a Stop that leaves one open). Without it, keep the
checklist in the notes' Phase 1 entry.

## Steps
1. **Pick the work item.** It is `$ARGUMENTS`; when that is empty or `next`, the top row of
   the **Queue** section in `docs/planning/tickets/BACKLOG.md` and its ticket doc. Rows under
   **Deliberate** are taken only when named. A rough idea not ready for a ticket becomes an
   intake doc (`docs/planning/_templates/intake.md` into `docs/planning/intake/`), and you
   stop there. If the user waives the pipeline ("just do it"), say so and work directly, still
   keeping the gate green and capturing lessons at the end.
2. **Pre-flight.** Fix any failure before going on:
   ```bash
   echo "cargo:    $(cargo --version 2>/dev/null || echo MISSING)"
   echo "gate:     $(test -x script/gates.sh && echo OK || echo MISSING)"
   echo "llvm-cov: $(cargo llvm-cov --version 2>/dev/null || echo 'cargo install cargo-llvm-cov; rustup component add llvm-tools-preview')"
   echo "nextest:  $(cargo nextest --version 2>/dev/null | head -1 || echo 'cargo install cargo-nextest')"
   echo "shear:    $(cargo shear --version 2>/dev/null || echo 'cargo install cargo-shear')"
   jq -r '.hooks.PreToolUse, .hooks.Stop' .claude/settings.json >/dev/null && echo "hooks: wired"
   ls docs/planning/pipeline/active/*.spec.md 2>/dev/null || echo "active: none"
   head -1 README.md | grep -q IMPORTANT && echo "readme marker: present" || echo "readme marker: ADD the two > [!IMPORTANT] lines before touching source"
   pgrep -fl "cargo (build|check|test|clippy|nextest|llvm-cov|doc)" || echo "cargo: idle"
   ```
   If an active pipeline exists and it is not this item, ask whether to resume it or archive
   it; never run two.
3. **Recall (§18.3).** Grep `docs/planning/knowledge/` for the item's terms (prevention rules
   and failures first) and `docs/planning/pipeline/completed/` for prior work on the same
   seams; ask the Rusty brain (`brain_ask`, or `rusty-cli brain ask`) the question this work
   decides. Write two to four bullets of what came back into the notes. "Nothing on this
   seam" is a valid result.
4. **Promote or mint.** A queued pair in `docs/planning/pipeline/queued/` (from `/spec`) moves
   into `active/`, and every seam it cites is re-verified against the code. Otherwise
   instantiate the pair from the templates with a real `pipeline_id`
   (`python3 -c 'import uuid;print(uuid.uuid4())'`). Remove the ticket's `BACKLOG.md` row:
   promotion leaves the queue. Set the ticket's `Pipeline doc` link and `Status: in-progress`.
5. **The prior-art sweep (§20)**, before any decision is locked. Three legs: the behavior maps
   (`docs/warp_architecture/`, `docs/zed_architecture/`, observed captures; research, not
   source), published material (docs, the LSP, MCP, ACP and CDP specs), and the code we
   already ship (Zed's crates and every dependency in `Cargo.lock`; reading them is
   adoption). Ask plainly whether a crate we already build owns this seam. Record what you
   found in `### Prior art`; a sweep that finds nothing says where it looked.
6. **The spec.** Title, scope (in and out), the EARS acceptance criteria, the locked
   decisions, the linked artifacts, a `## Reference (§20)` section (Warp for terminal, block
   and cockpit behavior with a `docs/warp_architecture/` citation; upstream Zed for editor and
   workspace behavior with the crate named; or `N/A — Marley-specific` and why) with its
   `### Prior art`, and a `## UI proof` section: the driven tests and the live drive for
   anything the user sees or types into, or a first line `N/A — no UI delta: <why>`.
   `enforce-warp-reference.sh` blocks a commit whose spec leaves either empty.
7. **The design, in the notes.**
   - **Approach:** how the change fits Marley and which modules and types it touches. §14
     applies: typed errors, no panics on input or response paths, testable `*_in(dir)` IO,
     spawns kept in adapters, Zed's `.rules`. A change to a Zed crate is the smallest additive
     diff, and each path outside the Marley-owned set will need its ledger row.
   - **File manifest:** each file to add or change, marked Marley crate or Zed crate.
   - **Test plan:** a table with at least one row per acceptance criterion: unit tests for
     pure code, gpui driven tests (`TestAppContext`/`VisualTestContext`) for UI paths, and
     the live drive. Name any path no test can reach, and why.
   - **Risks and decisions:** anything load-bearing and reversible. A decision worth keeping
     goes to the brain (`brain_decide`) at Complete.
8. **Present for review:** the classification, the scope, the acceptance criteria and the
   design. Wait for confirmation unless the user said the work runs autonomously.

## Closeout
- `status: Phase 1 — Plan PASS; ready for Phase 2 — Code` in the spec.
- `## Reference (§20)`, `### Prior art` and `## UI proof` are filled.
- Every task you created is resolved.
- Hand off: **"Phase 1 PASS. Run `/pipeline:code`."**

$ARGUMENTS
