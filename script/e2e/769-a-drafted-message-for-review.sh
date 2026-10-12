# shellcheck shell=bash
# #769's visual check: an agent drafts a message to another agent, and the person sends or drops
# it. A stand-in `claude`, first on the terminal's PATH, prints and logs each line it reads
# (#555's stand-in, without the hook). A scripted ACP agent, `Scripted`, runs an Agent Panel
# thread opened by a key, and logs every prompt. The stand-in MCP client (`planner-agent`):
# - drafts to the terminal: the Draft tab names the sender and the target, and nothing reaches the
#   agent (`769-01-draft`, REQ-001); Send types it at the agent's prompt with Enter and closes the
#   tab (`769-02-sent`, REQ-002);
# - drafts again and Drop closes the tab with nothing sent (`769-03-dropped`, REQ-003);
# - drafts to the thread, edited before Send, which the thread receives (`769-04-thread`, REQ-002);
# - is refused for an unknown terminal, a terminal with no agent, an unknown thread and an empty
#   text (REQ-004).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The Draft tab's buttons and its text, from the first run's shots; `STOP_AT=<shot>` ends the run
# after that shot.
SEND_X=${SEND_X:-296}
SEND_Y=${SEND_Y:-942}
DROP_X=${DROP_X:-343}
DROP_Y=${DROP_Y:-942}
TEXT_X=${TEXT_X:-900}
TEXT_Y=${TEXT_Y:-162}
STOP_AT=${STOP_AT:-}

export MCP_CLIENT_NAME=planner-agent

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

write_thread_agent() {
  cat >"$E2E_WORK/agent.py" <<'PY'
# A scripted ACP agent for #769's e2e test: it logs each message and answers "Noted: <prompt>".
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
            "protocolVersion": 1, "agentCapabilities": {"loadSession": True, "promptCapabilities": {}},
            "authMethods": [], "agentInfo": {"name": "scripted", "version": "0"}}})
    elif method == "session/new":
        sessions += 1
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "scripted-%d-%d" % (os.getpid(), sessions)}})
    elif method == "session/load":
        send({"jsonrpc": "2.0", "id": request_id, "result": {}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        text = "".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session, "update": {
            "sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": "Noted: " + text.strip()}}}})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  sed -e "s|@LOG@|$E2E_WORK/claude.log|" >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in agent: each line it reads, printed and logged.
import sys

print("claude (stand-in): type a line", flush=True)
for line in sys.stdin:
    line = line.rstrip("\n")
    with open("@LOG@", "a", encoding="utf-8") as log:
        log.write(f"got: {line}\n")
    print(f"got: {line}", flush=True)
FAKE
  chmod +x "$bin/claude"
  : >"$E2E_WORK/claude.log"
  write_thread_agent
  : >"$E2E_WORK/agent.log"
  profile_setting agent_servers.Scripted "{\"type\": \"custom\", \"command\": \"python3\", \"args\": [\"$E2E_WORK/agent.py\", \"$E2E_WORK/agent.log\"]}"
  # A key that opens an Agent Panel thread on Scripted.
  printf '%s\n' '[{"bindings": {"ctrl-alt-shift-z": ["agent::NewExternalAgentThread", {"agent": "Scripted"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  git init -q -b main "$repo"
  open_path "$repo"
}

# The tool's answer as JSON, from `mcp_agent tool`'s `  <tool>: <json>` line; empty when refused.
tool_json() {
  mcp_agent tool "$@" | sed -n "s/^  $1: //p"
}

# Calls `draft_message` with `$2` as its arguments, the whole reply in `$E2E_WORK/$1.txt`.
draft() {
  mcp_agent tool draft_message "$2" >"$E2E_WORK/$1.txt" 2>&1
  cat "$E2E_WORK/$1.txt"
}

replied() { grep -qF -- "$2" "$E2E_WORK/$1.txt"; }

