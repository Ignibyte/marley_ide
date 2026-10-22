# shellcheck shell=bash
# #865's visual check: an agent beside a terminal, Apply, and a running block's progress line.
# #825's set-up (Marley's own agent on a scripted ACP agent), which answers with a fix and `echo
# fixed` in a fenced block.
# - `sleep 4` running: its header's moving line (`865-01-running`, REQ-003).
# - `false`, then the failed block's Ask Marley and Return: the thread in the pane right of the
#   terminal, the block in its prompt, the answer and "Apply: echo fixed" (`865-02-beside`,
#   REQ-001, REQ-002).
# - Apply clicked: the terminal in front, `echo fixed` run and `fixed` printed (`865-03-applied`,
#   REQ-002).
# - Marley Day, the thread beside the terminal (`865-04-day`, REQ-001).
compositor sway

STOP_AT=${STOP_AT:-}
# The failed block's Ask Marley, Apply, and the thread's tab, from the first run's shots.
ASK_X=${ASK_X:-1492}
ASK_Y=${ASK_Y:-846}
APPLY_X=${APPLY_X:-1212}
APPLY_Y=${APPLY_Y:-467}
THREAD_X=${THREAD_X:-1250}
THREAD_Y=${THREAD_Y:-50}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

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
# A scripted ACP agent for #865's e2e test: it answers each prompt with a fix and the command to
# run in a fenced block. It logs every message.
import json
import sys

log_path = sys.argv[1]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def log(message):
    with open(log_path, "a") as log_file:
        log_file.write(json.dumps(message) + "\n")


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
            "authMethods": [], "agentInfo": {"name": "scripted", "version": "0"}}})
    elif method == "session/new":
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "marley-1"}})
    elif method == "session/prompt":
        answer = "`false` always fails. Run this instead:\n\n```sh\necho fixed\n```\n"
        send({"jsonrpc": "2.0", "method": "session/update", "params": {
            "sessionId": params.get("sessionId"),
            "update": {"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": answer}}}})
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

# Clicks `$1`,`$2` when the first run placed it.
click_placed() {
  if (($1 > 0)); then
    click "$1" "$2"
    settle "${3:-3}"
  fi
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== a running block"
  type_text "sleep 4"
  press "" Return
  settle 1.5
  pointer_to 900 300
  take 865-01-running || return 0
  settle 4

  echo "== a failed block, Ask Marley"
  type_text "false"
  press "" Return
  settle 3
  pointer_to 900 300
  settle 1
  take 865-01b-failed || return 0
  click_placed "$ASK_X" "$ASK_Y" 8
  # The block is in the message box; Return sends it.
  press "" Return
  settle 8
  pointer_to 900 300
  settle 1
  take 865-02-beside || return 0

  echo "== Apply"
  click_placed "$APPLY_X" "$APPLY_Y" 4
  pointer_to 900 300
  settle 1
  take 865-03-applied || return 0

  echo "== Marley Day, the thread"
  profile_setting theme '"Marley Day"'
  settle 3
  click_placed "$THREAD_X" "$THREAD_Y" 2
  pointer_to 900 300
  settle 1
  take 865-04-day || return 0
}
