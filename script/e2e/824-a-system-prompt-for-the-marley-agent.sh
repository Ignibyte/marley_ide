# shellcheck shell=bash
# #824's visual check: the Marley agent's system prompt. #798's set-up: Marley's own agent on a
# scripted ACP agent (MARLEY_ASSISTANT_ADAPTER), started from the repo row's +, New Agent...,
# Marley. The scripted agent logs the `_meta` of `session/new`, and the run checks its
# `systemPrompt.append` holds the prompt's four parts and the phrases older scenarios check
# (`824-01-thread`, REQ-001, REQ-002).
compositor sway

# The repo row's + and its New Agent..., as #822's run found them.
REPO_PLUS_X=${REPO_PLUS_X:-236}
REPO_PLUS_Y=${REPO_PLUS_Y:-136}
NEW_AGENT_X=${NEW_AGENT_X:-105}
NEW_AGENT_Y=${NEW_AGENT_Y:-233}

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
# A scripted ACP agent in the Claude adapter's place for #824's e2e test: it answers each prompt
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

# Whether the scripted agent's `session/new` carried the new prompt, printing its headings.
prompt_reached_the_agent() {
  python3 - "$E2E_WORK/agent.log" <<'PY'
import json, sys
for line in open(sys.argv[1]):
    message = json.loads(line)
    if message.get("method") != "session/new":
        continue
    meta = message.get("params", {}).get("_meta") or {}
    append = (meta.get("systemPrompt") or {}).get("append", "")
    headings = [line for line in append.splitlines() if line and not line.startswith("-")]
    print("  the prompt's lines that are not a bullet:")
    for heading in headings:
        print("   ", heading[:100])
    wanted = ["Marley's own agent", "What you do", "How you answer", "What you leave to others",
              "You do not edit files, run commands or change code here", "ask one question"]
    missing = [text for text in wanted if text not in append]
    print("  missing:", missing)
    sys.exit(0 if not missing else 1)
print("  no session/new in the log")
sys.exit(1)
PY
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== a Marley thread from the repo row's +"
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
  settle 6
  type_text "How do I make the agent text full width?"
  settle 10
  press "" Return
  settle 6
  shot 824-01-thread
  expect "its session/new carried the new prompt" prompt_reached_the_agent
}
