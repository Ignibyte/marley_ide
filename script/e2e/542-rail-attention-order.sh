# shellcheck shell=bash
# #542's visual check: the rail lists what needs the user first. Three projects, a, b and c, in
# one window (b and a handed to the running Marley, #513); in each a stand-in Claude Code acts out
# its steps through a FIFO with the plugin's real `event.py`, and b's first terminal is a plain
# shell. Marley runs on a private session bus whose notification server logs each banner, so none
# reaches the user's desktop. With a working, b waiting and c finished unseen, the rail lists b, c,
# a, and under b the waiting agent above the shell (REQ-001, REQ-003); b collapsed reads `1
# waiting` (REQ-004); while the pointer is over the rail, a turning waiting moves nothing
# (REQ-005); with the pointer gone, a and b, both waiting, lead in window order and b's shell,
# selected while held, stays selected as it moves (REQ-006, REQ-002, REQ-009). With `no_update_after_minutes` at 1, a working agent silent
# for 70 seconds sorts below one that works (REQ-007); `rail_order: window` lists a, b, c (REQ-008).
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks
# A point in the terminal, clear of the rail.
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the shots: the top project's disclosure under the one-entry Needs you inbox, and b's shell
# row while the order is held under the two-entry inbox.
DISCLOSURE_X=${DISCLOSURE_X:-23}
DISCLOSURE_Y=${DISCLOSURE_Y:-175}
SHELL_ROW_X=${SHELL_ROW_X:-120}
SHELL_ROW_Y=${SHELL_ROW_Y:-320}

# Sets the settings key path `$1` (dot-separated) to the JSON value `$2` in the run's copy of the
# settings.
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
  # The user's own bus, watched for a banner that should not reach it.
  stdbuf -oL busctl --address="unix:path=$XDG_RUNTIME_DIR/bus" monitor org.freedesktop.Notifications \
    >"$E2E_WORK/user-bus.log" 2>&1 &
  echo "$!" >"$E2E_WORK/user-bus.pid"
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  write_notification_server
  python3 -u "$E2E_WORK/notifications.py" "$E2E_WORK/banners.log" >"$E2E_WORK/notifications.out" 2>&1 &
  echo "$!" >"$E2E_WORK/notifications.pid"
  : >"$E2E_WORK/banners.log"
  write_steps
  write_stand_in "$bin/claude"
  for name in a b c; do
    mkfifo "$E2E_WORK/$name.fifo"
    git init -q -b main "$E2E_WORK/$name"
  done
  set_setting marley.no_update_after_minutes 1
  touch "$E2E_WORK/start-c"
  open_path "$E2E_WORK/c"
}

teardown() {
  local pid
  for pid in user-bus notifications; do
    [[ -f $E2E_WORK/$pid.pid ]] && kill "$(cat "$E2E_WORK/$pid.pid")" 2>/dev/null
  done
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

# Each stand-in's steps by name; a line written to its FIFO names the step it acts out.
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
  # Trusts c.
  press "" Return
  settle 3

  echo "== b, handed over: a shell, then a stand-in in a second terminal"
  hand_over "$E2E_WORK/b"
  settle 5
  press "" Return
  settle 4
  touch "$E2E_WORK/start-b"
  palette "workspace: new terminal"
  settle 4

  echo "== a, handed over"
  touch "$E2E_WORK/start-a"
  hand_over "$E2E_WORK/a"
  settle 5
  press "" Return
  settle 4

  echo "== a works, b waits, c finishes unseen"
  step_of a working
  step_of b waiting
  step_of c finished
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 3
  shot 542-01-order

  echo "== b collapsed"
  click "$DISCLOSURE_X" "$DISCLOSURE_Y"
  settle 2
  shot 542-02-collapsed

  echo "== held: b expanded, the pointer on the rail, a asks"
  click "$DISCLOSURE_X" "$DISCLOSURE_Y"
  settle 1
  step_of a asks
  settle 3
  shot 542-03-held

  echo "== released: b's shell selected, the pointer away"
  click "$SHELL_ROW_X" "$SHELL_ROW_Y"
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 3
  shot 542-04-released

  echo "== a silent past a minute while c works"
  step_of a resumes
  settle 50
  step_of c works
  settle 25
  shot 542-05-not-reporting

  echo "== rail_order window"
  set_setting marley.rail_order '"window"'
  settle 4
  shot 542-06-window-order

  cat "$E2E_WORK/banners.log"
  # Other programs post to the user's bus as they will; only Marley's name counts (#535's check).
  expect "no banner reached the user's bus" bash -c "! grep -q 'STRING \"Marley\"' '$E2E_WORK/user-bus.log'"
}
