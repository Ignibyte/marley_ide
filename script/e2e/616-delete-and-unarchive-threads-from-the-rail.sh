# shellcheck shell=bash
# #616's e2e test: delete a thread, and bring an archived one back, from the rail. #605's stand-in
# external agent (just enough ACP, in Python, a custom agent server in the run's settings) answers
# each prompt. The scratch project `repo` gets three of its threads from the rail's +: "first",
# "second" and "kept"; "first" is archived from its row.
#
# "kept", the thread its Agent Panel shows, is deleted from its row's menu after the prompt
# (`confirm`, `deleted`, REQ-001, REQ-004). The project's menu lists "first" under Archived
# Threads (`archived`, REQ-002), and Open Thread brings it back (`unarchived`, REQ-003).
compositor sway

# The project's + and how many steps down New Agent Thread's submenu the stand-in sits (#501's);
# the rows and Archive, as #605's shots found them.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-96}
STAND_IN_STEPS=${STAND_IN_STEPS:-2}
ROW_X=${ROW_X:-110}
ARCHIVE_X=${ARCHIVE_X:-234}
FIRST_Y=${FIRST_Y:-274}
KEPT_Y=${KEPT_Y:-182}
# Delete Thread… in kept's menu.
DELETE_X=${DELETE_X:-180}
DELETE_Y=${DELETE_Y:-222}
# Archived Threads in the project's menu, and the first archived thread in its submenu.
ARCHIVED_X=${ARCHIVED_X:-200}
ARCHIVED_Y=${ARCHIVED_Y:-172}
FIRST_ENTRY_X=${FIRST_ENTRY_X:-400}
FIRST_ENTRY_Y=${FIRST_ENTRY_Y:-176}

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
# A stand-in external agent for #616's e2e test (#605's): just enough ACP for Zed to start a session and
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

  echo "== three threads of the stand-in, first archived"
  new_thread "first"
  new_thread "second"
  new_thread "kept"
  pointer_to 700 600
  settle 2
  shot threads
  pointer_to "$ROW_X" "$FIRST_Y"
  settle 1
  pointer_to "$ARCHIVE_X" "$FIRST_Y"
  settle 2
  click "$ARCHIVE_X" "$FIRST_Y"
  settle 3
  pointer_to 700 600
  settle 1
  shot archived-first

  echo "== Delete Thread… on kept"
  click "$ROW_X" "$KEPT_Y" right
  settle 1.5
  shot kept-menu
  click "$DELETE_X" "$DELETE_Y"
  settle 2
  shot confirm
  press "" Return
  settle 4
  pointer_to 700 600
  settle 1
  shot deleted

  echo "== the project's Archived Threads"
  click "$ROW_X" "$PLUS_Y" right
  settle 1.5
  shot project-menu
  click "$ARCHIVED_X" "$ARCHIVED_Y"
  settle 1.5
  shot archived
  click "$FIRST_ENTRY_X" "$FIRST_ENTRY_Y"
  settle 4
  pointer_to 700 600
  settle 1
  shot unarchived

  echo "== after a restart: first and second listed, kept gone"
  quit_marley
  open_path ""
  launch_marley
  settle 15
  pointer_to 700 600
  settle 1
  shot restart
}

