# shellcheck shell=bash
# #919's visual check: the Threads page as the board draws it. #836's fixtures: two Claude Code CLI
# sessions in a scratch `CLAUDE_CONFIG_DIR`, one in the repository changed a minute ago, one in
# `sub` four days ago. A stand-in `claude` first on the terminals' PATH (F-claude-903), also
# `MARLEY_CLAUDE`: on `--resume` it sends the plugin's events for that session through
# `event.py`, ending in a permission ask, so the resumed session needs the user. Marley's own
# agent runs a scripted ACP agent that offers the modes Default and Plan and holds its turn, so a
# loaded Zed conversation has a Mode.
# - `919-01-night`: the header's count and New Agent, the search with its magnifier and words,
#   WHO rows with tiles and counts, cards with tiles and state dots (the resumed one pink), title
#   and time on one line, agent and folder pills (REQ-001, REQ-003 to REQ-005).
# - `919-02-preview`: the waiting card: "Needs you · <ask>" and the facts grid (REQ-006, REQ-007).
# - `919-03-preview-thread`: the Marley conversation: its state, Started and Mode (REQ-006,
#   REQ-007).
# - `919-04-new-agent`: New Agent clicked: the New Agent picker (REQ-002).
# - `919-05-day`: Marley Day, a card selected (REQ-001, REQ-003 to REQ-007).
compositor sway

AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-940}
HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks
# From the probe run's shots.
CLI_CARD_X=${CLI_CARD_X:-700}
CLI_CARD_Y=${CLI_CARD_Y:-305}
THREAD_CARD_X=${THREAD_CARD_X:-700}
THREAD_CARD_Y=${THREAD_CARD_Y:-235}
OPEN_X=${OPEN_X:-1241}
OPEN_Y=${OPEN_Y:-400}
NEW_AGENT_X=${NEW_AGENT_X:-1540}
NEW_AGENT_Y=${NEW_AGENT_Y:-113}

CLAUDE_ONE=11111111-9190-4919-8919-111111111111
CLAUDE_TWO=22222222-9190-4919-8919-222222222222

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

# A Claude Code transcript: `$1` the id, `$2` the folder, `$3` the entrypoint, `$4` the prompt,
# then `ai-title <text>` or `custom-title <text>`.
claude_session() {
  local id=$1 folder=$2 entry=$3 prompt=$4 kind=$5 title=$6
  local dir=$E2E_WORK/claude/projects/${folder//\//-}
  mkdir -p "$dir"
  python3 - "$dir/$id.jsonl" "$id" "$folder" "$entry" "$prompt" "$kind" "$title" <<'PY'
import json, sys
path, ident, folder, entry, prompt, kind, title = sys.argv[1:]
base = {"sessionId": ident, "cwd": folder, "entrypoint": entry, "isSidechain": False,
        "timestamp": "2026-10-11T10:00:00.000Z", "version": "2.1.296"}
records = [
    {**base, "type": "user", "message": {"role": "user", "content": prompt}},
    {**base, "type": "assistant", "message": {"role": "assistant",
                                               "content": [{"type": "text", "text": "On it."}]}},
    {**base, "type": "user", "isMeta": True, "message": {"role": "user",
                                                          "content": "Caveat: a meta record"}},
]
if kind == "ai-title":
    records.append({"type": "ai-title", "sessionId": ident, "aiTitle": title})
else:
    records.append({"type": "custom-title", "sessionId": ident, "customTitle": title})
with open(path, "w", encoding="utf-8") as file:
    for record in records:
        file.write(json.dumps(record) + "\n")
PY
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent (#915's), offering the modes Default and Plan: as session modes to
# its first session, as a config option of category mode to the next. "start": an edit with
# a diff, a command run through the client's terminal, then an open "Edit notes.md" call held until
# the release file exists.
import json
import os
import sys
import time

release = sys.argv[1]
next_id = [9000]
folders = {}
sessions = [0]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def request(method, params):
    next_id[0] += 1
    rid = next_id[0]
    send({"jsonrpc": "2.0", "id": rid, "method": method, "params": params})
    for answer in sys.stdin:
        if answer.strip() and json.loads(answer).get("id") == rid:
            return json.loads(answer)
    return {}


def update(session, body):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session, "update": body}})


def option(ident, name, category, current, values):
    return {"id": ident, "name": name, "category": category, "type": "select", "currentValue": current,
            "options": [{"value": value, "name": label} for value, label in values]}


