# shellcheck shell=bash
# #798's visual check: an agent thread's width button. #744's set-up: Marley's own agent on a
# scripted ACP agent (MARLEY_ASSISTANT_ADAPTER), started from the repo row's +, New Agent…,
# Marley, so a thread opens in a tab with no model.
# - The terminal's pane split right (Home opens in the new pane, #785) and the thread started there:
#   the button shows the thread pane's width in percent (`798-01-tab`, REQ-001).
# - A click steps the pane to 66% (`798-02-sixtysix`, REQ-002); `marley: cycle agent panel width`
#   then zooms it over the window (`798-02-zoomed`, REQ-003, REQ-006) and brings it back at 33%
#   (`798-03-back`, REQ-003). The pane's share is of its row.
# - The other panes and tabs closed, the thread's tab alone in its row: no button (`798-04-lone`,
#   REQ-005).
# - Zed's layout, the thread moved to the Agent Panel: the button shows the dock's share of the window
#   (`798-05-panel`, 50%, Zed's flexible default, REQ-001); the palette's action steps it to 67%,
#   zooms it and brings it back at 33% (`798-06-*`, REQ-002, REQ-003).
# - Quit and started again on the same profile: the panel comes back open at 33%, the width the
#   last step set (`798-07-relaunch`, REQ-004).
compositor sway

# The repo row's + and the thread pane's width button, as the first run's shots found them.
REPO_PLUS_X=${REPO_PLUS_X:-236}
REPO_PLUS_Y=${REPO_PLUS_Y:-136}
PANE_WIDTH_X=${PANE_WIDTH_X:-1002}
PANE_WIDTH_Y=${PANE_WIDTH_Y:-950}

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
  profile_setting use_system_path_prompts false
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #798's e2e test: it answers each prompt
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

# Runs `$1` from the command palette. One `wtype` types the name, waits for the list and sends
# Return: a second `wtype` brings its own keymap, and the key after the switch can be lost.
palette() {
  press "CTRL SHIFT" p
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "$1" -s 800 -k Return
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== the terminal's pane split right: Home in the new pane (#785)"
  palette "pane: split right"
  settle 3
  shot 798-00-split

  echo "== a Marley thread from the repo row's +, in the new pane beside the terminal"
  click "$REPO_PLUS_X" "$REPO_PLUS_Y"
  settle 1
  press "" Home
  press "" Down
  press "" Down
  press "" Return
  settle 2
  type_text "Marley"
  settle 1
  press "" Return
  settle 2
  press "" Return
  settle 6
  expect "the thread runs the scripted agent" holds "$E2E_WORK/agent.log" '"method": "session/new"'
  # The debug build draws the connected thread a few seconds after the agent answers.
  settle 8
  shot 798-01-tab

  echo "== the pane's steps: a click, then the palette's action"
  click "$PANE_WIDTH_X" "$PANE_WIDTH_Y"
  settle 2
  shot 798-02-sixtysix
  palette "marley: cycle agent panel width"
  settle 2
  shot 798-02-zoomed
  palette "marley: cycle agent panel width"
  settle 2
  shot 798-03-back

  echo "== the thread's tab alone in its row"
  palette "workspace: close inactive tabs and panes"
  settle 3
  shot 798-04-lone

  echo "== the thread in the Agent Panel, stepped by the palette's action"
  # The Marley layout keeps threads in tabs and hides Move Thread to Panel; Zed's layout has both.
  palette "marley: use zed layout"
  settle 3
  palette "marley: move thread to panel"
  settle 3
  shot 798-05-panel
  # Zed's flexible size opens the dock at half the room beside the center, so the steps go 67%,
  # the whole window, then 33%.
  palette "marley: cycle agent panel width"
  settle 2
  shot 798-06-sixtyseven
  palette "marley: cycle agent panel width"
  settle 2
  shot 798-06-zoomed
  palette "marley: cycle agent panel width"
  settle 2
  shot 798-06-third

  echo "== Marley started again on the same profile: the panel comes back open, at 33%"
  quit_marley
  settle 3
  launch_marley
  settle 12
  shot 798-07-relaunch
}
