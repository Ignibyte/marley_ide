# shellcheck shell=bash
# #801's visual check: component templates from files. The run's config folder holds
# `components/deploy_status.json` (a card with two stats and a table, laid out from built-ins with
# placeholders for its inputs).
# - The Component Library's Templates group, scrolled to the page's end, draws `deploy_status` from
#   its example with its file's path (`801-01-library`, REQ-001).
# - `release_note.json` copied in while Marley runs: it appears without a restart (`801-02-added`,
#   REQ-002).
# - `bad_one.json`, whose body uses an input its schema does not name: the group shows the error
#   and the path (`801-03-broken`, REQ-003).
# - Marley's own agent on a scripted ACP agent calls `component_show` with `deploy_status`: the
#   thread draws the expanded card (`801-04-thread`, REQ-004); a second call whose `deploys` is a
#   string shows the schema error with its path (`801-05-bad-inputs`, REQ-005).
# - A stand-in agent CLI in the terminal: `agent list` lists `deploy_status` and `release_note`,
#   and not `bad_one` (`801-06-list`, REQ-006, REQ-002).
compositor sway

REPO_PLUS_X=${REPO_PLUS_X:-236}
REPO_PLUS_Y=${REPO_PLUS_Y:-136}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin components=$E2E_PROFILE/config/components
  mkdir -p "$home" "$bin" "$E2E_WORK/repo" "$components" "$E2E_WORK/later"
  write_mcp_client
  cat >"$home/.bashrc" <<RC
PS1='\$ '
agent() {
  BRIDGE="$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge" \\
    MARLEY_MCP_ENDPOINT="$E2E_PROFILE/mcp-endpoint.json" \\
    python3 "$E2E_WORK/mcp-client.py" "\$@" 2>&1 | tee -a "$E2E_WORK/client.log"
}
RC
  terminal_env HOME "$home"
  : >"$E2E_WORK/client.log"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  cat >"$components/deploy_status.json" <<'JSON'
{
  "name": "deploy_status",
  "description": "Shows one environment's deploy: its version, its deploys today and its services.",
  "inputs": {
    "type": "object",
    "properties": {
      "environment": {"type": "string"},
      "version": {"type": "string"},
      "deploys": {"type": "integer"},
      "services": {"type": "array"}
    },
    "required": ["environment", "version"]
  },
  "example": {"environment": "staging", "version": "1.4.0", "deploys": 4,
              "services": [["api", "up"], ["worker", "slow"]]},
  "body": {"type": "card", "title": {"$input": "environment"}, "tone": "success", "children": [
    {"type": "row", "gap": "large", "children": [
      {"type": "stat", "label": "Version", "value": {"$input": "version"}},
      {"type": "stat", "label": "Deploys today", "value": {"$input": "deploys"}}]},
    {"type": "table", "columns": ["Service", "State"], "rows": {"$input": "services"}}]}
}
JSON
  cat >"$E2E_WORK/later/release_note.json" <<'JSON'
{
  "name": "release_note",
  "description": "Shows a release's headline and what changed.",
  "inputs": {"type": "object", "properties": {"headline": {"type": "string"}, "text": {"type": "string"}}},
  "example": {"headline": "1.5.0 is out", "text": "Templates load from files."},
  "body": {"type": "callout", "tone": "success", "title": {"$input": "headline"}, "text": {"$input": "text"}}
}
JSON
  cat >"$E2E_WORK/later/bad_one.json" <<'JSON'
{
  "name": "bad_one",
  "description": "Uses an input it does not name.",
  "inputs": {"type": "object", "properties": {"title": {"type": "string"}}},
  "example": {"title": "x"},
  "body": {"type": "text", "text": {"$input": "missing"}}
}
JSON
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/agent.log" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  profile_setting marley.assistant.enabled true
  profile_setting marley.assistant.agent '"claude_code"'
  profile_setting use_system_path_prompts false
  open_path "$E2E_WORK/repo"
}

write_mcp_client() {
  cat >"$E2E_WORK/mcp-client.py" <<'PY'
# A stand-in agent CLI for #801's e2e test: it runs the plugin's bridge, as Claude Code does, and
# calls Marley's component tools, printing what comes back (#801: a template's use).
import json
import os
import subprocess
import sys

bridge = subprocess.Popen([os.environ["BRIDGE"]], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                          stderr=subprocess.DEVNULL, text=True)
next_id = 0


def call(method, params=None):
    global next_id
    next_id += 1
    bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "id": next_id, "method": method,
                                   "params": params or {}}) + "\n")
    bridge.stdin.flush()
    for line in bridge.stdout:
        if line.strip():
            message = json.loads(line)
            if message.get("id") == next_id:
                return message
    sys.exit(f"{method}: the bridge closed")


