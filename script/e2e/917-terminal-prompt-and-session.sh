# shellcheck shell=bash
# #917's visual check: the terminal's prompt card, the sticky header's words, the session chips and
# Tell me when done. Project `repo` (two files changed), the prompt editor on (the default), a tmux
# server of the run's own (`TMUX_TMPDIR`) with a session `build` started in `repo`, and a private
# session bus whose notification server logs each banner, so none reaches the user's desktop.
# - `917-01-prompt`: `false` ran, `git log --o` typed: the card with the folder, branch, "2
#   changed" and "last: exit 1", `❯`, the suggestion and "→ to accept", the keys and Start an
#   agent here (REQ-001, REQ-002).
# - `917-02-start-agent`: Start an agent here clicked: New Agent asks only which agent (REQ-002).
# - `917-03-sticky`: `seq 300; sleep 8` running: the pinned row "… is above you · running N s" and
#   Jump to it (REQ-003).
# - `917-04-jumped`: Jump to it clicked: the block's start at the top (REQ-003).
# - `917-05-chips`: the tab row's right end: the shell, the folder and the branch (REQ-004).
# - `917-05-tmux`: `build` attached from its rail row: "tmux · build · 1 window" (REQ-004).
# - `917-06-tell`: `sleep 5` with Tell me when done clicked: "Will tell you"; after the tab moved
#   away and the block ended, the notification server's log holds the banner (REQ-005).
# - `917-07-day`: Marley Day, the prompt card and the chips (REQ-001, REQ-004).
compositor sway

AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-300}
# From the probe run's shots.
START_X=${START_X:-1531}
START_Y=${START_Y:-941}
JUMP_X=${JUMP_X:-1552}
JUMP_Y=${JUMP_Y:-77}
BUILD_X=${BUILD_X:-76}
BUILD_Y=${BUILD_Y:-226}
TELL_X=${TELL_X:-397}
TELL_Y=${TELL_Y:-935}

# tmux's socket path must stay under 108 bytes, which a run's work folder does not (#808).
SOCKETS=${XDG_RUNTIME_DIR:-/tmp}/marley-e2e-917
LOCAL_TMUX=$SOCKETS/local

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo" "$LOCAL_TMUX"
  printf "PS1=%q\nexport PATH=%q:\"\$PATH\"\n" '$ ' "$bin" >"$home/.bashrc"
  terminal_env HOME "$home"
  terminal_env TMUX_TMPDIR "$LOCAL_TMUX"
  export TMUX_TMPDIR=$LOCAL_TMUX
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  write_notification_server
  python3 -u "$E2E_WORK/notifications.py" "$E2E_WORK/banners.log" >"$E2E_WORK/notifications.out" 2>&1 &
  echo "$!" >"$E2E_WORK/notifications.pid"
  : >"$E2E_WORK/banners.log"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  git -C "$repo" add -A
  git -C "$repo" -c user.name=t -c user.email=t@t commit -qm init
  printf '# repo\n\nChanged.\n' >"$repo/README.md"
  printf 'notes\n' >"$repo/notes.md"
  TMUX_TMPDIR=$LOCAL_TMUX HOME=$home tmux -f /dev/null new-session -d -s build -c "$repo" \
    -x 100 -y 30 "sleep 600"
  open_path "$repo"
}

teardown() {
  [[ -f $E2E_WORK/notifications.pid ]] && kill "$(cat "$E2E_WORK/notifications.pid")" 2>/dev/null
  [[ -f $E2E_WORK/bus.pid ]] && kill "$(head -1 "$E2E_WORK/bus.pid")" 2>/dev/null
  TMUX_TMPDIR=$LOCAL_TMUX tmux kill-server 2>/dev/null || true
  rm -rf "$SOCKETS"
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

run() {
  type_text "$1"
  press "" Return
  settle "${2:-3}"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 12
  # Trusts repo.
  press "" Return
  settle 4
  run "git log --oneline -5 --graph"
  run false
  type_text "git log --o"
  settle 2
  away
  shot 917-01-prompt
  [ -n "${STOP_AT_PROMPT:-}" ] && return

  echo "== Start an agent here"
  click "$START_X" "$START_Y"
  settle 3
  away
  shot 917-02-start-agent
  press "" Escape
  settle 2
  press "CTRL" a
  press "" BackSpace
  settle 1

  echo "== the sticky header"
  run "seq 300; sleep 8" 2
  # Scrolled back into the output: the block's start is above the view.
  press SHIFT Page_Up
  settle 1
  away
  shot 917-03-sticky
  [ -n "${STOP_AT_STICKY:-}" ] && return
  click "$JUMP_X" "$JUMP_Y"
  settle 2
  away
  shot 917-04-jumped
  settle 6

  echo "== the session chips"
  away
  shot 917-05-chips
  click "$BUILD_X" "$BUILD_Y"
  settle 4
  away
  shot 917-05-tmux

  echo "== Tell me when done"
  press "CTRL" Page_Up
  settle 2
  run "sleep 5" 2
  click "$TELL_X" "$TELL_Y"
  settle 1
  away
  shot 917-06-tell
  press "CTRL" Page_Down
  settle 7
  echo "banners: $(tr '\n' ';' <"$E2E_WORK/banners.log")"
  expect "the block's end was told" holds "$E2E_WORK/banners.log" "sleep 5"

  echo "== Marley Day"
  press "CTRL" Page_Up
  settle 2
  profile_setting theme '"Marley Day"'
  settle 4
  away
  shot 917-07-day
}
