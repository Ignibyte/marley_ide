You are the **Delivery Gate** — `/commit`. The sole truth gate: `script/gates.sh --diff` runs here, then you commit (and open a PR if asked). Run after `/pipeline:complete`. FULL is a separate, deliberate audit over every Marley crate (`script/gates.sh`), run when the machine is idle — never part of /commit.

Read [CONSTITUTION.md](../../CONSTITUTION.md) §0 (no baselines, source-fix only) and §15 (transcript is truth). The `enforce-commit-gate.sh` hook blocks this commit unless `script/gates.sh` printed `GATE GREEN [diff]` (or `[full]`) and left a worktree-bound receipt after your last code change — so step 1 is mandatory, not advisory. `enforce-changelog.sh` blocks a Rust-source commit without a `CHANGELOG.md` entry (§21), and `enforce-warp-reference.sh` blocks a staged spec with an unfilled `## Reference (§20)`.

## Step 0 — TaskCreate
Create: "run diff gate", "stage", "commit", "(PR if asked)". Resolve all before Stop.

## Steps
1. **Run the DIFF gate and paste the real output:**
   ```bash
   script/gates.sh --diff
   ```
   It must end `GATE GREEN [diff]`. If any step is red, STOP and fix at the source — do not stage, do not weaken a gate, do not lower a floor. Re-run until green. One cargo command at a time; if another cargo is running on the box, wait for it. A `--diff` run whose change touches a Zed crate mutates in place: if the run is interrupted, `git diff` shows any leftover mutant — restore the file before continuing.
2. **Confirm the pipeline is complete** — `docs/planning/pipeline/active/` holds no spec for this work (archived at Phase 5) and the ticket doc is in `tickets/closed/`.
3. **Stage + review** — `git add -A` then show `git diff --cached --stat`; sanity-check nothing unintended (no `.mcp.json`, no `.env`, no secrets, no credential json, no `mutants.out`) is staged. The `README.md` review marker stays: only the human removes it.
4. **Commit** — a clear message: subject line, a body explaining *why*, the ticket id, ending with:
   ```
   Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
   ```
   Commit only when the user asked. If on the default branch and the user wants a branch, branch first.
5. **PR (only if the user asked)** — `gh pr create`. Zed's PR hygiene from `.rules` applies: an imperative title without a conventional-commit prefix or trailing punctuation, a `Release Notes:` section last, and a body ending:
   ```
   🤖 Generated with [Claude Code](https://claude.com/claude-code)
   ```

## Closeout
- Resolve all tasks. Report: gate result, commit SHA, branch/PR.
- Commit/push only what the user authorized.

$ARGUMENTS
