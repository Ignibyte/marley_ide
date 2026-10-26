# shellcheck shell=bash
# #934's visual check: Agent Activity lists calls to Rusty's tools, which Marley does not carry.
# Marley's entry runs a scripted ACP agent (`MARLEY_ASSISTANT_ADAPTER`); on a prompt it reports
# three of Claude Code's tool calls the way claude-agent-acp 0.94.0 does (`_meta.claudeCode.toolName`),
# each first in progress: `mcp__rusty__brain_decide` completed with "page written",
# `mcp__rusty__task_update` failed with "no task 42", and `mcp__rusty__brain_search` completed.
# A stand-in `claude`, first on the terminals' PATH through the scenario's `.bashrc`, runs the
# plugin's real `hooks/event.py` on a `PostToolUse` of `mcp__rusty__memory_add` at Enter and writes
# its sequence. No agent CLI runs; Rusty stays off.
# - `934-01-thread`: Agent Activity after the thread's turn: `rusty · brain_decide` with "Ship the
#   home page" and "page written", `rusty · task_update` with "no task 42" in red, no
#   `brain_search` (REQ-001, REQ-003, REQ-004).
# - `934-02-terminal`: after the terminal's stand-in: a CC row `rusty · memory_add` with "memory
#   saved" (REQ-002, REQ-005).
# - `934-03-day`: Agent Activity in Marley Day (REQ-001, REQ-002).
compositor sway

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
STOP_AT=${STOP_AT:-}
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-700}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin db
  mkdir -p "$home" "$bin"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s\n' "$E2E_WORK/scripted-agent.py" >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  write_stand_in "$bin/claude"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent CLAUDE_CONFIG_DIR=$E2E_WORK/claude-config
  profile_setting marley.assistant '{"enabled": true, "agent": "claude_code"}'
  profile_setting marley.tmux_hosts '[]'
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

write_stand_in() {
  sed -e "s|@HOOK@|$HOOK|" >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code: at Enter, the plugin's hook on a finished call to Rusty's memory_add.
import json
import os
import subprocess
import sys

EVENT = {"hook_event_name": "PostToolUse", "session_id": "e2e-rusty", "cwd": os.getcwd(),
         "tool_name": "mcp__rusty__memory_add", "tool_use_id": "t1",
         "tool_input": {"content": "The home page ships on Friday"},
         "tool_response": [{"type": "text", "text": "memory saved\nid 17"}]}
print("Claude Code (stand-in): press Enter", flush=True)
if sys.stdin.readline():
    answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps(EVENT),
                            capture_output=True, text=True, check=False).stdout
    sequence = json.loads(answer or "{}").get("terminalSequence")
    if sequence:
        sys.stdout.write(sequence)
        sys.stdout.flush()
    print("called memory_add", flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$1"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent for #934's check: on a prompt, three of Claude Code's calls to Rusty's
# tools, each in progress first and then finished, named as claude-agent-acp names them.
import json
import os
import sys
import time


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def update(session, body):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session, "update": body}})


def call(session, ident, tool, raw, status, text):
    meta = {"claudeCode": {"toolName": "mcp__rusty__" + tool}}
    update(session, {"sessionUpdate": "tool_call", "toolCallId": ident, "title": tool, "kind": "other",
                     "status": "in_progress", "rawInput": raw, "_meta": meta})
    time.sleep(0.3)
    update(session, {"sessionUpdate": "tool_call_update", "toolCallId": ident, "status": status,
                     "content": [{"type": "content", "content": {"type": "text", "text": text}}],
                     "_meta": meta})


for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "protocolVersion": 1,
            "agentCapabilities": {"loadSession": False, "promptCapabilities": {}},
            "authMethods": [], "agentInfo": {"name": "scripted", "version": "0"}}})
    elif method == "session/new":
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "rusty-%d" % os.getpid()}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        call(session, "decide-1", "brain_decide", {"title": "Ship the home page"}, "completed",
             "page written\ndecisions/ship-the-home-page")
        call(session, "update-1", "task_update", {"id": 42, "title": "Write the release notes"}, "failed",
             "no task 42")
        call(session, "search-1", "brain_search", {"query": "home page"}, "completed", "3 pages")
        update(session, {"sessionUpdate": "agent_message_chunk",
                         "content": {"type": "text", "text": "Recorded the decision."}})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 15
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== a thread calls Rusty's tools"
  palette "marley: new marley conversation" 10
  type_text "record it"
  settle 3
  press "" Return
  settle 6
  palette "marley: open agent activity" 3
  away
  take 934-01-thread || return 0

  echo "== Claude Code in a terminal calls Rusty's tools"
  palette "workspace: new terminal" 4
  type_text "claude"
  press "" Return
  settle 3
  press "" Return
  settle 3
  tail -3 "$E2E_PROFILE/agent_control/activity-$(date +%F).jsonl"
  palette "marley: open agent activity" 3
  away
  take 934-02-terminal || return 0

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  away
  shot 934-03-day
}
