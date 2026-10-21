# shellcheck shell=bash
# #833's visual check: the bottom row says what has the focus. Project `repo` with a terminal
# that runs `true` then `false`; `b`, handed over later, has `src/lib.rs` and runs #542's stand-in
# Claude Code, which waits on the user; `marley.tmux_hosts` is `[]`.
# - repo's terminal: `>_ bash`, `false · exit 1`, `no command running`, its blocks
#   (`833-01-terminal`, REQ-002); `sleep 30` typed: `running: sleep 30` (`833-02-running`).
# - b handed over and waiting: its terminal's row with the waiting agent and Open
#   (`833-03-waiting`, REQ-004). A Claude Code terminal's request takes no answer in place, so it
#   offers no Allow.
# - `src/lib.rs` opened: `rs src/lib.rs` beside the waiting agent (`833-04-file`, REQ-001).
# - Open clicked in the row: b's terminal, where the agent waits, shows (`833-05-opened`, REQ-005).
# - Marley Day (`833-06-day`, REQ-002, REQ-004).
compositor sway

HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the probe run's shots: the bottom row's Open while b's file shows.
OPEN_X=${OPEN_X:-715}
OPEN_Y=${OPEN_Y:-985}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
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
  mkfifo "$E2E_WORK/b.fifo"
  mkdir -p "$E2E_WORK/b/src"
  printf 'pub fn b() {}\n' >"$E2E_WORK/b/src/lib.rs"
  git init -q -b main "$E2E_WORK/b"
  mkdir -p "$E2E_WORK/repo/src"
  printf 'fn main() {}\n' >"$E2E_WORK/repo/src/main.rs"
  git init -q -b main "$E2E_WORK/repo"
  profile_setting marley.tmux_hosts '[]'
  open_path "$E2E_WORK/repo"
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

steps() {
  settle 12
  # Trusts repo.
  press "" Return
  settle 3
  type_text "true"
  press "" Return
  settle 1
  type_text "false"
  press "" Return
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 833-01-terminal

  echo "== a command runs"
  type_text "sleep 30"
  press "" Return
  settle 2
  shot 833-02-running

  echo "== b, handed over, waits"
  touch "$E2E_WORK/start-b"
  hand_over "$E2E_WORK/b"
  settle 6
  press "" Return
  settle 4
  step_of b waiting
  settle 3
  shot 833-03-waiting

  echo "== a file of b"
  press "CTRL" p
  settle 1
  type_text "lib.rs"
  settle 2
  press "" Return
  settle 3
  shot 833-04-file

  echo "== Open in the row"
  click "$OPEN_X" "$OPEN_Y"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 833-05-opened

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 833-06-day
}
