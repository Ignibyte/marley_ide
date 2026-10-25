# shellcheck shell=bash
# #907's visual check: the bottom row for a thread. Marley's entry runs a scripted ACP agent
# (`MARLEY_ASSISTANT_ADAPTER`): its session names a model, a thought level and a mode as config
# options; its first prompt reports token use, writes notes.md through the client and holds an
# "Edit notes.md" tool call open until the scenario touches a flag file; its second prompt asks for
# permission. No agent CLI runs.
# - `907-01-working`: mid-turn: the agent's tile and name, "Opus 5.5 · High", "Act", the working
#   chip with "Edit notes.md" and its time, Context with its bar and "48k of 200k", the files
#   changed, Full width (REQ-001 to REQ-006).
# - `907-02-ticked`: seconds later, the time moved (REQ-004).
# - `907-03-done`: the turn ended: the chip reads idle with the file changed (REQ-004, REQ-006).
# - `907-04-waiting`: the second prompt waits on a permission: the chip in the needs-you colour
#   with the ask (REQ-004).
# - `907-05-full-width`: Full width clicked: the thread's text runs the full width, the toggle lit
#   (REQ-007).
# - `907-06-day`: Marley Day (REQ-008).
compositor sway

AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-500}
# The bottom row's Full width, from the first run's shots.
FULL_X=${FULL_X:-990}
FULL_Y=${FULL_Y:-985}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin db
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/release" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  profile_setting marley.assistant '{"enabled": true, "agent": "claude_code"}'
  profile_setting agent.limit_content_width true
  profile_setting marley.tmux_hosts '[]'
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent for #907's check. Its session names a model, a thought level and a mode as
# config options. "start": token use, notes.md written through the client, and an open "Edit
# notes.md" tool call until the release file exists. "ask": a permission request.
import json
import os
import sys
import time

release = sys.argv[1]
next_id = [9000]
folders = {}


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


def option(ident, name, category, current, values):
    return {"id": ident, "name": name, "category": category, "type": "select", "currentValue": current,
            "options": [{"value": value, "name": label} for value, label in values]}


CONFIG = [
    option("model", "Model", "model", "opus", [("opus", "Opus 5.5"), ("sonnet", "Sonnet 5.5")]),
    option("effort", "Effort", "thought_level", "high", [("low", "Low"), ("high", "High")]),
    option("mode", "Mode", "mode", "act", [("plan", "Plan"), ("act", "Act")]),
]

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
        session = "row-%d" % os.getpid()
        folders[session] = params.get("cwd", ".")
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": session, "configOptions": CONFIG}})
    elif method == "session/set_config_option":
        send({"jsonrpc": "2.0", "id": request_id, "result": {"configOptions": CONFIG}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        folder = folders.get(session, ".")
        text = " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        if text.strip() == "ask":
            tool = {"toolCallId": "ask-1", "title": "Edit README.md", "kind": "edit", "status": "pending",
                    "rawInput": {"path": "README.md"}}
            update(session, {"sessionUpdate": "tool_call", **tool})
            request("session/request_permission", {
                "sessionId": session, "toolCall": tool,
                "options": [{"optionId": "allow", "name": "Allow", "kind": "allow_once"},
                            {"optionId": "reject", "name": "Reject", "kind": "reject_once"}]})
            update(session, {"sessionUpdate": "tool_call_update", "toolCallId": "ask-1", "status": "completed"})
        else:
            update(session, {"sessionUpdate": "usage_update", "used": 48000, "size": 200000})
            request("fs/write_text_file", {"sessionId": session, "path": folder + "/notes.md",
                                            "content": "# Notes\n\nThe bottom row.\nIts chips.\n"})
            update(session, {"sessionUpdate": "tool_call", "toolCallId": "edit-1", "title": "Edit notes.md",
                             "kind": "edit", "status": "in_progress"})
            while not os.path.exists(release):
                time.sleep(0.2)
            update(session, {"sessionUpdate": "tool_call_update", "toolCallId": "edit-1", "status": "completed"})
            update(session, {"sessionUpdate": "agent_message_chunk",
                             "content": {"type": "text", "text": "Wrote notes.md."}})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
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

  echo "== a Marley thread, mid-turn"
  palette "marley: new marley conversation" 10
  type_text "start"
  settle 4
  press "" Return
  settle 6
  away
  shot 907-01-working
  settle 8
  shot 907-02-ticked

  echo "== the turn ends"
  touch "$E2E_WORK/release"
  settle 4
  shot 907-03-done
  [ -n "${STOP_AT_DONE:-}" ] && return

  echo "== a permission waits"
  type_text "ask"
  settle 4
  press "" Return
  settle 5
  away
  shot 907-04-waiting

  echo "== Full width"
  click "$FULL_X" "$FULL_Y"
  settle 3
  away
  shot 907-05-full-width

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 907-06-day
}
