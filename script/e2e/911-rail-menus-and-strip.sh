# shellcheck shell=bash
# #911's visual check: the rail's menus and strip. A scratch repository `a` with a shell that ran
# `echo hello`, and a stand-in Claude Code (Python, so Zed names it by its script; never the
# box's own, PR-687, F-claude-903) that reports one finished turn through Marley's hooks on a
# private bus and echoes what it is sent.
# - `911-01-project-menu`: `a`'s header menu: Fold, then Close Project in red, last (REQ-001,
#   REQ-002).
# - `911-02-folded`: Fold chosen: `a` folded (REQ-001).
# - `911-03-row-menu`: the shell row's menu in the board's order, Close in red (REQ-003).
# - `911-04-turns`: Turns (1) chosen on the agent's row: its turns listed (REQ-004).
# - `911-05-sent`: Send Block to Agent on the shell row: the block reaches the stand-in (REQ-005).
# - `911-06-split`: Split Right: a new shell at the right (REQ-006).
# - `911-07-empty-menu`: the empty space's menu: "New Terminal in Default" (REQ-007).
# - `911-08-group-field`: New Group…: the field at the rail's foot (REQ-008).
# - `911-09-group-made`: "Docs" and Enter: the group made, the field gone (REQ-008).
# - `911-10-strip-plus`: the strip with its `+` at the foot (REQ-009).
# - `911-11-strip-popover`: the `+` clicked: the Add Project popover (REQ-009).
# - `911-12-day`: the row menu in Marley Day (REQ-003).
compositor sway

HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the probe run's shots.
HEADER_X=${HEADER_X:-120}
HEADER_Y=${HEADER_Y:-136}
FOLD_X=${FOLD_X:-144}
FOLD_Y=${FOLD_Y:-516}
SHELL_X=${SHELL_X:-120}
SHELL_Y=${SHELL_Y:-180}
AGENT_X=${AGENT_X:-120}
AGENT_Y=${AGENT_Y:-240}
TURNS_X=${TURNS_X:-131}
TURNS_Y=${TURNS_Y:-326}
SEND_X=${SEND_X:-131}
SEND_Y=${SEND_Y:-265}
SPLIT_X=${SPLIT_X:-131}
SPLIT_Y=${SPLIT_Y:-289}
EMPTY_X=${EMPTY_X:-120}
EMPTY_Y=${EMPTY_Y:-700}
GROUP_X=${GROUP_X:-131}
GROUP_Y=${GROUP_Y:-717}
PLUS_X=${PLUS_X:-30}
PLUS_Y=${PLUS_Y:-980}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/a"
  sed -e "s|@BIN@|$bin|" -e "s|@WORK@|$E2E_WORK|" >"$home/.bashrc" <<'RC'
PS1='\$ '
export PATH="@BIN@:$PATH"
if [[ -e @WORK@/start-${PWD##*/} ]]; then
  rm -f "@WORK@/start-${PWD##*/}"
  exec claude "${PWD##*/}"
fi
RC
  terminal_env HOME "$home"
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  write_notification_server
  python3 -u "$E2E_WORK/notifications.py" "$E2E_WORK/banners.log" >"$E2E_WORK/notifications.out" 2>&1 &
  echo "$!" >"$E2E_WORK/notifications.pid"
  write_stand_in "$bin/claude"
  profile_setting marley.tmux_hosts '[]'
  git init -q -b main "$E2E_WORK/a"
  printf '# a\n' >"$E2E_WORK/a/README.md"
  git -C "$E2E_WORK/a" add README.md
  git -C "$E2E_WORK/a" -c user.name=e2e -c user.email=e2e@example.invalid commit -q -m first
  open_path "$E2E_WORK/a"
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

write_stand_in() {
  sed -e "s|@HOOKS@|$HOOKS|" >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code: one finished turn through Marley's hooks, then each line it is sent,
# echoed.
import json
import os
import subprocess
import sys
import time

name = sys.argv[1] if len(sys.argv) > 1 else "agent"
COMMON = {"session_id": f"e2e-{name}", "transcript_path": "/tmp/e2e-transcript.jsonl",
          "cwd": os.getcwd(), "permission_mode": "default"}


def hook(event):
    answer = subprocess.run([sys.executable, "@HOOKS@/event.py"], input=json.dumps({**COMMON, **event}),
                            capture_output=True, text=True, check=False).stdout
    sequence = json.loads(answer or "{}").get("terminalSequence")
    if sequence:
        sys.stdout.write(sequence)
        sys.stdout.flush()


print("Claude Code (stand-in): ready", flush=True)
# One turn that changes the tree, so the rail lists it (#509): the checkpoint at the prompt, a
# file written, the checkpoint at the stop.
hook({"hook_event_name": "SessionStart", "source": "startup"})
hook({"hook_event_name": "UserPromptSubmit", "prompt": "Tidy the imports"})
time.sleep(3)
with open("notes.md", "w", encoding="utf-8") as notes:
    notes.write("# Notes\n\nTidied.\n")
time.sleep(1)
hook({"hook_event_name": "Stop", "last_assistant_message": "Tidied the imports."})
for line in sys.stdin:
    print("got:", line.rstrip(), flush=True)
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

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 15
  # Trusts a.
  press "" Return
  settle 4
  type_text "echo hello"
  press "" Return
  settle 2

  echo "== a's stand-in agent"
  touch "$E2E_WORK/start-a"
  palette "workspace: new terminal" 10
  [ -n "${STOP_AT_SETUP:-}" ] && { away; shot 911-00-probe; return; }

  echo "== the project menu"
  click "$HEADER_X" "$HEADER_Y" right
  settle 2
  shot 911-01-project-menu
  click "$FOLD_X" "$FOLD_Y"
  settle 2
  away
  shot 911-02-folded
  click "$HEADER_X" "$HEADER_Y" right
  settle 2
  click "$FOLD_X" "$FOLD_Y"
  settle 2
  [ -n "${STOP_AT_FOLD:-}" ] && return

  echo "== the row menus"
  click "$SHELL_X" "$SHELL_Y" right
  settle 2
  shot 911-03-row-menu
  press "" Escape
  settle 1
  click "$AGENT_X" "$AGENT_Y" right
  settle 2
  click "$TURNS_X" "$TURNS_Y"
  settle 2
  away
  shot 911-04-turns
  [ -n "${STOP_AT_TURNS:-}" ] && return

  echo "== Send Block to Agent"
  click "$SHELL_X" "$SHELL_Y" right
  settle 2
  click "$SEND_X" "$SEND_Y"
  settle 4
  away
  shot 911-05-sent

  echo "== Split Right"
  click "$SHELL_X" "$SHELL_Y" right
  settle 2
  click "$SPLIT_X" "$SPLIT_Y"
  settle 4
  away
  shot 911-06-split

  echo "== the empty space's menu"
  click "$EMPTY_X" "$EMPTY_Y" right
  settle 2
  shot 911-07-empty-menu
  click "$GROUP_X" "$GROUP_Y"
  settle 2
  shot 911-08-group-field
  type_text "Docs"
  press "" Return
  settle 3
  away
  shot 911-09-group-made

  echo "== the strip"
  palette "marley: toggle rail strip" 3
  away
  shot 911-10-strip-plus
  click "$PLUS_X" "$PLUS_Y"
  settle 2
  shot 911-11-strip-popover
  press "" Escape
  palette "marley: toggle rail strip" 3

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  click "$SHELL_X" "$SHELL_Y" right
  settle 2
  shot 911-12-day
  press "" Escape
}
