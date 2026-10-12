# shellcheck shell=bash
# #717's visual check: New Thread with Context. Two scripted ACP agents are custom agent servers:
# `Scripted A` answers each prompt, one answer carrying a GitHub-shaped token (built at run time,
# so this file holds none), one turn editing `notes.md` through a tool call, and "big" prompts
# answered with about 6 KB each; `Scripted B` answers each prompt with its size and its first and
# last lines. A key opens a thread tab on `Scripted A` in the scratch project.
# Shots: the tab's menu (REQ-001); the picker, A first (REQ-001); the new tab on B in a split on
# the right, its message box holding the history unsent, the token redacted, A's tab unchanged
# (REQ-002, REQ-003, REQ-005, REQ-007); B's reply naming the preamble and the person's words
# (REQ-006); a second handoff after big answers, its last line saying how many items were left out
# and naming the file, which holds the whole conversation with no token (REQ-004, REQ-005).
compositor sway

# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
A_TAB_X=${A_TAB_X:-553}
A_TAB_Y=${A_TAB_Y:-51}
MENU_ITEM_X=${MENU_ITEM_X:-649}
MENU_ITEM_Y=${MENU_ITEM_Y:-317}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

write_agent() {
  cat >"$E2E_WORK/agent.py" <<'PY'
# A scripted ACP agent for #717's e2e test. `A` answers by the prompt: "token" gets a key-shaped
# string, "edit" a tool call that edits notes.md, "big" about 6 KB, anything else a short answer.
# `B` answers with the prompt's size and its first and last lines.
import json
import os
import sys

log_path, mode = sys.argv[1], sys.argv[2]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def say(session, text):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session, "update": {
        "sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": text}}}})


sessions = 0
cwd = "/"
for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    with open(log_path, "a") as log_file:
        log_file.write(mode + " " + json.dumps(message) + "\n")
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "protocolVersion": 1, "agentCapabilities": {"loadSession": True, "promptCapabilities": {}},
            "authMethods": [], "agentInfo": {"name": "scripted-" + mode, "version": "0"}}})
    elif method == "session/new":
        sessions += 1
        cwd = params.get("cwd", "/")
        send({"jsonrpc": "2.0", "id": request_id,
              "result": {"sessionId": "scripted-%s-%d-%d" % (mode, os.getpid(), sessions)}})
    elif method == "session/load":
        send({"jsonrpc": "2.0", "id": request_id, "result": {}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        text = "".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        if mode == "B":
            lines = [line for line in text.splitlines() if line.strip()]
            say(session, "Got %d bytes. First line: %s Last line: %s" % (
                len(text.encode()), lines[0][:60] if lines else "", lines[-1] if lines else ""))
        elif "token" in text:
            say(session, "Here is the key: " + "gh" + "p_" + "abcdefghij" * 4)
        elif "edit" in text:
            send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session, "update": {
                "sessionUpdate": "tool_call", "toolCallId": "edit-1", "title": "Edit notes.md",
                "kind": "edit", "status": "completed", "locations": [{"path": cwd + "/notes.md"}]}}})
            say(session, "I changed notes.md.")
        elif "big" in text:
            say(session, "A long answer to " + text.strip() + ".\n" + ("x" * 79 + "\n") * 75)
        else:
            say(session, "An answer to " + text.strip() + ".")
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

setup() {
  mkdir -p "$E2E_WORK/repo"
  printf '# notes\n' >"$E2E_WORK/repo/notes.md"
  write_agent
  : >"$E2E_WORK/agent.log"
  profile_setting agent_servers.Scripted\ A "{\"type\": \"custom\", \"command\": \"python3\", \"args\": [\"$E2E_WORK/agent.py\", \"$E2E_WORK/agent.log\", \"A\"]}"
  profile_setting agent_servers.Scripted\ B "{\"type\": \"custom\", \"command\": \"python3\", \"args\": [\"$E2E_WORK/agent.py\", \"$E2E_WORK/agent.log\", \"B\"]}"
  # A key that opens a thread tab on Scripted A in the shown project.
  printf '%s\n' '[{"bindings": {"ctrl-alt-shift-z": ["marley::NewAgentThread", {"agent": "Scripted A"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  open_path "$E2E_WORK/repo"
}

say_to_agent() {
  type_text "$1"
  press "" Return
  settle 3
}

# The handoff files, newest last.
handoffs() { ls -1tr "$E2E_PROFILE"/handoffs/*.md 2>/dev/null || true; }
no_token_in_handoffs() { ! grep -q "ghp_abcdefghij" "$E2E_PROFILE"/handoffs/*.md; }
latest_holds() { grep -qF -- "$1" "$(handoffs | tail -n 1)"; }

# Right-clicks Scripted A's tab and chooses New Thread with Context, then Scripted B.
handoff_to_b() {
  click "$A_TAB_X" "$A_TAB_Y" right
  settle 1
  take "$1" || return 1
  # The item is the menu's last, under Zed's own Close and Pin items.
  click "$MENU_ITEM_X" "$MENU_ITEM_Y"
  settle 2
  take "$2" || return 1
  type_text "Scripted B"
  settle 1
  press "" Return
  settle 6
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== a conversation with Scripted A"
  press "CTRL ALT SHIFT" z
  settle 6
  say_to_agent "the first request"
  say_to_agent "show me the token"
  say_to_agent "please edit the notes"
  say_to_agent "the last request"
  if ((A_TAB_X == 0)); then
    take 717-00-a || return 0
    echo "Scripted A's tab is not placed yet: 717-00 is the last shot"
    return 0
  fi

  echo "== New Thread with Context, to Scripted B"
  handoff_to_b 717-01-menu 717-02-picker || return 0
  take 717-03-prefilled || return 0
  handoffs
  expect "a file holds the conversation" test -n "$(handoffs)"
  expect "no token in the file" no_token_in_handoffs
  expect "the file holds the last request" latest_holds "the last request"

  echo "== sent to Scripted B"
  type_text " carry on from here"
  press "" Return
  settle 4
  take 717-04-sent || return 0
  expect "B received the preamble and the person's words" \
    grep -q '^B .*Continuing .*carry on from here' "$E2E_WORK/agent.log"

  echo "== a second handoff after big answers"
  click "$A_TAB_X" "$A_TAB_Y"
  settle 2
  say_to_agent "big one"
  say_to_agent "big two"
  say_to_agent "big three"
  handoff_to_b 717-05a-menu 717-05b-picker || return 0
  press "CTRL" End
  settle 1
  take 717-05-budget || return 0
  expect "the newest file holds the third big answer" latest_holds "big three"
  expect "still no token in any file" no_token_in_handoffs
}
