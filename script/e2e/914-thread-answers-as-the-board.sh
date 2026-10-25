# shellcheck shell=bash
# #914's visual check: a thread's answers as the board draws them. Marley's entry runs a scripted
# ACP agent (`MARLEY_ASSISTANT_ADAPTER`): config options name its model and effort; "start" sends an
# edit with a diff, runs a command through the client's terminal, then holds until a flag file;
# "ask" asks a permission. No agent CLI runs.
# - `914-01-working`: the answer's header (tile, Marley, "Opus 5.5 · High", "Working · N s"), the
#   EDIT label filled with "+N −M", the RUN label filled with "passed" (REQ-001 to REQ-004).
# - `914-02-done`: the turn ended: no state, pill chips, the send square, the width text button,
#   the sent message plain in its bubble (REQ-001, REQ-005 to REQ-008).
# - `914-03-asks`: "ask": the newest answer's header reads "Needs you" (REQ-002).
# - `914-04-day`: Marley Day (REQ-001 to REQ-008).
compositor sway

AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-500}

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
# A scripted ACP agent for #914's check. "start": an edit tool call with a diff, a command run
# through the client's terminal, then held until the release file exists. "ask": a permission.
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
        session = "answers-%d" % os.getpid()
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
            update(session, {"sessionUpdate": "tool_call", "toolCallId": "hold-1", "title": "Check the notes",
                             "kind": "read", "status": "in_progress"})
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

  echo "== a Marley thread, mid-turn"
  palette "marley: new marley conversation" 10
  type_text "start"
  settle 4
  press "" Return
  settle 8
  away
  shot 914-01-working

  echo "== the turn ends"
  touch "$E2E_WORK/release"
  settle 4
  away
  shot 914-02-done

  echo "== a permission waits"
  type_text "ask"
  settle 4
  press "" Return
  settle 5
  away
  shot 914-03-asks

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 914-04-day
}
