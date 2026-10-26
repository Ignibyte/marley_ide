# shellcheck shell=bash
# #927's visual check: a thread's states as the Main and Threads boards draw them. Marley's entry
# runs a scripted ACP agent (`MARLEY_ASSISTANT_ADAPTER`) whose "start" reports a finished
# `browser_show_page` call for `report.html`, then asks permission for a `terminal_run` call and
# holds the turn there. The page itself is opened in a Browser tab through the Claude Code plugin's
# bridge from the project's terminal, as #802's stand-in calls it, with the offline Chromium of `browser-fixture.sh`. README.md
# is open in a pane left of the thread, so the thread's tab shares its row. No agent CLI runs.
# - `927-01-asks`: the width button "½", the waiting call's ASKS label and `terminal_run` chip,
#   Allow and Deny in one row (REQ-001, REQ-002).
# - `927-02-page`: the page card with its www tile, `report.html` and Show (REQ-003).
# - `927-03-shown`: Show clicked: the Browser tab on report.html in front (REQ-003).
# - `927-06-menu`: the thread tab's right-click menu: Rename, New Thread with Context…, Review Last
#   Reply, Archive, Delete Thread… (REQ-006).
# - `927-07-renamed`: Rename used: the tab reads "Release notes" (REQ-006).
# - `927-04-handoff`: New Thread with Context… for Marley's agent: the new thread's framed notice
#   with its counts (REQ-004).
# - `927-05-no-model`: a Zed Agent thread with no model: the centred card (REQ-005).
# - `927-08-day`: the asking thread and the no-model card in Marley Day (REQ-001 to REQ-005).
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

STOP_AT=${STOP_AT:-}
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-300}
# From the probe runs' shots.
SHOW_X=${SHOW_X:-1543}
SHOW_Y=${SHOW_Y:-261}
THREAD_TAB_X=${THREAD_TAB_X:-1205}
THREAD_TAB_Y=${THREAD_TAB_Y:-51}
RENAME_X=${RENAME_X:-1241}
RENAME_Y=${RENAME_Y:-317}
WITH_CONTEXT_X=${WITH_CONTEXT_X:-1300}
WITH_CONTEXT_Y=${WITH_CONTEXT_Y:-340}
NOTE_CLOSE_X=${NOTE_CLOSE_X:-1563}
NOTE_CLOSE_Y=${NOTE_CLOSE_Y:-879}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin db
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
  write_page_agent
  cat >"$home/.bashrc" <<RC
PS1='\$ '
show_page() {
  BRIDGE="$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge" \\
    MARLEY_MCP_ENDPOINT="$E2E_PROFILE/mcp-endpoint.json" \\
    python3 "$E2E_WORK/page-agent.py" "\$@" 2>&1 | tee -a "$E2E_WORK/client.log"
}
RC
  terminal_env HOME "$home"
  : >"$E2E_WORK/client.log"
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s\n' "$E2E_WORK/scripted-agent.py" >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  profile_setting marley.assistant '{"enabled": true, "agent": "claude_code"}'
  profile_setting marley.tmux_hosts '[]'
  # The details panels off, so four panes leave each thread room.
  profile_setting marley.thread_details false
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  cat >"$E2E_WORK/repo/report.html" <<'HTML'
<!doctype html><title>Report</title>
<body style="font: 20px sans-serif; margin: 32px"><h1>Report</h1><p>The release notes.</p></body>
HTML
  git init -q -b main "$E2E_WORK/repo"
  offline_chromium
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent for #927's check. "start": a finished browser_show_page call for
# report.html, as Claude Code's adapter names an MCP tool, then a terminal_run call that asks
# permission and holds the turn until it is answered.
import json
import os
import sys

next_id = [9000]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def request(method, params):
    next_id[0] += 1
    rid = next_id[0]
    send({"jsonrpc": "2.0", "id": rid, "method": method, "params": params})
    for answer in sys.stdin:
        if answer.strip() and json.loads(answer).get("id") == rid:
            return json.loads(answer)
    return {}


