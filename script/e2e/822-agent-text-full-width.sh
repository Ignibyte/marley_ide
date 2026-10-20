# shellcheck shell=bash
# #822's visual check: an agent thread's text runs the full width by default, and a toggle in the
# thread's bottom row narrows it to Zed's readable column and back. #798's set-up: Marley's own
# agent on a scripted ACP agent (MARLEY_ASSISTANT_ADAPTER), started from the repo row's +, New
# Agent..., Marley, so a thread opens in a tab alone in its row, where #798's width button hides.
# - A long prompt and its answer, with `agent.limit_content_width` taken out of the run's settings:
#   the text and the message box run the tab's full width, and the toggle shows (`822-01-full`,
#   REQ-001, REQ-002).
# - A click on the toggle: the run's settings say `limit_content_width: true` and the text sits in
#   the centered column (`822-02-column`, REQ-003).
# - A second click: `false`, and the full width again (`822-03-full-again`, REQ-003).
compositor sway

# The repo row's +, its New Agent... and the text-width toggle, as the first run's shots found them.
REPO_PLUS_X=${REPO_PLUS_X:-236}
REPO_PLUS_Y=${REPO_PLUS_Y:-136}
# New Agent... in the + menu, its fourth entry since #819 put New tmux Terminal second.
NEW_AGENT_X=${NEW_AGENT_X:-105}
NEW_AGENT_Y=${NEW_AGENT_Y:-233}
TEXT_WIDTH_X=${TEXT_WIDTH_X:-327}
TEXT_WIDTH_Y=${TEXT_WIDTH_Y:-950}
# In the column the bottom row moves in with the text, and the toggle with it.
COLUMN_TEXT_WIDTH_X=${COLUMN_TEXT_WIDTH_X:-572}

LONG_PROMPT="Every word of this prompt comes back in the answer, so the answer is long enough to show how wide the thread draws its text: a column of eight hundred and fifty pixels in the middle, or the whole width of the tab from the rail to the window's edge."

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
  # The user's own choice comes with the copied settings; the run shows Marley's default.
  python3 - "$E2E_PROFILE/config/settings.json" <<'UNSET'
import json, pathlib, sys

path = pathlib.Path(sys.argv[1])
settings = json.loads(path.read_text())
settings.get("agent", {}).pop("limit_content_width", None)
path.write_text(json.dumps(settings, indent=2))
UNSET
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #822's e2e test: it answers each prompt
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

# Runs `$1` from the command palette, as #798's scenario does.
palette() {
  press "CTRL SHIFT" p
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "$1" -s 800 -k Return
}

# Whether the run's settings set `agent.limit_content_width` to `$1`.
content_width_is() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'READ'
import json, re, sys

text = open(sys.argv[1]).read()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*', lambda m: m.group(1) or "", text)
text = re.sub(r",(\s*[}\]])", r"\1", text)
value = json.loads(text).get("agent", {}).get("limit_content_width")
sys.exit(0 if json.dumps(value) == sys.argv[2] else 1)
READ
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== a Marley thread from the repo row's +, alone in its row"
  click "$REPO_PLUS_X" "$REPO_PLUS_Y"
  settle 1
  shot 822-00-plus
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
  type_text "$LONG_PROMPT"
  # The debug build's message box takes the typed keys well after wtype sends them.
  settle 12
  press "" Return
  settle 6
  expect "the setting is unset" content_width_is null
  shot 822-01-full

  echo "== the toggle: the readable column"
  click "$TEXT_WIDTH_X" "$TEXT_WIDTH_Y"
  settle 3
  expect "the toggle wrote true" content_width_is true
  shot 822-02-column

  echo "== the toggle again: the full width"
  click "$COLUMN_TEXT_WIDTH_X" "$TEXT_WIDTH_Y"
  settle 3
  expect "the toggle wrote false" content_width_is false
  shot 822-03-full-again
}
