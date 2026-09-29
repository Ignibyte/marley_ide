# shellcheck shell=bash
# #572's visual check: a running command that prints a failure and keeps running, told from a
# terminal the user is not looking at, and its rail row's mark. Marley runs on a private session
# bus whose notification server logs each banner, so none reaches the user's desktop, with #565's
# layer on the replay provider and the scratch repository listed. Terminal 1 runs a stand-in dev
# server that prints on signals; terminal 2 holds the focus, and Alt+N moves it. Banners one
# project shows come at least five seconds apart (#538's cooldown).
#
# A failure line still running five seconds later marks the row and posts a banner (REQ-001,
# REQ-002); a recovery clears it and says so (REQ-003); a line the shapes leave open is asked with
# the lines around it, masked (REQ-007): in suggest it marks with a `?` and posts nothing, in act
# it flags as a shape (REQ-008), and a reading in the band changes nothing (REQ-009). An agent's
# and an SSH client's terminals are not watched (REQ-005); the terminal in front is marked with no
# banner (REQ-006); a command that ends inside the grace is not this ticket's (REQ-004); off does
# nothing (REQ-010); the mode is on the Marley page (REQ-011).
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The settings window's search field.
SEARCH_X=${SEARCH_X:-912}
SEARCH_Y=${SEARCH_Y:-59}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_PROFILE/system_one"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  # The user's own bus, watched for a banner that should not reach it.
  stdbuf -oL busctl --address="unix:path=$XDG_RUNTIME_DIR/bus" monitor org.freedesktop.Notifications \
    >"$E2E_WORK/user-bus.log" 2>&1 &
  echo "$!" >"$E2E_WORK/user-bus.pid"
  # A private session bus with a notification server that logs each banner.
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  write_notification_server
  python3 -u "$E2E_WORK/notifications.py" "$E2E_WORK/banners.log" >"$E2E_WORK/notifications.out" 2>&1 &
  echo "$!" >"$E2E_WORK/notifications.pid"
  : >"$E2E_WORK/banners.log"
  # A token assembled here, so no file of the repository holds one.
  local token
  token=$(printf 'gh%s_%s' p "$(printf 'Fake%.0s' {1..9})")
  echo "$token" >"$E2E_WORK/token"
  write_devserver "$bin/devserver" "$token"
  # Stand-ins for an agent CLI and an SSH client, each printing a failure and running on.
  printf '#!/usr/bin/env bash\nexec -a %s python3 -c %q\n' claude \
    'import time; print("error: boom", flush=True); time.sleep(7)' >"$bin/claude"
  printf '#!/usr/bin/env bash\nexec -a %s python3 -c %q\n' ssh \
    'import time; print("error: boom", flush=True); time.sleep(7)' >"$bin/ssh"
  chmod +x "$bin/devserver" "$bin/claude" "$bin/ssh"
  # The recorded answers: the open line a new failure, every time it is asked; the other in the
  # band.
  cat >"$E2E_PROFILE/system_one/replay.jsonl" <<'JSONL'
{"set": "running_error/1", "match": "line: worker 3: job exception", "repeat": true, "answers": {"new_failure": {"type": "noul", "noul": 0.9}, "recovered": {"type": "noul", "noul": 0.05}}}
{"set": "running_error/1", "match": "line: cache: fail count reset", "answers": {"new_failure": {"type": "noul", "noul": 0.5}, "recovered": {"type": "noul", "noul": 0.5}}}
JSONL
  system_one_setting "{\"enabled\": true, \"provider\": \"replay\", \"projects\": [\"$E2E_WORK/repo\"], \"uses\": {\"running_error\": \"shadow\"}}"
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  local pid
  for pid in user-bus notifications; do
    [[ -f $E2E_WORK/$pid.pid ]] && kill "$(cat "$E2E_WORK/$pid.pid")" 2>/dev/null
  done
  [[ -f $E2E_WORK/bus.pid ]] && kill "$(head -1 "$E2E_WORK/bus.pid")" 2>/dev/null
  [[ -f $E2E_WORK/devserver.pid ]] && kill "$(cat "$E2E_WORK/devserver.pid")" 2>/dev/null
  return 0
}

# Merges the JSON object `$1` into `marley.system_one` in the run's copy of the settings, `uses`
# one level deeper.
system_one_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'SETTINGS'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
# Comments and trailing commas out: the settings file allows them and JSON does not.
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
layer = settings.setdefault("marley", {}).setdefault("system_one", {})
for key, value in json.loads(sys.argv[2]).items():
    if key == "uses":
        layer.setdefault("uses", {}).update(value)
    else:
        layer[key] = value
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTINGS
}

mode() {
  system_one_setting "{\"uses\": {\"running_error\": \"$1\"}}"
  settle 2
}