call("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                    "clientInfo": {"name": "e2e", "version": "0"}})
bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
bridge.stdin.flush()

GOOD = {"type": "deploy_status", "inputs": {"environment": "production", "version": "1.5.0",
                                            "deploys": 2, "services": [["api", "up"]]}}
BAD = {"type": "deploy_status", "inputs": {"environment": "production", "version": "1.5.0",
                                           "deploys": "two"}}

verb = sys.argv[1]
if verb == "list":
    reply = call("tools/call", {"name": "component_list", "arguments": {}})
    components = reply["result"]["structuredContent"]["components"]
    print("types:", " ".join(entry["type"] for entry in components))
else:
    tree = GOOD if verb == "good" else BAD
    reply = call("tools/call", {"name": "component_show",
                                "arguments": {"component": tree, "title": "Deploy"}})
    print("answer:", json.dumps(reply["result"]["structuredContent"]))
bridge.stdin.close()
PY
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #801's e2e test: it answers a prompt with
# two completed `component_show` calls that use the `deploy_status` template, the second with an
# input of the wrong type.
import json
import sys

log_path = sys.argv[1]
GOOD = {"type": "deploy_status", "inputs": {"environment": "production", "version": "1.5.0",
                                            "deploys": 2, "services": [["api", "up"], ["search", "down"]]}}
BAD = {"type": "deploy_status", "inputs": {"environment": "production", "version": "1.5.0",
                                           "deploys": "two"}}


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


sessions = 0
for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    with open(log_path, "a") as log_file:
        log_file.write(json.dumps(message) + "\n")
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "protocolVersion": 1,
            "agentCapabilities": {"loadSession": False, "promptCapabilities": {}},
            "authMethods": [],
            "agentInfo": {"name": "scripted", "version": "0"},
        }})
    elif method == "session/new":
        sessions += 1
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "marley-%d" % sessions}})
    elif method == "session/prompt":
        session_id = params.get("sessionId")
        for call_id, tree in (("t1", GOOD), ("t2", BAD)):
            send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session_id, "update": {
                "sessionUpdate": "tool_call", "toolCallId": call_id,
                "title": "mcp__plugin_marley_marley__component_show", "kind": "other",
                "status": "completed", "rawInput": {"component": tree},
                "content": [{"type": "content", "content": {"type": "text", "text": "shown"}}]}}})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

# Runs `$1` from the command palette, the name, a pause and Return in one `wtype`.
palette() {
  press "CTRL SHIFT" p
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "$1" -s 800 -k Return
}

# Types a command at the terminal's prompt and runs it, in one `wtype`.
run_in_terminal() {
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "$1" -s 300 -k Return
}

# Scrolls the Component Library to its end, where the Templates group is.
to_templates() {
  pointer_to 1000 600
  scroll 400
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== the library's Templates group"
  palette "marley: open component library"
  settle 3
  to_templates
  shot 801-01-library

  echo "== a template added while Marley runs, and a broken one"
  cp "$E2E_WORK/later/release_note.json" "$E2E_PROFILE/config/components/"
  settle 3
  to_templates
  shot 801-02-added
  cp "$E2E_WORK/later/bad_one.json" "$E2E_PROFILE/config/components/"
  settle 3
  to_templates
  shot 801-03-broken

  echo "== the stand-in agent CLI lists the templates"
  click 400 51
  settle 2
  run_in_terminal "agent list"
  settle 6
  shot 801-06-list
  expect "component_list lists deploy_status and release_note" holds "$E2E_WORK/client.log" "deploy_status" "release_note"
  expect "component_list leaves bad_one out" bash -c "! grep -q bad_one '$E2E_WORK/client.log'"

  echo "== a thread whose agent uses the template"
  click "$REPO_PLUS_X" "$REPO_PLUS_Y"
  settle 1
  press "" Home
  press "" Down
  press "" Down
  press "" Return
  settle 2
  type_text "Marley"
  settle 1
  press "" Return
  settle 2
  press "" Return
  settle 10
  run_in_terminal "show production"
  settle 6
  shot 801-04-thread
  pointer_to 1000 500
  scroll 10
  settle 1
  shot 801-05-bad-inputs
}
