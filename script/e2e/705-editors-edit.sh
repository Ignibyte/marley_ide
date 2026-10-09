# shellcheck shell=bash
# #705's visual check: an agent edits and saves an open editor. #704's scripted MCP client
# (`e2e-agent`) opens `src/main.rs`, allowed for the session, and replaces `let answer = 42;` with
# `let answer = 43;`: the editor shows it, unsaved, the file still says 42 (`705-01-edited`,
# REQ-001). An `old_text` found nowhere is refused `no_match` (REQ-002). `editor_save` asks although
# the session was allowed (`705-02-save-asked`, REQ-003); after Allow the file says 43
# (`705-03-saved`). A second edit and one Ctrl+Z bring the text back to 43 (REQ-001).
compositor sway

# The question's Allow for This Session button, from #704's shots; the editor's text, for a click.
ALLOW_X=${ALLOW_X:-1225}
ALLOW_Y=${ALLOW_Y:-933}
EDITOR_X=${EDITOR_X:-700}
EDITOR_Y=${EDITOR_Y:-200}
AWAY_X=${AWAY_X:-700}
AWAY_Y=${AWAY_Y:-500}

# The plugin's bridge, the file Marley's marketplace ships.
BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge

setup() {
  mkdir -p "$E2E_WORK/repo/src" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  printf 'fn main() {\n    let answer = 42;\n    println!("{answer}");\n}\n' >"$E2E_WORK/repo/src/main.rs"
  write_client
  open_path "$E2E_WORK/repo"
}

write_client() {
  cat >"$E2E_WORK/mcp-client.py" <<'PY'
# A scripted MCP client for #705's e2e test: one tool call through the plugin's bridge, its
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
elif command == "edit":
    old, new = sys.argv[3], sys.argv[4]
    error, answer = tool("editor_edit", {"path": path, "old_text": old, "new_text": new})
    print(("refused " + answer.get("code", "")) if error else f"edited replaced={answer.get('replaced')} line={answer.get('first_line')} dirty={answer.get('dirty')}")
elif command == "save":
    error, answer = tool("editor_save", {"path": path})
    print(("refused " + answer.get("code", "")) if error else f"saved={answer.get('saved')}")
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
  local file=$E2E_WORK/repo/src/main.rs
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== editor_open, allowed for the session"
  client opening open "$file" &
  asking=$!
  settle 4
  click "$ALLOW_X" "$ALLOW_Y"
  wait "$asking"
  settle 2
  expect "editor_open opened main.rs" replied opening "opened line 3"

  echo "== editor_edit 42 to 43"
  client editing edit "$file" "let answer = 42;" "let answer = 43;"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  shot 705-01-edited
  expect "the edit replaced once, unsaved" replied editing "edited replaced=1 line=2 dirty=True"
  expect "the file on disk still says 42" grep -qF "let answer = 42;" "$file"
  client missing edit "$file" "nowhere to be found" "x"
  expect "an old_text found nowhere is refused" replied missing "refused no_match"

  echo "== editor_save asks, then saves"
  client saving save "$file" &
  asking=$!
  settle 4
  shot 705-02-save-asked
  click "$ALLOW_X" "$ALLOW_Y"
  wait "$asking"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  shot 705-03-saved
  expect "the save ran after Allow" replied saving "saved=True"
  expect "the file on disk says 43" grep -qF "let answer = 43;" "$file"

  echo "== one undo takes an edit back"
  client again edit "$file" "let answer = 43;" "let answer = 44;"
  click "$EDITOR_X" "$EDITOR_Y"
  settle 1
  press "CTRL" z
  settle 1
  client reading read "$file"
  expect "one Ctrl+Z took the second edit back" replied reading "let answer = 43;"
}
