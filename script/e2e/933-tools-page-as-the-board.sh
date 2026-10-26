# shellcheck shell=bash
# #933's visual check: MCP Servers and Agent Activity as the Tools board draws them. A stand-in
# `claude` (`MARLEY_CLAUDE`) answers `--version` with `2.1.304 (Claude Code)` and otherwise sleeps;
# `MARLEY_CODEX` names a file that is not there, so Codex is missing. A scratch `CLAUDE_CONFIG_DIR`
# keeps the user's Claude Code config away. `mcp_agent` plays an agent on Marley's own endpoint
# and makes the calls. No agent CLI runs.
# - `933-01-servers`: the subtitle, Claude Code's card (tile, version, path, "ready", chips),
#   Codex's ("missing"), Marley's own card with the tool count and its pill (REQ-001 to REQ-004).
# - `933-02-chip`: the pointer on an integration chip: its tooltip (REQ-003).
# - `933-03-activity`: Agent Activity after `terminal_run` of `echo hello` and one on a terminal
#   that does not exist: the cards, "exit 0", the refusal's reason in red (REQ-005 to REQ-007).
# - `933-04-home`: Home's AGENT ACTIVITY card, one line per call (REQ-008).
# - `933-05-day`, `933-06-day-activity`: MCP Servers and Agent Activity in Marley Day (REQ-001,
#   REQ-005).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

STOP_AT=${STOP_AT:-}
CHIP_X=${CHIP_X:-345}
CHIP_Y=${CHIP_Y:-198}
HOME_TAB_X=${HOME_TAB_X:-388}
HOME_TAB_Y=${HOME_TAB_Y:-50}
AWAY_X=${AWAY_X:-1500}
AWAY_Y=${AWAY_Y:-950}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config
  mkdir -p "$home" "$bin" "$config"
  printf '%s\n' "PS1='\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$bin/claude" <<'FAKE'
#!/bin/sh
# A stand-in Claude Code: its version, else it waits.
if [ "$1" = "--version" ]; then
  echo "2.1.304 (Claude Code)"
  exit 0
fi
exec sleep 600
FAKE
  chmod +x "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude MARLEY_CODEX=$bin/codex CLAUDE_CONFIG_DIR=$config
  export MCP_CLIENT_NAME="Stand-in agent"
  profile_setting marley.tmux_hosts '[]'
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== MCP Servers"
  palette "marley: open mcp servers" 4
  away
  take 933-01-servers || return 0

  echo "== a chip's tooltip"
  pointer_to "$CHIP_X" "$CHIP_Y"
  settle 2
  take 933-02-chip || return 0
  away

  echo "== two calls"
  mcp_agent terminal-run "" "echo hello" --wait 5
  mcp_agent refusal terminal_run '{"terminal": 99999, "command": "echo nowhere"}'
  settle 2
  tail -2 "$E2E_PROFILE/agent_control/activity-$(date +%F).jsonl"
  palette "marley: open agent activity" 3
  away
  take 933-03-activity || return 0

  echo "== Home"
  # Home and Agent Activity share the Home group; its tab brings Home forward.
  click "$HOME_TAB_X" "$HOME_TAB_Y"
  settle 2
  away
  take 933-04-home || return 0

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  palette "marley: open mcp servers" 3
  away
  shot 933-05-day
  palette "marley: open agent activity" 3
  away
  shot 933-06-day-activity
}
