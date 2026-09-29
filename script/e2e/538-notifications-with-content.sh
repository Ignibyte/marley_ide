# shellcheck shell=bash
# #538's visual check: banners that say what Claude Code did, and the terminal's unread mark.
# Marley runs on a private session bus whose notification server logs each banner, so none reaches
# the user's desktop. Two stand-in Claude Codes, A and D, act out their steps through FIFOs with the
# plugin's real `event.py`; a third, plain terminal holds the focus. A's finish shows
# `repo: Claude finished` over its last message cut to 180 characters and marks A (REQ-001,
# REQ-005, REQ-006); D's permission request two seconds later shows no second banner and still
# marks D (REQ-010); looking at A clears its mark (REQ-007); a manual compaction after the seen
# finish marks nothing (REQ-008); A's permission request in a new turn, its question and its failure
# each show their banner and mark (REQ-002, REQ-003, REQ-004, REQ-009); a cleared session shows none
# (REQ-011); an event in the focused terminal shows none and marks nothing (REQ-012); every banner
# is the event's own, one each (REQ-013).
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
  : >"$E2E_WORK/banners.log"
  write_steps
  mkfifo "$E2E_WORK/a.fifo" "$E2E_WORK/d.fifo"
  write_stand_in "$bin/claude"
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

# The steps each stand-in acts out, one per line written to its FIFO. A's first message is 300
# characters with `é`s at the cut.
write_steps() {
  python3 - "$E2E_WORK" <<'PY'
import json
import sys

work = sys.argv[1]
long_message = ("Refactored the parser. " * 8)[:177] + "ééé" + " More words to pass the limit." * 5
long_message = long_message[:300]
steps_a = [
    {"label": "a finish with a long message", "events": [
        {"hook_event_name": "SessionStart", "source": "startup"},
        {"hook_event_name": "UserPromptSubmit", "prompt": "Refactor the parser"},
        {"hook_event_name": "Stop", "last_assistant_message": long_message}]},
    {"label": "a manual compaction after the seen finish", "events": [
        {"hook_event_name": "PostCompact", "trigger": "manual"}]},
    {"label": "a permission in a new turn", "events": [
        {"hook_event_name": "UserPromptSubmit", "prompt": "List the files"},
        {"hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": "ls -la"}, "tool_use_id": "t2"},
        {"hook_event_name": "PermissionRequest", "tool_name": "Bash", "tool_input": {"command": "ls -la"}}]},
    {"label": "a question", "events": [
        {"hook_event_name": "PostToolUse", "tool_name": "Bash", "tool_input": {"command": "ls -la"}, "tool_use_id": "t2"},
        {"hook_event_name": "PreToolUse", "tool_name": "AskUserQuestion", "tool_input": {"questions": [{"question": "Ship the release now?", "options": [{"label": "Yes"}, {"label": "No"}]}]}, "tool_use_id": "t3"}]},
    {"label": "a failure", "events": [
        {"hook_event_name": "PostToolUse", "tool_name": "AskUserQuestion", "tool_input": {}, "tool_use_id": "t3"},
        {"hook_event_name": "StopFailure", "error": "rate_limit"}]},
    {"label": "a new turn, then the session cleared", "events": [
        {"hook_event_name": "UserPromptSubmit", "prompt": "Try again"},
        {"hook_event_name": "SessionStart", "source": "clear"}]},
]
steps_d = [
    {"label": "a permission in the cooldown", "events": [
        {"hook_event_name": "SessionStart", "source": "startup"},
        {"hook_event_name": "UserPromptSubmit", "prompt": "Fix the typo"},
        {"hook_event_name": "PreToolUse", "tool_name": "Edit", "tool_input": {"file_path": "README.md"}, "tool_use_id": "d1"},
        {"hook_event_name": "PermissionRequest", "tool_name": "Edit", "tool_input": {"file_path": "README.md"}}]},
    {"label": "a finish while focused", "events": [
        {"hook_event_name": "PostToolUse", "tool_name": "Edit", "tool_input": {"file_path": "README.md"}, "tool_use_id": "d1"},
        {"hook_event_name": "Stop", "last_assistant_message": "Fixed the typo."}]},
]
json.dump(steps_a, open(f"{work}/steps-a.json", "w"))
json.dump(steps_d, open(f"{work}/steps-d.json", "w"))
open(f"{work}/long-message.txt", "w").write(long_message)
PY
}

