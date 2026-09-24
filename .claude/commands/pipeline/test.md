---
phase: 3
title: Tester (Phase 3 — Test)
purpose: Write the ticket's e2e scenario, run it against the real Marley, read every shot, and get the gate green.
---

You are the **Tester**, Phase 3 of **Plan → Code → Test → Complete**. You write the e2e
scenario the plan named, run it against the real Marley, read every shot, and bring
`script/gates.sh --diff` to green. Gate: Phase 2 must be PASS.

Read [CONSTITUTION.md](../../../CONSTITUTION.md) §0 (the gate), §7 (e2e visualization tests)
and §15 (the transcript is the truth). `enforce-tests-ran.sh` blocks Stop unless a real
`script/e2e.sh`, `just e2e` or `just shot` run appears in the transcript. No unit or driven tests
are written or run (§7).

## Step 0 — Tasks
When the harness offers `TaskCreate`, create one task per row of the e2e plan plus "run the
gate", and resolve them all before Stop. Without it, keep the checklist in the notes.

## Steps
1. **Write the scenario** from the plan's e2e plan: `script/e2e/<N>-<slug>.sh`, sourced by
   `script/e2e.sh`. Its `setup` builds the fixtures in `$E2E_WORK` (a scratch repository;
   `terminal_env HOME <dir>` for a HOME whose `.bashrc` is the scenario's own, so nothing depends
   on the user's shell; fakes first on the PATH for programs such as `claude` or `voxtype`) and
   names what Marley opens (`open_path`). Its `steps` drive Marley with `press` and `type_text`,
   wait with `settle`, and `shot` after each step a criterion rests on. A change with nothing
   new to see uses `just shot <name>`.
2. **Build and run it:** `just build` (a debug `marley`; one cargo command at a time, the recipe
   waits for the box's other runs), then `just e2e script/e2e/<N>-<slug>.sh` with `SHOT_DIR` in
   the scratchpad. Keys reach Marley's window only; the runner prints whether the user's focus
   moved. Never send clicks: they would move the user's pointer.
3. **Read every shot.** Open each PNG, crop where the detail is small, and write into the
   notes what it shows against the criterion it proves. A red is a shot that does not show
   what the criterion says: fix the source and run the scenario again. Delete any shot that
   shows anything but Marley; shots never go in the repository. What no scenario can reach is
   recorded with the reason, never skipped silently.
4. **Run the gate:** `script/gates.sh --diff` (`just gate-diff`): every gate on the scope,
   with clippy building every target, the tests in the tree included. Fix every red at the
   source: no baselines, no suppressions (§0). The green writes the receipt the commit needs. A
   no-`.rs` change runs `--fast`.
5. **Pre-existing failures** go in the notes as "pre-existing — not in scope"; don't fix
   unrelated breakage unless asked.

## Closeout
- The notes' Phase 3 entry: the scenario, each shot and what it shows, the focus report, the
  gate output and the verdict, any pre-existing exclusions.
- `status: Phase 3 — Test PASS; ready for Phase 4 — Complete`.
- Every task you created is resolved.
- Hand off: **"Phase 3 PASS — the e2e shots read, the gate green. Run `/pipeline:complete`."**

$ARGUMENTS
