# shellcheck shell=bash
# #533's check: the harness's first contract request (MREQ-001) holds for what Marley serves. A
# stand-in MCP client, in a terminal through the plugin's bridge as Claude Code runs it, lists
# Marley's tools with each name against the pattern a Claude client takes (REQ-001), then calls
# `fleet_snapshot` and `session_surface_to_human` by those names (REQ-002): the fleet and a
# receipt, never an unknown-tool error. The retry ids of MREQ-002 have no caller in the app yet.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  write_mcp_agent
  # The stand-in client, as a command in the terminal.
  cat >"$bin/mcp" <<SH2
#!/usr/bin/env bash
BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge \\
  MARLEY_MCP_ENDPOINT=$E2E_PROFILE/mcp-endpoint.json exec python3 $E2E_WORK/mcp-agent.py "\$@"
SH2
  chmod +x "$bin/mcp"
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  type_text "clear; mcp names; mcp tool fleet_snapshot; mcp tool session_surface_to_human '{\"id\": \"none\"}'"
  press "" Return
  settle 6
  shot 533-01-names
  mcp_agent names | tee "$E2E_WORK/names.txt"
  expect "every listed tool's name fits a Claude client's pattern" \
    bash -c "grep -q '^  ok ' '$E2E_WORK/names.txt' && ! grep -q '^  bad' '$E2E_WORK/names.txt'"
  { mcp_agent tool fleet_snapshot; mcp_agent tool session_surface_to_human '{"id": "none"}'; } 2>&1 |
    tee "$E2E_WORK/calls.txt"
  expect "both verbs answered by their names" \
    bash -c "grep -q 'fleet_snapshot' '$E2E_WORK/calls.txt' && grep -q 'session_surface_to_human' '$E2E_WORK/calls.txt' && ! grep -qi 'unknown tool' '$E2E_WORK/calls.txt'"
}
