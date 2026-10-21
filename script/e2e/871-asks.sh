# shellcheck shell=bash
# #871's visual check: the agents' questions as the boards draw them. A stand-in MCP client, run
# in Marley's terminal through the plugin's bridge as Claude Code runs it (#491), calls
# `settings_change` and then `action_run`; each waits on the user (25 s at most).
# - Both questions as cards: the title, the change or the action, the file or the project, the
#   buttons in a row at the right, Decline and Deny quiet, Apply and Allow for This Session
#   primary (`871-01-asks`, REQ-001).
# - Marley Day, the same (`871-02-day`, REQ-001).
compositor sway

BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  # Each function calls one tool in the background, its answer to the run's log.
  cat >"$home/.bashrc" <<RC
PS1='\$ '
asksetting() { python3 "$E2E_WORK/ask.py" settings_change '{"key": "terminal.font_size", "value": 15}' >>"$E2E_WORK/asks.log" 2>&1 & }
askaction() { python3 "$E2E_WORK/ask.py" action_run '{"name": "workspace::NewFile"}' >>"$E2E_WORK/asks.log" 2>&1 & }
RC
  terminal_env HOME "$home"
  terminal_env BRIDGE "$BRIDGE"
  terminal_env MARLEY_MCP_ENDPOINT "$E2E_PROFILE/mcp-endpoint.json"
  write_client
  git init -q -b main "$E2E_WORK/repo"
  profile_setting marley.tmux_hosts '[]'
  open_path "$E2E_WORK/repo"
}

# A stand-in MCP client: it runs the bridge, initializes, and calls one tool with its arguments.
write_client() {
  cat >"$E2E_WORK/ask.py" <<'PY'
import json
import os
import subprocess
import sys

bridge = subprocess.Popen([os.environ["BRIDGE"]], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                          text=True)


def call(message_id, method, params):
    bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "id": message_id, "method": method,
                                   "params": params}) + "\n")
    bridge.stdin.flush()
    for line in bridge.stdout:
        if line.strip() and json.loads(line).get("id") == message_id:
            return json.loads(line)
    return None


call(1, "initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                       "clientInfo": {"name": "e2e", "version": "0"}})
bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
bridge.stdin.flush()
reply = call(2, "tools/call", {"name": sys.argv[1], "arguments": json.loads(sys.argv[2])})
print(f"{sys.argv[1]}: {json.dumps(reply)[:300]}")
bridge.stdin.close()
PY
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== two questions"
  type_text "asksetting"
  press "" Return
  settle 3
  type_text "askaction"
  press "" Return
  settle 4
  pointer_to 700 300
  settle 1
  shot 871-01-asks

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 871-02-day
  # Both calls end within their 25 s wait.
  settle 18
  cat "$E2E_WORK/asks.log"
}
