# shellcheck shell=bash
# #772's visual check: Reopen Thread…, and threads open in tabs. #739's set-up: the Marley entry on a
# scripted ACP agent that answers "Noted: <prompt>", replays a session's prompts on `session/load`,
# and logs every message; the copy's thread list emptied, so the picker lists the run's own.
# - The repo row's `+` lists Reopen Thread… under New Agent… (`plus`, REQ-007).
# - With the Marley thread's tab closed, Reopen Thread… lists it with its agent, folder and time
#   (`reopen`, REQ-001); Enter opens it in a repo tab with "first" and its answer (`reopened`,
#   REQ-002); chosen again, the same tab comes forward (`again`, REQ-003).
# - With the tab closed again, the thread's rail row opens it in a tab, no right dock (`row`,
#   REQ-004).
# - The thread tab's right-click menu has no Move Thread to Panel (`tab-menu`, REQ-006).
compositor sway

# The repo row's `+`, the thread's rail row and the thread's tab, as the first run's shots found
# them.
REPO_PLUS_X=${REPO_PLUS_X:-236}
REPO_PLUS_Y=${REPO_PLUS_Y:-136}
THREAD_ROW_X=${THREAD_ROW_X:-130}
THREAD_ROW_Y=${THREAD_ROW_Y:-230}
THREAD_TAB_X=${THREAD_TAB_X:-550}
THREAD_TAB_Y=${THREAD_TAB_Y:-51}

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
  # so the picker lists only the run's own.
  local db
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #772's e2e test: it answers each prompt
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

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 5

  echo "== a Marley thread asked \"first\", then its tab closed"
  press "CTRL ALT" n
  settle 2
  pick "Marley"
  press "" Return
  settle 2
  press "" Return
  settle 6
  expect "the thread runs the scripted agent" logged 1 session/new
  type_text "first"
  press "" Return
  settle 4
  press CTRL w
  settle 2

  echo "== the repo row's +, then Reopen Thread…"
  click "$REPO_PLUS_X" "$REPO_PLUS_Y"
  settle 1
  shot 772-00-plus
  press "" Home
  press "" Down
  press "" Down
  press "" Down
  press "" Return
  settle 2
  shot 772-01-reopen

  echo "== Enter: the conversation in a tab"
  press "" Return
  settle 6
  expect "the reopened thread loaded its session" logged 1 session/load
  shot 772-02-reopened

  echo "== Reopen Thread… again: the same tab"
  palette "marley: reopen thread"
  settle 2
  press "" Return
  settle 3
  shot 772-03-again

  echo "== the tab closed, the thread's rail row"
  press CTRL w
  settle 2
  click "$THREAD_ROW_X" "$THREAD_ROW_Y"
  settle 6
  shot 772-04-row

  echo "== the thread tab's menu"
  click "$THREAD_TAB_X" "$THREAD_TAB_Y" right
  settle 1
  shot 772-05-tab-menu
}
