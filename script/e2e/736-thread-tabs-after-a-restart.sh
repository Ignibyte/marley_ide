# shellcheck shell=bash
# #736's visual check: thread tabs come back after a restart. The Marley entry runs a scripted agent
# (`MARLEY_ASSISTANT_ADAPTER`) that answers each prompt "Noted: <prompt>", answers `read <path>` with
# Marley's `fs/read_text_file` reply, loads a session by replaying the prompts its log holds for
# it, and logs every message. Two keys bound to `marley::NewAgentThread` open a thread on `other/`,
# outside the project, and one in the project's root.
# - Both threads get a message (`before`, REQ-001).
# - Marley quits through its palette and starts again: both tabs are back in the project, the root
#   one showing "hello" and its answer (`after`, REQ-001).
# - "again" in the restored tab is answered (`typed`, REQ-002).
# - In the restored `other/` tab, `read <other>/notes.md` is answered from the file: the folder
#   joined the project again (`folder`, REQ-004).
compositor sway

# The `other/` thread's tab ("first"), as the first run's shots found it.
FIRST_TAB_X=${FIRST_TAB_X:-510}
FIRST_TAB_Y=${FIRST_TAB_Y:-51}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo" "$E2E_WORK/other"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  printf '# notes from other\n' >"$E2E_WORK/other/notes.md"
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/agent.log" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  profile_setting marley.assistant.enabled true
  profile_setting marley.assistant.agent '"claude_code"'
  printf '[{"bindings": {"ctrl-alt-shift-y": ["marley::NewAgentThread", {"agent": "Marley"}], "ctrl-alt-shift-u": ["marley::NewAgentThread", {"agent": "Marley", "folder": "%s"}]}}]\n' \
    "$E2E_WORK/other" >"$E2E_PROFILE/config/keymap.json"
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #736's e2e test: it answers each prompt
# with "Noted: <prompt>", answers "read <path>" with the file's first line as Marley's
# fs/read_text_file returns it, loads a session by replaying the prompts its log holds for it, and
# logs every message it reads.
import json
import os
import sys

log_path = sys.argv[1]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def log(message):
    with open(log_path, "a") as log_file:
        log_file.write(json.dumps(message) + "\n")


def update(session, kind, text):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {
        "sessionId": session,
        "update": {"sessionUpdate": kind, "content": {"type": "text", "text": text}},
    }})


def prompt_text(params):
    return " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")


def read_file(session, path):
    send({"jsonrpc": "2.0", "id": "read-1", "method": "fs/read_text_file",
          "params": {"sessionId": session, "path": path}})
    while True:
        line = sys.stdin.readline()
        if not line:
            return "Read failed: no answer"
        if not line.strip():
            continue
        message = json.loads(line)
        log(message)
        if message.get("id") == "read-1":
            if "error" in message:
                return "Read failed: " + str(message["error"].get("message"))
            content = (message.get("result") or {}).get("content", "")
            return "Read: " + content.splitlines()[0] if content else "Read: (empty)"


def earlier_prompts(session):
    prompts = []
    with open(log_path) as log_file:
        for line in log_file:
            message = json.loads(line)
            params = message.get("params") or {}
            if message.get("method") == "session/prompt" and params.get("sessionId") == session:
                prompts.append(prompt_text(params))
    return prompts


sessions = 0
for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    log(message)
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "protocolVersion": 1,
            "agentCapabilities": {"loadSession": True, "promptCapabilities": {}},
            "authMethods": [],
            "agentInfo": {"name": "scripted", "version": "0"},
        }})
    elif method == "session/new":
        sessions += 1
        send({"jsonrpc": "2.0", "id": request_id,
              "result": {"sessionId": "marley-%d-%d" % (os.getpid(), sessions)}})
    elif method == "session/load":
        session = params.get("sessionId")
        for text in earlier_prompts(session):
            update(session, "user_message_chunk", text)
            update(session, "agent_message_chunk", "Noted: " + text)
        send({"jsonrpc": "2.0", "id": request_id, "result": {}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        text = prompt_text(params)
        if text.startswith("read "):
            update(session, "agent_message_chunk", read_file(session, text[len("read "):].strip()))
        else:
            update(session, "agent_message_chunk", "Noted: " + text)
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

# Whether the scripted agent's log holds `$1` messages of `$2`.
logged() {
  [[ $(grep -c "\"method\": \"$2\"" "$E2E_WORK/agent.log") -ge $1 ]]
}

# Sends `$1` to the focused thread and waits for the answer.
ask() {
  type_text "$1"
  press "" Return
  settle 4
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== a thread on other/, then one in the project's root"
  press "CTRL ALT SHIFT" u
  settle 6
  expect "the other/ thread runs the scripted agent" logged 1 session/new
  ask "first"
  press "CTRL ALT SHIFT" y
  settle 6
  expect "the root thread runs the scripted agent" logged 2 session/new
  ask "hello"
  shot 736-01-before

  echo "== Marley quits and starts again"
  quit_marley
  launch_marley
  settle 15
  shot 736-02-after
  expect "a restored thread loaded its session" logged 1 session/load

  echo "== a message in the restored tab"
  ask "again"
  shot 736-03-typed
  expect "the restored tab's message reached the agent" holds "$E2E_WORK/agent.log" '"text": "again"'

  echo "== the restored other/ tab reads its folder"
  click "$FIRST_TAB_X" "$FIRST_TAB_Y"
  settle 2
  ask "read $E2E_WORK/other/notes.md"
  shot 736-04-folder
  expect "the agent read other/'s file after the restart" holds "$E2E_WORK/agent.log" '"content": "# notes from other'
}
