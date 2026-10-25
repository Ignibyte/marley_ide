# shellcheck shell=bash
# #925's visual check: toasts by kind, the plan's Needs you entry, a long command's banner and the
# phone push, as the Inbox board words them. Marley runs on a private session bus whose
# notification server logs each banner, and pushes to a fake ntfy on a loopback port that logs
# each request (#535, #551). A stand-in `claude` acts out a session, a step each time the scenario
# writes its name to a FIFO: the plugin's real `event.py` for each event, and for `plan` an
# ExitPlanMode approval through `marley approve`, as #881's does. `MARLEY_CLAUDE` names the
# stand-in and `CLAUDE_CONFIG_DIR` a scratch folder, so Connect Claude Code to Marley runs no real
# CLI. No agent CLI runs.
# - `925-02-plan`: the stand-in asks a plan titled "Tidy the notes": the entry reads "Plan to
#   review: “Tidy the notes”" with Open the plan in the accent (REQ-002).
# - `925-03-opened`: the terminal brought forward, then Open the plan clicked: the plan's tab in
#   front (REQ-002).
# - The push log: a turn ends while the window is on another workspace: Title "Turn finished",
#   body "repo · Claude" (REQ-004).
# - The banner log: `sleep 8; false` in a new terminal while the plan's tab is in front: "sleep 8;
#   false finished · exit 1", "… ran 8 s while you were in …" (REQ-003).
# - `925-01-toasts`: a success (the plugin connected), an error (the push server gone), a neutral
#   (`workspace: use classic layout`) and a warning (Wait on a working agent's close): green, red,
#   grey and yellow dots (REQ-001).
# - `925-04-day`: the toasts and the plan entry in Marley Day (REQ-001, REQ-002).
compositor sway

HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-300}
STOP_AT=${STOP_AT:-}
# From the probe run's shots.
CONNECT_X=${CONNECT_X:-515}
CONNECT_Y=${CONNECT_Y:-954}
OPEN_PLAN_X=${OPEN_PLAN_X:-196}
OPEN_PLAN_Y=${OPEN_PLAN_Y:-156}
WAIT_X=${WAIT_X:-903}
WAIT_Y=${WAIT_Y:-542}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/claude-config"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  export MARLEY_CLAUDE=$bin/claude CLAUDE_CONFIG_DIR=$E2E_WORK/claude-config
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  write_notification_server
  python3 -u "$E2E_WORK/notifications.py" "$E2E_WORK/banners.log" >"$E2E_WORK/notifications.out" 2>&1 &
  echo "$!" >"$E2E_WORK/notifications.pid"
  : >"$E2E_WORK/banners.log"
  write_fake_ntfy
  python3 -u "$E2E_WORK/ntfy.py" "$E2E_WORK/pushes.log" >"$E2E_WORK/ntfy.out" 2>&1 &
  echo "$!" >"$E2E_WORK/ntfy.pid"
  local port=
  for _ in $(seq 50); do
    port=$(grep -oE 'port [0-9]+' "$E2E_WORK/ntfy.out" | cut -d' ' -f2 || true)
    [[ -n $port ]] && break
    sleep 0.1
  done
  [[ -n $port ]] || { echo "the fake ntfy did not start" >&2; return 1; }
  : >"$E2E_WORK/pushes.log"
  mkfifo "$E2E_WORK/steps.fifo"
  write_steps
  write_stand_in "$bin/claude"
  git init -q -b main "$E2E_WORK/repo"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.plan_review true
  profile_setting marley.long_command_seconds 2
  profile_setting marley.push "{\"url\": \"http://127.0.0.1:$port\", \"topic\": \"marley-e2e\"}"
  open_path "$E2E_WORK/repo"
}

teardown() {
  local pid
  for pid in notifications ntfy; do
    [[ -f $E2E_WORK/$pid.pid ]] && kill "$(cat "$E2E_WORK/$pid.pid")" 2>/dev/null
  done
  [[ -f $E2E_WORK/bus.pid ]] && kill "$(head -1 "$E2E_WORK/bus.pid")" 2>/dev/null
  return 0
}