# A stand-in dev server: ready, then on each signal the lines a build tool or a server prints.
write_devserver() {
  sed -e "s|@PID@|$E2E_WORK/devserver.pid|" -e "s|@TOKEN@|$2|" >"$1" <<'PY'
#!/usr/bin/env python3
import os
import signal

open("@PID@", "w").write(str(os.getpid()))


def say(*lines):
    for line in lines:
        print(line, flush=True)


signal.signal(signal.SIGUSR1, lambda *_: say(
    "error: Failed to compile ./src/App.tsx",
    "  12 | import App from './App'",
    "     | Module not found"))
signal.signal(signal.SIGUSR2, lambda *_: say("Compiled successfully."))
signal.signal(signal.SIGRTMIN, lambda *_: say(
    "auth: signed in with @TOKEN@",
    "worker 3: job exception, retrying",
    "worker 3: next try in 5 s"))
signal.signal(signal.SIGRTMIN + 1, lambda *_: say("cache: fail count reset"))
say("ready in 120 ms")
while True:
    signal.pause()
PY
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

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

banners() { wc -l <"$E2E_WORK/banners.log"; }

# Whether the log's last banner is exactly this summary and body.
last_banner_is() {
  local last
  last=$(tail -1 "$E2E_WORK/banners.log")
  echo "last banner: $last"
  [[ $last == "Marley|$1|$2" ]]
}

signal_server() { kill "-$1" "$(cat "$E2E_WORK/devserver.pid")"; }

# The running error's call rows, one JSON object a line.
calls() {
  cat "$E2E_PROFILE"/system_one/calls-*.jsonl 2>/dev/null | grep '"row":"call"' |
    grep '"use":"running_error"' || true
}

# Writes the last call row to `$1.json` and prints its gist.
last_call() {
  calls | tail -n 1 >"$E2E_WORK/$1.json"
  python3 - "$E2E_WORK/$1.json" <<'PY'
import json, sys

row = json.load(open(sys.argv[1]))
print(f"  {row['use']} {row['mode']} {row['provider']} {row['set']}: {row['reading']}")
print("  state:", (row.get("state") or "").replace("\n", " | "))
PY
}

# Types `$1` in terminal 2 and runs it, then gives terminal 1 the focus at once.
run_in_two() {
  press ALT 2
  settle 1
  type_text "$1"
  press "" Return
  press ALT 1
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  palette "workspace: new terminal"
  settle 3
  press ALT 1
  settle 1
  type_text "devserver"
  press "" Return
  settle 2
  press ALT 2
  settle 1

  echo "== shadow: a failure line, still running, not looked at"
  signal_server USR1
  settle 7
  shot 572-01-error-flag
  expect "the failure was told" last_banner_is "repo: devserver printed an error" \
    "error: Failed to compile ./src/App.tsx"
  last_call flag
  expect "the shapes' flag is a rules row" holds "$E2E_WORK/flag.json" '"provider":"rules"' \
    '"set":"running_error/1"' 'new_failure'

  echo "== a recovery"
  settle 4
  signal_server USR2
  settle 2
  shot 572-02-recovered
  expect "the recovery was told" last_banner_is "repo: devserver recovered" "Compiled successfully."

  echo "== suggest: an open line the reading calls a failure"
  mode suggest
  local before
  before=$(banners)
  signal_server RTMIN
  settle 7
  shot 572-03-suggest
  expect "a questioned mark posts nothing" test "$(banners)" = "$before"
  last_call suggest
  expect "the open line was asked with the lines around it, masked" holds "$E2E_WORK/suggest.json" \
    '"provider":"replay"' '"mode":"suggest"' 'line: worker 3: job exception, retrying' \
    'before: auth: signed in with [redacted' 'after: worker 3: next try in 5 s'
  expect "the token never left" bash -c "! grep -q FakeFake '$E2E_WORK/suggest.json'"
  signal_server USR2
  settle 2
  expect "a questioned episode's recovery posts nothing" test "$(banners)" = "$before"

  echo "== act: the reading flags as a shape"
  mode act
  signal_server RTMIN
  settle 7
  shot 572-04-open-case
  expect "the reading's failure was told" last_banner_is "repo: devserver printed an error" \
    "worker 3: job exception, retrying"

  echo "== a reading in the band"
  settle 4
  signal_server USR2
  settle 6
  before=$(banners)
  signal_server RTMIN+1
  settle 7
  shot 572-05-no-signal
  expect "the band changed nothing" test "$(banners)" = "$before"
  last_call band
  expect "the band's call was logged" holds "$E2E_WORK/band.json" 'line: cache: fail count reset'
  expect "a last line sends no line after it" bash -c "! grep -q 'after:' '$E2E_WORK/band.json'"

  echo "== an agent's and an SSH client's terminals"
  run_in_two claude
  settle 8
  run_in_two ssh
  settle 8
  shot 572-05b-agent-and-ssh
  expect "neither was watched" test "$(banners)" = "$before"

  echo "== the terminal in front"
  signal_server USR1
  settle 7
  shot 572-06-focused
  expect "the front terminal posted nothing" test "$(banners)" = "$before"
  signal_server USR2
  settle 2

  echo "== a command that ends inside the grace"
  run_in_two "sh -c 'echo error: bad; sleep 1; exit 1'"
  settle 7
  shot 572-07-exit-not-ours
  expect "an end inside the grace posted nothing" test "$(banners)" = "$before"

  echo "== off"
  mode off
  local rows
  rows=$(calls | wc -l)
  press ALT 2
  settle 1
  signal_server USR1
  settle 7
  shot 572-08-off
  expect "off posted nothing" test "$(banners)" = "$before"
  expect "off made no call" test "$(calls | wc -l)" = "$rows"

  echo "== the mode on the Marley page"
  palette "marley: open settings"
  settle 4
  click "$SEARCH_X" "$SEARCH_Y"
  settle 1
  type_text "Running Error"
  settle 2
  shot 572-09-setting

  echo "== the user's own bus got nothing"
  cat "$E2E_WORK/banners.log"
  # Other programs post to the user's bus as they will; only Marley's name counts (#535's check).
  expect "no banner reached the user's bus" bash -c "! grep -q 'STRING \"Marley\"' '$E2E_WORK/user-bus.log'"
}
