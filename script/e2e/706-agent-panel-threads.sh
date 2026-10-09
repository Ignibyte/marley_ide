# shellcheck shell=bash
# #706's visual check: an agent works the Agent Panel's threads. The Marley entry runs a scripted
# ACP agent (`MARLEY_ASSISTANT_ADAPTER`): it answers each prompt with "Noted: <prompt>". Told
# "ask permission", it asks for a tool call's permission and reports "Answered: <option>". A
# Marley thread gets "hello". #704's scripted MCP client (`e2e-agent`) then:
# - lists the thread and reads it (REQ-001, REQ-002);
# - posts into it, asked first (`706-01-post-asked`) and sent after Allow (`706-02-posted`,
#   REQ-003);
# - answers the thread's permission, asked every time (`706-03-answer-asked`) and given after
#   Allow (`706-04-answered`, REQ-004).
compositor sway

# The project's +, how many steps down New Agent Thread's submenu Marley sits, and the question's
# Allow for This Session button, as earlier runs' shots found them.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-136}
MARLEY_STEPS=${MARLEY_STEPS:-2}
ALLOW_X=${ALLOW_X:-1225}
ALLOW_Y=${ALLOW_Y:-933}
AWAY_X=${AWAY_X:-700}
AWAY_Y=${AWAY_Y:-500}

# The plugin's bridge, the file Marley's marketplace ships.
BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge

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
  write_client
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #706's e2e test: it answers each prompt
# with "Noted: <prompt>"; told "ask permission", it asks for a tool call's permission first and
# reports the option chosen. It logs every message it reads.
import json
import sys

log_path = sys.argv[1]
PERMISSION_ID = 900


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def say(session, text):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {
        "sessionId": session,
        "update": {"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": text}},
    }})


sessions = 0
waiting = None
for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    with open(log_path, "a") as log_file:
        log_file.write(json.dumps(message) + "\n")
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method is None and request_id == PERMISSION_ID and waiting is not None:
        prompt_id, session = waiting
        waiting = None
        outcome = (message.get("result") or {}).get("outcome") or {}
        chosen = outcome.get("optionId") or outcome.get("outcome") or "none"
        send({"jsonrpc": "2.0", "method": "session/update", "params": {
            "sessionId": session,
            "update": {"sessionUpdate": "tool_call_update", "toolCallId": "write-1", "status": "completed"},
        }})
        say(session, "Answered: " + chosen)
        send({"jsonrpc": "2.0", "id": prompt_id, "result": {"stopReason": "end_turn"}})
    elif method == "initialize":
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
        if "ask permission" in text:
            tool_call = {"toolCallId": "write-1", "title": "Write notes.txt", "kind": "edit", "status": "pending"}
            send({"jsonrpc": "2.0", "method": "session/update", "params": {
                "sessionId": session, "update": dict(tool_call, sessionUpdate="tool_call"),
            }})
            send({"jsonrpc": "2.0", "id": PERMISSION_ID, "method": "session/request_permission", "params": {
                "sessionId": session,
                "toolCall": tool_call,
                "options": [
                    {"optionId": "allow", "name": "Allow", "kind": "allow_once"},
                    {"optionId": "reject", "name": "Reject", "kind": "reject_once"},
                ],
            }})
            waiting = (request_id, session)
        else:
            say(session, "Noted: " + text)
            send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

write_client() {
  cat >"$E2E_WORK/mcp-client.py" <<'PY'
# A scripted MCP client for #706's e2e test: one tool call on the Agent Panel's first thread
# through the plugin's bridge, its outcome printed.
import json
import os
import subprocess
import sys

bridge = subprocess.Popen([os.environ["BRIDGE"]], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
next_id = 0


def call(method, params):
    global next_id
    next_id += 1
    bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "id": next_id, "method": method, "params": params}) + "\n")
    bridge.stdin.flush()
    for line in bridge.stdout:
        message = json.loads(line)
        if message.get("id") == next_id:
            return message
    sys.exit(f"{method}: the bridge closed")


def tool(name, arguments):
    result = call("tools/call", {"name": name, "arguments": arguments}).get("result") or {}
    return result.get("isError", False), result.get("structuredContent") or {}


call("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "e2e-agent", "version": "0"}})
bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
bridge.stdin.flush()
command = sys.argv[1]
error, listed = tool("thread_list", {})
threads = listed.get("threads", [])
if command == "list":
    for thread in threads:
        pending = thread.get("pending") or {}
        print(f"{thread['title']} agent={thread['agent']} project={thread['project']} running={thread['running']} pending={pending.get('tool')} answerable={pending.get('answerable')}")
elif not threads:
    print("no threads")
else:
    thread = threads[0]["id"]
    if command == "read":
        error, answer = tool("thread_read", {"thread": thread})
        print(("refused " + answer.get("code", "")) if error else answer.get("text", ""))
    elif command == "post":
        error, answer = tool("thread_post", {"thread": thread, "message": sys.argv[2]})
        print(("refused " + answer.get("code", "")) if error else f"sent={answer.get('sent')}")
    elif command == "answer":
        error, answer = tool("thread_answer", {"thread": thread, "allow": True})
        print(("refused " + answer.get("code", "")) if error else f"answered {answer.get('tool')} allowed={answer.get('allowed')}")
bridge.stdin.close()
bridge.wait(timeout=10)
PY
}

# The client, run from the harness, its reply in `$E2E_WORK/$1.txt`.
client() {
  BRIDGE=$BRIDGE MARLEY_MCP_ENDPOINT=$E2E_PROFILE/mcp-endpoint.json \
    python3 "$E2E_WORK/mcp-client.py" "${@:2}" >"$E2E_WORK/$1.txt" 2>&1
  cat "$E2E_WORK/$1.txt"
}

# Whether the reply `$1` holds `$2`.
replied() { grep -qF -- "$2" "$E2E_WORK/$1.txt"; }

steps() {
  local step asking
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

  echo "== thread_list and thread_read"
  client listing list
  expect "thread_list names the idle thread" replied listing "running=False pending=None"
  client reading read
  expect "thread_read holds the agent's answer" replied reading "Noted: hello"

  echo "== thread_post asks first, then sends"
  client posting post "posted by e2e-agent" &
  asking=$!
  settle 4
  shot 706-01-post-asked
  click "$ALLOW_X" "$ALLOW_Y"
  wait "$asking"
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  shot 706-02-posted
  expect "the post was sent" replied posting "sent=True"
  expect "the agent got the post" holds "$E2E_WORK/agent.log" '"text": "posted by e2e-agent"'

  echo "== thread_answer asks every time, then answers"
  client asking-agent post "ask permission"
  expect "the second post went without a question" replied asking-agent "sent=True"
  settle 3
  client pending list
  expect "thread_list shows the pending permission" replied pending "pending=Write notes.txt answerable=True"
  client answering answer &
  asking=$!
  settle 4
  shot 706-03-answer-asked
  click "$ALLOW_X" "$ALLOW_Y"
  wait "$asking"
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  shot 706-04-answered
  expect "the answer was given" replied answering "allowed=True"
  expect "the agent got Allow" holds "$E2E_WORK/agent.log" '"optionId": "allow"'
}
