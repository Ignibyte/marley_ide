# shellcheck shell=bash
# #745's visual check: Threads in the rail. #772's set-up: the Marley entry on a scripted ACP agent
# that answers "Noted: <prompt>", replays a session's prompts on `session/load`, and logs every
# message; the copy's thread list emptied. Two Marley conversations, "alpha" then "beta".
# - The rail's header reads PROJECTS (pressed) and THREADS; the status bar has no Threads button
#   (`header`, REQ-001, REQ-009).
# - THREADS lists beta then alpha, each with its agent and folder, in place of the project rows
#   (`threads`, REQ-002, REQ-006's click).
# - "alp" in the search field: alpha alone (`search`, REQ-003).
# - alpha's tab closed, its row clicked: alpha in a tab with its turn (`open`, REQ-004).
# - beta's row clicked while its tab is open: beta's tab in front, one beta tab (`forward`,
#   REQ-005).
# - Ctrl+G in the rail: the projects again (`key`, REQ-006).
# - Home's Threads button: the Threads page (`home`, REQ-008).
compositor sway

# The header's THREADS, the Threads view's second row, the alpha tab, a free spot of
# the rail, the status bar's Home and Home's Threads button, as the first run's shots found them.
THREADS_X=${THREADS_X:-95}
THREADS_Y=${THREADS_Y:-17}
SECOND_ROW_X=${SECOND_ROW_X:-130}
SECOND_ROW_Y=${SECOND_ROW_Y:-141}
ALPHA_TAB_X=${ALPHA_TAB_X:-520}
ALPHA_TAB_Y=${ALPHA_TAB_Y:-51}
RAIL_SPOT_X=${RAIL_SPOT_X:-130}
RAIL_SPOT_Y=${RAIL_SPOT_Y:-700}
HOME_X=${HOME_X:-1375}
HOME_Y=${HOME_Y:-985}
HOME_THREADS_X=${HOME_THREADS_X:-450}
HOME_THREADS_Y=${HOME_THREADS_Y:-225}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
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
  # The run's profile is a copy of the user's, conversations and all: the copy's list is emptied,
  # so the rail lists only the run's own.
  local db
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #745's e2e test: it answers each prompt
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

# Whether the agent has read `$2` `$1` times or more.
logged() {
  [[ $(grep -c "\"method\": \"$2\"" "$E2E_WORK/agent.log") -ge $1 ]]
}

# Types `$1` into the open picker and waits for its list.
pick() {
  type_text "$1"
  settle 1
}

# Runs `$1` from the command palette.
palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Starts a Marley thread in the repo and asks it `$1`.
marley_thread() {
  press "CTRL ALT" n
  settle 2
  pick "Marley"
  press "" Return
  settle 2
  press "" Return
  settle 6
  type_text "$1"
  press "" Return
  settle 4
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 5

  echo "== two Marley conversations"
  marley_thread alpha
  marley_thread beta
  expect "both threads run the scripted agent" logged 2 session/new
  shot 745-01-header

  echo "== THREADS"
  click "$THREADS_X" "$THREADS_Y"
  settle 2
  shot 745-02-threads

  echo "== the search"
  type_text "alp"
  settle 2
  shot 745-03-search
  press "" Escape
  settle 1

  echo "== alpha's tab closed, its row clicked"
  click "$ALPHA_TAB_X" "$ALPHA_TAB_Y"
  settle 1
  press CTRL w
  settle 2
  click "$SECOND_ROW_X" "$SECOND_ROW_Y"
  settle 6
  expect "the reopened thread loaded its session" logged 1 session/load
  shot 745-04-open

  echo "== beta's row, its tab open"
  # alpha's reopen made it the newest, so beta is the second row now.
  click "$SECOND_ROW_X" "$SECOND_ROW_Y"
  settle 3
  shot 745-05-forward

  echo "== Ctrl+G in the rail"
  click "$RAIL_SPOT_X" "$RAIL_SPOT_Y"
  settle 1
  press CTRL g
  settle 2
  shot 745-06-key

  echo "== Home's Threads button"
  click "$HOME_X" "$HOME_Y"
  settle 2
  shot 745-07a-home
  click "$HOME_THREADS_X" "$HOME_THREADS_Y"
  settle 2
  shot 745-07-home
}
