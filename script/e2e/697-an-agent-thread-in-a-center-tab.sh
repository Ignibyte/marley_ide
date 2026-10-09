# shellcheck shell=bash
# #697's visual check: an Agent Panel thread in a center tab and back. The Marley entry runs a
# scripted agent (`MARLEY_ASSISTANT_ADAPTER`) that answers each prompt with "Noted: <prompt>". A
# Marley thread, once it runs the scripted agent (PR-687), gets "hello". Then
# `marley: open thread in center` shows it in a center tab, and the panel turns to a new draft
# (`in-center`, REQ-001). "again", typed in the tab, reaches the agent and is answered there
# (`typed`, REQ-002). `marley: move thread to panel` shows it in the panel again and closes the tab
# (`back`, REQ-003). The rail row's Open in Center moves it once more (`from-rail`, REQ-004).
compositor sway

# The project's +, how many steps down New Agent Thread's submenu Marley sits, and the thread's
# row in the rail, as the first run's shots found them.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-92}
MARLEY_STEPS=${MARLEY_STEPS:-2}
THREAD_ROW_X=${THREAD_ROW_X:-130}
THREAD_ROW_Y=${THREAD_ROW_Y:-175}

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
  # The Marley entry on Claude Code by name, so no run looks for the user's agents.
  profile_setting marley.assistant.enabled true
  profile_setting marley.assistant.agent '"claude_code"'
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #697's e2e test: it answers each prompt
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
  local step
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== a Marley thread on the scripted agent"
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  # A menu opens with nothing chosen (Zed #64365); Home chooses New Terminal.
  press "" Home
  press "" Down
  press "" Down
  press "" Right
  settle 1
  for ((step = 0; step < MARLEY_STEPS; step++)); do
    press "" Down
  done
  settle 1
  press "" Return
  settle 6
  expect "the thread runs the scripted agent" holds "$E2E_WORK/agent.log" '"method": "session/new"'
  type_text "hello"
  press "" Return
  settle 4

  echo "== open thread in center"
  palette "marley: open thread in center"
  shot 697-01-in-center

  echo "== a message typed in the tab"
  type_text "again"
  press "" Return
  settle 4
  shot 697-02-typed
  expect "the tab's message reached the agent" holds "$E2E_WORK/agent.log" '"text": "again"'

  echo "== move thread to panel"
  palette "marley: move thread to panel"
  shot 697-03-back

  echo "== the rail row's Open in Center"
  click "$THREAD_ROW_X" "$THREAD_ROW_Y" right
  settle 1
  press "" Home
  press "" Return
  settle 3
  shot 697-04-from-rail
}
