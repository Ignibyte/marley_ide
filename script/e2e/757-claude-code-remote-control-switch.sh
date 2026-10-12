# shellcheck shell=bash
# #757's visual check: Claude Code's Remote Control switch. #535's set-up: a fake ntfy on a loopback
# port logs each push, and Marley runs on a private session bus whose notification server logs each
# banner, so none reaches the user's desktop. A stand-in `claude` logs its arguments, reports a
# session through the plugin's real `event.py`, and acts out one finished turn when the scenario
# writes to a FIFO, then exits. Each Claude Code starts the way #557's English path starts one: a
# sentence at the shell's prompt and Ctrl+Shift+Enter, with no agent running.
# Shots: with `marley.claude_code_remote_control` on, the new Claude Code's row shows `remote`
# beside its title (`757-01-chip`, REQ-001, REQ-002; its arguments hold `--remote-control --`);
# its turn ends while Marley's window is away, and ntfy logs nothing (`757-02-quiet`, REQ-003);
# with the setting off, a second Claude Code starts without the flag and its row has no chip
# (`757-03-off`, REQ-004), and its turn pushes one line.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  # A private session bus with a notification server that logs each banner.
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  write_notification_server
  python3 -u "$E2E_WORK/notifications.py" "$E2E_WORK/banners.log" >"$E2E_WORK/notifications.out" 2>&1 &
  echo "$!" >"$E2E_WORK/notifications.pid"
  write_fake_ntfy
  python3 -u "$E2E_WORK/ntfy.py" "$E2E_WORK/pushes.log" >"$E2E_WORK/ntfy.out" 2>&1 &
  echo "$!" >"$E2E_WORK/ntfy.pid"
  for _ in $(seq 50); do
    PORT=$(grep -oE 'port [0-9]+' "$E2E_WORK/ntfy.out" | cut -d' ' -f2 || true)
    [[ -n $PORT ]] && break
    sleep 0.1
  done
  [[ -n $PORT ]] || { echo "the fake ntfy did not start" >&2; return 1; }
  : >"$E2E_WORK/pushes.log"
  : >"$E2E_WORK/args.log"
  mkfifo "$E2E_WORK/steps.fifo"
  write_stand_in "$bin/claude"
  profile_setting marley.push "{\"url\": \"http://127.0.0.1:$PORT\", \"topic\": \"marley-e2e\"}"
  profile_setting marley.claude_code_remote_control true
  git init -q -b main "$E2E_WORK/repo"
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
# A fake ntfy on a free loopback port: logs each request's method, path, headers and body, one
# JSON line each, and answers as ntfy does.
import http.server
import json
import sys

LOG = sys.argv[1]


class Handler(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length") or 0)
        record = {"method": "POST", "path": self.path, "headers": dict(self.headers.items()),
                  "body": self.rfile.read(length).decode("utf-8", "replace")}
        with open(LOG, "a", encoding="utf-8") as log:
            log.write(json.dumps(record) + "\n")
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

# A stand-in Claude Code: it logs its arguments, starts a session through the plugin's hook, and
# on the FIFO's first line acts out a finished turn, then exits.
write_stand_in() {
  sed -e "s|@HOOKS@|$HOOKS|" -e "s|@FIFO@|$E2E_WORK/steps.fifo|" -e "s|@ARGS@|$E2E_WORK/args.log|" \
    >"$1" <<'FAKE'
#!/usr/bin/env python3
import json
import os
import subprocess
import sys

if sys.argv[1:2] == ["plugin"]:
    sys.exit(0)
with open("@ARGS@", "a", encoding="utf-8") as log:
    log.write(json.dumps(sys.argv[1:]) + "\n")
COMMON = {"session_id": "e2e-" + str(os.getpid()), "transcript_path": "/tmp/e2e-transcript.jsonl",
          "cwd": os.getcwd(), "permission_mode": "default"}


def write(answer):
    sequence = json.loads(answer or "{}").get("terminalSequence")
    if sequence:
        sys.stdout.write(sequence)
        sys.stdout.flush()


def event(fields):
    write(subprocess.run([sys.executable, "@HOOKS@/event.py"], input=json.dumps({**COMMON, **fields}),
                         capture_output=True, text=True, check=False).stdout)


print("Claude Code (stand-in): " + " ".join(sys.argv[1:]), flush=True)
event({"hook_event_name": "SessionStart", "source": "startup"})
with open("@FIFO@", encoding="utf-8") as fifo:
    fifo.readline()
event({"hook_event_name": "UserPromptSubmit", "prompt": "Tidy the imports"})
event({"hook_event_name": "Stop", "last_assistant_message": "Tidied."})
write(subprocess.run(["sh", "@HOOKS@/notify.sh", "finished"], capture_output=True, text=True,
                     check=False).stdout)
print("the turn finished", flush=True)
FAKE
  chmod +x "$1"
}

# The stand-in's turn, with Marley's window on another workspace so the user is not looking.
turn_away() {
  sway_msg workspace 2 >/dev/null
  settle 1
  echo next >"$E2E_WORK/steps.fifo"
  settle 3
  sway_msg workspace 1 >/dev/null
  settle 1
}

# Whether the fake ntfy has logged exactly this many requests.
pushed() {
  local count
  count=$(wc -l <"$E2E_WORK/pushes.log")
  echo "pushes so far: $count"
  [[ $count -eq $1 ]]
}

# Starts Claude Code with `$1` as its prompt, the way #557's English path does, from the first tab.
ask_in_english() {
  press ALT 1
  settle 1
  type_text "$1"
  settle 1
  press "CTRL SHIFT" Return
  settle 5
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== the switch on: Claude Code starts with --remote-control"
  ask_in_english "find the large files in this repo"
  cat "$E2E_WORK/args.log"
  expect "the first Claude Code has the flag and -- before its prompt" \
    holds "$E2E_WORK/args.log" '["--remote-control", "--", "find the large files in this repo"]'
  take 757-01-chip || return 0

  echo "== its turn ends while the window is away: no push"
  turn_away
  take 757-02-quiet || return 0
  expect "a Remote Control session pushes nothing" pushed 0

  echo "== the switch off: a second Claude Code, no flag, and its push"
  profile_setting marley.claude_code_remote_control false
  settle 3
  ask_in_english "find the old logs"
  cat "$E2E_WORK/args.log"
  expect "the second Claude Code has no flag" holds "$E2E_WORK/args.log" '["find the old logs"]'
  take 757-03-off || return 0
  turn_away
  cat "$E2E_WORK/pushes.log"
  expect "the second session pushed its line" pushed 1
}
