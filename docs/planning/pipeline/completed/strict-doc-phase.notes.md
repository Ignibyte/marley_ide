# Strict documentation phase — Notes

- **Forge ticket:** #3 `368f2d21-74ec-4e21-bee1-47dcc09984e4` (chore), claimed `dc7df9b5-…`.
- **AAR:** `0c98d79e-be52-4aa9-a95f-743d5becb770`.
- **Local ticket doc:** `docs/planning/tickets/open/TICKET-doc-phase.md`.
- **Branch:** `ticket-strict-doc-phase` (stacked on 001/intake).

## Phase 1 — Plan

- **Request:** force a CHANGELOG + architecture-doc update each ticket, unskippably;
  audit phase-skip resistance. Per the session goal, before resuming the crate stack.
- **Classification / tier:** work pipeline, **chore (gate-is-test infra)**, no `.rs`.
- **Forge recall (§18.3):** `AD-claude-receipt-scope-001` (41a6fc62) — the
  commit-hook that blocks on a changeset condition (`.rs` in changeset → require a
  receipt). The CHANGELOG hook is the same shape (`.rs` in changeset → require a
  CHANGELOG.md change). Reuse its git-commit detection regex + `.rs`-trigger logic.

### Phase-skip-resistance audit (REQ-008) — verdict
Empirically observed across TICKET-000/001:

| Hook | Blocks | Verdict |
|---|---|---|
| `enforce-phase-gate` | a `Write`/`Edit` to `crates/*/src/**` before the prior phase is PASS | **BITES** — at 001 it allowed `lib.rs` only at implement (Design PASS); design/plan writes to crates are blocked. |
| `enforce-phase-tasks` | `Stop` with zero tasks created this phase, or any unresolved | **BITES** — every phase created + resolved tasks; an unresolved task blocks Stop. |
| `enforce-pipeline-completion` | `Stop` with a placeholder / `IN PROGRESS` / `<…>` status | **BITES** — status must be advanced to a real `Phase N … PASS`. |
| `enforce-tests-ran` | `Stop` at `validate` without a real `cargo test`/`nextest`/`gates.sh` in the transcript | **BITES** — proven at 001 validate (nextest + FULL gate ran). |
| `enforce-commit-gate` | a `git commit` including `.rs` without a receipt matching the worktree | **BITES** — proven at 001 (allowed only because the receipt matched; a stale/absent receipt blocks). |

**New hard check added here:** `enforce-changelog` — a `.rs` commit must also touch
`CHANGELOG.md`. Sixth unskippable, evidence-based gate.

**Disclosed soft spots (NOT new holes — already §15/AD-recorded, out of scope):**
1. **Self-reported phase status.** The hooks check the `status:` line, not whether
   the work (e.g. inspect critics) actually ran — §15 says explicitly the scaffold
   catches *omissions*, not deliberate *fabrication*; the commit receipt + the new
   changelog hook are the hard checks. (A future ratchet could require the transcript
   to show inspect `Agent` calls before validate — deferred; false-positive risk.)
2. **No-`.rs` gate-defining-file receipt bypass** (`AD-claude-receipt-scope-001`) —
   a change to `gates.sh`/hooks commits without a receipt; the fingerprint already
   binds those files so a *later* `.rs` commit re-verifies; full closure deferred to
   a FULL-greenable workspace.

No NEW hole requiring a fix. The ticket's contribution is the changelog gate + the
strict `complete` step.

### Edit surface (for Design)
- NEW `.claude/hooks/enforce-changelog.sh` (mirror `enforce-commit-gate.sh:23-65`).
- `.claude/settings.json` — add it under PreToolUse(Bash).
- NEW `CHANGELOG.md` (root).
- `.claude/commands/pipeline/complete.md` — step 1 strict + a TaskCreate line.
- `CONSTITUTION.md` — new §21; cite in §3 + the §0/§15 summary.

**Phase 1 status:** PASS (autonomous-through-commit per session goal). → Phase 2 Design.

## Phase 2 — Design

Gate-is-test (zero `.rs`). Verification = the hook's exit codes + negative smokes.

### `enforce-changelog.sh` (PreToolUse Bash) — algorithm
Mirror `enforce-commit-gate.sh` exactly so the two stay in lockstep:
```
set -euo pipefail
INPUT=$(cat); command -v jq >/dev/null 2>&1 || exit 0
source lib-hook-helpers.sh            # PROJECT_ROOT only
CMD=$(echo "$INPUT" | jq -r '.tool_input.command // empty'); [ -n "$CMD" ] || exit 0
# same git-commit detector as the commit gate (git … commit; -C/-c/env/(); not after a separator)
echo "$CMD" | grep -qE '(^|[^[:alnum:]_.-])git[[:space:]]+([^;&|]*[[:space:]])?commit([[:space:]]|$)' || exit 0
# standalone --dry-run/--help/-h (not chained) → allow
if echo "$CMD" | grep -qE -- '(--dry-run|--help|(^|[[:space:]])-h([[:space:]]|$))'; then
    echo "$CMD" | grep -qE '[;&|]' || exit 0
fi
cd "$PROJECT_ROOT" 2>/dev/null || exit 0
CODE=$(git status --porcelain 2>/dev/null | grep -E '\.rs"?$' || true)
[ -n "$CODE" ] || exit 0              # no .rs in changeset → exempt (docs/config/tooling)
LOG=$(git status --porcelain 2>/dev/null | grep -E '(^|[ /])CHANGELOG\.md"?$' || true)
[ -n "$LOG" ] && exit 0               # CHANGELOG.md in the changeset → allow
{ echo ""; echo "COMMIT BLOCKED — code change without a CHANGELOG entry (CONSTITUTION §21)."
  echo "Add an entry to CHANGELOG.md describing this change, then commit."; } >&2
exit 2
```
- Trigger parity with the receipt gate: keys on `.rs"?$` (C-quoted-path safe), so a
  no-`.rs` change (this ticket) is exempt — atomic "code ⇒ changelog", and infra/docs
  commits aren't burdened.
