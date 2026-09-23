#!/usr/bin/env bash
# =============================================================================
# enforce-warp-reference.sh — no committing a pipeline SPEC without a filled
# `## Reference (§20)` section (PreToolUse: Bash)
# =============================================================================
# CONSTITUTION §20: every spec names its reference behavior (Warp for the
# terminal / blocks / cockpit, from docs/warp_architecture; upstream Zed for the
# editor and workspace, from the tree itself; or N/A) + how Marley matches it,
# so clean-room-from-Warp is disciplined, not ad-hoc. This
# hook makes the SECTION binding — a `git commit` that stages a pipeline spec
# whose `## Reference (§20)` is missing or empty is blocked.
#
# It mirrors enforce-changelog.sh's trigger EXACTLY: same git-commit detection,
# same staged-index key. Keep the two in lockstep — if the commit-gate's
# git-detection regex changes, change it here too (PR-claude-detection-tracks-runner).
# The hook gates PRESENCE + non-emptiness, NOT correctness (whether the behavior
# match is right is judged in the Code phase's self-review, §20).
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

# Detect a real `git ... commit` in any common spelling (same as enforce-changelog).
grep -qE '(^|[^[:alnum:]_.-])git[[:space:]]+([^;&|]*[[:space:]])?commit([[:space:]]|$)' <<<"$CMD" || exit 0
# Allow a STANDALONE dry-run/help (writes no commit) — but NOT when chained.
if grep -qE -- '(--dry-run|--help|(^|[[:space:]])-h([[:space:]]|$))' <<<"$CMD"; then
    grep -qE '[;&|]' <<<"$CMD" || exit 0
fi

cd "$PROJECT_ROOT" 2>/dev/null || exit 0
STAGED=$(git diff --cached --name-only 2>/dev/null || true)
[ -n "$STAGED" ] || exit 0

# Only real pipeline specs are gated — active/ + completed/. The _templates/ path
# is NOT active|completed, so the template (which carries only the guidance comment)
# is auto-exempt. A commit with no staged spec is exempt (like the §21 CHANGELOG gate).
SPECS=$(printf '%s\n' "$STAGED" | grep -E '^docs/planning/pipeline/(active|completed)/.*\.spec\.md$' || true)
[ -n "$SPECS" ] || exit 0

FAIL=""
OLDIFS=$IFS
IFS='
'
for f in $SPECS; do
    [ -n "$f" ] || continue
    # The STAGED (index) content — not the worktree (an unstaged edit must not satisfy the gate).
    BLOB=$(git show ":$f" 2>/dev/null || true)
    # Anchor the EXACT `## Reference (§20)` heading — a prefix match would let a `## References`
    # (plural) / `## Reference Material` section satisfy the gate while the real one is absent.
    if ! grep -qE '^## Reference \(§20\)' <<<"$BLOB"; then
        FAIL="${FAIL}  ${f} — missing the '## Reference (§20)' section
"
        continue
    fi
    # The section body = lines after the `## Reference (§20)` heading up to the next `## ` heading.
    SECTION=$(printf '%s\n' "$BLOB" | awk '/^## Reference \(§20\)/{c=1;next} c&&/^## /{c=0} c{print}')
    # Strip the HTML-comment guidance, then blank lines + bare placeholders; if nothing real remains
    # the section is unfilled. Strip SAME-LINE comments FIRST (`s/<!--.*-->//g`) — a POSIX sed
    # RANGE (`/<!--/,/-->/d`) can't close an open+close on one line, so it would swallow following
    # prose (a false-block when an editor collapses the comment or an author inlines one).
    MEANINGFUL=$(printf '%s\n' "$SECTION" \
        | sed 's/<!--.*-->//g' \
        | sed '/<!--/,/-->/d' \
        | grep -vE '^[[:space:]]*$' \
        | grep -vE '^[[:space:]]*(…|<[^>]*>|TODO)[[:space:]]*$' || true)
    if [ -z "$MEANINGFUL" ]; then
        FAIL="${FAIL}  ${f} — the '## Reference (§20)' section is empty (only the template comment/placeholder)
"
        continue
    fi
    # The PRIOR-ART sweep (§20). The wall says what we may not READ (Warp's source); it does not excuse
    # reinventing what is already ours to take. The sweep's highest-yield leg is the code we already ship:
    # Zed's own crates and every dependency in Cargo.lock. Reading them is ADOPTION — it has twice killed
    # work before it was written (#336's forks; #339's D-EMPTY-ADVANCE, which `regex::find_iter` already
    # owned). Gates PRESENCE only; "none: checked gpui/terminal/regex, no owner" is a PASS. Silence is not.
    if ! grep -qE '^### Prior art' <<<"$SECTION"; then
        FAIL="${FAIL}  ${f} — '## Reference (§20)' has no '### Prior art' subsection (CONSTITUTION §20)
"
        continue
    fi
    PRIOR=$(printf '%s\n' "$SECTION" \
        | awk '/^### Prior art/{c=1;next} c&&/^#{2,3} /{c=0} c{print}' \
        | sed 's/<!--.*-->//g' \
        | sed '/<!--/,/-->/d' \
        | grep -vE '^[[:space:]]*$' \
        | grep -vE '^[[:space:]]*(…|<[^>]*>|TODO)[[:space:]]*$' || true)
    if [ -z "$PRIOR" ]; then
        FAIL="${FAIL}  ${f} — '### Prior art' is empty; record the sweep (behavior maps · published material · the code we already ship, Zed's crates included — does a crate we already build own this seam?), or say 'none: checked X/Y, no owner'
"
    fi
done
IFS=$OLDIFS

[ -z "$FAIL" ] && exit 0

{ echo ""
  echo "COMMIT BLOCKED — a staged pipeline spec has an unfilled '## Reference (§20)' (CONSTITUTION §20)."
  printf '%s' "$FAIL"
  echo "  · Reference: Warp (terminal/blocks/cockpit, cite docs/warp_architecture or an observed capture),"
  echo "    upstream Zed (editor/workspace behavior, cite the crate), or 'N/A — Marley-specific + why'."
  echo "  · ### Prior art: the sweep — the behavior maps, published material, and THE CODE WE ALREADY SHIP"
  echo "    (Zed's crates and every dependency). Reading them is ADOPTION. Ask: does a crate we already"
  echo "    build own this seam? (It killed #336's forks and dissolved #339's D-EMPTY-ADVANCE outright.)"
  echo "    Found nothing? That is a PASS — write 'none: checked gpui/terminal/regex, no owner'."
} >&2
exit 2
