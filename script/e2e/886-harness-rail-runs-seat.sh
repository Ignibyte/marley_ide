# shellcheck shell=bash
# #886's visual check: the rail's Harness section, a scheduled task's runs and the New Harness
# Seat form as the boards draw them. #719's scratch harness: a real runtime of the harness's built
# `rh` on a scratch root, harness writes on, a task `checks` every minute whose first run finds
# something and second finds nothing.
# - The rail before any run: the Harness header's dot (`886-01-rail`, REQ-001).
# - After two runs, the task's row: its history as a card, each run's outcome a chip
#   (`886-02-runs`, REQ-002).
# - `marley: new harness seat`: the agent toggle, Cancel and Create (`886-03-seat`, REQ-003).
# - Marley Day, the form (`886-04-day`, REQ-003). Escape closes it; no seat is made.
compositor sway

RH=${RH:-/srv/stacks/rustal-harness/target/debug/rh}
# The task's row in the rail once the finding's Needs you card is above it, from #719's shots.
TASK_Y=${TASK_Y:-390}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

# The harness's CLI on the scratch root, outside any tmux.
rh() { env -u TMUX -u TMUX_PANE "$RH" --state "$ROOT" "$@"; }

bytes() { python3 -c 'import json, sys; print(json.dumps(list(sys.argv[1].encode())))' "$1"; }

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

# An actor script that takes the run's prompt, works a moment, and reports `$1` with the summary
# `$2` in the step that ends the send.
write_actor() {
  cat <<JSON
{"version": 1, "steps": [
  {"step": "publish", "state": "idle", "value": {}},
  {"step": "wait", "key": "task"},
  {"step": "output", "bytes": $(bytes $'checking the parser\n')},
  {"step": "publish", "state": "running", "value": {}},
  {"step": "sleep", "millis": 1500},
  {"step": "publish", "state": "idle", "value": {"run_report": {"outcome": "$1", "summary": "$2"}}},
  {"step": "sleep", "millis": 60000}]}
JSON
}

setup() {
  local repo=$E2E_WORK/repo home=$E2E_WORK/home bin=$E2E_WORK/bin tries=50
  "$RH" schedule --help >/dev/null 2>&1 || {
    echo "no rh with scheduled tasks: build the harness at its TICKET-118 first" >&2
    return 1
  }
  # A short root: the harness's sockets live under it.
  ROOT=$XDG_RUNTIME_DIR/rh719-$$
  echo "$ROOT" >"$E2E_WORK/root"
  mkdir -m 0700 "$ROOT" "$ROOT/profiles"
  (rh serve >"$ROOT/serve.log" 2>&1 & echo "$!" >"$E2E_WORK/serve.pid")
  until grep -q '"ready":true' "$ROOT/serve.log" 2>/dev/null || ((tries-- == 0)); do
    sleep 0.2
  done
  mkdir -p "$repo" "$home" "$bin"
  write_actor finding "two tests fail in the parser" >"$ROOT/run.json"
  printf '{"version": 1, "kind": "actor", "cwd": "%s", "script": "%s"}\n' "$repo" "$ROOT/run.json" \
    >"$ROOT/profiles/tester.json"
  chmod 0600 "$ROOT/profiles/tester.json"
  rh schedule add checks --profile tester --cwd "$repo" --prompt "Run the checks" --every 1 >/dev/null
  rh schedule add nightly --profile tester --cwd "$repo" --prompt "Run the nightly checks" \
    --daily 03:00 >/dev/null
  rh schedule pause nightly >/dev/null
  # Marley's private session bus, with a notification server that logs each banner. The runtime
  # started before it: it launches a run's session through the user's own bus.
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  write_notification_server
  python3 -u "$E2E_WORK/notifications.py" "$E2E_WORK/banners.log" >"$E2E_WORK/notifications.out" 2>&1 &
  echo "$!" >"$E2E_WORK/notifications.pid"
  : >"$E2E_WORK/banners.log"
  # Marley reaches the harness outside any tmux too.
  cat >"$bin/rh" <<SH
#!/bin/bash
exec env -u TMUX -u TMUX_PANE "$RH" "\$@"
SH
  chmod +x "$bin/rh"
  profile_setting marley.harness "{\"command\": \"$bin/rh\", \"args\": [\"--state\", \"$ROOT\", \"mcp\", \"--grant\", \"write\"]}"
  profile_setting marley.harness_writes true
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  open_path "$repo"
}

teardown() {
  [[ -f $E2E_WORK/root ]] || return 0
  ROOT=$(cat "$E2E_WORK/root")
  chmod 0700 "$ROOT" 2>/dev/null
  rh schedule remove checks >/dev/null 2>&1
  rh schedule remove nightly >/dev/null 2>&1
  rh shutdown --stop-backend >/dev/null 2>&1
  # A runtime that did not take the shutdown.
  [[ -f $E2E_WORK/serve.pid ]] && kill "$(cat "$E2E_WORK/serve.pid")" 2>/dev/null
  rm -rf "${ROOT:?}"
  [[ -f $E2E_WORK/notifications.pid ]] && kill "$(cat "$E2E_WORK/notifications.pid")" 2>/dev/null
  [[ -f $E2E_WORK/bus.pid ]] && kill "$(head -1 "$E2E_WORK/bus.pid")" 2>/dev/null
  return 0
}

# How many of the task `checks`'s runs have ended.
ended_runs() {
  rh schedule runs checks | python3 -c 'import json, sys
print(sum(1 for run in json.load(sys.stdin)["runs"] if run["outcome"]))'
}

# Waits until `checks` has `$1` ended runs, at most four minutes.
wait_runs() {
  local tries=120
  until (($(ended_runs) >= $1)) || ((tries-- == 0)); do
    sleep 2
  done
  rh schedule runs checks
}

# How many banners say a task found something.
finding_banners() {
  grep -c 'found something' "$E2E_WORK/banners.log" || true
}

steps() {
  ROOT=$(cat "$E2E_WORK/root")
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 18

  echo "== the rail's Harness section"
  pointer_to 900 500
  take 886-01-rail || return 0

  echo "== two runs, then the task's history"
  wait_runs 1
  write_actor nothing "the checks passed" >"$ROOT/run.json"
  wait_runs 2
  settle 20
  click 120 "$TASK_Y"
  settle 4
  pointer_to 900 700
  take 886-02-runs || return 0

  echo "== the seat form"
  press "CTRL SHIFT" p
  settle 2
  type_text "marley: new harness seat"
  settle 3
  press "" Return
  settle 3
  pointer_to 1500 950
  take 886-03-seat || return 0

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  take 886-04-day || return 0
  press "" Escape
  settle 1
}