# The id of a terminal: the one whose title names `$1`, or with `--other $1`, any but terminal `$1`.
terminal_id() {
  mcp_agent terminals >&2
  tool_json terminal_list | python3 -c '
import json, sys
terminals = json.load(sys.stdin).get("terminals", [])
if sys.argv[1] == "--other":
    found = (t["id"] for t in terminals if str(t["id"]) != sys.argv[2])
else:
    found = (t["id"] for t in terminals if sys.argv[1] in t["title"])
print(next(found, ""))' "$@"
}

steps() {
  local terminal thread lines
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "claude"
  press "" Return
  settle 3
  terminal=$(terminal_id repo)
  echo "the agent's terminal: $terminal"
  expect "terminal_list names the agent's terminal" test -n "$terminal"

  echo "== a draft to the agent CLI, then Send"
  draft first "{\"to\": {\"terminal\": $terminal}, \"text\": \"Please run the tests and tell me what fails.\"}"
  expect "the tool answered opened" replied first '"opened": true'
  settle 2
  take 769-01-draft || return 0
  expect "nothing reached the agent" test ! -s "$E2E_WORK/claude.log"
  if ((SEND_X == 0)); then
    echo "the Draft tab's buttons are not placed yet: 769-01 is the last shot"
    return 0
  fi
  click "$SEND_X" "$SEND_Y"
  settle 3
  take 769-02-sent || return 0
  cat "$E2E_WORK/claude.log"
  expect "the agent read the draft" holds "$E2E_WORK/claude.log" "got: Please run the tests and tell me what fails."

  echo "== a second draft, dropped"
  lines=$(wc -l <"$E2E_WORK/claude.log")
  draft second "{\"to\": {\"terminal\": $terminal}, \"text\": \"This one is dropped.\"}"
  settle 2
  click "$DROP_X" "$DROP_Y"
  settle 2
  take 769-03-dropped || return 0
  expect "nothing more reached the agent" test "$(wc -l <"$E2E_WORK/claude.log")" = "$lines"

  echo "== a draft to a thread, edited, then Send"
  press "CTRL ALT SHIFT" z
  settle 6
  type_text "hello"
  press "" Return
  settle 3
  thread=$(tool_json thread_list '{}' | python3 -c '
import json, sys
print(next((t["id"] for t in json.load(sys.stdin).get("threads", [])), ""))')
  echo "the thread: $thread"
  expect "thread_list names the thread" test -n "$thread"
  draft third "{\"to\": {\"thread\": \"$thread\"}, \"text\": \"Review the plan in notes.md.\"}"
  settle 2
  click "$TEXT_X" "$TEXT_Y"
  press "CTRL" End
  type_text " Keep it short."
  settle 1
  take 769-04a-thread-draft || return 0
  click "$SEND_X" "$SEND_Y"
  settle 4
  take 769-04-thread || return 0
  expect "the thread received the edited draft" \
    grep -q '"text": "Review the plan in notes.md. Keep it short."' "$E2E_WORK/agent.log"

  echo "== refusals"
  draft unknown-terminal '{"to": {"terminal": 99999}, "text": "hello"}'
  expect "an unknown terminal is refused" replied unknown-terminal "refused"
  palette_command "workspace: new terminal"
  settle 3
  draft no-agent "{\"to\": {\"terminal\": $(terminal_id --other "$terminal")}, \"text\": \"hello\"}"
  expect "a terminal with no agent is refused" replied no-agent "no agent CLI runs in that terminal"
  draft unknown-thread '{"to": {"thread": "nope"}, "text": "hello"}'
  expect "an unknown thread is refused" replied unknown-thread "no window holds a thread nope"
  draft empty "{\"to\": {\"terminal\": $terminal}, \"text\": \"  \"}"
  expect "an empty text is refused" replied empty "give the message's text"
  take 769-05-no-new-tabs || return 0
}

palette_command() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}
