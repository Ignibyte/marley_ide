# shellcheck shell=bash
# #535's e2e test: Claude Code's events pushed to the phone through ntfy. A fake ntfy on a loopback
# port logs each request; Marley runs on a private session bus whose notification server logs each
# desktop banner, so none reaches the user's desktop. A stand-in `claude` acts out a session, a
# step each time the scenario writes to a FIFO (keys cannot reach a terminal that is not focused):
# it runs the plugin's real `event.py` for every event and `notify.sh` where Claude Code runs it.
# Marley's window leaves the active workspace for each event that should push, and comes back for
# its shot. Checks: pushes only with `marley.push` set, only for Claude Code's events, only while
# the user is not looking at the terminal; the line and nothing else; the token from a 0600 file
# and none from a readable one; no push to a server off this machine; one toast when the server
# is gone, the banner still shown; and no banner on the user's own bus.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks

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
  write_fake_ntfy
  python3 -u "$E2E_WORK/ntfy.py" "$E2E_WORK/pushes.log" >"$E2E_WORK/ntfy.out" 2>&1 &
  echo "$!" >"$E2E_WORK/ntfy.pid"
  for _ in $(seq 50); do
    PORT=$(grep -oE 'port [0-9]+' "$E2E_WORK/ntfy.out" | cut -d' ' -f2 || true)
    [[ -n $PORT ]] && break
    sleep 0.1
  done
  [[ -n $PORT ]] || { echo "the fake ntfy did not start" >&2; return 1; }
  : >"$E2E_WORK/pushes.log"
  mkfifo "$E2E_WORK/steps.fifo"
  write_steps
  write_stand_in "$bin/claude"
  git init -q -b push "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  local pid
  for pid in user-bus notifications ntfy; do
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

write_fake_ntfy() {
  cat >"$E2E_WORK/ntfy.py" <<'PY'
# A fake ntfy on a free loopback port: logs each request's method, path, headers and body, one
# JSON line each, and answers as ntfy does.
import http.server
import json
import sys

LOG = sys.argv[1]


class Handler(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length") or 0)
        record = {"method": "POST", "path": self.path, "headers": dict(self.headers.items()),
                  "body": self.rfile.read(length).decode("utf-8", "replace")}
        with open(LOG, "a", encoding="utf-8") as log:
            log.write(json.dumps(record) + "\n")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(b'{"id":"e2e","event":"message"}')

    def log_message(self, *args):
        pass


server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
print(f"port {server.server_address[1]}", flush=True)
server.serve_forever()
PY
}

# The session the stand-in acts out; `notify` is the argument Claude Code's Notification or Stop
# hook gives notify.sh, when it runs it.
write_steps() {
  cat >"$E2E_WORK/steps.json" <<'JSON'
[
  {"label": "a turn with no push set", "notify": "finished", "events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "Tidy the imports"},
    {"hook_event_name": "Stop", "last_assistant_message": "Tidied."}]},
  {"label": "a permission asked", "notify": "permission", "events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "Add a README"},
    {"hook_event_name": "PreToolUse", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# repo"}, "tool_use_id": "t1"},
    {"hook_event_name": "PermissionRequest", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# repo"}}]},
  {"label": "the turn finished", "notify": "finished", "events": [
    {"hook_event_name": "PostToolUse", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# repo"}, "tool_use_id": "t1"},
    {"hook_event_name": "Stop", "last_assistant_message": "Added README.md."}]},
  {"label": "a turn failed", "events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "Run the tests"},
    {"hook_event_name": "StopFailure", "error": "rate_limit"}]},
  {"label": "a turn while the terminal has focus", "notify": "finished", "events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "Check the lints"},
    {"hook_event_name": "Stop", "last_assistant_message": "No lints."}]},
  {"label": "a turn with the token file", "notify": "finished", "events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "Format the code"},
    {"hook_event_name": "Stop", "last_assistant_message": "Formatted."}]},
  {"label": "a turn with the token file readable by others", "notify": "finished", "events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "Update the lockfile"},
    {"hook_event_name": "Stop", "last_assistant_message": "Updated."}]},
  {"label": "a turn with a server off this machine", "notify": "finished", "events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "Rename the crate"},
    {"hook_event_name": "Stop", "last_assistant_message": "Renamed."}]},
  {"label": "a turn with the server gone", "notify": "finished", "events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "Write the changelog"},
    {"hook_event_name": "Stop", "last_assistant_message": "Written."}]}
]
JSON
}

