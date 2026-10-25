# shellcheck shell=bash
# #910's visual check: rail rows say what is going on. Projects `a`, `b` and `c` in one window.
# Marley's own agent runs a scripted ACP agent (`MARLEY_ASSISTANT_ADAPTER`, #907's): "start" holds
# an "Edit notes.md" tool call until a flag file, "ask" asks a permission. In `a` a shell ran
# `false` and a second sits idle at the root. `b` runs #827's stand-in Claude Code (Marley's hooks
# on a private bus), waiting on the user; `c` is quiet. Stand-ins sit first on the terminals' PATH
# through `.bashrc` (PR-687, F-claude-903).
# - `910-01-working`: the thread row reads "Edit notes.md" in amber (REQ-001).
# - `910-04-idle`: the turn finished in front, seen: "Opus 5.5 · idle", muted (REQ-004).
# - `910-03-done`: a second turn finished unseen: "1 file changed" in the done colour (REQ-003).
# - `910-02-asks`: "Allow Edit README.md?" in pink (REQ-002).
# - `910-05-shells`: "false · exit 1 · Ask an agent" in red; the idle shell's folder (REQ-005,
#   REQ-006).
# - `910-06-asked`: Ask an agent clicked: the stand-in Claude Code asked to fix it (REQ-007).
# - `910-07-headers`: pills on `a` and `b`, "N rows" on `c`, the Default tile (REQ-008 to
#   REQ-010).
# - `910-08-day`: Marley Day (REQ-001 to REQ-010).
compositor sway

HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the probe run's shots.
ASK_X=${ASK_X:-180}
ASK_Y=${ASK_Y:-296}
THREAD_TAB_X=${THREAD_TAB_X:-660}
TAB_Y=${TAB_Y:-50}
DEFAULT_X=${DEFAULT_X:-23}
DEFAULT_Y=${DEFAULT_Y:-284}
SHELL_X=${SHELL_X:-130}
SHELL_Y=${SHELL_Y:-232}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin name db
  mkdir -p "$home" "$bin"
  # A terminal opened while its project's start mark is there runs the stand-in in place of the
  # shell.
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
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/release" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  profile_setting marley.assistant '{"enabled": true, "agent": "claude_code"}'
  profile_setting marley.tmux_hosts '[]'
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  for name in a b c; do
    mkfifo "$E2E_WORK/$name.fifo"
    git init -q -b main "$E2E_WORK/$name"
  done
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
if not os.path.exists(f"@WORK@/{name}.fifo"):
    # Started by Ask an agent with a request, not by a project's start mark: it waits.
    for _ in sys.stdin:
        pass
    sys.exit(0)
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

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent for #907's check. Its session names a model, a thought level and a mode as
# config options. "start": token use, notes.md written through the client, and an open "Edit
# notes.md" tool call until the release file exists. "ask": a permission request.
import json
import os
import sys
import time

release = sys.argv[1]
next_id = [9000]
folders = {}
# Each "start" holds until its own release file: release1, release2.
starts = [0]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def request(method, params):
    next_id[0] += 1
    rid = next_id[0]
    send({"jsonrpc": "2.0", "id": rid, "method": method, "params": params})
    for answer in sys.stdin:
        if answer.strip() and json.loads(answer).get("id") == rid:
            return json.loads(answer)
    return {}


def update(session, body):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session, "update": body}})


def option(ident, name, category, current, values):
    return {"id": ident, "name": name, "category": category, "type": "select", "currentValue": current,
            "options": [{"value": value, "name": label} for value, label in values]}


CONFIG = [
    option("model", "Model", "model", "opus", [("opus", "Opus 5.5"), ("sonnet", "Sonnet 5.5")]),
    option("effort", "Effort", "thought_level", "high", [("low", "Low"), ("high", "High")]),
    option("mode", "Mode", "mode", "act", [("plan", "Plan"), ("act", "Act")]),
]

