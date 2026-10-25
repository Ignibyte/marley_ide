# shellcheck shell=bash
# #915's visual check: the thread's composer and details panel. Marley's entry runs a scripted ACP
# agent (`MARLEY_ASSISTANT_ADAPTER`) that offers the modes Default and Plan; "start" runs a command
# through the client's terminal, then holds an "Edit notes.md" call open until a flag file. No
# agent CLI runs.
# - `915-01-details`: the details panel as cards: THIS THREAD with "Working · Edit notes.md", the
#   context bar, the Folder and Mode tiles; ITS TERMINAL with Open in a tab (REQ-003, REQ-004).
# - `915-02-chips`: "@notes" accepted in the composer: a "notes.md" chip above the text (REQ-001).
# - `915-03-plan`: Plan clicked: the switch on Plan, the Mode tile "Plan" (REQ-002, REQ-004).
# - `915-04-terminal`: Open in a tab: the agent's terminal in a tab of its own, named for its
#   command (REQ-005).
# - `915-05-day`: Marley Day (REQ-001 to REQ-004).
# - `915-06-options`: a second thread, whose agent reports its mode as a config option as Claude
#   Code's adapter does, its default named Ask: Plan / Act beside Zed's option menu (REQ-002).
# - `915-07-options-plan`: Plan clicked there: the option set to Plan (REQ-002).
compositor sway

AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-500}
# From the probe run's shot.
PLAN_X=${PLAN_X:-1143}
PLAN_Y=${PLAN_Y:-933}
OPTIONS_PLAN_X=${OPTIONS_PLAN_X:-1166}
OPTIONS_PLAN_Y=${OPTIONS_PLAN_Y:-943}
OPEN_X=${OPEN_X:-1380}
OPEN_Y=${OPEN_Y:-530}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin db
  mkdir -p "$home" "$bin"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/release" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  profile_setting marley.assistant '{"enabled": true, "agent": "claude_code"}'
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
# A scripted ACP agent for #915's check, offering the modes Default and Plan: as session modes to
# its first session, as a config option of category mode to the next. "start": an edit with
# a diff, a command run through the client's terminal, then an open "Edit notes.md" call held until
# the release file exists.
import json
import os
import sys
import time

release = sys.argv[1]
next_id = [9000]
folders = {}
sessions = [0]


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
    option("mode", "Mode", "mode", "default", [("default", "Ask"), ("plan", "Plan")]),
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
        session = "composer-%d-%d" % (os.getpid(), sessions[0])
        folders[session] = params.get("cwd", ".")
        # Counted on disk: each conversation may start its own agent process.
        counter = os.path.join(os.path.dirname(release), "sessions")
        sessions[0] = (int(open(counter).read()) if os.path.exists(counter) else 0) + 1
        with open(counter, "w") as handle:
            handle.write(str(sessions[0]))
        if sessions[0] == 1:
            send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": session, "modes": {
                "currentModeId": "default",
                "availableModes": [{"id": "default", "name": "Default"}, {"id": "plan", "name": "Plan"}]}}})
        else:
            send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": session, "configOptions": CONFIG}})
    elif method == "session/set_config_option":
        for item in CONFIG:
            if item["id"] == params.get("configId"):
                item["currentValue"] = params.get("value")
        send({"jsonrpc": "2.0", "id": request_id, "result": {"configOptions": CONFIG}})
    elif method == "session/set_mode":
        send({"jsonrpc": "2.0", "id": request_id, "result": {}})
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
            update(session, {"sessionUpdate": "agent_message_chunk",
                             "content": {"type": "text", "text": "I will add the notes."}})
            update(session, {"sessionUpdate": "tool_call", "toolCallId": "edit-1", "title": "Edit notes.md",
                             "kind": "edit", "status": "completed", "locations": [{"path": folder + "/notes.md"}],
                             "content": [{"type": "diff", "path": folder + "/notes.md", "oldText": "# Notes\n",
                                          "newText": "# Notes\n\nThe answer's header.\nIts chips.\n"}]})
            created = request("terminal/create", {"sessionId": session, "command": "sh",
                                                  "args": ["-c", "echo Compiling; echo Finished"], "cwd": folder})
            terminal = (created.get("result") or {}).get("terminalId")
            if terminal:
                update(session, {"sessionUpdate": "tool_call", "toolCallId": "run-1", "title": "cargo build",
                                 "kind": "execute", "status": "in_progress",
                                 "content": [{"type": "terminal", "terminalId": terminal}]})
                request("terminal/wait_for_exit", {"sessionId": session, "terminalId": terminal})
                update(session, {"sessionUpdate": "tool_call_update", "toolCallId": "run-1", "status": "completed"})
            update(session, {"sessionUpdate": "tool_call", "toolCallId": "hold-1", "title": "Edit notes.md",
                             "kind": "edit", "status": "in_progress"})
            while not os.path.exists(release):
                time.sleep(0.2)
            update(session, {"sessionUpdate": "tool_call_update", "toolCallId": "hold-1", "status": "completed"})
            update(session, {"sessionUpdate": "agent_message_chunk",
                             "content": {"type": "text", "text": " The notes are in."}})
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
  printf '# Notes\n' >"$E2E_WORK/repo/notes.md"

  echo "== a Marley thread, mid-turn"
  palette "marley: new marley conversation" 10
  type_text "start"
  settle 4
  press "" Return
  settle 8
  away
  shot 915-01-details
  [ -n "${STOP_AT_DETAILS:-}" ] && return

  echo "== a mention in the draft"
  type_text "Look at @notes"
  settle 3
  press "" Tab
  settle 2
  away
  shot 915-02-chips

  echo "== Plan"
  click "$PLAN_X" "$PLAN_Y"
  settle 3
  away
  shot 915-03-plan

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 915-05-day
  profile_setting theme '"Marley Night"'
  settle 3

  echo "== Open in a tab"
  click "$OPEN_X" "$OPEN_Y"
  settle 3
  away
  shot 915-04-terminal
  touch "$E2E_WORK/release"
  settle 2

  echo "== a thread whose mode is a config option"
  palette "marley: new marley conversation" 10
  away
  shot 915-06-options
  [ -n "${STOP_AT_OPTIONS:-}" ] && return
  click "$OPTIONS_PLAN_X" "$OPTIONS_PLAN_Y"
  settle 3
  away
  shot 915-07-options-plan
}