- `(^|[ /])CHANGELOG\.md"?$` matches the root `CHANGELOG.md` (porcelain ` M CHANGELOG.md`)
  and tolerates a C-quoted path; bash-3.2/BSD-safe, shellcheck-clean.

### `.claude/settings.json`
Add a second hook to the existing **PreToolUse → matcher "Bash"** block (next to
`enforce-commit-gate.sh`), `timeout: 10`.

### `CHANGELOG.md` (root, Keep a Changelog)
`## [Unreleased]` (empty) + `## [0.0.0] — 2026-06-28` with **Added**: the Marley
quality pipeline (TICKET-000) and `marley_text_offsets` (TICKET-001). Plus THIS
ticket's own entry (dogfood) under `[Unreleased]`: "Added — strict documentation
phase (enforce-changelog hook + CONSTITUTION §21)."

### `complete.md` step 1 (strict)
Replace "Update project docs **if** behavior/architecture changed" with: "**Required:
(a)** add a `CHANGELOG.md` entry for this ticket; **(b)** update the relevant
`docs/marley_architecture/` (and any component doc) so the architecture record stays
current (CONSTITUTION §21). Skipping either is a §21 violation; `enforce-changelog`
blocks the code commit without (a)." Add the matching Step-0 TaskCreate item.

### `CONSTITUTION.md` — new §21 (binding)
"Every pipeline's Phase 5 (Complete) **shall** add a `CHANGELOG.md` entry and update
the architecture docs, so context is never lost. The CHANGELOG half is
machine-enforced (`enforce-changelog.sh` blocks a `.rs` commit lacking a CHANGELOG
change, mirroring the §15 receipt — a no-`.rs` change is exempt). The architecture-doc
half is a required, inspect-verified Phase-5 step." Cite §21 in the §3 pipeline list
(at `/pipeline:complete`) and in the §15 enforcement summary (add the changelog hook
beside the commit receipt as the two hard, evidence-based commit-time checks).

### File manifest (gate-is-test — zero `.rs`)
| File | Change |
|---|---|
| `.claude/hooks/enforce-changelog.sh` | NEW (algorithm above) |
| `.claude/settings.json` | wire it into PreToolUse(Bash) |
| `CHANGELOG.md` | NEW (backfill 000/001 + this ticket) |
| `.claude/commands/pipeline/complete.md` | step 1 → strict + TaskCreate item |
| `CONSTITUTION.md` | new §21 + §3/§15 citations |

### Regression test plan — negative smokes
| Smoke | Proves | Method |
|---|---|---|
| code-without-CHANGELOG blocked | REQ-002 | temp `crates/marley_text_offsets/src/lib.rs` comment edit (do NOT commit) → feed a synthetic `git commit` JSON to `enforce-changelog.sh` → **exit 2** |
| CHANGELOG present ⇒ allowed | REQ-003 | with the temp `.rs` edit + a temp `CHANGELOG.md` edit → hook **exit 0** |
| no-`.rs` exempt | REQ-004 | clean docs/config changeset (this ticket) → hook **exit 0** |
| wired + shellcheck-clean | REQ-005 | grep settings.json; `gates.sh --fast` gate:11 |
| `gates.sh --fast` green | REQ-009 | run it |
| revert | — | restore the temp `.rs`; the only committed CHANGELOG change is this ticket's real entry |

### Risks
- **R1 — detector drift from the commit gate.** If `enforce-commit-gate`'s
  git-detection regex is ever changed, this hook must change in lockstep (they share
  the trigger). Note it in both files. (Mirrors `PR-claude-detection-tracks-runner`.)
- **R2 — porcelain CHANGELOG match.** A rename/odd path could miss; anchored to
  `(^|[ /])CHANGELOG\.md"?$`. Inspect re-checks the bypass surface.

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement

Built the manifest (zero `.rs`):
- `.claude/hooks/enforce-changelog.sh` (chmod +x) — mirrors `enforce-commit-gate.sh`
  (same git-commit detector + `.rs"?$` trigger); blocks a Rust-source commit lacking
  a `CHANGELOG.md` change; no-`.rs` exempt.
