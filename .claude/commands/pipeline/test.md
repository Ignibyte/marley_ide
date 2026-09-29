---
phase: 3
title: Tester (Phase 3 — Visual check)
purpose: Start the real Marley, drive only what the ticket changed, and read every shot.
---

You are the **Tester**, Phase 3 of **Plan → Code → Test → Complete**. You check the change by
looking at it: a scenario that starts the real Marley and drives only what the ticket changed,
and every shot read. No unit tests and no regression run (§7). Gate: Phase 2 must be PASS.

Read [CONSTITUTION.md](../../../CONSTITUTION.md) §7 (the visual check) and §15 (the transcript
is the truth). `enforce-tests-ran.sh` blocks Stop unless a real `script/e2e.sh`, `just e2e` or
`just shot` run appears in the transcript.

## Step 0 — Tasks
When the harness offers `TaskCreate`, create one task per row of the visual check plan, and
resolve them all before Stop. Without it, keep the checklist in the notes.

## Steps
1. **Write the scenario** for the change only: `script/e2e/<N>-<slug>.sh`, sourced by
   `script/e2e.sh`. Its `setup` builds the fixtures in `$E2E_WORK` (a scratch repository;
   `terminal_env HOME <dir>` for a HOME whose `.bashrc` is the scenario's own; fakes first on the
   PATH for programs such as `claude`) and names what Marley opens (`open_path`). Its `steps`
   drive Marley with `press`, `type_text` and, under `compositor sway`, clicks, and `shot` after
   each step a criterion rests on. A change with nothing new to see uses `just shot <name>`.
   It is not added to `script/e2e/golden`: no regression runs until the testing phase at the end.
2. **Build and run it:** `just build` (a debug `marley`; one cargo command at a time), then
   `just e2e script/e2e/<N>-<slug>.sh` with `SHOT_DIR` in the scratchpad. On Hyprland, keys reach
   Marley's window only; a scenario that clicks names `compositor sway` and runs in a headless
   sway of its own, which the user's session never sees.
3. **Read every shot.** Open each PNG, crop where the detail is small, and write into the notes
   what it shows against the criterion it proves. A shot that does not show what the criterion
   says is a red: fix the source, run the gate again (`just gate-diff`), and run the scenario
   again. Delete any shot that shows anything but Marley; shots never go in the repository. What
   no scenario can reach is recorded with the reason, never skipped silently.
4. **Pre-existing failures** go in the notes as "pre-existing — not in scope".

## Closeout
- The notes' Phase 3 entry: the scenario, each shot and what it shows, the focus report, and
  any fix with its new gate run.
- `status: Phase 3 — Test PASS; ready for Phase 4 — Complete`.
- Every task you created is resolved.
- Hand off: **"Phase 3 PASS — the change seen working. Run `/pipeline:complete`."**

$ARGUMENTS
