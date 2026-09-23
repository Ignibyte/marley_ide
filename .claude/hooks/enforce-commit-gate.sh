#!/usr/bin/env bash
# =============================================================================
# enforce-commit-gate.sh — no committing code without a green gate (PreToolUse: Bash)
# =============================================================================
# CONSTITUTION §0/§15: script/gates.sh is the truth gate. This hook makes it
# binding — a `git commit` that includes Rust source is BLOCKED unless
# script/gates.sh left a RECEIPT (.git/ignibyte-gate-receipt) proving a FULL
# or DIFF green ran on the EXACT current worktree (FAST writes none). Every
# commit, Rust or not, must also pass gate:16's ledger check (CONSTITUTION §14).
#
# The receipt is a CONTENT FINGERPRINT (gate_state_hash), not a transcript
# string. That closes the holes a string-match would have:
#   - can't be forged by printing/echoing "GATE GREEN" or by reading a file that
#     contains the literal (no receipt is written by either);
#   - any post-green edit by ANY tool (Write, Edit, or a Bash `cat >`/`sed -i`)
#     changes the fingerprint, so a stale green is rejected;
#   - a FAST run writes no receipt, so `--fast` can never satisfy a commit;
#   - lowering a floor can't help — the gate clamps floors to the §0 minimums
#     before it will print green and write the receipt.
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

# Detect a real `git ... commit` in any common spelling: `git commit`,
# `git -C . commit`, `git -c k=v commit`, `env X=1 git commit`, `(git commit)`,
# `eval "git commit"`, `sh -c "git commit"`. (A literal `git` token, then a
# `commit` token, allowing intervening flags/args but not a command separator.)
grep -qE '(^|[^[:alnum:]_.-])git[[:space:]]+([^;&|]*[[:space:]])?commit([[:space:]]|$)' <<<"$CMD" || exit 0
# Allow a STANDALONE dry-run/help (writes no commit) — but NOT when chained with
# another command (which could smuggle a real commit past the skip).
if grep -qE -- '(--dry-run|--help|(^|[[:space:]])-h([[:space:]]|$))' <<<"$CMD"; then
    grep -qE '[;&|]' <<<"$CMD" || exit 0
fi

cd "$PROJECT_ROOT" 2>/dev/null || exit 0
# Every commit, Rust or not, is checked against the Zed touchpoint ledger
# (CONSTITUTION §14): gate:16's own check over the index and the work tree
# about to be committed. The receipt below covers neither the ledger nor a
# non-Rust Zed file, so this is the check that makes a Bash edit unrecordable.
if ! LEDGER_REPORT=$(zed_ledger_check "$(upstream_base)"); then
    { echo ""
      echo "COMMIT BLOCKED — the tree differs from upstream Zed in ways $ZED_LEDGER does not record (gate:16)."
      echo "$LEDGER_REPORT"
      echo "CONSTITUTION §14: give each changed Zed path its row (path · what changed · why · on merge),"
      echo "and remove the rows of changes that are gone."
    } >&2
    exit 2
fi
# Only gate when Rust SOURCE is in the change set (staged, unstaged, or an
# untracked file — `--untracked-files=all` lists a new crate's files instead of
# collapsing them to one directory entry, which would let a fresh crate skip the
# receipt). The
# optional trailing quote matches git's C-quoted paths (names with spaces/tabs).
CODE_CHANGED=$(git status --porcelain --untracked-files=all 2>/dev/null | grep -E '\.rs"?$' || true)
[ -n "$CODE_CHANGED" ] || exit 0

# The gate's receipt must exist AND still match the current worktree fingerprint.
GITDIR=$(git rev-parse --git-dir 2>/dev/null || true)
RECEIPT="${GITDIR:-.git}/ignibyte-gate-receipt"
if [ -f "$RECEIPT" ] && [ "$(cat "$RECEIPT" 2>/dev/null)" = "$(gate_state_hash)" ]; then
    exit 0
fi

{ echo ""
  echo "COMMIT BLOCKED — no green gate (DIFF or FULL) for the current worktree."
  echo "CONSTITUTION §0: run  script/gates.sh --diff  (GATE GREEN [diff]; a [full]"
  echo "green also counts) AFTER your last code change, then commit. Fix every red at"
  echo "the source — no baselines, no suppressions, no lowering a floor."
  if [ -f "$RECEIPT" ]; then
    echo "(a receipt exists but its fingerprint no longer matches — code changed since the"
    echo " gate ran; re-run script/gates.sh.)"
  fi
} >&2
exit 2
