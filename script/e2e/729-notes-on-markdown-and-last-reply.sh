# shellcheck shell=bash
# #729's visual check: notes on a Markdown file and on an agent's last reply. A scratch repository
# with a `README.md`; in its terminal a stand-in Claude Code (#522's: Marley's plugin's hook makes
# it idle, and it echoes each line it reads) that also sends a `Stop` frame naming a scratch
# transcript with two assistant entries around a tool's result; and a scripted ACP agent in a
# thread tab (#717's way, behind a key), answering each prompt with "Noted:" and a second paragraph.
# Shots: `marley: review markdown` on README.md, the tab with Send Notes (`729-01-file`, REQ-001);
# a note on a passage, Send Notes, the picker with the agent ready (`729-02a-picker`), and Enter:
# the stand-in echoes `File:`, the lines, `Quote:` and `User comment:` (`729-02-file-sent`,
# REQ-002); Review Last Reply from the terminal's rail menu: the transcript's reply, its thinking
# left out (`729-03-terminal-reply`, REQ-003); a note and Send Notes: the stand-in echoes "Notes on
# your last reply:" (`729-04-terminal-sent`, REQ-004) and the tab marks the note Sent
# (`729-04b-marked`); in the thread tab after one prompt, `marley: review last reply`
# (`729-05-thread-reply`, REQ-005); a note and Send Notes: the thread shows the notes as the user's
# message and the agent's answer (`729-06-thread-sent`, REQ-006). Each note's drag and its `c` have a
# shot of their own (`…-dragged`, `…-dragged-c`).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
FILE_FROM_X=${FILE_FROM_X:-279}
FILE_FROM_Y=${FILE_FROM_Y:-151}
FILE_TO_X=${FILE_TO_X:-594}
FILE_TO_Y=${FILE_TO_Y:-151}
SEND_X=${SEND_X:-1278}
SEND_Y=${SEND_Y:-331}
ROW_X=${ROW_X:-130}
ROW_Y=${ROW_Y:-175}
REVIEW_ITEM_X=${REVIEW_ITEM_X:-197}
REVIEW_ITEM_Y=${REVIEW_ITEM_Y:-238}
REPLY_FROM_X=${REPLY_FROM_X:-279}
REPLY_FROM_Y=${REPLY_FROM_Y:-175}
REPLY_TO_X=${REPLY_TO_X:-497}
REPLY_TO_Y=${REPLY_TO_Y:-175}
REPLY_TAB_X=${REPLY_TAB_X:-727}
REPLY_TAB_Y=${REPLY_TAB_Y:-51}
THREAD_FROM_X=${THREAD_FROM_X:-279}
THREAD_FROM_Y=${THREAD_FROM_Y:-135}
THREAD_TO_X=${THREAD_TO_X:-474}
THREAD_TO_Y=${THREAD_TO_Y:-135}
THREAD_TAB_X=${THREAD_TAB_X:-920}
THREAD_TAB_Y=${THREAD_TAB_Y:-51}
AWAY_X=${AWAY_X:-1700}
AWAY_Y=${AWAY_Y:-1000}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$config/plugins" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/$SHIPPED", "version": "$SHIPPED"}]}}
JSON
  export CLAUDE_CONFIG_DIR=$config
  cat >"$repo/README.md" <<'MD'
# Search cache

The search reads the index on every keystroke.
A cache in front of it would cut the reads.

## Next steps

1. Measure the reads.
2. Add the cache.
MD
  write_transcript "$E2E_WORK/transcript.jsonl"
  write_fake_claude "$bin/claude"
  write_agent
  : >"$E2E_WORK/agent.log"
  profile_setting agent_servers.Scripted "{\"type\": \"custom\", \"command\": \"python3\", \"args\": [\"$E2E_WORK/agent.py\", \"$E2E_WORK/agent.log\"]}"
  # A key that opens a thread tab on the scripted agent in the shown project.
  printf '%s\n' '[{"bindings": {"ctrl-alt-shift-z": ["marley::NewAgentThread", {"agent": "Scripted"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  open_path "$repo"
}