write_notification_server() {
  cat >"$E2E_WORK/notifications.py" <<'PY'
# A desktop notification server on the scenario's private bus: logs each banner and answers as a
# real server does.
import sys

import dbus
import dbus.mainloop.glib
import dbus.service
from gi.repository import GLib

LOG = sys.argv[1]
INTERFACE = "org.freedesktop.Notifications"
dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)


class Notifications(dbus.service.Object):
    def __init__(self, bus):
        super().__init__(bus, "/org/freedesktop/Notifications")
        self.last = 0

    @dbus.service.method(INTERFACE, in_signature="susssasa{sv}i", out_signature="u")
    def Notify(self, app, replaces, icon, summary, body, actions, hints, timeout):
        self.last += 1
        with open(LOG, "a", encoding="utf-8") as log:
            log.write(f"{app}|{summary}|{body}\n")
        return self.last

    @dbus.service.method(INTERFACE, in_signature="", out_signature="as")
    def GetCapabilities(self):
        return ["body", "actions"]

    @dbus.service.method(INTERFACE, in_signature="u", out_signature="")
    def CloseNotification(self, id):
        pass

    @dbus.service.method(INTERFACE, in_signature="", out_signature="ssss")
    def GetServerInformation(self):
        return ("e2e", "marley", "1", "1.2")


bus = dbus.SessionBus()
name = dbus.service.BusName(INTERFACE, bus)
server = Notifications(bus)
print("ready", flush=True)
GLib.MainLoop().run()
PY
}

write_fake_ntfy() {
  cat >"$E2E_WORK/ntfy.py" <<'PY'
# A fake ntfy on a free loopback port: logs each request's path, headers and body, one JSON line
# each, and answers as ntfy does.
import http.server
import json
import sys

LOG = sys.argv[1]


class Handler(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length") or 0)
        record = {"path": self.path, "headers": dict(self.headers.items()),
                  "body": self.rfile.read(length).decode("utf-8", "replace")}
        with open(LOG, "a", encoding="utf-8") as log:
            log.write(json.dumps(record, ensure_ascii=False) + "\n")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(b'{"id":"e2e","event":"message"}')

    def log_message(self, *args):
        pass


server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
print(f"port {server.server_address[1]}", flush=True)
server.serve_forever()
PY
}

# Each step's events; `notify` is the argument Claude Code's Stop hook gives notify.sh.
write_steps() {
  cat >"$E2E_WORK/steps.json" <<'JSON'
{
  "working": {"events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "Run the build"},
    {"hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": "make"}, "tool_use_id": "t1"}]},
  "stops": {"notify": "finished", "events": [
    {"hook_event_name": "PostToolUse", "tool_name": "Bash", "tool_input": {"command": "make"}, "tool_use_id": "t1"},
    {"hook_event_name": "Stop", "last_assistant_message": "The build is green."}]}
}
JSON
}