CONFIG = [
    option("mode", "Mode", "mode", "default", [("default", "Ask"), ("plan", "Plan")]),
]

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
        session = "composer-%d-%d" % (os.getpid(), sessions[0])
        folders[session] = params.get("cwd", ".")
        # Counted on disk: each conversation may start its own agent process.
        counter = os.path.join(os.path.dirname(release), "sessions")
        sessions[0] = (int(open(counter).read()) if os.path.exists(counter) else 0) + 1
        with open(counter, "w") as handle:
            handle.write(str(sessions[0]))
        if sessions[0] == 1:
            send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": session, "modes": {
                "currentModeId": "default",
                "availableModes": [{"id": "default", "name": "Default"}, {"id": "plan", "name": "Plan"}]}}})
        else:
            send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": session, "configOptions": CONFIG}})
    elif method == "session/set_config_option":
        for item in CONFIG:
            if item["id"] == params.get("configId"):
                item["currentValue"] = params.get("value")
        send({"jsonrpc": "2.0", "id": request_id, "result": {"configOptions": CONFIG}})
    elif method == "session/set_mode":
        send({"jsonrpc": "2.0", "id": request_id, "result": {}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        folder = folders.get(session, ".")
        text = " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        if text.strip() == "ask":
            tool = {"toolCallId": "ask-1", "title": "Edit README.md", "kind": "edit", "status": "pending",
                    "rawInput": {"path": "README.md"}}
            update(session, {"sessionUpdate": "tool_call", **tool})
            request("session/request_permission", {
                "sessionId": session, "toolCall": tool,
                "options": [{"optionId": "allow", "name": "Allow", "kind": "allow_once"},
                            {"optionId": "reject", "name": "Reject", "kind": "reject_once"}]})
            update(session, {"sessionUpdate": "tool_call_update", "toolCallId": "ask-1", "status": "completed"})
        else:
            update(session, {"sessionUpdate": "agent_message_chunk",
                             "content": {"type": "text", "text": "I will add the notes."}})
            update(session, {"sessionUpdate": "tool_call", "toolCallId": "edit-1", "title": "Edit notes.md",
                             "kind": "edit", "status": "completed", "locations": [{"path": folder + "/notes.md"}],
                             "content": [{"type": "diff", "path": folder + "/notes.md", "oldText": "# Notes\n",
                                          "newText": "# Notes\n\nThe answer's header.\nIts chips.\n"}]})
            created = request("terminal/create", {"sessionId": session, "command": "sh",
                                                  "args": ["-c", "echo Compiling; echo Finished"], "cwd": folder})
            terminal = (created.get("result") or {}).get("terminalId")
            if terminal:
                update(session, {"sessionUpdate": "tool_call", "toolCallId": "run-1", "title": "cargo build",
                                 "kind": "execute", "status": "in_progress",
                                 "content": [{"type": "terminal", "terminalId": terminal}]})
                request("terminal/wait_for_exit", {"sessionId": session, "terminalId": terminal})
                update(session, {"sessionUpdate": "tool_call_update", "toolCallId": "run-1", "status": "completed"})
            update(session, {"sessionUpdate": "tool_call", "toolCallId": "hold-1", "title": "Edit notes.md",
                             "kind": "edit", "status": "in_progress"})
            while not os.path.exists(release):
                time.sleep(0.2)
            update(session, {"sessionUpdate": "tool_call_update", "toolCallId": "hold-1", "status": "completed"})
            update(session, {"sessionUpdate": "agent_message_chunk",
                             "content": {"type": "text", "text": " The notes are in."}})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

write_stand_in() {
  sed -e "s|@HOOKS@|$HOOKS|" >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code: on --resume it sends the plugin's events for that session, ending in a
# permission ask, then waits.
import json
import os
import subprocess
import sys

print("Claude Code (stand-in): " + " ".join(sys.argv[1:]), flush=True)
if "--resume" in sys.argv:
    session = sys.argv[sys.argv.index("--resume") + 1]
    common = {"session_id": session, "transcript_path": "/tmp/e2e-transcript.jsonl",
              "cwd": os.getcwd(), "permission_mode": "default"}
    events = [
        {"hook_event_name": "SessionStart", "source": "resume"},
        {"hook_event_name": "UserPromptSubmit", "prompt": "Run the build"},
        {"hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": "make"},
         "tool_use_id": "t1"},
        {"hook_event_name": "PermissionRequest", "tool_name": "Bash",
         "tool_input": {"command": "make"}},
    ]
    for event in events:
        answer = subprocess.run([sys.executable, "@HOOKS@/event.py"],
                                input=json.dumps({**common, **event}),
                                capture_output=True, text=True, check=False).stdout
        sequence = json.loads(answer or "{}").get("terminalSequence")
        if sequence:
            sys.stdout.write(sequence)
            sys.stdout.flush()
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$1"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo db
  mkdir -p "$home" "$bin" "$repo/sub"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  git init -q -b main "$repo"
  write_stand_in "$bin/claude"
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/release" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  export MARLEY_CLAUDE=$bin/claude CLAUDE_CONFIG_DIR=$E2E_WORK/claude
  claude_session "$CLAUDE_ONE" "$repo" cli "fix the flaky parser test in tokenizer.rs" \
    ai-title "Fix the flaky tokenizer test"
  claude_session "$CLAUDE_TWO" "$repo/sub" cli "write the release notes" \
    custom-title "Release notes for sub"
  touch -d '-1 minutes' "$CLAUDE_CONFIG_DIR/projects/"*"/$CLAUDE_ONE.jsonl"
  touch -d '-4 days' "$CLAUDE_CONFIG_DIR/projects/"*"/$CLAUDE_TWO.jsonl"
  profile_setting marley.assistant '{"enabled": true, "agent": "claude_code"}'
  profile_setting marley.tmux_hosts '[]'
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  open_path "$repo"
}

steps() {
  settle 15
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== a Marley conversation, mid-turn"
  palette "marley: new marley conversation" 10
  type_text "start"
  settle 3
  press "" Return
  settle 6

  echo "== the tokenizer session resumed"
  palette "marley: open threads" 4
  click "$CLI_CARD_X" "$CLI_CARD_Y"
  settle 1
  click "$OPEN_X" "$OPEN_Y"
  settle 8

  echo "== the Threads page"
  palette "marley: open threads" 4
  away
  shot 919-01-night
  [ -n "${STOP_AT_PAGE:-}" ] && return

  echo "== the waiting card"
  click "$CLI_CARD_X" "$CLI_CARD_Y"
  settle 2
  away
  shot 919-02-preview

  echo "== the Marley conversation's card"
  click "$THREAD_CARD_X" "$THREAD_CARD_Y"
  settle 2
  away
  shot 919-03-preview-thread

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 919-05-day
  profile_setting theme '"Marley Night"'
  settle 3

  echo "== New Agent"
  click "$NEW_AGENT_X" "$NEW_AGENT_Y"
  settle 3
  away
  shot 919-04-new-agent
  press "" Escape
  touch "$E2E_WORK/release"
}
