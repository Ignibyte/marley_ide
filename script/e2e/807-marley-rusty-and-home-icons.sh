# shellcheck shell=bash
# #807's visual check: Chad's icons for Home, Marley and Rusty. #778's set-up: the Marley entry on
# a scripted agent, Rusty on through its stand-in `rusty-mcp`, the copy's thread list emptied.
# - The title bar's Rusty and Marley buttons draw the butler and the cat, and the status bar's Home
#   the house (`807-01-title-bar`, REQ-001 to REQ-003).
# - `rusty: open home`: Home's tab and its rail row draw the house (`807-02-home`, REQ-001).
# - The New Agent picker: the Marley and Rusty entries draw the cat and the butler
#   (`807-03-picker`, REQ-002, REQ-003).
# - `marley: talk to marley`: the thread's tab and its rail row draw the cat (`807-04-thread`,
#   REQ-002).
# - A light theme: the icons take its colors (`807-05-light`, REQ-004).
compositor sway

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
  # The run's profile is a copy of the user's: its thread list is emptied, so talk-to finds only
  # the run's own conversation.
  local db
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #807's e2e test: it answers each prompt
# with "Noted: <prompt>" and logs every message it reads.
import json
import os
import sys

log_path = sys.argv[1]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


sessions = 0
for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    with open(log_path, "a") as log_file:
        log_file.write(json.dumps(message) + "\n")
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
        send({"jsonrpc": "2.0", "id": request_id,
              "result": {"sessionId": "marley-%d-%d" % (os.getpid(), sessions)}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        text = " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        send({"jsonrpc": "2.0", "method": "session/update", "params": {
            "sessionId": session,
            "update": {"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": "Noted: " + text}},
        }})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

# Runs `$1` from the command palette, the name, a pause and Return in one `wtype`.
palette() {
  press "CTRL SHIFT" p
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "$1" -s 800 -k Return
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 5
  shot 807-01-title-bar

  echo "== Home"
  palette "rusty: open home"
  settle 3
  shot 807-02-home

  echo "== the New Agent picker"
  press "CTRL ALT" n
  settle 2
  shot 807-03-picker
  press "" Escape
  settle 1

  echo "== a Marley thread"
  palette "marley: talk to marley"
  settle 6
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "hello" -s 300 -k Return
  settle 4
  shot 807-04-thread

  echo "== a light theme"
  palette "theme selector: toggle"
  settle 2
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "One Light" -s 800 -k Return
  settle 3
  shot 807-05-light
}