write_stand_in() {
  sed -e "s|@HOOKS@|$HOOKS|" -e "s|@STEPS@|$E2E_WORK/steps.json|" -e "s|@FIFO@|$E2E_WORK/steps.fifo|" \
    >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code. Each line written to the FIFO acts out the next step: the plugin's
# event.py for every event, and notify.sh where Claude Code runs it, each answer's sequence
# written to the terminal as Claude Code writes it.
import json
import os
import subprocess
import sys

STEPS = json.load(open("@STEPS@", encoding="utf-8"))
COMMON = {"session_id": "e2e-session", "transcript_path": "/tmp/e2e-transcript.jsonl",
          "cwd": os.getcwd(), "permission_mode": "default"}


def write(answer):
    sequence = json.loads(answer or "{}").get("terminalSequence")
    if sequence:
        sys.stdout.write(sequence)
        sys.stdout.flush()


print("Claude Code (stand-in): steps come through the FIFO", flush=True)
done = 0
while done < len(STEPS):
    with open("@FIFO@", encoding="utf-8") as fifo:
        for _ in fifo:
            if done >= len(STEPS):
                break
            step = STEPS[done]
            done += 1
            for event in step["events"]:
                write(subprocess.run([sys.executable, "@HOOKS@/event.py"],
                                     input=json.dumps({**COMMON, **event}),
                                     capture_output=True, text=True, check=False).stdout)
            if step.get("notify"):
                write(subprocess.run(["sh", "@HOOKS@/notify.sh", step["notify"]],
                                     capture_output=True, text=True, check=False).stdout)
            print(f"step {done}: {step['label']}", flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$1"
}

# Sets `marley.push` in the profile's settings, or removes it with `none`, and waits for the reload.
push_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$@" <<'PY'
import json
import re
import sys

path, url, *rest = sys.argv[1:]
text = open(path).read()
text = re.sub(r'\n\s*"push": \{[^}]*\},', "", text)
if url != "none":
    entry = {"url": url, "topic": rest[0]}
    if len(rest) > 1:
        entry["token_file"] = rest[1]
    line = '"push": ' + json.dumps(entry) + ","
    if '"marley": {' in text:
        text = text.replace('"marley": {', '"marley": {\n    ' + line, 1)
    else:
        lines = text.split("\n")
        at = next(i for i, text_line in enumerate(lines) if text_line.startswith("{"))
        lines.insert(at + 1, '  "marley": {' + line + "},")
        text = "\n".join(lines)
open(path, "w").write(text)
PY
  settle 2
}

# The stand-in's next step, with Marley's window on another workspace (`away`) or where it is.
next_step() { echo next >"$E2E_WORK/steps.fifo"; settle 2; }
away() { sway_msg workspace 2 >/dev/null; settle 1; }
back() { sway_msg workspace 1 >/dev/null; settle 1; }

# Whether the fake ntfy has logged exactly this many requests.
pushed() {
  local count
  count=$(wc -l <"$E2E_WORK/pushes.log")
  echo "pushes so far: $count"
  [[ $count -eq $1 ]]
}

# Whether the last request is a push of this line, with these headers.
last_push() {
  tail -1 "$E2E_WORK/pushes.log"
  python3 - "$E2E_WORK/pushes.log" "$@" <<'PY'
import json
import sys

path, line, priority, tag, *authorization = sys.argv[1:]
push = json.loads(open(path).read().splitlines()[-1])
headers = {key.lower(): value for key, value in push["headers"].items()}
ok = (push["path"] == "/marley-e2e" and push["body"] == line
      and headers.get("title") == "Marley" and headers.get("priority") == priority
      and headers.get("tags") == tag
      and headers.get("authorization") == (authorization[0] if authorization else None))
sys.exit(0 if ok else 1)
PY
}

