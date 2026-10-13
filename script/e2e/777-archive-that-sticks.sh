# shellcheck shell=bash
# #777's visual check: archive that sticks, and an archived thread that comes back when opened.
# #772's set-up: the Marley entry on a scripted ACP agent that answers "Noted: <prompt>", replays a
# session's prompts on `session/load`, and logs every message; the copy's thread list emptied.
# - A Marley thread "first" in a repo tab, its row in the rail (`thread`).
# - Archive on its row: the tab closes and the row goes (`archived`, REQ-001); the rail's Threads
#   view does not list it (`threads-view`, REQ-001).
# - The Threads page, Show archived: "first" under ARCHIVED (`page`); opened from there, it is back
#   in a tab with its turn (`reopened`, REQ-003) and listed in the Threads view
#   (`threads-view-back`, REQ-003).
# - Archived again with its tab open, then quit and started: no tab and no row (`restart`,
#   REQ-002).
# - Talk to Marley: a new conversation, not "first" (`talk`, REQ-004).
# - "second" asked there, then Delete Thread… from its row's menu and its question
#   (`confirm`): the store drops it and its tab closes (`deleted`, REQ-005).
compositor sway

# The repo's thread row and its Archive (lower once the Threads page's tab has its own row), the
# header's PROJECTS and THREADS, and the Threads page's Show archived and its first archived row,
# as the first runs' shots found them.
ROW_X=${ROW_X:-130}
ROW_Y=${ROW_Y:-221}
LOWER_ROW_Y=${LOWER_ROW_Y:-267}
ARCHIVE_X=${ARCHIVE_X:-234}
PROJECTS_X=${PROJECTS_X:-40}
PROJECTS_Y=${PROJECTS_Y:-17}
THREADS_X=${THREADS_X:-95}
THREADS_Y=${THREADS_Y:-17}
SHOW_ARCHIVED_X=${SHOW_ARCHIVED_X:-335}
SHOW_ARCHIVED_Y=${SHOW_ARCHIVED_Y:-211}
ARCHIVED_ROW_X=${ARCHIVED_ROW_X:-400}
ARCHIVED_ROW_Y=${ARCHIVED_ROW_Y:-277}
FREE_X=${FREE_X:-900}
FREE_Y=${FREE_Y:-600}

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
  # The user's own Claude Code and Codex sessions stay out of the lists, so a click finds only
  # the run's threads.
  profile_setting marley.list_cli_sessions false
  # The run's profile is a copy of the user's, conversations and all: the copy's list is emptied,
  # so the rail and the Threads page list only the run's own.
  local db
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #777's e2e test: it answers each prompt
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

# Whether Zed's store in the run's profile keeps `$1` Marley threads.
kept_threads() {
  local db count=0 rows
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    rows=$(sqlite3 "$db" "select count(*) from sidebar_threads where agent_id = 'Marley'" 2>/dev/null) || rows=0
    count=$((count + rows))
  done
  [[ $count -eq $1 ]]
}

# Runs `$1` from the command palette.
palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Archive on the thread row at height `$1`, the pointer on the row first so its Archive shows.
archive_row() {
  pointer_to "$ROW_X" "$1"
  settle 1
  pointer_to "$ARCHIVE_X" "$1"
  settle 1
  click "$ARCHIVE_X" "$1"
  settle 3
  pointer_to "$FREE_X" "$FREE_Y"
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 5

  echo "== a Marley thread asked \"first\""
  press "CTRL ALT" n
  settle 2
  type_text "Marley"
  settle 1
  press "" Return
  settle 2
  press "" Return
  settle 6
  expect "the thread runs the scripted agent" logged 1 session/new
  type_text "first"
  press "" Return
  settle 4
  pointer_to "$FREE_X" "$FREE_Y"
  settle 1
  shot 777-00-thread

  echo "== Archive on its row, its tab open"
  archive_row "$ROW_Y"
  shot 777-01-archived
  click "$THREADS_X" "$THREADS_Y"
  settle 2
  shot 777-02-threads-view
  click "$PROJECTS_X" "$PROJECTS_Y"
  settle 1

  echo "== the Threads page, Show archived, first opened"
  palette "marley: open threads"
  settle 3
  shot 777-03a-page-folded
  click "$SHOW_ARCHIVED_X" "$SHOW_ARCHIVED_Y"
  settle 2
  shot 777-03-page
  click "$ARCHIVED_ROW_X" "$ARCHIVED_ROW_Y"
  settle 6
  expect "the reopened thread loaded its session" logged 1 session/load
  shot 777-04-reopened
  click "$THREADS_X" "$THREADS_Y"
  settle 2
  shot 777-05-threads-view
  click "$PROJECTS_X" "$PROJECTS_Y"
  settle 1

  echo "== archived again with its tab open, then quit and started"
  archive_row "$LOWER_ROW_Y"
  shot 777-05a-archived-again
  quit_marley
  launch_marley
  settle 15
  shot 777-06-restart

  echo "== Talk to Marley: a new conversation"
  palette "marley: talk to marley"
  settle 6
  expect "Talk to Marley started a new session" logged 2 session/new
  shot 777-07-talk

  echo "== \"second\" asked, then deleted from its row's menu"
  type_text "second"
  press "" Return
  settle 4
  expect "the store keeps first and second" kept_threads 2
  click "$ROW_X" "$ROW_Y" right
  settle 1
  press "" End
  press "" Return
  settle 2
  shot 777-08a-confirm
  press "" Return
  settle 3
  expect "the store keeps first alone" kept_threads 1
  pointer_to "$FREE_X" "$FREE_Y"
  settle 1
  shot 777-08-deleted
}
