# shellcheck shell=bash
# #685's e2e test: what the Agent Panel does with an ACP agent that speaks between turns, the way a
# manager's thread through rustal-harness's `rh acp` will. A scripted agent (#605's stand-in grown,
# a custom agent server in the run's settings) answers the first prompt and ends its turn; three
# seconds later it sends a report with a `messageId` of its own, and two seconds after that asks
# permission with Allow and Deny while no turn runs. The shots show the reply (`replied`), the
# report under it (`report`, REQ-001, REQ-002) and the request (`permission`, REQ-003); a click on
# Allow answers it, and the agent's log holds the choice (`allowed`, REQ-004); a second prompt is
# answered after it (`second-turn`, REQ-005).
compositor sway

# The project's + in the rail, how many steps down New Agent Thread's submenu the scripted agent
# sits, Allow on the request, and the thread's message editor, as the first run's shots found them.
PLUS_X=${PLUS_X:-224}
PLUS_Y=${PLUS_Y:-123}
AGENT_STEPS=${AGENT_STEPS:-2}
ALLOW_X=${ALLOW_X:-1020}
ALLOW_Y=${ALLOW_Y:-410}
EDITOR_X=${EDITOR_X:-1200}
EDITOR_Y=${EDITOR_Y:-845}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  write_scripted_agent
  # The agent joins the run's agent servers, inside the copied settings' own block when they have
  # one, so it does not shadow the user's (#605's way).
  python3 - "$E2E_PROFILE/config/settings.json" "$E2E_WORK/scripted-agent.py" "$E2E_WORK/agent.log" <<'PY'
import json, re, sys
path, script, log = sys.argv[1:]
entry = '"Scripted": %s,' % json.dumps({"type": "custom", "command": "python3", "args": [script, log]})
text = open(path).read()
match = re.search(r'"agent_servers"\s*:\s*\{', text)
if match:
    text = text[:match.end()] + "\n    " + entry + text[match.end():]
else:
    at = text.index("{") + 1
    text = text[:at] + '\n  "agent_servers": {' + entry + '},' + text[at:]
open(path, "w").write(text)
PY
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent for #685's e2e test: it answers each prompt, and after the first one's turn
# has ended it sends a report and then asks permission, as a manager posting on its own would.
# Every message it reads or sends goes to the log named first, one JSON line each.
import json
import sys
import threading

log_path = sys.argv[1]
lock = threading.Lock()


def log(direction, message):
    with lock, open(log_path, "a") as log_file:
        log_file.write(json.dumps({"direction": direction, "message": message}) + "\n")


def send(message):
    with lock:
        sys.stdout.write(json.dumps(message) + "\n")
        sys.stdout.flush()
    log("sent", message)


def chunk(session_id, message_id, text):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {
        "sessionId": session_id,
        "update": {"sessionUpdate": "agent_message_chunk", "messageId": message_id,
                   "content": {"type": "text", "text": text}},
    }})


def report(session_id):
    chunk(session_id, "report-1", "Report between turns: the build passed.")
    threading.Timer(2.0, ask, args=(session_id,)).start()


def ask(session_id):
    send({"jsonrpc": "2.0", "id": "permission-1", "method": "session/request_permission", "params": {
        "sessionId": session_id,
        "toolCall": {"toolCallId": "confirm-1", "title": "Merge the branch into main?",
                     "kind": "other", "status": "pending"},
        "options": [
            {"optionId": "allow", "name": "Allow", "kind": "allow_once"},
            {"optionId": "deny", "name": "Deny", "kind": "reject_once"},
        ],
    }})


sessions = 0
prompts = 0
for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    log("read", message)
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
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "scripted-%d" % sessions}})
    elif method == "session/prompt":
        prompts += 1
        session_id = params.get("sessionId")
        text = " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        chunk(session_id, "reply-%d" % prompts, "Noted: " + text)
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
        if prompts == 1:
            threading.Timer(3.0, report, args=(session_id,)).start()
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "the scripted agent does not do " + str(method)}})
PY
}

# Whether the agent's log holds the response to its permission request with the allow option.
allowed_in_log() {
  python3 - "$E2E_WORK/agent.log" <<'PY'
import json, sys
for line in open(sys.argv[1]):
    entry = json.loads(line)
    message = entry["message"]
    if entry["direction"] == "read" and message.get("id") == "permission-1":
        print("  the response:", json.dumps(message.get("result") or message.get("error")))
        outcome = (message.get("result") or {}).get("outcome") or {}
        sys.exit(0 if outcome.get("optionId") == "allow" else 1)
print("  no response to permission-1 in the log")
sys.exit(1)
PY
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== a thread of the scripted agent, prompted"
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  shot 685-00-menu
  press "" Down
  press "" Down
  press "" Right
  settle 1
  for ((step = 0; step < AGENT_STEPS; step++)); do
    press "" Down
  done
  settle 1
  shot 685-00-submenu
  press "" Return
  settle 6
  type_text "start"
  press "" Return
  settle 1
  shot 685-01-replied

  echo "== the report, after the turn ended"
  settle 3
  shot 685-02-report

  echo "== the permission request, with no turn running"
  settle 3
  shot 685-03-permission

  echo "== Allow answers it"
  click "$ALLOW_X" "$ALLOW_Y"
  settle 2
  shot 685-04-allowed
  expect "the agent got the allow option" allowed_in_log

  echo "== a second prompt after it"
  click "$EDITOR_X" "$EDITOR_Y"
  settle 1
  type_text "again"
  press "" Return
  settle 3
  shot 685-05-second-turn
  expect "the agent answered the second prompt" grep -q '"Noted: again"' "$E2E_WORK/agent.log"
}
