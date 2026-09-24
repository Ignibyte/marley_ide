#!/bin/sh
# Marley's hook for Claude Code. It asks Claude Code to send its terminal an OSC 777 notify,
# which Marley shows as a desktop notification while you are not looking at that terminal, and
# says nothing in any other terminal.
[ "${TERM_PROGRAM-}" = zed ] || exit 0
case "${1-}" in
    permission) message='needs your permission' ;;
    waiting) message='is waiting for you' ;;
    finished) message='finished' ;;
    *) exit 0 ;;
esac
# The project's name, without what would break the JSON or the escape.
project=$(basename "${CLAUDE_PROJECT_DIR:-$PWD}" | tr -d '\000-\037"\;')
printf '{"terminalSequence":"\\u001b]777;notify;Claude Code;%s %s\\u0007"}\n' "$project" "$message"
