# shellcheck shell=bash
# #859's visual check: a project's mark in its notifications and the rail's switcher. A project
# with `logo.png`; Marley on a private session bus whose notification server logs each banner
# with its image hint (#538's); two terminals.
# - The first terminal prints an OSC 777 notify four seconds on, while the second has the focus:
#   the banner's logged line names the project's logo file, printed in the second terminal
#   (`859-01-banner`, REQ-001).
# - The rail's switcher: each row's project mark beside its name (`859-02-switcher`, REQ-002),
#   and in Marley Day (`859-03-day`, REQ-002).
compositor sway

STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  magick -size 64x64 xc:'#ff8c42' -fill '#9be15d' -draw 'circle 32,32 32,12' "$repo/logo.png"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  write_notification_server
  python3 -u "$E2E_WORK/notifications.py" "$E2E_WORK/banners.log" >"$E2E_WORK/notifications.out" 2>&1 &
  echo "$!" >"$E2E_WORK/notifications.pid"
  : >"$E2E_WORK/banners.log"
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

teardown() {
  [[ -f $E2E_WORK/notifications.pid ]] && kill "$(cat "$E2E_WORK/notifications.pid")" 2>/dev/null
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
            log.write(f"{summary}|{body}|image-path={hints.get('image-path', '')}\n")
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
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== a notify from the first terminal, the second in front"
  type_text "sleep 4; printf '\\033]777;notify;Build;finished\\a'"
  press "" Return
  palette "workspace: new terminal" 3
  settle 6
  type_text "cat $E2E_WORK/banners.log"
  press "" Return
  settle 2
  cat "$E2E_WORK/banners.log"
  pointer_to 1500 950
  settle 1
  take 859-01-banner || return 0

  echo "== the rail's switcher"
  palette "agents sidebar: toggle thread switcher" 2
  pointer_to 1500 950
  settle 1
  take 859-02-switcher || return 0
  press "" Escape
  settle 1

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  palette "agents sidebar: toggle thread switcher" 2
  pointer_to 1500 950
  settle 1
  take 859-03-day || return 0
  press "" Escape
}
