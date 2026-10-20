# shellcheck shell=bash
# #823's visual check: under the window order a group's terminal, Browser tab and thread rows are
# one run, so any of them drags to any place among them. `repo` gets a terminal (its first), a
# Browser tab and a Marley thread on #798's scripted ACP agent, all from the repo row's +.
# - The rows as they come: the terminal, the Browser tab, the thread (`823-01-sections`, REQ-003).
# - The thread's row dragged over the terminal's: the drop line above it (`823-02-line`), and
#   dropped, the thread first (`823-03-thread-first`, REQ-001).
# - The Browser tab's row dropped on the terminal's from below: the Browser tab between the thread
#   and the terminal (`823-04-browser-between`, REQ-001).
# - Quit and started again with no path: the same order (`823-05-restart`, REQ-002).
compositor sway

# The repo row's + and its menu's entries, as #822's run found them; the rows as this one's first
# run found them.
ROW_X=${ROW_X:-110}
REPO_PLUS_X=${REPO_PLUS_X:-236}
REPO_PLUS_Y=${REPO_PLUS_Y:-136}
NEW_BROWSER_X=${NEW_BROWSER_X:-105}
NEW_BROWSER_Y=${NEW_BROWSER_Y:-210}
NEW_AGENT_X=${NEW_AGENT_X:-105}
NEW_AGENT_Y=${NEW_AGENT_Y:-233}
# At the start: the terminal, the Browser tab and the thread, top to bottom.
TERMINAL_Y=${TERMINAL_Y:-175}
BROWSER_Y=${BROWSER_Y:-233}
THREAD_Y=${THREAD_Y:-291}
# Once the thread is first: the thread, the terminal, the Browser tab.
MOVED_TERMINAL_Y=${MOVED_TERMINAL_Y:-233}
MOVED_BROWSER_Y=${MOVED_BROWSER_Y:-291}

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
  profile_setting marley.rail_order '"window"'
  profile_setting use_system_path_prompts false
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #823's e2e test: it answers each prompt
# with "Noted: <prompt>" and logs every message it reads.
import json
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
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "marley-%d" % sessions}})
    elif method == "session/prompt":
        text = " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        send({"jsonrpc": "2.0", "method": "session/update", "params": {
            "sessionId": params.get("sessionId"),
            "update": {"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": "Noted: " + text}},
        }})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

# Presses on the row at y `$1` and moves to y `$2` in steps, the button still down, as a hand does.
drag_over() {
  local from=$1 to=$2 step
  pointer_to "$ROW_X" "$from"
  settle 0.3
  pointer_down
  settle 0.3
  for step in 1 2 3 4 5 6; do
    pointer_to "$ROW_X" "$((from + (to - from) * step / 6))"
    sleep 0.1
  done
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== a Browser tab and a Marley thread from the repo row's +"
  click "$REPO_PLUS_X" "$REPO_PLUS_Y"
  settle 1
  click "$NEW_BROWSER_X" "$NEW_BROWSER_Y"
  settle 4
  click "$REPO_PLUS_X" "$REPO_PLUS_Y"
  settle 1
  click "$NEW_AGENT_X" "$NEW_AGENT_Y"
  settle 2
  type_text "Marley"
  settle 1
  press "" Return
  settle 2
  press "" Return
  settle 6
  expect "the thread runs the scripted agent" holds "$E2E_WORK/agent.log" '"method": "session/new"'
  settle 8
  shot 823-01-sections

  echo "== the thread over the terminal, then dropped"
  drag_over "$THREAD_Y" "$TERMINAL_Y"
  shot 823-02-line
  pointer_up
  settle 3
  shot 823-03-thread-first

  echo "== the Browser tab onto the terminal from below"
  drag_over "$MOVED_BROWSER_Y" "$MOVED_TERMINAL_Y"
  pointer_up
  settle 3
  shot 823-04-browser-between

  echo "== a restart"
  quit_marley
  # No path, so Marley restores the window it had (L-claude-818).
  open_path ""
  launch_marley
  settle 15
  shot 823-05-restart
}
