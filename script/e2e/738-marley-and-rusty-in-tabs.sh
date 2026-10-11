# shellcheck shell=bash
# #738's visual check: the Marley agent and Rusty in tabs. Both entries run a scripted agent
# (`MARLEY_ASSISTANT_ADAPTER`, #736's: "Noted: <prompt>", sessions replayed on load) that logs
# `session/new`'s `cwd`; Rusty is on through `marley_rusty`'s stand-in `rusty-mcp` (#696's set-up).
# - `marley: talk to marley` in the project: a Marley tab working in `assistant/marley` under the
#   profile's data folder (`marley`, REQ-001).
# - Home shown, the command again: the project shown with the same tab in front, no new session
#   (`again`, REQ-002).
# - `marley: talk to rusty`: a Rusty tab in `assistant/rusty` (`rusty`, REQ-003).
# - `marley: new marley conversation`: a second Marley tab (`new`, REQ-004).
compositor sway

# Home's header in the rail, as the first run's shots found it.
HOME_X=${HOME_X:-73}
HOME_Y=${HOME_Y:-89}

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
# A scripted ACP agent in the Claude adapter's place for #738's e2e test: it answers each prompt
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

# Sends `$1` to the focused thread and waits for the answer.
ask() {
  type_text "$1"
  press "" Return
  settle 4
}

# Runs the palette command `$1`; with `$2`, shoots the palette first.
palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  if [[ -n ${2:-} ]]; then
    shot "$2"
  fi
  press "" Return
  settle 4
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== talk to marley"
  palette "marley: talk to marley" 738-00-palette
  settle 2
  expect "Marley's tab runs the scripted agent" sessions 1
  expect "Marley works in its own folder" holds "$E2E_WORK/agent.log" 'assistant/marley"'
  ask "hello"
  shot 738-01-marley

  echo "== Home shown, then talk to marley again"
  click "$HOME_X" "$HOME_Y"
  settle 2
  shot 738-02a-home
  palette "marley: talk to marley"
  settle 2
  shot 738-02-again
  expect "no new session started" sessions 1

  echo "== talk to rusty"
  palette "marley: talk to rusty"
  settle 2
  expect "Rusty's tab runs the scripted agent" sessions 2
  expect "Rusty works in its own folder" holds "$E2E_WORK/agent.log" 'assistant/rusty"'
  shot 738-03-rusty

  echo "== new marley conversation"
  palette "marley: new marley conversation"
  settle 2
  expect "a new Marley conversation started" sessions 3
  shot 738-04-new
}
