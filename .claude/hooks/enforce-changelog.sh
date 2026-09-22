#!/usr/bin/env bash
# =============================================================================
# enforce-changelog.sh — no committing CODE without a CHANGELOG entry (PreToolUse: Bash)
# =============================================================================
# CONSTITUTION §21: every code change carries a CHANGELOG.md entry so context is
# never lost. This hook makes that half BINDING — a `git commit` that includes
# Rust source is blocked unless CHANGELOG.md is in the same changeset.
#
# It mirrors enforce-commit-gate.sh's trigger EXACTLY: same git-commit detection,
# same `.rs`-in-changeset key — so a no-.rs change (docs/config/tooling) is exempt,
# just like the gate receipt. Keep the two in lockstep: if the commit gate's
# git-detection regex changes, change it here too (PR-claude-detection-tracks-runner).
#
# Always-on (not gated on a pipeline session). Exit 0 = allow, 2 = block.
# Bash 3.2 + BSD-grep safe.
# =============================================================================
set -euo pipefail
INPUT=$(cat)
command -v jq >/dev/null 2>&1 || exit 0
source "$(dirname "$0")/lib-hook-helpers.sh"

CMD=$(echo "$INPUT" | jq -r '.tool_input.command // empty')
[ -n "$CMD" ] || exit 0

# Detect a real `git ... commit` in any common spelling (same as enforce-commit-gate).
grep -qE '(^|[^[:alnum:]_.-])git[[:space:]]+([^;&|]*[[:space:]])?commit([[:space:]]|$)' <<<"$CMD" || exit 0
# Allow a STANDALONE dry-run/help (writes no commit) — but NOT when chained.
if grep -qE -- '(--dry-run|--help|(^|[[:space:]])-h([[:space:]]|$))' <<<"$CMD"; then
    grep -qE '[;&|]' <<<"$CMD" || exit 0
fi

cd "$PROJECT_ROOT" 2>/dev/null || exit 0
# Gate the COMMIT, which records the STAGED index — NOT `git status --porcelain`
# (the worktree). An untracked or merely-modified-but-unstaged CHANGELOG.md is not
# in the commit and must not satisfy the gate. Our flow stages with `git add -A` as
# its own step before `git commit`, so the staged set IS the changeset. (A chained
# `git add … && git commit` stages AFTER this PreToolUse hook runs and is not
# introspected — stage first, then commit.)
STAGED=$(git diff --cached --name-only 2>/dev/null || true)
[ -n "$STAGED" ] || exit 0

# Rust application source in the commit = crates/<crate>/{src,examples}/**.rs.
# A no-source commit (docs/config/tooling) is exempt, like the §15 receipt.
CODE_CHANGED=$(printf '%s\n' "$STAGED" | grep -E '^crates/[^/]+/(src|examples)/.*\.rs$' || true)
[ -n "$CODE_CHANGED" ] || exit 0

# The ROOT CHANGELOG.md staged in the same commit satisfies §21 (a subdir
# CHANGELOG.md does not — anchored to the repo root).
LOG_CHANGED=$(printf '%s\n' "$STAGED" | grep -E '^CHANGELOG\.md$' || true)
[ -n "$LOG_CHANGED" ] && exit 0

{ echo ""
  echo "COMMIT BLOCKED — Rust source staged without a CHANGELOG entry (CONSTITUTION §21)."
  echo "Add an entry to the root CHANGELOG.md for this change, stage it, then commit."
  echo "(A no-source commit — docs/config/tooling — is exempt; this only gates crates/*/src.)"
} >&2
exit 2
