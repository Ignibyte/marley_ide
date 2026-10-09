# shellcheck shell=bash
# #703's visual check: agent activity and the kill switch. A scripted MCP client, run from the
# harness through the plugin's bridge as Claude Code reaches Marley, calls `terminal_run` with
# `echo hello` on the project's terminal; Agent Activity lists it, done (`703-01-activity`,
# REQ-001). `marley: stop agent control` stops agents' write tools (`703-02-stopped`, REQ-003). The
# client's next `terminal_run` is refused with `agent_control_stopped` while its `terminal_list`
# still answers, and the tab lists the refusal (`703-03-refused`, REQ-002). Home's AGENT ACTIVITY
# card shows the state, Resume and the rows (`703-04-home-card`, REQ-003). After `marley: resume
# agent control` the client's call runs again.
compositor sway

# The plugin's bridge, the file Marley's marketplace ships.
BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge

# The rail's row of Home's page, under the Home header, from the first run's shots.
HOME_X=${HOME_X:-100}
HOME_Y=${HOME_Y:-129}
AWAY_X=${AWAY_X:-1000}
AWAY_Y=${AWAY_Y:-950}

setup() {
  mkdir -p "$E2E_WORK/repo" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  write_client
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

write_client() {
  cat >"$E2E_WORK/mcp-client.py" <<'PY'
# A scripted MCP client for #703's e2e test: it runs the plugin's bridge, as Claude Code does,
# calls one tool and prints how it ended.
import json
import os
import subprocess
import sys

bridge = subprocess.Popen(
    [os.environ["BRIDGE"]],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.DEVNULL,
    text=True,
)
next_id = 0


def call(method, params):
    global next_id
    next_id += 1
    bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "id": next_id, "method": method, "params": params}) + "\n")
    bridge.stdin.flush()
    for line in bridge.stdout:
        message = json.loads(line)
        if message.get("id") == next_id:
            return message
    sys.exit(f"{method}: the bridge closed")


call("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "e2e-agent", "version": "0"}})
bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
bridge.stdin.flush()
terminals = call("tools/call", {"name": "terminal_list", "arguments": {}})["result"]["structuredContent"]["terminals"]
if sys.argv[1] == "list":
    print(f"listed {len(terminals)}")
else:
    reply = call("tools/call", {"name": "terminal_run", "arguments": {"terminal": terminals[0]["id"], "command": "echo hello", "wait_seconds": 5}})
    result = reply.get("result") or {}
    if result.get("isError"):
        print("refused " + json.dumps(result.get("structuredContent") or result.get("content")))
    else:
        print("done")
bridge.stdin.close()
bridge.wait(timeout=10)
PY
}

# The client, run from the harness, its reply in `$E2E_WORK/$1.txt`.
client() {
  BRIDGE=$BRIDGE MARLEY_MCP_ENDPOINT=$E2E_PROFILE/mcp-endpoint.json \
    python3 "$E2E_WORK/mcp-client.py" "$2" >"$E2E_WORK/$1.txt" 2>&1
  cat "$E2E_WORK/$1.txt"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
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

  echo "== an agent's terminal_run, then Agent Activity"
  client first run
  expect "the first terminal_run ran" holds "$E2E_WORK/first.txt" "done"
  palette "marley: open agent activity"
  settle 2
  away
  shot 703-01-activity

  echo "== stop agent control"
  palette "marley: stop agent control"
  settle 2
  away
  shot 703-02-stopped

  echo "== the next terminal_run, and a read"
  client second run
  client reading list
  expect "the second terminal_run was refused as stopped" holds "$E2E_WORK/second.txt" "agent_control_stopped"
  expect "terminal_list still answered" holds "$E2E_WORK/reading.txt" "listed"
  settle 2
  away
  shot 703-03-refused

  echo "== Home's card"
  click "$HOME_X" "$HOME_Y"
  settle 3
  away
  shot 703-04-home-card

  echo "== resume, then the call runs again"
  palette "marley: resume agent control"
  settle 2
  client third run
  expect "terminal_run ran after the resume" holds "$E2E_WORK/third.txt" "done"
  expect "the day file holds the rows" holds "$E2E_PROFILE/agent_control/activity-$(date +%F).jsonl" "agent_control_stopped"
}
