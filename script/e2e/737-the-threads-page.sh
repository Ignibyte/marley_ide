# shellcheck shell=bash
# #737's visual check: the Threads page. One scripted agent (#736's: it answers "Noted: <prompt>"
# and loads a session by replaying its log) runs both the Marley entry and a custom agent server,
# Scripted. Three keys bound to `marley::NewAgentThread` start Marley in the project's root and
# Scripted on `other/` and on `second/`; each gets a message.
# - `marley: open threads`: Marley's conversation under Marley and Rusty, the two Scripted ones
#   under All conversations, each with agent, folder and time (`page`, REQ-001, REQ-002).
# - `second` in the search: only that conversation (`search`, REQ-003).
# - The Marley chip: only Marley's (`chip`, REQ-004).
# - The `second/` tab closed, then its row clicked: a tab with its turn, loaded (`open`, REQ-005).
# The run's profile is a copy of the user's, so the page may list the user's own conversations
# below the run's.
compositor sway

# The Marley chip, the All chip, the `second/` conversation's row and its tab, as the first run's
# shots found them.
MARLEY_CHIP_X=${MARLEY_CHIP_X:-395}
MARLEY_CHIP_Y=${MARLEY_CHIP_Y:-177}
ALL_CHIP_X=${ALL_CHIP_X:-288}
ALL_CHIP_Y=${ALL_CHIP_Y:-177}
SECOND_ROW_X=${SECOND_ROW_X:-340}
SECOND_ROW_Y=${SECOND_ROW_Y:-322}
SECOND_TAB_X=${SECOND_TAB_X:-760}
SECOND_TAB_Y=${SECOND_TAB_Y:-51}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo" "$E2E_WORK/other" "$E2E_WORK/second"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/agent.log" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  profile_setting marley.assistant.enabled true
  profile_setting marley.assistant.agent '"claude_code"'
  profile_setting agent_servers.Scripted "{\"type\": \"custom\", \"command\": \"$bin/scripted-agent\"}"
  printf '[{"bindings": {"ctrl-alt-shift-y": ["marley::NewAgentThread", {"agent": "Marley"}], "ctrl-alt-shift-u": ["marley::NewAgentThread", {"agent": "Scripted", "folder": "%s"}], "ctrl-alt-shift-i": ["marley::NewAgentThread", {"agent": "Scripted", "folder": "%s"}]}}]\n' \
    "$E2E_WORK/other" "$E2E_WORK/second" >"$E2E_PROFILE/config/keymap.json"
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #737's e2e test: it answers each prompt
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

# Runs the palette command `$1`.
palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 3
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== three conversations"
  press "CTRL ALT SHIFT" y
  settle 6
  expect "Marley's thread runs the scripted agent" logged 1 session/new
  ask "marley first"
  press "CTRL ALT SHIFT" u
  settle 6
  expect "the other/ thread runs the scripted agent" logged 2 session/new
  ask "other notes"
  press "CTRL ALT SHIFT" i
  settle 6
  expect "the second/ thread runs the scripted agent" logged 3 session/new
  ask "second plan"

  echo "== the Threads page"
  palette "marley: open threads"
  shot 737-01-page

  echo "== the search"
  type_text "second"
  settle 2
  shot 737-02-search
  press CTRL a
  press "" BackSpace
  settle 1

  echo "== the Marley chip"
  click "$MARLEY_CHIP_X" "$MARLEY_CHIP_Y"
  settle 2
  shot 737-03-chip
  click "$ALL_CHIP_X" "$ALL_CHIP_Y"
  settle 1

  echo "== a closed conversation opened from the page"
  click "$SECOND_TAB_X" "$SECOND_TAB_Y"
  settle 1
  press CTRL w
  settle 2
  palette "marley: open threads"
  shot 737-04a-before-open
  click "$SECOND_ROW_X" "$SECOND_ROW_Y"
  settle 6
  shot 737-04-open
  expect "the reopened conversation loaded its session" logged 1 session/load
}
