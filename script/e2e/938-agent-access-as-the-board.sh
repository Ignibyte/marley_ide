# shellcheck shell=bash
# #938's visual check: what agents may do, as the AgentAccess board draws it. A scratch repository
# with `src/main.rs`; a stand-in MCP client run in Marley's terminal through the plugin's bridge
# (L-claude-871), so the caller is that terminal; one paired client in the run's `clients.json`.
# - The client opens `src/main.rs` while the editors' mode is `allow`; the mode turns to
#   `ask_every` and the client calls `editor_edit` 42 → 43: the question with the caller's tile,
#   "wants to edit main.rs", the terminal and project, "− let answer = 42;" and
#   "+ let answer = 43;", the footnote and the buttons (`938-01-asked`, REQ-001 to REQ-004);
#   the same in Marley Day (`938-02-day`).
# - `marley: open agent activity`: "Kept from agents" with the eight default globs and the note
#   (`938-03-kept`, REQ-005); `marley.redact_secrets_for_agents` false: the note says redaction is
#   off (`938-04-unredacted`, REQ-005).
# - The Settings window searched for "editors": Off / Ask Every Time / Ask First / Allow
#   (`938-05-modes`, REQ-006).
# - `marley: outside clients`: the paired client's levels (`938-06-clients`, REQ-007).
compositor sway

STOP_AT=${STOP_AT:-0}
# The question's Deny button, from the first run's shot.
DENY_X=${DENY_X:-1199}
DENY_Y=${DENY_Y:-929}

BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo/src" "$E2E_PROFILE/mcp"
  printf 'fn main() {\n    let answer = 42;\n    println!("{answer}");\n}\n' >"$repo/src/main.rs"
  # Each function calls one tool in the background, its answer to the run's log.
  cat >"$home/.bashrc" <<RC
PS1='\$ '
askopen() { python3 "$E2E_WORK/ask.py" editor_open '{"path": "$repo/src/main.rs"}' >>"$E2E_WORK/asks.log" 2>&1 & }
askedit() { python3 "$E2E_WORK/ask.py" editor_edit '{"path": "$repo/src/main.rs", "old_text": "let answer = 42;", "new_text": "let answer = 43;"}' >>"$E2E_WORK/asks.log" 2>&1 & }
RC
  terminal_env HOME "$home"
  terminal_env BRIDGE "$BRIDGE"
  terminal_env MARLEY_MCP_ENDPOINT "$E2E_PROFILE/mcp-endpoint.json"
  write_client
  printf '{"clients": [{"name": "laptop", "write": true, "allowed_at": 1761600000, "power": "write"}]}\n' \
    >"$E2E_PROFILE/mcp/clients.json"
  chmod 600 "$E2E_PROFILE/mcp/clients.json"
  git init -q -b main "$repo"
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.agent_control '{"editors": "allow"}'
  open_path "$repo"
}

# A stand-in MCP client (#871's): it runs the bridge, initializes, and calls one tool.
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

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the edit question"
  type_text "askopen"
  press "" Return
  settle 4
  profile_setting marley.agent_control '{"editors": "ask_every"}'
  settle 2
  click 400 51
  settle 1
  type_text "askedit"
  press "" Return
  settle 4
  pointer_to 700 300
  settle 1
  shot 938-01-asked
  if [[ $STOP_AT == 1 ]]; then
    return
  fi
  profile_setting theme '"Marley Day"'
  settle 4
  shot 938-02-day
  profile_setting theme '"Marley Night"'
  settle 3
  click "$DENY_X" "$DENY_Y"
  settle 2
  expect "the edit was denied" holds "$E2E_WORK/asks.log" "agent_control_declined"
  expect "the file still says 42" grep -qF "let answer = 42;" "$E2E_WORK/repo/src/main.rs"

  echo "== Kept from agents"
  palette "marley: open agent activity" 4
  pointer_to 1500 950
  settle 1
  shot 938-03-kept
  profile_setting marley.redact_secrets_for_agents false
  settle 3
  shot 938-04-unredacted

  echo "== the modes in the Settings window"
  palette "marley: open settings" 5
  press "CTRL" f
  settle 1
  type_text "editors"
  settle 3
  pointer_to 1500 950
  settle 1
  shot 938-05-modes
  press "CTRL" w
  settle 2

  echo "== Outside Clients"
  palette "marley: outside clients" 4
  pointer_to 1500 950
  settle 1
  shot 938-06-clients
  press "" Escape
  settle 1
  cat "$E2E_WORK/asks.log"
}