# A transcript in Claude Code's form: a prompt, a reply with thinking and text, a tool's result,
# and the reply's end.
write_transcript() {
  python3 - "$1" <<'PY'
import json, sys
entries = [
    {"type": "user", "message": {"role": "user", "content": "Look at the search"}},
    {"type": "assistant", "message": {"role": "assistant", "content": [
        {"type": "thinking", "thinking": "A private thought."},
        {"type": "text", "text": "I read the search code.\n\nThe index is read **twice** per keystroke."},
        {"type": "tool_use", "id": "t1", "name": "Read", "input": {}}]}},
    {"type": "user", "message": {"role": "user", "content": [
        {"type": "tool_result", "tool_use_id": "t1", "content": "ok"}]}},
    {"type": "assistant", "message": {"role": "assistant", "content": [
        {"type": "text", "text": "A cache would halve the reads. Shall I add one?"}]}},
]
with open(sys.argv[1], "w") as out:
    for entry in entries:
        out.write(json.dumps(entry) + "\n")
PY
}

# A fake `claude` that sends a SessionStart and then a Stop naming the transcript through Marley's
# plugin hook, so its seat is idle and knows the transcript, and echoes each line it reads.
write_fake_claude() {
  sed -e "s|@HOOK@|$HOOK|" -e "s|@TRANSCRIPT@|$E2E_WORK/transcript.jsonl|" >"$1" <<'FAKE'
#!/usr/bin/env python3
import json
import os
import subprocess
import sys

if sys.argv[1:2] == ["plugin"]:
    sys.exit(0)
print("stand-in Claude Code, idle", flush=True)
session = "s-" + str(os.getpid())


def hook(event):
    event.update({"cwd": os.getcwd(), "session_id": session, "transcript_path": "@TRANSCRIPT@"})
    answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps(event),
                            capture_output=True, text=True, check=False).stdout
    sequence = json.loads(answer or "{}").get("terminalSequence")
    if sequence:
        sys.stdout.write(sequence)
        sys.stdout.flush()


hook({"hook_event_name": "SessionStart", "source": "startup", "permission_mode": "default"})
hook({"hook_event_name": "Stop", "last_assistant_message": "A cache would halve the reads."})
for line in sys.stdin:
    print("got: " + line.rstrip("\n"), flush=True)
FAKE
  chmod +x "$1"
}

write_agent() {
  cat >"$E2E_WORK/agent.py" <<'PY'
# A scripted ACP agent for #729's visual check: each prompt is logged and answered with "Noted:"
# and a second paragraph.
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
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "protocolVersion": 1, "agentCapabilities": {"loadSession": True, "promptCapabilities": {}},
            "authMethods": [], "agentInfo": {"name": "scripted", "version": "0"}}})
    elif method == "session/new":
        sessions += 1
        send({"jsonrpc": "2.0", "id": request_id,
              "result": {"sessionId": "scripted-%d-%d" % (os.getpid(), sessions)}})
    elif method == "session/load":
        send({"jsonrpc": "2.0", "id": request_id, "result": {}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        text = "".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        with open(log_path, "a") as log_file:
            log_file.write(json.dumps({"prompt": text}) + "\n")
        first = text.strip().splitlines()[0] if text.strip() else ""
        send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session, "update": {
            "sessionUpdate": "agent_message_chunk", "content": {"type": "text",
            "text": "Noted: " + first + "\n\nThe cache should live in memory, keyed by the query."}}}})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Drags over a passage from ($1, $2) to ($3, $4), takes the shot `$6`, presses `c`, and writes the
