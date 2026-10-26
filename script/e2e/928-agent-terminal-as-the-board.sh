# shellcheck shell=bash
# #928's visual check: an agent's command card, Always run … here, the typing bar and the rich
# input's key hints, as the board draws them. A scratch repository; a stand-in `cargo` and a
# stand-in Claude Code first on the terminal's PATH through the `.bashrc`; the stand-in MCP
# client on Marley's own endpoint (`mcp_agent`, #811) calls `terminal_run` and `terminal_type`;
# `marley.agent_commands_outside_lists` asks and `marley.agent_terminal_writes` never asks. No
# agent CLI runs.
# - `928-01-card`: `terminal_run` of `cargo test -p demo`: the card, who asks, the folder, "not on
#   either list", the command, Run and Refuse with their keys, Always run cargo test here (REQ-001).
# - `928-02-always`: Always run cargo test here clicked: the command ran and the settings hold the
#   folder's pattern; `cargo test --doc` then runs with no card (REQ-002).
# - `928-03-typing`: `terminal_type` into `python3`: "… is typing into this terminal's program"
#   with Take Over and its key (REQ-003).
# - `928-04-hints`: the stand-in Claude Code running, Ctrl-G: the rich input with its key row
#   (REQ-004).
# - `928-05-day-card`, `928-05-day-hints`: the card and the hints in Marley Day (REQ-001, REQ-004).
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

STOP_AT=${STOP_AT:-}
TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-300}
# From the probe run's shot.
ALWAYS_X=${ALWAYS_X:-519}
ALWAYS_Y=${ALWAYS_Y:-835}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  printf '#!/bin/sh\necho "running $* (stand-in cargo)"\n' >"$bin/cargo"
  printf '#!/bin/bash\necho "Claude Code (stand-in)"\nexec -a claude cat >/dev/null\n' >"$bin/claude"
  chmod +x "$bin/cargo" "$bin/claude"
  export MCP_CLIENT_NAME="Stand-in agent"
  git init -q -b main "$repo"
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.agent_commands_outside_lists '"ask"'
  profile_setting marley.agent_terminal_writes '"never_ask"'
  open_path "$repo"
}

# Starts a run in the background; its output goes to `$1`.
run_later() {
  local out=$1
  shift
  mcp_agent terminal-run repo "$@" >"$out" 2>&1 &
  RUNNER=$!
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1

  echo "== a command outside the lists waits on the card"
  run_later "$E2E_WORK/run1.txt" "cargo test -p demo"
  settle 3
  away
  take 928-01-card || return 0

  echo "== Always run cargo test here"
  click "$ALWAYS_X" "$ALWAYS_Y"
  wait "$RUNNER" || true
  cat "$E2E_WORK/run1.txt"
  expect "it ran" holds "$E2E_WORK/run1.txt" "'cargo test -p demo', exit 0"
  settle 2
  grep -A3 agent_command_allowlist_by_folder "$E2E_PROFILE/config/settings.json"
  expect "the folder's pattern is kept" holds "$E2E_PROFILE/config/settings.json" \
    "agent_command_allowlist_by_folder" "$E2E_WORK/repo"
  mcp_agent terminal-run repo "cargo test --doc" | tee "$E2E_WORK/run2.txt"
  expect "the next cargo test ran with no card" holds "$E2E_WORK/run2.txt" "'cargo test --doc', exit 0"
  settle 1
  away
  take 928-02-always || return 0

  echo "== the card in Marley Day"
  run_later "$E2E_WORK/run3.txt" "cargo build --release"
  settle 3
  profile_setting theme '"Marley Day"'
  settle 4
  away
  shot 928-05-day-card
  press "" Escape
  wait "$RUNNER" || true
  profile_setting theme '"Marley Night"'
  settle 3

  echo "== an agent types into python3"
  type_text "python3 -q"
  press "" Return
  settle 3
  mcp_agent terminal-type repo --submit "print(6 * 7)" | tee "$E2E_WORK/type.txt"
  settle 2
  away
  take 928-03-typing || return 0
  type_text "exit()"
  press "" Return
  settle 2

  echo "== the stand-in Claude Code, Ctrl-G"
  type_text "claude"
  press "" Return
  settle 3
  press CTRL g
  settle 2
  type_text "Run the tests"
  settle 1
  away
  take 928-04-hints || return 0

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  away
  shot 928-05-day-hints
  press "" Escape
  settle 1
}
