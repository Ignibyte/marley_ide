# shellcheck shell=bash
# #551's visual check: a long command's end, and a password prompt, told from a terminal the user
# is not looking at, and the terminal's rail row with its command. Marley runs on a private
# session bus whose notification server logs each banner, so none reaches the user's desktop, with
# `marley.long_command_seconds` at 2. Terminal 1 runs the commands; terminal 2 holds the focus, and
# Alt+N moves between them; banners one project shows come at least five seconds apart (#538's
# cooldown). A failed long command posts `exit 1 after 3 s` and its row reads the
# exit in red (REQ-001, REQ-004); a short one posts nothing (REQ-002); a running one's row says so,
# then `done` (REQ-003, REQ-004); `read -s` says it waits for a password (REQ-005); a long command
# in the focused terminal posts nothing (REQ-002); the rail's filter finds the terminal by its
# command (REQ-006); an agent's long block posts nothing (REQ-007); the setting is on the Marley
# page (REQ-008).
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The rail's filter field, and where the settings window scrolls to its Terminal section.
FILTER_X=${FILTER_X:-100}
FILTER_Y=${FILTER_Y:-54}
SETTINGS_X=${SETTINGS_X:-1200}
SETTINGS_Y=${SETTINGS_Y:-500}
SETTINGS_SCROLL=${SETTINGS_SCROLL:-20}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin"
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
  # A stand-in agent whose session is one long block.
  printf '#!/usr/bin/env bash\nexec -a claude sleep 4\n' >"$bin/claude"
  chmod +x "$bin/claude"
  long_commands_after 2
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  local pid
  for pid in user-bus notifications; do
    [[ -f $E2E_WORK/$pid.pid ]] && kill "$(cat "$E2E_WORK/$pid.pid")" 2>/dev/null
  done
  [[ -f $E2E_WORK/bus.pid ]] && kill "$(head -1 "$E2E_WORK/bus.pid")" 2>/dev/null
  return 0
}

# Sets `marley.long_command_seconds` in the run's copy of the settings.
long_commands_after() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'SETTINGS'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
# Comments and trailing commas out: the settings file allows them and JSON does not.
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
settings.setdefault("marley", {})["long_command_seconds"] = int(sys.argv[2])
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTINGS
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

# Types `$1` in terminal 1 and runs it, then gives terminal 2 the focus at once.
run_away() {
  press ALT 1
  settle 1
  type_text "$1"
  press "" Return
  press ALT 2
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  palette "workspace: new terminal"
  settle 3

  echo "== a failed long command, not looked at: a banner, and its row"
  run_away "sleep 3; false"
  settle 6
  shot 551-01-failed-row
  expect "its end was told with its exit" last_banner_is "sleep 3; false" "exit 1 after 3 s"

  echo "== a short command: nothing"
  local before
  before=$(banners)
  run_away "true"
  settle 3
  expect "a short command told nothing" test "$(banners)" = "$before"

  echo "== a running command's row, then its end"
  run_away "sleep 7"
  settle 3
  shot 551-02-running-row
  settle 7
  shot 551-03-done-row
  expect "its end was told as done" last_banner_is "sleep 7" "done in 7 s"

  echo "== a password prompt"
  # Past the last banner's cooldown; the prompt comes a second late, once the focus has moved.
  settle 5
  run_away "sleep 1; read -s -p 'Password: ' x"
  settle 4
  shot 551-04-password-row
  expect "the prompt was told once" last_banner_is "sleep 1; read -s -p 'Password: ' x" "waiting for a password"
  press ALT 1
  settle 1
  type_text "secret"
  press "" Return
  settle 1

  echo "== a long command in the focused terminal: nothing"
  settle 5
  before=$(banners)
  type_text "sleep 3; false"
  press "" Return
  settle 6
  expect "the focused terminal's end told nothing" test "$(banners)" = "$before"

  echo "== the rail's filter finds the terminal by its command"
  click "$FILTER_X" "$FILTER_Y"
  settle 1
  type_text "sleep"
  settle 2
  shot 551-05-filter
  press "" Escape
  settle 1

  echo "== an agent's long block: nothing"
  settle 5
  before=$(banners)
  run_away "claude"
  settle 7
  expect "the agent's block told nothing" test "$(banners)" = "$before"

  echo "== the setting on the Marley page"
  palette "marley: open settings"
  settle 4
  pointer_to "$SETTINGS_X" "$SETTINGS_Y"
  settle 1
  scroll "$SETTINGS_SCROLL"
  settle 2
  shot 551-06-setting

  echo "== the user's own bus got nothing"
  cat "$E2E_WORK/banners.log"
  expect "no banner reached the user's bus" bash -c "! grep -q 'Member=Notify' '$E2E_WORK/user-bus.log'"
}
