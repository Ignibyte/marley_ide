# shellcheck shell=bash
# #704's visual check: Zed's editors for agents. A scripted MCP client (`e2e-agent`), run from the
# harness through the plugin's bridge, opens `src/main.rs` at line 3: the editors area asks first
# (`704-01-asked`, REQ-003). Allow for This Session opens it there, in front (`704-02-opened`,
# REQ-004), and the client's next `editor_open` runs without a question. A line typed into the
# editor is read back unsaved by `editor_read`, `editor_list` names the file, and `.env` is
# refused as a secret file; Agent Activity lists the opens and the reads (`704-03-activity`,
# REQ-001, REQ-002).
compositor sway

# The question's Allow for This Session button, from the first run's shots.
ALLOW_X=${ALLOW_X:-1225}
ALLOW_Y=${ALLOW_Y:-933}
AWAY_X=${AWAY_X:-700}
AWAY_Y=${AWAY_Y:-500}

# The plugin's bridge, the file Marley's marketplace ships.
BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge

setup() {
  mkdir -p "$E2E_WORK/repo/src" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  printf 'fn main() {\n    let answer = 42;\n    println!("{answer}");\n}\n' >"$E2E_WORK/repo/src/main.rs"
  printf 'API_TOKEN=not-a-real-token\n' >"$E2E_WORK/repo/.env"
  write_client
  open_path "$E2E_WORK/repo"
}

write_client() {
  cat >"$E2E_WORK/mcp-client.py" <<'PY'
# A scripted MCP client for #704's e2e test: one tool call through the plugin's bridge, its
# outcome printed.
import json
import os
import subprocess
import sys

bridge = subprocess.Popen([os.environ["BRIDGE"]], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
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


def tool(name, arguments):
    result = call("tools/call", {"name": name, "arguments": arguments}).get("result") or {}
    return result.get("isError", False), result.get("structuredContent") or {}


call("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "e2e-agent", "version": "0"}})
bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
bridge.stdin.flush()
command, path = sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else ""
if command == "open":
    error, answer = tool("editor_open", {"path": path, "line": 3})
    print(("refused " + answer.get("code", "")) if error else f"opened line {answer.get('line')}")
elif command == "list":
    error, answer = tool("editor_list", {})
    for editor in answer.get("editors", []):
        selections = editor.get("selections") or [{}]
        print(f"{os.path.basename(editor['path'])} active={editor['active']} dirty={editor['dirty']} language={editor['language']} cursor={selections[0].get('start')}")
elif command == "read":
    error, answer = tool("editor_read", {"path": path})
    if error:
        print("refused " + answer.get("code", ""))
    else:
        print(f"dirty={answer.get('dirty')} lines={answer.get('total_lines')}")
        print(answer.get("text", ""))
bridge.stdin.close()
bridge.wait(timeout=10)
PY
}

# The client, run from the harness, its reply in `$E2E_WORK/$1.txt`.
client() {
  BRIDGE=$BRIDGE MARLEY_MCP_ENDPOINT=$E2E_PROFILE/mcp-endpoint.json \
    python3 "$E2E_WORK/mcp-client.py" "${@:2}" >"$E2E_WORK/$1.txt" 2>&1
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

# Whether the reply `$1` holds `$2`.
replied() { grep -qF -- "$2" "$E2E_WORK/$1.txt"; }

steps() {
  local asking
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== editor_open asks first"
  client first open "$E2E_WORK/repo/src/main.rs" &
  asking=$!
  settle 4
  shot 704-01-asked

  echo "== Allow for This Session"
  click "$ALLOW_X" "$ALLOW_Y"
  wait "$asking"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  shot 704-02-opened
  expect "the first editor_open opened it at line 3" replied first "opened line 3"

  echo "== the session needs no second question"
  client second open "$E2E_WORK/repo/src/main.rs"
  expect "the second editor_open ran without asking" replied second "opened line 3"

  echo "== an unsaved line, then list and read"
  type_text "// unsaved by e2e"
  settle 1
  client listing list
  client reading read "$E2E_WORK/repo/src/main.rs"
  expect "editor_list names main.rs, active and dirty" replied listing "main.rs active=True dirty=True language=Rust"
  expect "editor_read gives the unsaved text" replied reading "// unsaved by e2e"
  client env-open open "$E2E_WORK/repo/.env"
  client env-read read "$E2E_WORK/repo/.env"
  expect ".env is refused as a secret file" replied env-read "refused secret_file"

  echo "== Agent Activity"
  palette "marley: open agent activity"
  settle 2
  shot 704-03-activity
}