write_stand_in() {
  sed -e "s|@HOOKS@|$HOOKS|" -e "s|@WORK@|$E2E_WORK|" >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code named by its argument. Each line written to its FIFO acts out its next
# step: the plugin's event.py for every event, each answer's sequence written to the terminal as
# Claude Code writes it.
import json
import os
import subprocess
import sys

name = sys.argv[1]
STEPS = json.load(open(f"@WORK@/steps-{name}.json", encoding="utf-8"))
COMMON = {"session_id": f"e2e-{name}", "transcript_path": "/tmp/e2e-transcript.jsonl",
          "cwd": os.getcwd(), "permission_mode": "default"}


def write(answer):
    sequence = json.loads(answer or "{}").get("terminalSequence")
    if sequence:
        sys.stdout.write(sequence)
        sys.stdout.flush()


print(f"Claude Code (stand-in {name}): steps come through its FIFO", flush=True)
done = 0
while done < len(STEPS):
    with open(f"@WORK@/{name}.fifo", encoding="utf-8") as fifo:
        for _ in fifo:
            if done >= len(STEPS):
                break
            step = STEPS[done]
            done += 1
            for event in step["events"]:
                write(subprocess.run([sys.executable, "@HOOKS@/event.py"],
                                     input=json.dumps({**COMMON, **event}),
                                     capture_output=True, text=True, check=False).stdout)
            print(f"step {done}: {step['label']}", flush=True)
for _ in sys.stdin:
    pass
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

# The next step of stand-in `$1`.
step_of() { echo next >"$E2E_WORK/$1.fifo"; settle 2; }

# How many banners the private bus logged, and how many mention Claude.
banners() { wc -l <"$E2E_WORK/banners.log"; }
claude_banners() { grep -c "|repo: Claude " "$E2E_WORK/banners.log" || true; }

# Whether the log's last banner is exactly this summary and body.
last_banner_is() {
  local last
  last=$(tail -1 "$E2E_WORK/banners.log")
  echo "last banner: $last"
  [[ $last == "Marley|$1|$2" ]]
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== three terminals: stand-ins A and D, and a plain one that holds the focus"
  type_text "claude a"
  press "" Return
  settle 2
  palette "workspace: new terminal"
  settle 3
  type_text "claude d"
  press "" Return
  settle 2
  palette "workspace: new terminal"
  settle 3
  shot 538-00-terminals

  echo "== A's finish: a banner with its message cut, and A marked"
  step_of a
  settle 1
  shot 538-01-marked
  cat "$E2E_WORK/banners.log"
  expect "one banner, titled for the finish" test "$(claude_banners)" = 1
  expect "its body is the message cut to 179 characters and an ellipsis" python3 - "$E2E_WORK" <<'PY'
import sys

work = sys.argv[1]
line = open(f"{work}/banners.log", encoding="utf-8").read().splitlines()[-1]
app, summary, body = line.split("|", 2)
message = " ".join(open(f"{work}/long-message.txt", encoding="utf-8").read().split())
sys.exit(0 if summary == "repo: Claude finished" and len(body) == 180 and body.endswith("…")
         and body[:179] == message[:179] else 1)
PY

  echo "== D's permission two seconds later: no second banner, D still marked"
  step_of d
  shot 538-02-held-back
  expect "the cooldown held D's banner back" test "$(claude_banners)" = 1

  echo "== looking at A clears its mark"
  press ALT 1
  settle 2
  shot 538-03-seen
  press ALT 3
  settle 1

  echo "== a manual compaction after the seen finish: nothing"
  settle 4
  step_of a
  shot 538-04-repeat-ping
  expect "the compaction showed no banner" test "$(claude_banners)" = 1

  echo "== A's permission in a new turn: a banner and a mark"
  settle 4
  step_of a
  shot 538-05-new-state
  expect "the permission's banner says what it asks" last_banner_is "repo: Claude needs input" "Using Bash: ls -la"

  echo "== A's question: a banner with its text"
  settle 6
  step_of a
  expect "the question's banner holds its text" last_banner_is "repo: Claude needs input" "Ship the release now?"

  echo "== A's failure: a banner naming its kind"
  settle 6
  step_of a
  expect "the failure's banner names its kind" last_banner_is "repo: Claude failed" "rate limit"

  echo "== a cleared session: no banner"
  settle 6
  local before
  before=$(banners)
  step_of a
  expect "the cleared session showed no banner" test "$(banners)" = "$before"

  echo "== an event in the focused terminal: no banner, no mark"
  press ALT 2
  settle 6
  before=$(banners)
  step_of d
  shot 538-06-focused
  expect "the focused terminal's finish showed no banner" test "$(banners)" = "$before"

  echo "== the user's own bus got nothing"
  cat "$E2E_WORK/banners.log"
  expect "no banner reached the user's bus" bash -c "! grep -q 'Member=Notify' '$E2E_WORK/user-bus.log'"
}
