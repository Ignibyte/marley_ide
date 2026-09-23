#!/usr/bin/env bash
# enforce-phase-tasks.sh — a phase resolves the tasks it created (Stop hook).
# A pipeline phase that called TaskCreate must mark every task it created
# completed or deleted before Stop. A phase that created none is not blocked:
# some harnesses have no TaskCreate, and the checklist then lives in the notes.
# Exit 0 = allow, 2 = block.
set -euo pipefail
INPUT=$(cat)
command -v jq &>/dev/null || exit 0
source "$(dirname "$0")/lib-hook-helpers.sh"
TRANSCRIPT_PATH=$(echo "$INPUT" | jq -r '.transcript_path // empty')
[ -n "$TRANSCRIPT_PATH" ] && [ -f "$TRANSCRIPT_PATH" ] || exit 0
is_pipeline_session "$TRANSCRIPT_PATH" || exit 0

# Only enforce inside an actual pipeline phase (not /spec, not chat).
PHASE=$(detect_active_command "$TRANSCRIPT_PATH")
[ -n "$PHASE" ] || exit 0

# A fully archived pipeline (the completer moved its doc to completed/) has no
# open phase to enforce. Without this guard, a context-summarized transcript can
# leave detect_active_command pinned to an old phase marker indefinitely, blocking
# Stop on a pipeline that is already complete. In-flight pipelines always have an
# active doc, so this does not weaken their enforcement.
[ -n "$(get_active_pipeline_doc)" ] || exit 0

IDX=$(index_of_latest_phase_advance "$TRANSCRIPT_PATH")
[ -n "$IDX" ] || exit 0   # no phase advance recorded; nothing to scope

CREATED=$(count_tool_uses_after_index "$TRANSCRIPT_PATH" "$IDX" "TaskCreate")
RESOLVED=$(count_terminal_task_updates_after_index "$TRANSCRIPT_PATH" "$IDX")

if [ "$RESOLVED" -lt "$CREATED" ]; then
    { echo ""; echo "STOP BLOCKED — /pipeline:$PHASE has unresolved tasks ($RESOLVED/$CREATED terminal)."
      echo "Before leaving a phase, TaskUpdate every task to completed (done) or deleted (n/a)."
      echo "Call TaskList to see what's still open."; } >&2
    exit 2
fi
exit 0
