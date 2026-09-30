# shellcheck shell=bash
# #597's check: the fleet contract's new fields as Marley's MCP server serves them. A stand-in
# `claude` runs the plugin's real hook once with a UserPromptSubmit, so the fleet holds a seat.
# Then a stand-in MCP client, in the terminal through the plugin's bridge as Claude Code runs it,
# prints the snapshot, whose seat declares no capabilities and so has no `capabilities` key
# (REQ-004), and a `session_surface_to_human` call, still answered by its name (REQ-001). Without
# a `session.write` grant the answer is its refusal; `fleet_snapshot`'s output schema (REQ-006) is
# not in `tools/list`, which lists only the served families.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
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
  # A stand-in Claude Code: one prompt through the plugin's hook, whose answer it writes to the
  # terminal as Claude Code writes a hook's `terminalSequence`.
  sed -e "s|@HOOK@|$HOOK|" >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
import json, os, subprocess, sys

event = {"hook_event_name": "UserPromptSubmit", "prompt": "Check the contract",
         "session_id": "e2e-597", "transcript_path": "/tmp/e2e-597.jsonl",
         "cwd": os.getcwd(), "permission_mode": "default"}
answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps(event),
                        capture_output=True, text=True, check=False).stdout
sequence = json.loads(answer or "{}").get("terminalSequence")
if sequence:
    sys.stdout.write(sequence)
    sys.stdout.flush()
print("Claude Code (stand-in): one prompt sent through the hook", flush=True)
FAKE
  chmod +x "$bin/mcp" "$bin/claude"
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
  type_text "claude"
  press "" Return
  settle 3
  type_text "clear; mcp tool fleet_snapshot; mcp tool session_surface_to_human '{\"id\": \"none\"}'"
  press "" Return
  settle 6
  shot 597-01-contract

  mcp_agent tool fleet_snapshot | tee "$E2E_WORK/snapshot.txt"
  expect "the fleet holds the stand-in's seat" holds "$E2E_WORK/snapshot.txt" '"seats": [{'
  expect "a seat that declares nothing has no capabilities key" \
    bash -c "! grep -q '\"capabilities\"' '$E2E_WORK/snapshot.txt'"
  mcp_agent tool session_surface_to_human '{"id": "none"}' 2>&1 | tee "$E2E_WORK/surface.txt"
  expect "the surface verb answers by its name" \
    holds "$E2E_WORK/surface.txt" 'session_surface_to_human' '"result":"refused"'
  expect "no unknown-tool error" bash -c "! grep -qi 'unknown tool' '$E2E_WORK/surface.txt'"
}
