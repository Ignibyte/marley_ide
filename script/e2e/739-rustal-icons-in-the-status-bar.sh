# shellcheck shell=bash
# #739's visual check: Home, Rusty, Threads and Marley at the right of the status bar. #738's set-up:
# the Marley entry on a scripted agent, Rusty on through its stand-in `rusty-mcp`, the copy's thread
# list emptied.
# - The status bar shows the four buttons, and the rail's header no Rusty button (`bar`, REQ-001,
#   REQ-005).
# - Home, Rusty and Threads each open their page in a tab of the repo (`home`, `rusty`, `threads`,
#   REQ-002).
# - Marley starts a Marley conversation in a repo tab (`marley`, REQ-003).
# - Threads again brings the same tab forward (`again`, REQ-004).
compositor sway

# The four buttons, as the first run's shots found them; `STOP_AT_BAR=1` ends the run at the
# first shot, to find them.
HOME_BUTTON_X=${HOME_BUTTON_X:-1309}
RUSTY_BUTTON_X=${RUSTY_BUTTON_X:-1329}
THREADS_BUTTON_X=${THREADS_BUTTON_X:-1349}
MARLEY_BUTTON_X=${MARLEY_BUTTON_X:-1369}
BUTTON_Y=${BUTTON_Y:-985}
STOP_AT_BAR=${STOP_AT_BAR:-0}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo" "$E2E_WORK/rusty"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/agent.log" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY'
import json, sys
path = sys.argv[1]
settings = json.load(open(path))
marley = settings.setdefault("marley", {})
marley["assistant"] = {"enabled": True, "agent": "claude_code"}
marley["rusty"] = {"enabled": True}
json.dump(settings, open(path, "w"), indent=2)
PY
  # The run's profile is a copy of the user's, conversations and all: the copy's list is emptied,
  # so talk-to finds only the run's own and never opens one of the user's.
  local db
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #739's e2e test: it answers each prompt
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

# Whether the scripted agent has started exactly `$1` sessions.
sessions() {
  [[ $(grep -c '"method": "session/new"' "$E2E_WORK/agent.log") -eq $1 ]]
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 3
  shot 739-01-bar
  if [[ $STOP_AT_BAR == 1 ]]; then
    return 0
  fi

  echo "== Home"
  click "$HOME_BUTTON_X" "$BUTTON_Y"
  settle 2
  shot 739-02-home

  echo "== Rusty"
  click "$RUSTY_BUTTON_X" "$BUTTON_Y"
  settle 3
  shot 739-03-rusty

  echo "== Threads"
  click "$THREADS_BUTTON_X" "$BUTTON_Y"
  settle 2
  shot 739-04-threads

  echo "== Marley"
  click "$MARLEY_BUTTON_X" "$BUTTON_Y"
  settle 6
  expect "Marley's tab runs the scripted agent" sessions 1
  shot 739-05-marley

  echo "== Threads again"
  click "$THREADS_BUTTON_X" "$BUTTON_Y"
  settle 2
  shot 739-06-again
}
