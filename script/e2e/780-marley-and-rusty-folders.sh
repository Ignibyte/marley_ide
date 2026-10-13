# shellcheck shell=bash
# #780's visual check: Marley and Rusty start in their own folders, set in settings. #746's set-up:
# the Marley entry on a scripted ACP agent that logs every message (each `session/new` with its
# `cwd`), Rusty on through its stand-in `rusty-mcp`, whose vault is a scratch folder; the copy's
# thread list emptied.
# - New Agent, "Marley", Enter: a Marley conversation at once, no Where; its `cwd` is the run's
#   Marley settings folder (`marley`, REQ-001, REQ-003).
# - New Agent, "Rusty", Enter: a Rusty conversation at once; its `cwd` is the stand-in's vault
#   (`rusty`, REQ-002, REQ-003).
# - `marley.assistant.folder` set to a scratch folder that does not exist yet: a new Marley
#   conversation starts there, the folder made (`moved`, REQ-004).
# - The Settings window's Marley page lists Marley's Folder (`settings`) and Rusty Agent's Folder
#   (`settings-rusty`), REQ-005.
compositor sway

# The Settings window's Marley Agent and Rusty entries in its navigation, after the search for
# "Folder", as the first run's shots found them.
NAV_MARLEY_AGENT_X=${NAV_MARLEY_AGENT_X:-884}
NAV_MARLEY_AGENT_Y=${NAV_MARLEY_AGENT_Y:-211}
NAV_RUSTY_X=${NAV_RUSTY_X:-860}
NAV_RUSTY_Y=${NAV_RUSTY_Y:-239}

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
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY2'
import json, sys
path = sys.argv[1]
settings = json.load(open(path))
marley = settings.setdefault("marley", {})
marley["assistant"] = {"enabled": True, "agent": "claude_code"}
marley["rusty"] = {"enabled": True}
marley["list_cli_sessions"] = False
json.dump(settings, open(path, "w"), indent=2)
PY2
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
# A scripted ACP agent in the Claude adapter's place for #780's e2e test: it answers each prompt
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

# Runs `$1` from the command palette.
palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Whether the agent's newest `session/new` asked for `cwd` `$1`.
started_in() {
  python3 - "$E2E_WORK/agent.log" "$1" <<'PY2'
import json, os, sys
log_path, want = sys.argv[1:]
last = None
for line in open(log_path):
    message = json.loads(line)
    if message.get("method") == "session/new":
        last = (message.get("params") or {}).get("cwd")
print("session/new cwd:", last)
sys.exit(0 if last and os.path.realpath(last) == os.path.realpath(want) else 1)
PY2
}

# Starts a thread of the New Agent picker's entry matching `$1`; with `$2`, a shot of the picker by
# that name first.
new_agent() {
  press "CTRL ALT" n
  settle 2
  type_text "$1"
  settle 1
  if [[ -n ${2:-} ]]; then
    shot "$2"
  fi
  press "" Return
  settle 6
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 5

  echo "== New Agent, Marley"
  new_agent "Marley" 780-00-picker
  shot 780-01-marley
  expect "Marley started in its settings folder" started_in "$E2E_PROFILE/config"

  echo "== New Agent, Rusty"
  new_agent "Rusty"
  shot 780-02-rusty
  expect "Rusty started in its vault" started_in "$E2E_WORK/rusty/vault"

  echo "== Marley's folder set"
  python3 - "$E2E_PROFILE/config/settings.json" "$E2E_WORK/marley-here" <<'PY2'
import json, sys
path, folder = sys.argv[1:]
settings = json.load(open(path))
settings["marley"]["assistant"]["folder"] = folder
json.dump(settings, open(path, "w"), indent=2)
PY2
  settle 4
  palette "marley: new marley conversation"
  settle 6
  shot 780-03-moved
  expect "the new conversation started in the set folder" started_in "$E2E_WORK/marley-here"
  expect "the folder was made" test -d "$E2E_WORK/marley-here"

  echo "== the Settings window's Marley page"
  palette "zed: open settings"
  settle 3
  type_text "Folder"
  settle 2
  click "$NAV_MARLEY_AGENT_X" "$NAV_MARLEY_AGENT_Y"
  settle 2
  shot 780-04-settings
  click "$NAV_RUSTY_X" "$NAV_RUSTY_Y"
  settle 2
  shot 780-05-settings-rusty
}
