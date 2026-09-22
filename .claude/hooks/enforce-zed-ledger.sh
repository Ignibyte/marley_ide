#!/usr/bin/env bash
# =============================================================================
# enforce-zed-ledger.sh — no change to a Zed path without its ledger row
# (PreToolUse: Write|Edit)
# =============================================================================
# CONSTITUTION §14: every change outside the Marley-owned paths is recorded in
# docs/marley/zed-touchpoints.md in the same change, so an upstream merge starts
# from a complete list. This hook blocks a Write or Edit to such a path until the
# ledger's Touchpoints table names it; add the row first, then make the change.
# Bash edits (sed -i, heredocs) bypass it by design: gate:16 and the ledger check
# in enforce-commit-gate.sh catch those. The owned-path set is marley_owned_path
# in lib-hook-helpers.sh.
#
# The file is judged by the checkout it lives in, never by the session's working
# directory: git resolves its repository, and only this repository's checkouts
# (the main one and its linked worktrees) are gated, each against its own ledger.
#
# Always-on (not gated on a pipeline session). Exit 0 = allow, 2 = block.
# Bash 3.2 + BSD-grep safe.
# =============================================================================
set -euo pipefail
INPUT=$(cat)
command -v jq >/dev/null 2>&1 || exit 0
source "$(dirname "$0")/lib-hook-helpers.sh"

FILE_PATH=$(jq -r '.tool_input.file_path // empty' <<<"$INPUT")
[ -n "$FILE_PATH" ] || exit 0

block() {
    { echo ""
      echo "ZED TOUCHPOINT BLOCKED — $1"
      echo "CONSTITUTION §14: add the row first (path · what changed · why · on merge),"
      echo "then make the change, with a // Marley: comment on a code hunk."
    } >&2
    exit 2
}

# A ledger row is one line, so a name with a newline could pose as two rows.
case "$FILE_PATH" in
    *$'\n'*) block "a path with a newline in it cannot have a row in $ZED_LEDGER." ;;
esac
case "$FILE_PATH" in
    /*) ;;
    *) FILE_PATH="$PWD/$FILE_PATH" ;;
esac

# The nearest directory that exists (a Write may create the rest), and git's
# answer for it: the checkout's top level, the directory's physical path below
# it, and the repository it belongs to. Outside any work tree, including inside
# a .git directory, there is nothing to gate.
DIR=$(dirname -- "$FILE_PATH")
while [ ! -d "$DIR" ]; do DIR=$(dirname -- "$DIR"); done
TOP=$(git -C "$DIR" rev-parse --show-toplevel 2>/dev/null) || exit 0
PREFIX=$(git -C "$DIR" rev-parse --show-prefix 2>/dev/null) || exit 0
FILE_REPO=$(git -C "$DIR" rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || exit 0
HOOK_REPO=$(git -C "$(dirname "$0")" rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || exit 0
[ "$FILE_REPO" = "$HOOK_REPO" ] || exit 0
REL="${PREFIX}${FILE_PATH#"$DIR"/}"

case "/$REL/" in
    */../*)
        # A `..` in the part of the path that does not exist yet can climb out
        # of an owned or an ignored directory, so only an exact row lets it by.
        ;;
    *)
        marley_owned_path "$REL" && exit 0
        # Ignored files (build output, local config such as .mcp.json) are not
        # divergence. The ./ keeps git from reading a leading `:` as pathspec
        # magic.
        git -C "$TOP" check-ignore -q -- "./$REL" 2>/dev/null && exit 0
        ;;
esac
line_in_list "$REL" "$(zed_ledger_rows "$TOP")" && exit 0

block "$REL is upstream Zed's, and $ZED_LEDGER has no row for it."
