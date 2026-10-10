# shellcheck shell=bash
# #734's visual check: an agent thread starts in a center tab, in any group, on a folder Marley
# names. The Marley entry runs a scripted agent (`MARLEY_ASSISTANT_ADAPTER`) that answers each
# prompt "Noted: <prompt>", answers `read <path>` by asking Marley for the file with
# `fs/read_text_file`, and logs every message, `session/new`'s `cwd` among them.
# - The project's `+` → New Agent Thread → Marley: a tab in the project, its session in the
#   project's root, and `read <repo>/README.md` answered from the file (`project-thread`, REQ-003,
#   REQ-004, REQ-005).
# - A key bound to `marley::NewAgentThread` with the Marley agent and `other/`, a folder outside
#   the project: a tab there, and `read <other>/notes.md` answered, through the hidden worktree
#   (`folder`, REQ-007).
# - Home's `+` → New Agent Thread → Marley: a tab in Home, its row under Home, its session in
#   Marley's home folder (`home-thread`, REQ-001, REQ-002, REQ-004).
compositor sway

# The rail's two `+` buttons and how many steps down New Agent Thread's submenu Marley sits, as
# the first run's shots found them.
HOME_PLUS_X=${HOME_PLUS_X:-236}
HOME_PLUS_Y=${HOME_PLUS_Y:-89}
PROJECT_PLUS_X=${PROJECT_PLUS_X:-236}
PROJECT_PLUS_Y=${PROJECT_PLUS_Y:-136}
MARLEY_STEPS=${MARLEY_STEPS:-2}
HOME_MARLEY_STEPS=${HOME_MARLEY_STEPS:-3}
# `STOP_AT_LAYOUT=1` ends the run at the first shot, to find the buttons.
STOP_AT_LAYOUT=${STOP_AT_LAYOUT:-0}

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
  # The Marley entry on Claude Code by name, so no run looks for the user's agents.
  profile_setting marley.assistant.enabled true
  profile_setting marley.assistant.agent '"claude_code"'
  # The key that starts a thread on `other/`, a folder the project does not hold.
  printf '[{"bindings": {"ctrl-alt-shift-y": ["marley::NewAgentThread", {"agent": "Marley", "folder": "%s"}]}}]\n' \
    "$E2E_WORK/other" >"$E2E_PROFILE/config/keymap.json"
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #734's e2e test: it answers each prompt
# with "Noted: <prompt>", answers "read <path>" with the file's first line as Marley's
# fs/read_text_file returns it, and logs every message it reads.
import json
import sys

log_path = sys.argv[1]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def log(message):
    with open(log_path, "a") as log_file:
        log_file.write(json.dumps(message) + "\n")


def say(session, text):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {
        "sessionId": session,
        "update": {"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": text}},
    }})


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
            "agentCapabilities": {"loadSession": False, "promptCapabilities": {}},
            "authMethods": [],
            "agentInfo": {"name": "scripted", "version": "0"},
        }})
    elif method == "session/new":
        sessions += 1
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "marley-%d" % sessions}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        text = " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        if text.startswith("read "):
            say(session, read_file(session, text[len("read "):].strip()))
        else:
            say(session, "Noted: " + text)
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

# Opens the `+` menu at ($1, $2) and picks the entry $3 steps down New Agent Thread's submenu,
# Marley's place there; with $4, shoots the submenu first.
new_marley_thread() {
  local step
  click "$1" "$2"
  settle 1
  # A menu opens with nothing chosen (Zed #64365); Home chooses New Terminal.
  press "" Home
  press "" Down
  press "" Down
  press "" Right
  settle 1
  for ((step = 0; step < $3; step++)); do
    press "" Down
  done
  settle 1
  if [[ -n ${4:-} ]]; then
    shot "$4"
  fi
  press "" Return
  settle 6
}

# Whether the scripted agent has started `$1` sessions: the thread just opened runs it, not one of
# the user's own agents (PR-claude-687), so nothing is typed into a real one.
sessions_started() {
  [[ $(grep -c '"method": "session/new"' "$E2E_WORK/agent.log") -ge $1 ]]
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
  shot 734-00-layout
  if [[ $STOP_AT_LAYOUT == 1 ]]; then
    return 0
  fi

  echo "== the project's + → New Agent Thread → Marley"
  new_marley_thread "$PROJECT_PLUS_X" "$PROJECT_PLUS_Y" "$MARLEY_STEPS"
  expect "the project's thread runs the scripted agent" sessions_started 1
  expect "the project's thread runs in the project's root" holds "$E2E_WORK/agent.log" "\"cwd\": \"$E2E_WORK/repo\""
  ask "read $E2E_WORK/repo/README.md"
  shot 734-03-project-thread
  expect "the agent read the project's file" holds "$E2E_WORK/agent.log" '"content": "# repo'

  echo "== the bound key: a thread on other/"
  press "CTRL ALT SHIFT" y
  settle 6
  expect "the bound thread runs the scripted agent" sessions_started 2
  expect "the bound thread runs in other/" holds "$E2E_WORK/agent.log" "\"cwd\": \"$E2E_WORK/other\""
  ask "read $E2E_WORK/other/notes.md"
  shot 734-02-folder
  expect "the agent read other/'s file" holds "$E2E_WORK/agent.log" '"content": "# notes from other'

  echo "== Home's + → New Agent Thread → Marley"
  new_marley_thread "$HOME_PLUS_X" "$HOME_PLUS_Y" "$HOME_MARLEY_STEPS" 734-01a-home-menu
  expect "Home's thread runs the scripted agent" sessions_started 3
  expect "Home's thread runs in Marley's home folder" holds "$E2E_WORK/agent.log" "\"cwd\": \"$HOME\""
  ask "hello"
  shot 734-01-home-thread
}
