# shellcheck shell=bash
# #688's visual check: a notice for the manager's reports. Marley runs on a private session bus whose
# notification server logs each banner (#538's), so none reaches the user's desktop. A scratch
# harness (its built `bin/rh`) runs one terminal, `mgr`, whose fixture script reports idle, is
# designated the root's manager with `rh manager`, and posts a report to the thread through `rh mcp
# --grant agent`'s `thread_post` each time the scenario drops a cue file. `marley.harness` is `rh
# --state <root> mcp --grant write` and `marley.harness_writes` is on.
#
# A report while the project is in front shows its banner and a Needs you entry, `Manager · Report:
# …` (`688-01-needs-you`, REQ-001, REQ-002); a click on it opens the Manager thread, and with the
# thread in front the entry goes (`688-02-in-front`) and a second report shows no banner but
# arrives in the thread (`688-03-streamed`, REQ-002).
compositor sway

RH=${RH:-/srv/stacks/rustal-harness/bin/rh}
REAL_RH=/srv/stacks/rustal-harness/target/debug/rh
# The Needs you entry's place, from the first run's shot; 0 stops after it.
ENTRY_Y=${ENTRY_Y:-165}

# The harness's CLI on the scratch root, outside any tmux.
rh() { env -u TMUX -u TMUX_PANE "$RH" --state "$ROOT" "$@"; }

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

# The manager's fixture: reports idle, then posts a report for each cue file, `post1` and `post2`.
write_manager() {
  cat >"$E2E_WORK/post.py" <<'PY'
# Posts one record to the thread as the root's manager, through `rh mcp --grant agent` from inside
# the manager's harness terminal.
import json
import subprocess
import sys

rh, root, kind, text = sys.argv[1:5]
server = subprocess.Popen([rh, "--state", root, "mcp", "--grant", "agent"], stdin=subprocess.PIPE,
                          stdout=subprocess.PIPE, text=True)


def call(message):
    server.stdin.write(json.dumps(message) + "\n")
    server.stdin.flush()
    return json.loads(server.stdout.readline()) if "id" in message else None


call({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
    "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "fixture", "version": "0"}}})
call({"jsonrpc": "2.0", "method": "notifications/initialized"})
print(json.dumps(call({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
    "name": "thread_post", "arguments": {"kind": kind, "text": text}}})))
PY
  cat >"$E2E_WORK/mgr.sh" <<SH
#!/bin/bash
env -u TMUX "$REAL_RH" --state "$ROOT" report --source fixture --seq 1 idle
for cue in post1 post2; do
  while [[ ! -f "$ROOT/\$cue" ]]; do sleep 0.3; done
  python3 "$E2E_WORK/post.py" "$REAL_RH" "$ROOT" report "The gate passed on TICKET-\$cue
the rest of the report" >>"$E2E_WORK/posted.log" 2>&1
done
sleep 600
SH
  chmod +x "$E2E_WORK/mgr.sh"
}

setup() {
  [[ -x $REAL_RH ]] || {
    echo "no built rh: build the harness first (its scripts/setup.sh)" >&2
    return 1
  }
  # A private session bus with a notification server that logs each banner.
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  write_notification_server
  python3 -u "$E2E_WORK/notifications.py" "$E2E_WORK/banners.log" >"$E2E_WORK/notifications.out" 2>&1 &
  echo "$!" >"$E2E_WORK/notifications.pid"
  : >"$E2E_WORK/banners.log"
  # A short root: the harness's sockets live under it.
  ROOT=$XDG_RUNTIME_DIR/rh688-$$
  echo "$ROOT" >"$E2E_WORK/root"
  mkdir -m 0700 "$ROOT"
  (rh serve >"$ROOT/serve.log" 2>&1 &)
  local tries=50
  until grep -q '"ready":true' "$ROOT/serve.log" 2>/dev/null || ((tries-- == 0)); do
    sleep 0.2
  done
  write_manager
  rh new mgr --cwd "$ROOT" -- "$E2E_WORK/mgr.sh" >/dev/null
  local id
  tries=20
  until [[ -n ${id:-} ]] || ((tries-- == 0)); do
    id=$(rh fleet | python3 -c 'import json, sys
for seat in json.load(sys.stdin).get("seats", []):
    if seat["id"].startswith("agent/"):
        print(seat["id"])')
    [[ -n $id ]] || sleep 0.5
  done
  rh manager "$id" >/dev/null
  profile_setting marley.harness "{\"command\": \"$RH\", \"args\": [\"--state\", \"$ROOT\", \"mcp\", \"--grant\", \"write\"]}"
  profile_setting marley.harness_writes true
  mkdir -p "$E2E_WORK/repo" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

teardown() {
  [[ -f $E2E_WORK/root ]] || return 0
  ROOT=$(cat "$E2E_WORK/root")
  chmod 0700 "$ROOT" 2>/dev/null
  local ws
  for ws in $(rh list 2>/dev/null | python3 -c 'import json, sys
print(" ".join(w["id"] for w in json.load(sys.stdin).get("workspaces", []) if w.get("state") == "running"))'); do
    rh stop "$ws" >/dev/null 2>&1
  done
  rh shutdown --stop-backend >/dev/null 2>&1
  rm -rf "$ROOT"
  [[ -f $E2E_WORK/notifications.pid ]] && kill "$(cat "$E2E_WORK/notifications.pid")" 2>/dev/null
  [[ -f $E2E_WORK/bus.pid ]] && kill "$(head -1 "$E2E_WORK/bus.pid")" 2>/dev/null
  return 0
}

# How many banners name the manager.
manager_banners() {
  grep -c 'Manager: Report' "$E2E_WORK/banners.log" || true
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== a report while the project is in front"
  touch "$(cat "$E2E_WORK/root")/post1"
  settle 5
  cat "$E2E_WORK/posted.log"
  pointer_to 900 400
  settle 1
  shot 688-01-needs-you
  cat "$E2E_WORK/banners.log"
  expect "the banner names the manager and the report's first line" holds "$E2E_WORK/banners.log" \
    "Manager: Report" "The gate passed on TICKET-post1"
  if ((ENTRY_Y == 0)); then
    echo "the entry's place is not set yet: the Needs you shot only"
    return 0
  fi

  echo "== the entry opens the Manager thread, and the thread in front quiets the next"
  click 120 "$ENTRY_Y"
  settle 5
  shot 688-02-in-front
  touch "$(cat "$E2E_WORK/root")/post2"
  settle 5
  shot 688-03-streamed
  cat "$E2E_WORK/banners.log"
  expect "a report with the thread in front shows no banner" [ "$(manager_banners)" = 1 ]
}
