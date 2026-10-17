# shellcheck shell=bash
# #800's visual check: agents draw catalog components with `component_show`.
# - A stand-in MCP client in the repo's terminal, through the Claude Code plugin's bridge as an
#   agent CLI runs it: `agent list` prints every catalog type (`800-07-list`, REQ-005); `agent bad`
#   prints `shown: false` and the error's JSON path (`800-05-answer`, REQ-003); `agent good` opens a
#   Component tab titled "Deploy" beside the terminal (`800-06-terminal`, REQ-004).
# - Marley's own agent on a scripted ACP agent (#744's set-up) answers a prompt with four completed
#   `component_show` calls, named as agents name them: by the title alone, as Claude Code's adapter
#   reports an MCP tool (`mcp__plugin_marley_marley__component_show`); by `_meta.tool_name`
#   `marley.component_show`, a dotted name; by `_meta.tool_name` `component_show`, as Zed's agent
#   names a context-server tool; and one whose tree has an unknown type. The thread draws each tree in
#   its call's row (`800-01-thread`, `800-02-more`, REQ-001) and the bad one's error
#   (`800-03-error`, REQ-003); the first row opened shows its raw input (`800-04-raw`, REQ-002).
compositor sway

# The repo row's + and the first tool call's disclosure, as the first run's shots found them.
REPO_PLUS_X=${REPO_PLUS_X:-236}
REPO_PLUS_Y=${REPO_PLUS_Y:-136}
DISCLOSURE_X=${DISCLOSURE_X:-1330}
DISCLOSURE_Y=${DISCLOSURE_Y:-147}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
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
# A stand-in agent CLI for #800's e2e test: it runs the plugin's bridge, as Claude Code does, and
# calls Marley's component tools, printing what comes back.
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

GOOD = {"type": "card", "title": "staging", "tone": "success", "children": [
    {"type": "stat", "label": "Deploys today", "value": 4, "change": "+1 since noon", "tone": "success"},
    {"type": "progress", "label": "Rollout", "value": 80}]}
BAD = {"type": "row", "children": [{"type": "text", "text": "ok"}, {"type": "gauge"}]}

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
# A scripted ACP agent in the Claude adapter's place for #800's e2e test: it answers a prompt with
# four completed `component_show` calls, named the ways agents name them, then a line of text.
import json
import sys

log_path = sys.argv[1]

TABLE = {"type": "table", "columns": ["Service", {"label": "Latency", "align": "right"}, "State"],
         "rows": [["api", "42 ms", {"type": "badge", "text": "up", "tone": "success"}],
                  ["worker", "310 ms", {"type": "badge", "text": "slow", "tone": "warning"}]]}
CHART = {"type": "bar_chart", "title": "Build time", "unit": " s",
         "bars": [{"label": "editor", "value": 48}, {"label": "gpui", "value": 77, "tone": "warning"}]}
STATS = {"type": "row", "gap": "large", "children": [
    {"type": "stat", "label": "Tests", "value": 1284},
    {"type": "stat", "label": "Failures", "value": 0, "change": "-3", "tone": "success"}]}
BAD = {"type": "column", "children": [{"type": "gauge", "value": 3}]}

CALLS = [
    ("c1", "mcp__plugin_marley_marley__component_show", None, TABLE),
    ("c2", "Show a component", "marley.component_show", CHART),
    ("c3", "component_show", "component_show", STATS),
    ("c4", "mcp__plugin_marley_marley__component_show", None, BAD),
]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def update(session_id, body):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session_id, "update": body}})


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
        for call_id, title, tool_name, tree in CALLS:
            body = {"sessionUpdate": "tool_call", "toolCallId": call_id, "title": title, "kind": "other",
                    "status": "completed", "rawInput": {"component": tree},
                    "content": [{"type": "content", "content": {"type": "text", "text": "shown"}}]}
            if tool_name:
                body["_meta"] = {"tool_name": tool_name}
            update(session_id, body)
        update(session_id, {"sessionUpdate": "agent_message_chunk",
                            "content": {"type": "text", "text": "Drawn above."}})
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

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== a stand-in agent CLI in the terminal"
  run_in_terminal "agent list"
  settle 6
  shot 800-07-list
  expect "component_list named the catalog's types" holds "$E2E_WORK/client.log" "types: row column card divider text heading code table key_value stat bar_chart sparkline badge progress callout"
  run_in_terminal "agent bad"
  settle 6
  shot 800-05-answer
  expect "a bad tree answered shown: false with its path" holds "$E2E_WORK/client.log" '"shown": false' '"path": "$.children[1]"'
  run_in_terminal "agent good"
  settle 6
  shot 800-06-terminal
  expect "a good tree opened a tab" holds "$E2E_WORK/client.log" '"where": "tab"'

  echo "== a Marley thread whose agent calls component_show"
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
  run_in_terminal "show the deploy"
  settle 6
  shot 800-01-thread
  pointer_to 1000 500
  scroll 10
  settle 1
  shot 800-02-more
  scroll 10
  settle 1
  shot 800-03-error
  if ((DISCLOSURE_X > 0)); then
    scroll -40
    settle 1
    pointer_to "$DISCLOSURE_X" "$DISCLOSURE_Y"
    settle 1
    click "$DISCLOSURE_X" "$DISCLOSURE_Y"
    settle 2
    shot 800-04-raw
  fi
}
