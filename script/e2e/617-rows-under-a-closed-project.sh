# shellcheck shell=bash
# #617's e2e test: a closed project's threads and ports under its header. Before Marley starts, a
# server listens in `repo-b`. Marley opens `repo`, and a second launch hands it `repo-b` (#513),
# where #605's stand-in agent (just enough ACP, in Python, a custom agent server in the run's
# settings, here able to load a session again) gets two threads from the project's +, "first 617"
# and then "second 617", which its Agent Panel keeps showing. `repo` shown, Marley starts again
# with no path, so `repo-b` is closed: its dimmed header lists both threads and the port
# (`closed`, REQ-001), and its chevron folds them and brings them back (`folded`, `unfolded`,
# REQ-004). A click on "first 617", which a restored panel would not show, opens `repo-b` and that
# thread (`thread`, REQ-002). `repo` shown and Marley started again, a double-click on the port row
# opens `repo-b` and a Browser tab on the port (`port`, REQ-003). Chromium runs offline.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The project's + and how many steps down New Agent Thread's submenu the stand-in sits (#616's).
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-96}
STAND_IN_STEPS=${STAND_IN_STEPS:-2}
# After a restart: `repo-b`'s chevron, its "first 617" row (under "second 617", newest first) and
# its port row.
ROW_X=${ROW_X:-110}
CHEVRON_X=${CHEVRON_X:-24}
REPO_B_Y=${REPO_B_Y:-96}
FIRST_Y=${FIRST_Y:-182}
PORT_Y=${PORT_Y:-228}

setup() {
  local home=$E2E_WORK/home name
  mkdir -p "$home"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  for name in repo repo-b; do
    mkdir -p "$E2E_WORK/$name"
    printf '# %s\n' "$name" >"$E2E_WORK/$name/README.md"
  done
  printf '<!doctype html><title>repo-b</title><h1>Served from repo-b</h1>\n' \
    >"$E2E_WORK/repo-b/index.html"
  offline_chromium
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
  read -r PORT < <(python3 -c '
import socket
held = socket.socket()
held.bind(("127.0.0.1", 0))
print(held.getsockname()[1])
')
  echo "the port: $PORT"
  (cd "$E2E_WORK/repo-b" && exec python3 -u -m http.server --bind 127.0.0.1 "$PORT" \
    >"$E2E_WORK/server.log" 2>&1) &
  echo "$!" >>"$E2E_WORK/servers"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

write_stand_in_agent() {
  cat >"$E2E_WORK/stand-in-agent.py" <<'PY'
# A stand-in external agent for #617's e2e test (#605's): just enough ACP for Zed to start a session,
# answer each prompt with one message, and load a session again after Marley restarts, from the
# messages it keeps in `sessions/` beside this script.
import json
import os
import sys
import uuid

kept = os.path.join(os.path.dirname(os.path.abspath(__file__)), "sessions")
os.makedirs(kept, exist_ok=True)


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def update(session_id, kind, text):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {
        "sessionId": session_id,
        "update": {"sessionUpdate": kind, "content": {"type": "text", "text": text}},
    }})


def history(session_id):
    path = os.path.join(kept, session_id + ".json")
    return json.load(open(path)) if os.path.exists(path) else []


for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "protocolVersion": 1,
            "agentCapabilities": {"loadSession": True, "promptCapabilities": {}},
            "authMethods": [],
            "agentInfo": {"name": "stand-in", "version": "0"},
        }})
    elif method == "session/new":
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "stand-in-" + uuid.uuid4().hex[:8]}})
    elif method == "session/load":
        session_id = params.get("sessionId")
        for kind, text in history(session_id):
            update(session_id, kind, text)
        send({"jsonrpc": "2.0", "id": request_id, "result": {}})
    elif method == "session/prompt":
        session_id = params.get("sessionId")
        text = " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        update(session_id, "agent_message_chunk", "Noted: " + text)
        messages = history(session_id) + [["user_message_chunk", text], ["agent_message_chunk", "Noted: " + text]]
        json.dump(messages, open(os.path.join(kept, session_id + ".json"), "w"))
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "the stand-in does not do " + str(method)}})
PY
}

# A second launch on this profile with `$1`, which hands it to the running Marley (#513).
hand_off() {
  env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" \
    SWAYSOCK="$SWAY_SOCK" "$E2E_MARLEY" --user-data-dir "$E2E_PROFILE" "$1" \
    >"$E2E_WORK/second.log" 2>&1 </dev/null
}

# A thread of the stand-in from the top project's +, prompted with `$1`.
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

# Two clicks at `$1`,`$2`, close enough together to count as one double-click (#604's).
double_click() {
  pointer_to "$1" "$2"
  sleep 0.05
  pointer_down
  sleep 0.05
  pointer_up
  sleep 0.08
  pointer_down
  sleep 0.05
  pointer_up
}

# Shows `repo`, then starts Marley again with no path, which leaves `repo-b` closed (#606).
restart_with_repo_b_closed() {
  hand_off "$E2E_WORK/repo"
  settle 5
  quit_marley
  open_path ""
  launch_marley
  settle 15
  pointer_to 700 600
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== repo-b, with two threads of the stand-in"
  hand_off "$E2E_WORK/repo-b"
  settle 6
  press "" Return
  settle 3
  new_thread "first 617"
  new_thread "second 617"
  pointer_to 700 600
  settle 2
  shot before

  echo "== repo shown, then Marley started again: repo-b closed"
  restart_with_repo_b_closed
  shot closed

  echo "== repo-b's chevron folds its rows, and brings them back"
  click "$CHEVRON_X" "$REPO_B_Y"
  settle 2
  pointer_to 700 600
  settle 1
  shot folded
  click "$CHEVRON_X" "$REPO_B_Y"
  settle 2
  pointer_to 700 600
  settle 1
  shot unfolded

  echo "== a click on first 617 opens repo-b and that thread"
  click "$ROW_X" "$FIRST_Y"
  settle 10
  pointer_to 700 600
  settle 1
  shot thread

  echo "== started again; a double-click on the port row"
  restart_with_repo_b_closed
  shot closed-again
  double_click "$ROW_X" "$PORT_Y"
  settle 12
  pointer_to 700 600
  settle 1
  shot port
}
