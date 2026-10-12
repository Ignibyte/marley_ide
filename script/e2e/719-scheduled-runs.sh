# shellcheck shell=bash
# #719's visual check: scheduled runs from the harness. A real runtime of the harness's built `rh`
# (with its TICKET-118: `rh schedule`, the read tools, a run's labels and an actor's `run_report`)
# on a scratch root is the harness Marley follows, with harness writes on. An actor profile,
# `tester`, takes the run's prompt and reports through a `publish` step; the scenario swaps its
# script after the first run, so the first run of the task `checks` (`--every 1`) reports `finding`
# and the second `nothing`. A second task, `nightly` (`--daily 03:00`), is paused. Marley runs on a
# private session bus whose notification server logs each banner (#688's), so none reaches the
# user's desktop.
#
# Shots: the Harness section lists both tasks, "every 1 min · next …" and "daily 03:00 · paused",
# and no run's session among the sessions (`719-01-rail`, REQ-001, REQ-007); after the first run,
# Needs you lists "checks found something: …" and one banner shows (`719-02-finding`, REQ-002);
# after the second run, still one banner (REQ-003); the task's history, newest first, a nothing then
# a finding (`719-03-history`, REQ-004); the finding's entry opens the run's session, the entry is
# gone and the session carries `finding.seen` (`719-04-run`, REQ-005, REQ-006).
compositor sway

RH=${RH:-/srv/stacks/rustal-harness/target/debug/rh}
# Points in the rail, from the first run's shots; 0 stops after the shot before the click.
TASK_Y=${TASK_Y:-342}
ENTRY_Y=${ENTRY_Y:-116}
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

  echo "== the tasks in the Harness section, before any run"
  pointer_to 900 500
  take 719-01-rail || return 0

  echo "== the first run finds something: one Needs you entry, one banner"
  wait_runs 1
  # The next run reports nothing.
  write_actor nothing "the checks passed" >"$ROOT/run.json"
  settle 20
  take 719-02-finding || return 0
  cat "$E2E_WORK/banners.log"
  expect "one banner names the task and the run's first line" holds "$E2E_WORK/banners.log" \
    "checks found something|two tests fail in the parser"

  echo "== the second run finds nothing: no second banner"
  wait_runs 2
  settle 20
  cat "$E2E_WORK/banners.log"
  expect "a run that found nothing shows no banner" [ "$(finding_banners)" = 1 ]
  if ((TASK_Y == 0)); then
    echo "the task's row is not placed yet: 719-02 is the last shot"
    return 0
  fi

  echo "== the task's history"
  click 120 "$TASK_Y"
  settle 4
  take 719-03-history || return 0
  if ((ENTRY_Y == 0)); then
    echo "the entry's place is not set yet: 719-03 is the last shot"
    return 0
  fi

  echo "== the finding's entry opens the run's session and marks it seen"
  click 120 "$ENTRY_Y"
  settle 6
  pointer_to 900 500
  take 719-04-run || return 0
  rh fleet >"$E2E_WORK/fleet.json"
  expect "the run's session carries finding.seen" holds "$E2E_WORK/fleet.json" '"finding.seen": "1"'
}