def update(session, body):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session, "update": body}})


for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "protocolVersion": 1,
            "agentCapabilities": {"loadSession": False, "promptCapabilities": {}},
            "authMethods": [], "agentInfo": {"name": "scripted", "version": "0"}}})
    elif method == "session/new":
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "states-%d" % os.getpid()}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        text = " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        if text.strip() == "start":
            update(session, {"sessionUpdate": "agent_message_chunk",
                             "content": {"type": "text", "text": "The report is open; now the tests."}})
            update(session, {"sessionUpdate": "tool_call", "toolCallId": "page-1",
                             "title": "mcp__plugin_marley_marley__browser_show_page", "kind": "other",
                             "status": "completed", "rawInput": {"path": "report.html"}})
            tool = {"toolCallId": "run-1", "title": "mcp__plugin_marley_marley__terminal_run",
                    "kind": "other", "status": "pending", "rawInput": {"command": "cargo test"}}
            update(session, {"sessionUpdate": "tool_call", **tool})
            request("session/request_permission", {
                "sessionId": session, "toolCall": tool,
                "options": [{"optionId": "allow", "name": "Allow once", "kind": "allow_once"},
                            {"optionId": "always", "name": "Always allow terminal_run", "kind": "allow_always"},
                            {"optionId": "reject", "name": "Reject", "kind": "reject_once"}]})
            update(session, {"sessionUpdate": "tool_call_update", "toolCallId": "run-1", "status": "completed"})
        else:
            update(session, {"sessionUpdate": "agent_message_chunk",
                             "content": {"type": "text", "text": "Read the context."}})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

write_page_agent() {
  cat >"$E2E_WORK/page-agent.py" <<'PY'
# Calls browser_show_page through the plugin's bridge, as #802's stand-in agent does.
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
reply = call("tools/call", {"name": "browser_show_page", "arguments": {"path": sys.argv[1]}})
print("answer:", json.dumps(reply.get("result", {}).get("structuredContent", reply)))
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

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 15
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the page in a Browser tab, then README.md with a pane to its right"
  # From the project's terminal, which Marley's bridge trusts as an agent CLI started there.
  type_text "show_page report.html"
  press "" Return
  settle 8
  cat "$E2E_WORK/client.log"
  press CTRL p
  settle 2
  type_text "README.md"
  settle 2
  press "" Return
  settle 2
  palette "pane: split right" 3

  echo "== a Marley thread in the right pane, asking"
  palette "marley: new marley conversation" 10
  type_text "start"
  settle 4
  press "" Return
  settle 8
  away
  take 927-01-asks || return 0
  take 927-02-page || return 0

  echo "== Show"
  click "$SHOW_X" "$SHOW_Y"
  settle 3
  away
  take 927-03-shown || return 0

  echo "== the thread tab's menu"
  click "$THREAD_TAB_X" "$THREAD_TAB_Y" right
  settle 1
  # Off the tab, so its tooltip does not cover the menu.
  pointer_to 1100 700
  settle 2
  take 927-06-menu || return 0

  echo "== Rename"
  click "$RENAME_X" "$RENAME_Y"
  settle 2
  press CTRL a
  type_text "Release notes"
  press "" Return
  settle 3
  away
  take 927-07-renamed || return 0

  echo "== New Thread with Context, Marley's agent"
  click "$THREAD_TAB_X" "$THREAD_TAB_Y" right
  settle 2
  click "$WITH_CONTEXT_X" "$WITH_CONTEXT_Y"
  settle 3
  type_text "marley"
  settle 2
  press "" Return
  settle 10
  away
  take 927-04-handoff || return 0

  echo "== a Zed Agent thread with no model"
  press "CTRL ALT" n
  settle 2
  type_text "zed"
  settle 1
  press "" Return
  settle 2
  press "" Return
  settle 8
  click "$NOTE_CLOSE_X" "$NOTE_CLOSE_Y"
  settle 2
  away
  take 927-05-no-model || return 0

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  away
  shot 927-08-day
}
