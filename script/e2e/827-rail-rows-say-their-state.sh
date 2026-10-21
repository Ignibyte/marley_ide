# shellcheck shell=bash
# #827's visual check: rail rows say their state. Projects `a`, `b` and `c` in one window, each
# with #542's stand-in Claude Code (on a private notification bus): `a` works, `b` waits on the
# user, `c` finishes while unseen. `a` also has a shell running `sleep 60` and `c` one whose
# `false` failed. The run's own tmux server has `build`.
# - The rail in Marley Night: agent-terminal tiles with amber, pink and green dots and lines, the
#   shells' tiles (working, a failed command), the tmux row's tile (`827-01-night`, REQ-001,
#   REQ-002).
# - `b` folded: its summary in pink (`827-02-folded`, REQ-003).
# - Marley Day (`827-03-day`, REQ-001, REQ-002).
compositor sway

HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the probe run's shots: b's disclosure.
B_DISCLOSURE_X=${B_DISCLOSURE_X:-23}
B_DISCLOSURE_Y=${B_DISCLOSURE_Y:-424}

# tmux's socket path must stay under 108 bytes, which a run's work folder does not (#808).
SOCKETS=${XDG_RUNTIME_DIR:-/tmp}/marley-e2e-827
LOCAL_TMUX=$SOCKETS/local

set_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'SETTINGS'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
# Comments and trailing commas out: the settings file allows them and JSON does not.
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
*parents, last = sys.argv[2].split(".")
node = settings
for key in parents:
    node = node.setdefault(key, {})
node[last] = json.loads(sys.argv[3])
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTINGS
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin name
  mkdir -p "$home" "$bin" "$LOCAL_TMUX"
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
  terminal_env TMUX_TMPDIR "$LOCAL_TMUX"
  export TMUX_TMPDIR=$LOCAL_TMUX
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  write_notification_server
  python3 -u "$E2E_WORK/notifications.py" "$E2E_WORK/banners.log" >"$E2E_WORK/notifications.out" 2>&1 &
  echo "$!" >"$E2E_WORK/notifications.pid"
  write_steps
  write_stand_in "$bin/claude"
  for name in a b c; do
    mkfifo "$E2E_WORK/$name.fifo"
    git init -q -b main "$E2E_WORK/$name"
  done
  TMUX_TMPDIR=$LOCAL_TMUX HOME=$home tmux -f /dev/null new-session -d -s build -c "$home" -x 100 -y 30
  touch "$E2E_WORK/start-c"
  open_path "$E2E_WORK/c"
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

write_steps() {
  cat >"$E2E_WORK/steps.json" <<'JSON'
{
  "working": [
    {"hook_event_name": "SessionStart", "source": "startup"},
    {"hook_event_name": "UserPromptSubmit", "prompt": "Run the build"},
    {"hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": "make"}, "tool_use_id": "t1"}
  ],
  "waiting": [
    {"hook_event_name": "SessionStart", "source": "startup"},
    {"hook_event_name": "UserPromptSubmit", "prompt": "Write the README"},
    {"hook_event_name": "PreToolUse", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# b"}, "tool_use_id": "t2"},
    {"hook_event_name": "PermissionRequest", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# b"}}
  ],
  "asks": [
    {"hook_event_name": "PermissionRequest", "tool_name": "Bash", "tool_input": {"command": "make"}}
  ],
  "resumes": [
    {"hook_event_name": "PostToolUse", "tool_name": "Bash", "tool_input": {"command": "make"}, "tool_use_id": "t1"},
    {"hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": "make test"}, "tool_use_id": "t3"}
  ],
  "finished": [
    {"hook_event_name": "SessionStart", "source": "startup"},
    {"hook_event_name": "UserPromptSubmit", "prompt": "Tidy the imports"},
    {"hook_event_name": "Stop", "last_assistant_message": "Tidied the imports."}
  ],
  "works": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "Now the docs"},
    {"hook_event_name": "PreToolUse", "tool_name": "Read", "tool_input": {"file_path": "README.md"}, "tool_use_id": "t4"}
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

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

hand_over() {
  timeout 20 env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" \
    SWAYSOCK="$SWAY_SOCK" "$E2E_MARLEY" --user-data-dir "$E2E_PROFILE" "$1" \
    >>"$E2E_WORK/second.log" 2>&1 </dev/null || true
}

step_of() {
  timeout 5 bash -c "echo $2 >'$E2E_WORK/$1.fifo'" ||
    echo "step_of: stand-in $1 is not reading" >&2
}

steps() {
  settle 12
  # Trusts c.
  press "" Return
  settle 3
  echo "== c: a shell whose command fails"
  palette "workspace: new terminal"
  settle 4
  type_text "false"
  press "" Return
  settle 2

  echo "== b, handed over: its stand-in"
  touch "$E2E_WORK/start-b"
  hand_over "$E2E_WORK/b"
  settle 5
  press "" Return
  settle 4

  echo "== a, handed over: its stand-in, and a shell that runs"
  touch "$E2E_WORK/start-a"
  hand_over "$E2E_WORK/a"
  settle 5
  press "" Return
  settle 4
  palette "workspace: new terminal"
  settle 4
  type_text "sleep 60"
  press "" Return
  settle 1

  echo "== a works, b waits, c finishes unseen"
  step_of a working
  step_of b waiting
  step_of c finished
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 3
  shot 827-01-night

  echo "== b folded"
  click "$B_DISCLOSURE_X" "$B_DISCLOSURE_Y"
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 2
  shot 827-02-folded

  echo "== Marley Day"
  click "$B_DISCLOSURE_X" "$B_DISCLOSURE_Y"
  settle 1
  set_setting theme '"Marley Day"'
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 4
  shot 827-03-day
}
