# shellcheck shell=bash
# #830's visual check: the rail folds to a strip of project marks. Two projects in one window: b,
# opened first, runs a stand-in Claude Code (#542's FIFO stand-in on a private notification bus)
# that waits on the user, and a, handed over second, is the shown one with a plain shell.
# - The open rail with its new Shrink button (`830-01-open`, REQ-001); the button clicked, the
#   60 px strip in Marley Night with a ringed and b's badge 1 (`830-02-strip-night`, REQ-001,
#   REQ-004, REQ-005).
# - The pointer on b's tile: b's header and its waiting row beside the strip (`830-03-flyout`,
#   REQ-006).
# - Ctrl F with the rail focused, `wait` typed: the filter and its match in a popover
#   (`830-04-search`, REQ-007).
# - The theme set to Marley Day: the strip (`830-05-strip-day`, REQ-004).
# - Marley quit and started again on the same profile: still the strip, with
#   `marley.rail_shrunk` printed from the profile (`830-06-restart`, REQ-003).
# - The strip's Open clicked: the rail at its width again (`830-07-open-again`, REQ-002).
compositor sway

HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the shots: the open rail's Shrink button, the strip's Open, search and b's tile.
SHRINK_X=${SHRINK_X:-247}
SHRINK_Y=${SHRINK_Y:-17}
OPEN_X=${OPEN_X:-30}
OPEN_Y=${OPEN_Y:-17}
B_TILE_X=${B_TILE_X:-30}
B_TILE_Y=${B_TILE_Y:-181}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin name
  mkdir -p "$home" "$bin"
  # A terminal whose project has a start mark runs that project's stand-in in place of the shell.
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
  write_steps
  write_stand_in "$bin/claude"
  for name in a b; do
    mkfifo "$E2E_WORK/$name.fifo"
    git init -q -b main "$E2E_WORK/$name"
  done
  touch "$E2E_WORK/start-b"
  # The restart quits with b's stand-in still waiting, which would ask first (#550).
  profile_setting marley.ask_before_ending_a_working_agent false
  open_path "$E2E_WORK/b"
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

# The stand-in's one step: it asks to write a file and waits on the user.
write_steps() {
  cat >"$E2E_WORK/steps.json" <<'JSON'
{
  "waiting": [
    {"hook_event_name": "SessionStart", "source": "startup"},
    {"hook_event_name": "UserPromptSubmit", "prompt": "Write the README"},
    {"hook_event_name": "PreToolUse", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# b"}, "tool_use_id": "t2"},
    {"hook_event_name": "PermissionRequest", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# b"}}
  ]
}
JSON
}

write_stand_in() {
  sed -e "s|@HOOKS@|$HOOKS|" -e "s|@WORK@|$E2E_WORK|" >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code named by its argument. Each line written to its FIFO names the step it
# acts out: the plugin's event.py for every event, each answer's sequence written to the terminal
# as Claude Code writes it.
import json
import os
import subprocess
import sys

name = sys.argv[1]
STEPS = json.load(open("@WORK@/steps.json", encoding="utf-8"))
COMMON = {"session_id": f"e2e-{name}", "transcript_path": "/tmp/e2e-transcript.jsonl",
          "cwd": os.getcwd(), "permission_mode": "default"}

print(f"Claude Code (stand-in {name}): steps come through its FIFO", flush=True)
while True:
    with open(f"@WORK@/{name}.fifo", encoding="utf-8") as fifo:
        for line in fifo:
            step = line.strip()
            for event in STEPS[step]:
                answer = subprocess.run([sys.executable, "@HOOKS@/event.py"],
                                        input=json.dumps({**COMMON, **event}),
                                        capture_output=True, text=True, check=False).stdout
                sequence = json.loads(answer or "{}").get("terminalSequence")
                if sequence:
                    sys.stdout.write(sequence)
                    sys.stdout.flush()
            print(f"{name}: {step}", flush=True)
FAKE
  chmod +x "$1"
}

# Hands `path` to the running Marley, as a second launch does (#513).
hand_over() {
  timeout 20 env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" \
    SWAYSOCK="$SWAY_SOCK" "$E2E_MARLEY" --user-data-dir "$E2E_PROFILE" "$1" \
    >>"$E2E_WORK/second.log" 2>&1 </dev/null || true
}

# Stand-in `$1` acts out step `$2`; a stand-in that is not reading fails the step, not the run.
step_of() {
  timeout 5 bash -c "echo $2 >'$E2E_WORK/$1.fifo'" ||
    echo "step_of: stand-in $1 is not reading" >&2
}

# `marley.rail_shrunk` in the run's copy of the settings, as Marley wrote it.
shrunk_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY'
import json, pathlib, re, sys

text = pathlib.Path(sys.argv[1]).read_text()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
print(json.loads(text).get("marley", {}).get("rail_shrunk"))
PY
}

steps() {
  settle 12
  # Trusts b.
  press "" Return
  settle 3

  echo "== a, handed over"
  hand_over "$E2E_WORK/a"
  settle 5
  press "" Return
  settle 4
  step_of b waiting
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 3
  shot 830-01-open

  echo "== shrunk"
  click "$SHRINK_X" "$SHRINK_Y"
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 2
  shot 830-02-strip-night
  echo "rail_shrunk after the click: $(shrunk_setting)"

  echo "== the flyout"
  pointer_to "$B_TILE_X" "$B_TILE_Y"
  settle 2
  shot 830-03-flyout

  echo "== the search popover"
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  click "$B_TILE_X" "$((OPEN_Y + 33))"
  settle 1
  type_text "claude"
  settle 2
  shot 830-04-search
  press "" Escape
  settle 1

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 4
  shot 830-05-strip-day

  echo "== a restart"
  quit_marley
  settle 2
  launch_marley
  settle 12
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 2
  shot 830-06-restart

  echo "== opened again"
  click "$OPEN_X" "$OPEN_Y"
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 2
  shot 830-07-open-again
}
