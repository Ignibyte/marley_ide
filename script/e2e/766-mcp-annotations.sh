# shellcheck shell=bash
# #766's check: MCP annotations on every tool. No screen draws them (N/A — no UI delta), so the run
# takes one shot of Marley and checks what a client reads: a stand-in client calls `tools/list`
# through the plugin's bridge and prints each tool's hints, and the guide is checked to list every
# tool the server serves.
compositor sway

# The plugin's bridge, the file Marley's marketplace ships.
BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge

write_client() {
  cat >"$E2E_WORK/annotations.py" <<'PY'
# `tools/list` through the plugin's bridge: each tool's name and annotations, a line each.
import json
import os
import subprocess
import sys

bridge = subprocess.Popen([os.environ["BRIDGE"]], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                          stderr=subprocess.DEVNULL, text=True)
next_id = 0


def call(method, params):
    global next_id
    next_id += 1
    bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "id": next_id, "method": method,
                                   "params": params}) + "\n")
    bridge.stdin.flush()
    for line in bridge.stdout:
        message = json.loads(line)
        if message.get("id") == next_id:
            return message
    sys.exit(f"{method}: the bridge closed")


call("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                    "clientInfo": {"name": "e2e-agent", "version": "0"}})
bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
bridge.stdin.flush()
tools = call("tools/list", {})["result"]["tools"]
print(f"tools: {len(tools)}, annotated: {sum(1 for tool in tools if 'annotations' in tool)}")
for tool in tools:
    print(tool["name"], json.dumps(tool.get("annotations"), sort_keys=True))
bridge.stdin.close()
bridge.wait(timeout=10)
PY
}

setup() {
  local repo=$E2E_WORK/repo
  mkdir -p "$repo"
  write_client
  git init -q -b main "$repo"
  open_path "$repo"
}

# Whether the client's count of tools equals its count of annotated ones.
all_annotated() {
  local line
  line=$(head -1 "$E2E_WORK/annotations.txt")
  [[ $line =~ ^tools:\ ([0-9]+),\ annotated:\ ([0-9]+)$ ]] && [[ ${BASH_REMATCH[1]} == "${BASH_REMATCH[2]}" ]]
}

# The annotations line of the tool `$1`.
hints() {
  grep "^$1 " "$E2E_WORK/annotations.txt"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  shot 766-01-marley
  BRIDGE=$BRIDGE MARLEY_MCP_ENDPOINT=$E2E_PROFILE/mcp-endpoint.json \
    python3 "$E2E_WORK/annotations.py" | tee "$E2E_WORK/annotations.txt"
  expect "every listed tool is annotated" all_annotated
  expect "a read is read-only" bash -c "grep '^browser_tabs ' '$E2E_WORK/annotations.txt' | grep -q '\"readOnlyHint\": true'"
  expect "terminal_type is destructive" bash -c "grep '^terminal_type ' '$E2E_WORK/annotations.txt' | grep -q '\"destructiveHint\": true'"
  expect "browser_scroll acts but destroys nothing" bash -c "grep '^browser_scroll ' '$E2E_WORK/annotations.txt' | grep -q '\"destructiveHint\": false, \"idempotentHint\": false, \"openWorldHint\": true, \"readOnlyHint\": false'"
  expect "editor_list stays in the closed world" bash -c "grep '^editor_list ' '$E2E_WORK/annotations.txt' | grep -q '\"openWorldHint\": false'"
  expect "browser_color_scheme has its title" bash -c "grep '^browser_color_scheme ' '$E2E_WORK/annotations.txt' | grep -q '\"title\": \"Browser: color scheme\"'"
  hints browser_click
  # Every served tool has a row in the guide.
  python3 - "$E2E_WORK/annotations.txt" docs/marley/guide.md <<'PY' | tee "$E2E_WORK/guide.txt"
import sys
names = [line.split()[0] for line in open(sys.argv[1]).read().splitlines()[1:]]
guide = open(sys.argv[2]).read()
missing = [name for name in names if f"`{name}`" not in guide]
print(f"listed: {len(names)}, missing from the guide: {missing or 'none'}")
PY
  expect "the guide lists every listed tool" grep -q "missing from the guide: none" "$E2E_WORK/guide.txt"
}