for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "protocolVersion": 1,
            "agentCapabilities": {"loadSession": False, "promptCapabilities": {}},
            "authMethods": [], "agentInfo": {"name": "scripted", "version": "0"}}})
    elif method == "session/new":
        session = "row-%d" % os.getpid()
        folders[session] = params.get("cwd", ".")
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": session, "configOptions": CONFIG}})
    elif method == "session/set_config_option":
        send({"jsonrpc": "2.0", "id": request_id, "result": {"configOptions": CONFIG}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        folder = folders.get(session, ".")
        text = " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        if text.strip() == "ask":
            tool = {"toolCallId": "ask-1", "title": "Edit README.md", "kind": "edit", "status": "pending",
                    "rawInput": {"path": "README.md"}}
            update(session, {"sessionUpdate": "tool_call", **tool})
            request("session/request_permission", {
                "sessionId": session, "toolCall": tool,
                "options": [{"optionId": "allow", "name": "Allow", "kind": "allow_once"},
                            {"optionId": "reject", "name": "Reject", "kind": "reject_once"}]})
            update(session, {"sessionUpdate": "tool_call_update", "toolCallId": "ask-1", "status": "completed"})
        else:
            update(session, {"sessionUpdate": "usage_update", "used": 48000, "size": 200000})
            request("fs/write_text_file", {"sessionId": session, "path": folder + "/notes.md",
                                            "content": "# Notes\n\nThe bottom row.\nIts chips.\n"})
            update(session, {"sessionUpdate": "tool_call", "toolCallId": "edit-1", "title": "Edit notes.md",
                             "kind": "edit", "status": "in_progress"})
            starts[0] += 1
            while not os.path.exists(release + str(starts[0])):
                time.sleep(0.2)
            update(session, {"sessionUpdate": "tool_call_update", "toolCallId": "edit-1", "status": "completed"})
            update(session, {"sessionUpdate": "agent_message_chunk",
                             "content": {"type": "text", "text": "Wrote notes.md."}})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
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

hand_over() {
  timeout 20 env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" \
    SWAYSOCK="$SWAY_SOCK" "$E2E_MARLEY" --user-data-dir "$E2E_PROFILE" "$1" \
    >>"$E2E_WORK/second.log" 2>&1 </dev/null || true
}

step_of() {
  timeout 5 bash -c "echo $2 >'$E2E_WORK/$1.fifo'" ||
    echo "step_of: stand-in $1 is not reading" >&2
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

  echo "== a: a shell whose command fails, and an idle one"
  type_text "false"
  press "" Return
  settle 2
  palette "workspace: new terminal" 4

  echo "== a: a Marley thread, mid-turn"
  palette "marley: new marley conversation" 10
  type_text "start"
  settle 4
  press "" Return
  settle 6
  away
  shot 910-01-working

  echo "== the turn ends in front: seen"
  touch "$E2E_WORK/release1"
  settle 4
  away
  shot 910-04-idle

  echo "== a second turn ends while a shell is in front: unseen"
  type_text "start"
  settle 4
  press "" Return
  settle 3
  click "$SHELL_X" "$SHELL_Y"
  settle 2
  touch "$E2E_WORK/release2"
  settle 4
  away
  shot 910-03-done
  click "$THREAD_TAB_X" "$TAB_Y"
  settle 3
  [ -n "${STOP_AT_IDLE:-}" ] && return

  echo "== a permission waits"
  type_text "ask"
  settle 4
  press "" Return
  settle 5
  click "$SHELL_X" "$SHELL_Y"
  settle 2
  away
  shot 910-02-asks
  shot 910-05-shells

  echo "== Ask an agent"
  click "$ASK_X" "$ASK_Y"
  settle 6
  away
  shot 910-06-asked

  echo "== b waits, c is quiet"
  touch "$E2E_WORK/start-b"
  hand_over "$E2E_WORK/b"
  settle 5
  press "" Return
  settle 4
  step_of b waiting
  hand_over "$E2E_WORK/c"
  settle 5
  press "" Return
  settle 4
  # The Default group folded: nothing to count, so "quiet".
  click "$DEFAULT_X" "$DEFAULT_Y"
  settle 2
  away
  shot 910-07-headers

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 910-08-day
}