write_stand_in() {
  sed -e "s|@HOOKS@|$HOOKS|" -e "s|@WORK@|$E2E_WORK|" >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code. `claude plugin …`, as Connect runs it, succeeds and does nothing. A
# session acts out each step named on the FIFO: the plugin's event.py for each event and
# notify.sh where Claude Code runs it; `plan` asks for an ExitPlanMode approval in a thread, since
# the call waits for the answer.
import json
import os
import subprocess
import sys
import threading

if sys.argv[1:2] == ["plugin"]:
    sys.exit(0)

BIN = os.environ.get("MARLEY_BIN", "")
STEPS = json.load(open("@WORK@/steps.json", encoding="utf-8"))
COMMON = {"session_id": "e2e-session", "transcript_path": "/tmp/e2e-transcript.jsonl",
          "cwd": os.getcwd(), "permission_mode": "default"}
PLAN = """# Tidy the notes

1. Merge the duplicate notes.
2. Sort them by date.
"""


def write(answer):
    sequence = json.loads(answer or "{}").get("terminalSequence")
    if sequence:
        sys.stdout.write(sequence)
        sys.stdout.flush()


def ask_plan():
    done = subprocess.run(
        [BIN, "approve", "--source", "mod:claude-code", "--call", "toolu_plan", "--tool",
         "ExitPlanMode"], input=json.dumps({"plan": PLAN}), capture_output=True, text=True,
        check=False)
    print(f"plan: exit {done.returncode} {done.stdout.strip()}", flush=True)


print("Claude Code (stand-in): steps come through the FIFO", flush=True)
while True:
    with open("@WORK@/steps.fifo", encoding="utf-8") as fifo:
        for line in fifo:
            name = line.strip()
            if name == "plan":
                threading.Thread(target=ask_plan, daemon=True).start()
            else:
                step = STEPS[name]
                for event in step["events"]:
                    write(subprocess.run([sys.executable, "@HOOKS@/event.py"],
                                         input=json.dumps({**COMMON, **event}),
                                         capture_output=True, text=True, check=False).stdout)
                if step.get("notify"):
                    write(subprocess.run(["sh", "@HOOKS@/notify.sh", step["notify"]],
                                         capture_output=True, text=True, check=False).stdout)
            print(f"step: {name}", flush=True)
FAKE
  chmod +x "$1"
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

step_of() {
  timeout 5 bash -c "echo $1 >'$E2E_WORK/steps.fifo'" ||
    echo "step_of: the stand-in is not reading" >&2
  settle 2
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

# Whether the last push is titled `$1` with the body `$2`.
last_push() {
  tail -1 "$E2E_WORK/pushes.log"
  python3 - "$E2E_WORK/pushes.log" "$1" "$2" <<'PY'
import json
import sys

path, title, body = sys.argv[1:]
lines = open(path, encoding="utf-8").read().splitlines()
push = json.loads(lines[-1]) if lines else {}
headers = {key.lower(): value for key, value in push.get("headers", {}).items()}
sys.exit(0 if headers.get("title") == title and push.get("body") == body else 1)
PY
}

steps() {
  settle 15
  # Trusts the scratch repository.
  press "" Return
  settle 3
  type_text "claude"
  press "" Return
  settle 4
  away
  take 925-00-connect || return 0

  echo "== Connect Claude Code to Marley: a success"
  click "$CONNECT_X" "$CONNECT_Y"
  settle 4

  echo "== the plan"
  step_of plan
  settle 3
  away
  take 925-02-plan || return 0
  # The terminal in front, so the click shows the plan's tab coming forward.
  palette "pane: activate previous item" 2
  click "$OPEN_PLAN_X" "$OPEN_PLAN_Y"
  settle 3
  away
  take 925-03-opened || return 0

  echo "== a turn ends while the window is away"
  sway_msg workspace 2 >/dev/null
  settle 1
  step_of working
  step_of stops
  settle 2
  sway_msg workspace 1 >/dev/null
  settle 2
  cat "$E2E_WORK/pushes.log"
  expect "the push is titled with the event, the body the project and the agent" \
    last_push "Turn finished" "repo · Claude"

  echo "== a long command while the plan is in front"
  settle 5
  palette "workspace: new terminal" 3
  type_text "sleep 8; false"
  press "" Return
  palette "pane: activate previous item" 2
  settle 10
  cat "$E2E_WORK/banners.log"
  expect "the banner is titled with the command and its exit" \
    holds "$E2E_WORK/banners.log" "|sleep 8; false finished · exit 1|"
  expect "the banner says where the user was" \
    holds "$E2E_WORK/banners.log" " ran 8 s while you were in "

  echo "== the push server gone: an error"
  kill "$(cat "$E2E_WORK/ntfy.pid")"
  sway_msg workspace 2 >/dev/null
  settle 1
  step_of working
  step_of stops
  settle 2
  sway_msg workspace 1 >/dev/null
  settle 2

  echo "== a neutral toast"
  palette "workspace: use classic layout" 3

  echo "== a working agent's close: Wait, a warning"
  palette "pane: activate previous item" 2
  step_of working
  palette "pane: close active item" 2
  away
  take 925-01a-close || return 0
  click "$WAIT_X" "$WAIT_Y"
  settle 2
  away
  shot 925-01-toasts

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  away
  shot 925-04-day
}
