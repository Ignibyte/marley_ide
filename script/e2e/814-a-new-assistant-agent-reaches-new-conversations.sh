# shellcheck shell=bash
# #814's visual check: a changed `marley.assistant.agent` reaches the next Marley and Rusty
# conversations with no restart, and a conversation already open keeps its agent. #813's set-up
# and scripted agent (sourced from its scenario): each answer says what its session was given,
# and `sandbox: no CODEX_CONFIG` marks a process started for Claude Code.
# - On Claude Code, a Rusty conversation (`814-01-rusty-claude`).
# - `marley.assistant.agent` set to `codex` while Marley runs; a new Rusty conversation starts a
#   new process with Codex's configuration (`814-02-rusty-codex`, REQ-001), and so does a new
#   Marley conversation, read-only (`814-03-marley-codex`, REQ-002).
# - The first conversation answers again, on its Claude Code process (`814-04-still-claude`,
#   REQ-003).
. script/e2e/813-rusty-runs-commands-and-edits-files.sh

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== Claude Code: a Rusty conversation"
  palette "marley: new rusty conversation"
  settle 2
  expect "Rusty's conversation runs the scripted agent" sessions 1
  ask "what were you given"
  shot 814-01-rusty-claude
  expect "Rusty runs on a Claude Code process" answer_holds 1 \
    "- entry: Rusty" "- sandbox: no CODEX_CONFIG"

  echo "== Codex, with no restart"
  profile_setting marley.assistant.agent '"codex"'
  settle 6
  local mark
  mark=$(wc -l <"$E2E_WORK/agent.log")
  palette "marley: new rusty conversation"
  settle 2
  expect "the new Rusty conversation starts a session" session_since "$mark"
  ask "what were you given"
  shot 814-02-rusty-codex
  expect "the new Rusty conversation runs on a Codex process" answer_holds 2 \
    "- entry: Rusty" "- INITIAL_AGENT_MODE: unset" "- sandbox: none"
  mark=$(wc -l <"$E2E_WORK/agent.log")
  palette "marley: new marley conversation"
  settle 2
  expect "the new Marley conversation starts a session" session_since "$mark"
  ask "what were you given"
  shot 814-03-marley-codex
  expect "the new Marley conversation runs on Codex, read-only" answer_holds 3 \
    "- entry: Marley" "- INITIAL_AGENT_MODE: read-only" "- sandbox: read-only"

  echo "== the first conversation"
  palette "pane: activate previous item"
  palette "pane: activate previous item"
  ask "are you still there"
  shot 814-04-still-claude
  expect "the first conversation still answers from its Claude Code process" answer_holds 4 \
    "- entry: Rusty" "- sandbox: no CODEX_CONFIG"
}
