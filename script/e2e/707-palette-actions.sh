# shellcheck shell=bash
# #707's visual check: an agent runs Zed's palette actions. #704's scripted MCP client
# (`e2e-agent`) runs them. The profile allows the editors area (so opening `src/main.rs` asks
# nothing) and names `editor::SelectAll` and `workspace::SendKeystrokes` in `actions_allowed`.
# - `action_list` names no unknown action (REQ-001).
# - `workspace::ToggleRightDock` asks first (`707-01-run-asked`), then hides the right dock's
#   project panel (`707-02-dock-toggled`). A second toggle asks nothing and gives the panel the focus, where
#   `pane::SplitRight` is `not_available`; from the editor it splits (`707-03-split`) (REQ-002).
# - `editor::SelectAll` runs once named; `editor::DuplicateLineDown` is not allowed (REQ-003).
# - `project_panel::Delete`, `zed::Quit` and `workspace::SendKeystrokes`, though named, are
#   refused (REQ-004).
compositor sway

# The question's Allow for This Session button, from #704's shots.
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
  profile_setting marley.agent_control.editors '"allow"'
  profile_setting marley.agent_control.actions_allowed '["editor::SelectAll", "workspace::SendKeystrokes"]'
  write_client
  open_path "$E2E_WORK/repo"
}

write_client() {
  cat >"$E2E_WORK/mcp-client.py" <<'PY'
# A scripted MCP client for #707's e2e test: one tool call through the plugin's bridge, its
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
command, argument = sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else ""
if command == "open":
    error, answer = tool("editor_open", {"path": argument, "line": 2})
    print(("refused " + answer.get("code", "")) if error else f"opened line {answer.get('line')}")
elif command == "list":
    error, answer = tool("action_list", {})
    names = [action["name"] + (" named" if action.get("named_by_user") else "") for action in answer.get("actions", [])]
    print(f"actions={len(names)} unknown={answer.get('unknown')}")
    print("\n".join(names))
elif command == "run":
    error, answer = tool("action_run", {"name": argument})
    print(("refused " + answer.get("code", "")) if error else f"ran {answer.get('name')} dispatched={answer.get('dispatched')}")
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

# Whether the reply `$1` holds `$2`.
replied() { grep -qF -- "$2" "$E2E_WORK/$1.txt"; }

steps() {
  local asking
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== action_list"
  client listing list
  expect "no allowlisted name is unknown" replied listing "unknown=[]"
  expect "the list holds the dock toggle" replied listing "workspace::ToggleRightDock"
  expect "the list holds the user's named action" replied listing "editor::SelectAll named"

  echo "== an editor, opened with no question"
  client opening open "$E2E_WORK/repo/src/main.rs"
  expect "main.rs opened" replied opening "opened line 2"
  settle 2

  echo "== the first run asks"
  client toggling run workspace::ToggleRightDock &
  asking=$!
  settle 4
  shot 707-01-run-asked
  click "$ALLOW_X" "$ALLOW_Y"
  wait "$asking"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  shot 707-02-dock-toggled
  expect "the dock toggle ran" replied toggling "ran workspace::ToggleRightDock dispatched=True"

  echo "== later runs ask nothing"
  client back run workspace::ToggleRightDock
  expect "the second toggle ran without a question" replied back "ran workspace::ToggleRightDock"
  # The reopened dock took the focus, and a split does nothing from the project panel.
  client panel-split run pane::SplitRight
  expect "a split from the panel's focus is not available" replied panel-split "refused not_available"
  click "$EDITOR_X" "$EDITOR_Y"
  settle 1
  client splitting run pane::SplitRight
  expect "the split ran from the editor" replied splitting "ran pane::SplitRight"
  settle 2
  shot 707-03-split

  echo "== named, not allowed, refused"
  client selecting run editor::SelectAll
  expect "a named action runs" replied selecting "ran editor::SelectAll"
  client duplicating run editor::DuplicateLineDown
  expect "an unnamed action is not allowed" replied duplicating "refused action_not_allowed"
  client deleting run project_panel::Delete
  expect "a delete is refused" replied deleting "refused action_refused"
  client quitting run zed::Quit
  expect "quitting is refused" replied quitting "refused action_refused"
  client keystrokes run workspace::SendKeystrokes
  expect "keystrokes are refused though named" replied keystrokes "refused action_refused"
}