# note `$5`.
note_on() {
  pointer_to "$1" "$2"
  pointer_down
  pointer_to "$3" "$4"
  pointer_up
  settle 1
  take "$6" || return 1
  press "" c
  settle 1
  take "$6-c" || return 1
  type_text "$5"
  press "" Return
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the stand-in Claude Code, idle, its transcript named"
  type_text "claude"
  press "" Return
  settle 4

  echo "== README.md in a review tab"
  press "CTRL" p
  settle 1
  type_text "README"
  settle 1
  press "" Return
  settle 2
  palette "marley: review markdown"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 729-01-file || return 0
  if ((FILE_FROM_X == 0)); then
    echo "the file's points are not placed yet: 729-01 is the last shot"
    return 0
  fi

  echo "== a note on the file, sent through the picker"
  note_on "$FILE_FROM_X" "$FILE_FROM_Y" "$FILE_TO_X" "$FILE_TO_Y" "Say which index" 729-01a-dragged ||
    return 0
  click "$SEND_X" "$SEND_Y"
  settle 1
  take 729-02a-picker || return 0
  press "" Return
  settle 3
  take 729-02-file-sent || return 0
  mcp_agent terminal-read claude >"$E2E_WORK/file-sent.txt" || true
  cat "$E2E_WORK/file-sent.txt"
  expect "the agent got the file's note" holds "$E2E_WORK/file-sent.txt" \
    "got: File: README.md" "got: Line" 'got: Quote: "' 'got: User comment: "Say which index"'
  if ((ROW_X == 0)); then
    echo "the rail row is not placed yet: 729-02 is the last shot"
    return 0
  fi

  echo "== Review Last Reply from the terminal's rail menu"
  click "$ROW_X" "$ROW_Y" right
  settle 1
  take 729-03a-menu || return 0
  if ((REVIEW_ITEM_X == 0)); then
    echo "the menu item is not placed yet: 729-03a is the last shot"
    return 0
  fi
  click "$REVIEW_ITEM_X" "$REVIEW_ITEM_Y"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 729-03-terminal-reply || return 0
  if ((REPLY_FROM_X == 0)); then
    echo "the reply's points are not placed yet: 729-03 is the last shot"
    return 0
  fi

  echo "== a note on the reply, sent back to its terminal"
  note_on "$REPLY_FROM_X" "$REPLY_FROM_Y" "$REPLY_TO_X" "$REPLY_TO_Y" "Only for file names" \
    729-03b-dragged || return 0
  click "$SEND_X" "$SEND_Y"
  settle 3
  take 729-04-terminal-sent || return 0
  mcp_agent terminal-read claude >"$E2E_WORK/reply-sent.txt" || true
  cat "$E2E_WORK/reply-sent.txt"
  expect "the agent got the reply's note" holds "$E2E_WORK/reply-sent.txt" \
    "got: Notes on your last reply:" 'got: Quote: "' 'got: User comment: "Only for file names"'
  click "$REPLY_TAB_X" "$REPLY_TAB_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 729-04b-marked || return 0

  echo "== a thread's last reply"
  press "CTRL ALT SHIFT" z
  settle 6
  type_text "check the cache"
  press "" Return
  settle 4
  palette "marley: review last reply"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 729-05-thread-reply || return 0
  if ((THREAD_FROM_X == 0)); then
    echo "the thread reply's points are not placed yet: 729-05 is the last shot"
    return 0
  fi

  echo "== a note on the thread's reply, sent as the user's message"
  note_on "$THREAD_FROM_X" "$THREAD_FROM_Y" "$THREAD_TO_X" "$THREAD_TO_Y" \
    "Keyed by the query and the folder" 729-05b-dragged || return 0
  click "$SEND_X" "$SEND_Y"
  settle 4
  click "$THREAD_TAB_X" "$THREAD_TAB_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 729-06-thread-sent || return 0
  cat "$E2E_WORK/agent.log"
  expect "the thread's agent got the notes" holds "$E2E_WORK/agent.log" \
    "Notes on your last reply:" "Keyed by the query and the folder"
}