- `.claude/settings.json` — wired into PreToolUse(Bash) (now 2 Bash hooks).
- `CHANGELOG.md` (root) — Keep a Changelog; `[Unreleased]` = this ticket; `[0.0.0]`
  = backfilled TICKET-000 + TICKET-001.
- `.claude/commands/pipeline/complete.md` — Step 0 task + Step 1 made STRICT (§21).
- `CONSTITUTION.md` — new **§21 — Documentation Phase**; cited in the §3 pipeline
  list + the §15 enforcement summary (changelog hook beside the receipt).

Verified: `bash -n` + `shellcheck` clean; settings.json valid; functional check —
a no-`.rs` `git commit` through the hook → **exit 0 (exempt)**; `scripts/gates.sh
--fast` → **GATE GREEN [fast] 12/12**.

No deviations from design.

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5)

2 critics (hook-bypass · doc-consistency), each verifying by running the hook
against synthetic staged states + grepping the docs.

| # | Sev | Finding | Verdict | Fix |
|---|---|---|---|---|
| 1 | **HIGH** | The hook checked `git status --porcelain` (the **worktree**), not the staged index. An untracked/unstaged `CHANGELOG.md` merely present satisfied the gate — and since `CHANGELOG.md` was itself untracked, the gate was **LIVE-inert** (every `.rs` commit exempt for free). A subdir `CHANGELOG.md` decoy also passed (unanchored). | **REAL** (verified: staged `.rs` + untracked CHANGELOG → exit 0 under the old hook) | Gate on `git diff --cached --name-only` (the commit's staged index); require `^CHANGELOG\.md$` (root-anchored); trigger on `^crates/[^/]+/(src\|examples)/.*\.rs$`. Re-verified: staged `.rs`+untracked CHANGELOG → **2**; +staged CHANGELOG → **0**; subdir decoy → **2**; nothing staged → **0**. forge `BF-changelog-hook-worktree-vs-staged-001` + `PR-…commit-hook-checks-staged-index-001`. |
| 2 | MED | Commit detector misses `commit` followed by `"`/`)`/`;` (e.g. `sh -c "git commit"`) — shared verbatim with `enforce-commit-gate`. | **ACCEPTED (not fixed)** | In lockstep with the commit-gate; the `sh -c "git commit"` edge is deliberate *circumvention* (§15 scopes that out, the receipt is the hard check), and adding `"` to the closer class would false-block the quoted mention `echo "git commit"` (which must stay exempt). Documented limitation; both hooks consistent. |
| 3 | LOW | CHANGELOG/notes said "five enforcement hooks" but 7 are wired (`enforce-quality` unmentioned). | **REAL** (doc precision) | Reworded to "the five **skip-resistance** hooks … (`enforce-quality` covers formatting separately)". |

**Verified CORRECT (Critic B):** §21 binding + cited in §3 and §15; `complete.md`
step 1 strict (no "if behavior changed") + Step-0 task; CHANGELOG well-formed
Keep-a-Changelog (000/001/this) and outside gate:14's `docs/` scan; the
skip-audit's 3 spot-checks (phase-gate gates `crates/*/src`, tests-ran requires a
real run, commit-gate requires a matching receipt) are accurate, not overclaims;
settings.json valid + wired.

Post-fix: shellcheck clean; `gates.sh --fast` GREEN 12/12.

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate

Gate-is-test (zero `.rs`) → no unit tests; verification = the hook's exit codes +
negative smokes.

**enforce-changelog negative smokes (validate-of-record, self-reverting):**
| Smoke | Expect | Got |
|---|---|---|
| (a) staged `crates/…/src/zz_tmp.rs`, CHANGELOG untracked-not-staged | exit 2 BLOCK | **2 ✓** (REQ-002) |
| (b) + `git add CHANGELOG.md` | exit 0 ALLOW | **0 ✓** (REQ-003) |
| (c) nothing staged (exempt) | exit 0 | **0 ✓** (REQ-004) |
| (d) subdir `crates/…/CHANGELOG.md` decoy staged | exit 2 BLOCK | **2 ✓** (root-anchor) |

No temp artifacts left (`git reset` + `rm`; `git status` clean of `zz_tmp`).

**Structural (REQ-005/006/007):** `enforce-changelog.sh` wired in
`.claude/settings.json` PreToolUse(Bash); `complete.md` step 1 strict (`REQUIRED —
CONSTITUTION §21`; old "if behavior" text gone); `CONSTITUTION.md` carries §21 with
3 references (heading + §3 + §15).

**REQ-009 gate:** `scripts/gates.sh --fast` → **GATE GREEN [fast], 12/12** (the new
hook is shellcheck-clean, gate:11).

**FULL gate / receipt:** not required — this ticket stages **no** `crates/*/src`
`.rs`, so both `enforce-commit-gate` and the new `enforce-changelog` exit 0 at
commit (no-source path, §15/§21). Dogfood: this ticket's own change is recorded in
`CHANGELOG.md [Unreleased]`.

**Pre-existing failures:** none.

**Phase 4 status:** PASS — smokes 4/4 + gate green. → Phase 5 Complete.
