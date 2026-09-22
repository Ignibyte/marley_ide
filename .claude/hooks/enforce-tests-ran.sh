#!/usr/bin/env bash
# enforce-tests-ran.sh — tests must actually RUN at validate (Stop hook).
# CONSTITUTION §7/§15: writing a test file is not testing. At /pipeline:validate
# the transcript must show a real cargo-test invocation. Exit 0 = allow, 2 = block.
set -euo pipefail
INPUT=$(cat)
command -v jq &>/dev/null || exit 0
source "$(dirname "$0")/lib-hook-helpers.sh"
TRANSCRIPT_PATH=$(echo "$INPUT" | jq -r '.transcript_path // empty')
[ -n "$TRANSCRIPT_PATH" ] && [ -f "$TRANSCRIPT_PATH" ] || exit 0
is_pipeline_session "$TRANSCRIPT_PATH" || exit 0

# Only enforce when the latest pipeline command is validate.
[ "$(latest_pipeline_command "$TRANSCRIPT_PATH")" = "pipeline:validate" ] || \
[ "$(detect_active_command "$TRANSCRIPT_PATH")" = "validate" ] || exit 0

CMDS=$(extract_bash_commands "$TRANSCRIPT_PATH")
RUST_OK=false
# Anchor the runner to a command position (line start or after a shell
# separator) and drop --help/--version/--list, so a bare `echo "cargo test"`
# or `cargo test --help` doesn't satisfy the gate. This is a NUDGE against
# omission — it can't prove the run passed (that's the /commit gate's job,
# enforced by enforce-commit-gate.sh).
# Separator class `[;&|(]` catches line-start and after ; & | ( — and `&&`/`||`
# match on their 2nd char (e.g. "x && cargo test" → "& cargo test"). Avoids the
# `\|` ERE ambiguity in BSD grep. A bare `echo "cargo test"` (cargo preceded by
# a quote) does not match.
RUNNER_AT='(^|[;&|(])[[:space:]]*'
# COUNT survivors instead of `grep -vq`: the quiet grep exits at its first hit and
# closes the pipe while the upstream grep is still writing — on a LARGE transcript
# (thousands of extracted commands) that SIGPIPE + `pipefail` read as failure and
# the hook FALSE-BLOCKED a compliant session (reproduced at 49MB/9k commands;
# short transcripts fit the pipe buffer, which is why this never fired before).
# `grep -vc` consumes all input — deterministic, semantics identical.
RUNNER_HITS=$(echo "$CMDS" | grep -E "${RUNNER_AT}(cargo (test|nextest|llvm-cov)|(\./)?script/gates\.sh)" | grep -vcE -- '--help|--version|--list' || true)
[ "${RUNNER_HITS:-0}" -gt 0 ] && RUST_OK=true

if ! $RUST_OK; then
    { echo ""; echo "STOP BLOCKED — /pipeline:validate but tests did not execute:"; echo ""
      echo "  VIOLATION: Rust tests never ran. Run: cargo nextest run -p <the touched crates>  (or script/gates.sh)."
      echo ""; echo "CONSTITUTION §15: if it didn't happen in the transcript, it didn't happen."; } >&2
    exit 2
fi
exit 0