# How many banners so far hold the text.
banners_with() { grep -c -- "$1" "$E2E_WORK/banners.log" || true; }

# Whether more banners hold the text than the count given.
more_banners() {
  local now
  now=$(banners_with "$1")
  echo "banners with '$1': $2 before, $now now"
  [[ $now -gt $2 ]]
}

# Whether every push carries only these headers: none of them is the agent's.
only_known_headers() {
  python3 - "$E2E_WORK/pushes.log" <<'PY'
import json
import sys

known = {"title", "priority", "tags", "content-type", "content-length", "authorization", "host",
         "accept", "accept-encoding", "user-agent"}
for text in open(sys.argv[1]).read().splitlines():
    extra = {key.lower() for key in json.loads(text)["headers"]} - known
    if extra:
        print("unexpected headers:", sorted(extra))
        sys.exit(1)
PY
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  push_setting "http://127.0.0.1:$PORT" marley-e2e
  echo "== a plain program's notifications, with push set"
  type_text "sleep 3; printf '\\033]9;built\\007\\033]777;notify;make;done\\007'"
  press "" Return
  away
  settle 4
  back
  expect "a program that is not an agent pushes nothing" pushed 0
  expect "its notifications still reach the desktop" \
    holds "$E2E_WORK/banners.log" "|built" "|make|done"
  push_setting none
  type_text "claude"
  press "" Return
  settle 3
  echo "== no push set"
  away
  next_step
  back
  expect "with no push set, nothing is pushed" pushed 0
  push_setting "http://127.0.0.1:$PORT" marley-e2e
  echo "== needs input"
  away
  next_step
  back
  shot 535-01-needs-input
  expect "a permission request pushes needs input" pushed 1
  expect "the push is the line, titled Marley" last_push "repo: Claude needs input" 4 question
  settle 5
  echo "== finished"
  away
  next_step
  back
  shot 535-02-finished
  expect "a turn's end pushes finished" pushed 2
  expect "the push says finished" last_push "repo: Claude finished" 3 white_check_mark
  settle 5
  echo "== failed"
  away
  next_step
  back
  shot 535-03-failed
  expect "a failed turn pushes failed" pushed 3
  expect "the push says failed" last_push "repo: Claude failed" 4 x
  settle 5
  echo "== the terminal has focus"
  next_step
  shot 535-04-focused
  expect "nothing is pushed while the user looks at the terminal" pushed 3
  expect "no push carries anything of the agent's" only_known_headers
  settle 5
  echo "== the token, from a file only its owner reads"
  printf 'tk_e2e\n' >"$E2E_WORK/token"
  chmod 600 "$E2E_WORK/token"
  push_setting "http://127.0.0.1:$PORT" marley-e2e "$E2E_WORK/token"
  away
  next_step
  back
  expect "the push carries the token" last_push "repo: Claude finished" 3 white_check_mark "Bearer tk_e2e"
  settle 5
  echo "== the token file readable by others"
  chmod 644 "$E2E_WORK/token"
  away
  next_step
  back
  expect "a token file others can read stops the push" pushed 4
  expect "Marley's log says why" holds "$E2E_PROFILE/logs/Marley.log" "can be read by others"
  settle 5
  echo "== a server off this machine"
  push_setting "http://192.0.2.1:9" marley-e2e
  away
  next_step
  back
  expect "nothing is pushed to a server off this machine" pushed 4
  expect "Marley's log says why" holds "$E2E_PROFILE/logs/Marley.log" "is not a server on this machine"
  push_setting "http://127.0.0.1:$PORT" marley-e2e
  settle 5
  echo "== the server gone"
  local finished
  finished=$(banners_with "repo finished")
  kill "$(cat "$E2E_WORK/ntfy.pid")"
  away
  next_step
  settle 2
  back
  shot 535-05-no-server
  expect "the desktop banner still shows" more_banners "repo finished" "$finished"
  expect "no banner reached the user's own bus" bash -c "! grep -qE 'STRING \"(Marley|Claude Code)\"' '$E2E_WORK/user-bus.log'"
}
