# shellcheck shell=bash
# #605's e2e test: archive an agent thread from the rail. A stand-in external agent (just enough
# ACP, in Python, as #501's, a custom agent server in the run's settings) answers each prompt. The
# scratch project `repo` gets three of its threads from the rail's +: "first", "second" and
# "kept" (`threads`). The pointer on first's row shows Archive and its tooltip (`hover`,
# REQ-001); a click takes the row away without opening it (`archived`, REQ-002). Second's
# right-click menu offers Archive Thread (`menu`), which takes it away (`menu-archived`,
# REQ-003). After a restart with no path, kept is listed and the other two are not (`restart`,
# REQ-004).
compositor sway

# The project's + and how many steps down New Agent Thread's submenu the stand-in sits (501's);
# the rows and Archive, as the first run's shots found them.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-96}
STAND_IN_STEPS=${STAND_IN_STEPS:-2}
ROW_X=${ROW_X:-110}
ARCHIVE_X=${ARCHIVE_X:-234}
FIRST_Y=${FIRST_Y:-274}
SECOND_Y=${SECOND_Y:-228}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  write_stand_in_agent
  # The stand-in joins the run's agent servers, inside the copied settings' own block when they
  # have one, so it does not shadow the user's.
  python3 - "$E2E_PROFILE/config/settings.json" "$E2E_WORK/stand-in-agent.py" <<'PY'
import json, re, sys
path, script = sys.argv[1:]
entry = '"Stand-in": %s,' % json.dumps({"type": "custom", "command": "python3", "args": [script]})
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

write_stand_in_agent() {
  cat >"$E2E_WORK/stand-in-agent.py" <<'PY'
# A stand-in external agent for #605's e2e test: just enough ACP for Zed to start a session and
# answer each prompt with one message.
import json
import sys


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


# Zed sends its request ids as strings, so sessions are counted apart.
sessions = 0
for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "protocolVersion": 1,
            "agentCapabilities": {"loadSession": False, "promptCapabilities": {}},
            "authMethods": [],
            "agentInfo": {"name": "stand-in", "version": "0"},
        }})
    elif method == "session/new":
        sessions += 1
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "stand-in-%d" % sessions}})
    elif method == "session/prompt":
        text = " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        send({"jsonrpc": "2.0", "method": "session/update", "params": {
            "sessionId": params.get("sessionId"),
            "update": {"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": "Noted: " + text}},
        }})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "the stand-in does not do " + str(method)}})
PY
}

# A thread of the stand-in from the project's +, prompted with `$1`.
new_thread() {
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  press "" Down
  press "" Down
  press "" Right
  settle 1
  for ((step = 0; step < STAND_IN_STEPS; step++)); do
    press "" Down
  done
  settle 1
  press "" Return
  settle 6
  type_text "$1"
  press "" Return
  settle 5
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== three threads of the stand-in"
  new_thread "first"
  shot threads-1
  new_thread "second"
  new_thread "kept"
  pointer_to 700 600
  settle 2
  shot threads

  echo "== Archive on first's row"
  pointer_to "$ROW_X" "$FIRST_Y"
  settle 1
  pointer_to "$ARCHIVE_X" "$FIRST_Y"
  settle 2
  shot hover
  click "$ARCHIVE_X" "$FIRST_Y"
  settle 3
  pointer_to 700 600
  settle 1
  shot archived

  echo "== Archive Thread from second's menu"
  click "$ROW_X" "$SECOND_Y" right
  settle 1
  shot menu
  press "" Return
  settle 3
  shot menu-archived

  echo "== quit, and start again with no path"
  quit_marley
  open_path ""
  launch_marley
  settle 15
  shot restart
}
