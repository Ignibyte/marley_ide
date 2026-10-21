# shellcheck shell=bash
# #825's visual check: the agent thread with its details panel, bubbles and the composer card.
# #822's set-up: Marley's own agent on a scripted ACP agent (MARLEY_ASSISTANT_ADAPTER), opened with
# `marley: talk to marley`. The agent answers each prompt and, for one, asks Marley to write
# notes.md through ACP's `fs/write_text_file`.
# - Two prompts answered: the user's messages as bubbles on the right, the message box as a
#   rounded card, and the details panel with the agent, its model, the context line and
#   notes.md under FILES CHANGED (`825-01-night`, REQ-001 to REQ-004).
# - The panel's close button: the panel gone, `marley.thread_details` printed (`825-02-hidden`,
#   REQ-005).
# - The tab's show button, then Marley Day (`825-03-day`, REQ-001 to REQ-004).
compositor sway

AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the probe run's shots: the panel's close button and the tab's show button.
HIDE_X=${HIDE_X:-1580}
HIDE_Y=${HIDE_Y:-85}
SHOW_X=${SHOW_X:-1585}
SHOW_Y=${SHOW_Y:-75}

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
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.thread_details true
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #825's e2e test: it answers each prompt
# with "Noted: <prompt>", and for a prompt that says "write notes" first asks the client to write
# notes.md in the session's folder, so the thread has a changed file. It logs every message.
import json
import sys

log_path = sys.argv[1]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def log(message):
    with open(log_path, "a") as log_file:
        log_file.write(json.dumps(message) + "\n")


sessions = 0
folders = {}
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
        session = "marley-%d" % sessions
        folders[session] = params.get("cwd", ".")
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": session}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        text = " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        if "write notes" in text:
            send({"jsonrpc": "2.0", "id": 9001, "method": "fs/write_text_file", "params": {
                "sessionId": session,
                "path": folders.get(session, ".") + "/notes.md",
                "content": "# Notes\n\nThe thread wrote these.\nA second line.\n",
            }})
            for answer in sys.stdin:
                if not answer.strip():
                    continue
                reply = json.loads(answer)
                log(reply)
                if reply.get("id") == 9001 and "method" not in reply:
                    break
        send({"jsonrpc": "2.0", "method": "session/update", "params": {
            "sessionId": session,
            "update": {"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": "Noted: " + text}},
        }})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "$1" -s 800 -k Return
}

# `marley.thread_details` in the run's copy of the settings.
details_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY'
import json, pathlib, re, sys

text = pathlib.Path(sys.argv[1]).read_text()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
print(json.loads(text).get("marley", {}).get("thread_details"))
PY
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== Marley's thread"
  palette "marley: talk to marley"
  settle 8
  expect "the thread runs the scripted agent" holds "$E2E_WORK/agent.log" '"method": "session/new"'
  settle 6
  type_text "Please write notes for this project."
  # The debug build's message box takes the typed keys well after wtype sends them.
  settle 10
  press "" Return
  settle 6
  type_text "Thanks, that reads well."
  settle 10
  press "" Return
  settle 6
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 825-01-night
  echo "notes.md: $(cat "$E2E_WORK/repo/notes.md" 2>&1 | head -1)"

  echo "== the panel hidden"
  click "$HIDE_X" "$HIDE_Y"
  settle 2
  echo "thread_details: $(details_setting)"
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 825-02-hidden

  echo "== shown again, Marley Day"
  click "$SHOW_X" "$SHOW_Y"
  settle 2
  profile_setting theme '"Marley Day"'
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 4
  shot 825-03-day
}
