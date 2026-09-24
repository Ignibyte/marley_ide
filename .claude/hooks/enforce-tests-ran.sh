#!/usr/bin/env bash
# enforce-tests-ran.sh — the e2e test must actually RUN in the Test phase (Stop hook).
# CONSTITUTION §7/§15: writing a scenario is not testing. At /pipeline:test the
# transcript must show a real run of the e2e runner. Exit 0 = allow, 2 = block.
set -euo pipefail
INPUT=$(cat)
command -v jq &>/dev/null || exit 0
source "$(dirname "$0")/lib-hook-helpers.sh"
TRANSCRIPT_PATH=$(echo "$INPUT" | jq -r '.transcript_path // empty')
[ -n "$TRANSCRIPT_PATH" ] && [ -f "$TRANSCRIPT_PATH" ] || exit 0
is_pipeline_session "$TRANSCRIPT_PATH" || exit 0

# Only enforce when the latest pipeline command is test.
[ "$(latest_pipeline_command "$TRANSCRIPT_PATH")" = "pipeline:test" ] || \
[ "$(detect_active_command "$TRANSCRIPT_PATH")" = "test" ] || exit 0

CMDS=$(extract_bash_commands "$TRANSCRIPT_PATH")
# The runner counts at a command position: a line start or a shell separator
# (`[;&|(]`, where `&&` and `||` match on their second character), then any
# `NAME=value` assignments and a `timeout <n>` in front of it. So a bare
# `echo "script/e2e.sh"` (the runner after a quote) and `script/e2e.sh --help`
# do not count. This is a NUDGE against omission: it cannot prove the shots were
# read, which the notes' Phase 3 entry records (§7).
# COUNT survivors instead of `grep -vq`: the quiet grep exits at its first hit and
# closes the pipe while the upstream grep is still writing; on a LARGE transcript
# (thousands of extracted commands) that SIGPIPE and `pipefail` read as failure and
# the hook false-blocked a compliant session (reproduced at 49MB, 9k commands).
# `grep -vc` consumes all input, with the same meaning.
RUNNER_AT='(^|[;&|(])[[:space:]]*([A-Za-z_][A-Za-z0-9_]*=[^[:space:]]*[[:space:]]+)*(timeout[[:space:]]+[0-9]+[smhd]?[[:space:]]+)?'
RUNNER_HITS=$(echo "$CMDS" | grep -E "${RUNNER_AT}((\./)?script/e2e\.sh|just[[:space:]]+(e2e|shot))([[:space:]]|$)" | grep -vcE -- '--help' || true)

if [ "${RUNNER_HITS:-0}" -eq 0 ]; then
    { echo ""; echo "STOP BLOCKED — /pipeline:test but no e2e test ran:"; echo ""
      echo "  VIOLATION: the ticket's e2e scenario never ran. Run: just e2e script/e2e/<ticket>-<slug>.sh"
      echo "  (or script/e2e.sh <scenario>; just shot <name> for a change with nothing new to see), then read every shot."
      echo ""; echo "CONSTITUTION §7/§15: if it didn't happen in the transcript, it didn't happen."; } >&2
    exit 2
fi
exit 0
