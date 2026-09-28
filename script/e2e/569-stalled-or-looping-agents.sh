# shellcheck shell=bash
# #569's e2e test: stalled or looping agents. #566's stand-in `claude`, named through MARLEY_CLAUDE,
# acts out one step at each Enter through the plugin's real hook, in one session with a prompt id
# per turn, and can start and end a child that burns a core, standing in for a tool at work. The
# profile sets the first quiet check at 10 seconds (the next at 20, 40 and 80) and turns #565's
# layer on with the scratch repository listed; recorded answers stand in for Jev. A tool whose
# processes burn CPU is a long task and asks nothing (REQ-002); the same `Bash: cargo test` ended
# three times reads `looping?` from the rule alone (REQ-001); a quiet turn with nothing running is
# asked about at its check with the state masked and cut (REQ-003) and reads `stalled?` with a mark
# (REQ-004), whose tooltip gives the reading (REQ-005) and whose labels agents read (REQ-008); the
# next event takes the flag off and logs the outcome (REQ-009); `cannot_tell` flags nothing
# (REQ-006); `act` posts one banner on the scenario's private bus while the terminal is out of
# sight (REQ-007); `off` makes no call (REQ-010); the stand-in reads nothing but the scenario's
# Enters (REQ-011); and the settings page shows both items (REQ-012).
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The warning mark on the rail's agent row, for its tooltip, from the first run's shots.
MARK_X=${MARK_X:-207}
MARK_Y=${MARK_Y:-136}
# A point in the Settings window's page to scroll at.
SETTINGS_X=${SETTINGS_X:-1200}
SETTINGS_Y=${SETTINGS_Y:-500}

# Merges the JSON object `$1` into `marley` in the profile copy's settings, one level deep for
# `system_one` (565's `system_one_setting`, widened).
marley_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'PY'
import json, pathlib, sys

path, changes = pathlib.Path(sys.argv[1]), json.loads(sys.argv[2])
text = path.read_text() if path.exists() else "{}"
def skip_blank(index):
    while index < len(text):
        if text[index] in " \t\r\n":
            index += 1
        elif text.startswith("//", index):
            end = text.find("\n", index)
            index = len(text) if end < 0 else end
        elif text.startswith("/*", index):
            end = text.find("*/", index + 2)
            index = len(text) if end < 0 else end + 2
        else:
            break
    return index


out, index, in_string = [], 0, False
while index < len(text):
    character = text[index]
    if in_string:
        out.append(character)
        if character == "\\" and index + 1 < len(text):
            out.append(text[index + 1])
            index += 2
            continue
        if character == '"':
            in_string = False
        index += 1
        continue
    if character == '"':
        in_string = True
    elif text.startswith("//", index) or text.startswith("/*", index):
        index = skip_blank(index)
        continue
    elif character == ",":
        ahead = skip_blank(index + 1)
        if ahead < len(text) and text[ahead] in "}]":
            index += 1
            continue
    out.append(character)
    index += 1
cleaned = "".join(out).strip()
settings = json.loads(cleaned) if cleaned else {}
marley = settings.setdefault("marley", {})
for key, value in changes.items():
    if key == "system_one":
        layer = marley.setdefault("system_one", {})
        for inner, setting in value.items():
            if inner == "uses":
                layer.setdefault("uses", {}).update(setting)
            else:
                layer[inner] = setting
    else:
        marley[key] = value
path.write_text(json.dumps(settings, indent=2) + "\n")
PY
}

# Sets the stall kind's mode, and the provider when a second argument names one.
stall_mode() {
  local provider=${2:+, \"provider\": \"$2\"}
  marley_setting "{\"system_one\": {\"uses\": {\"stall_kind\": \"$1\"}$provider}}"
  settle 2
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config
  mkdir -p "$home" "$bin" "$config/plugins" "$E2E_PROFILE/system_one"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  # The plugin listed at the version Marley ships, so the agent bar offers no update.
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/$SHIPPED", "version": "$SHIPPED"}]}}
JSON
  export CLAUDE_CONFIG_DIR=$config
  # A private session bus whose notification server logs each banner (535's), so none reaches the
  # user's desktop.
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  write_notification_server
  : >"$E2E_WORK/banners.log"
  python3 -u "$E2E_WORK/notifications.py" "$E2E_WORK/banners.log" >"$E2E_WORK/notifications.out" 2>&1 &
  echo "$!" >"$E2E_WORK/notifications.pid"
  # A token assembled here, so no file of the repository holds one.
  local token
  token=$(printf 'gh%s_%s' p "$(printf 'Fake%.0s' {1..9})")
  python3 - "$E2E_WORK/steps.json" "$token" <<'PY'
import json, sys

path, token = sys.argv[1:]


def call(tool, target, number, prompt_id=None):
    key = "command" if tool == "Bash" else "file_path"
    fields = {"tool_name": tool, "tool_input": {key: target}, "tool_use_id": f"s1-{number}"}
    if prompt_id:
        fields["prompt_id"] = prompt_id
    return fields


def begins(tool, target, number, prompt_id=None):
    return {"hook_event_name": "PreToolUse", **call(tool, target, number, prompt_id)}


def ends(tool, target, number, prompt_id=None):
    return {"hook_event_name": "PostToolUse", **call(tool, target, number, prompt_id)}


def prompt(text):
    return {"hook_event_name": "UserPromptSubmit", "prompt": text}


def stop(prompt_id, message):
    return {"hook_event_name": "Stop", "prompt_id": prompt_id, "last_assistant_message": message}


flaky = "Fix the flaky test in the scheduler crate. " + "It fails one run in ten on the timer. " * 9
loop = [event for number in (11, 12, 13)
        for event in (begins("Bash", "cargo test flaky", number), ends("Bash", "cargo test flaky", number))]
steps = [
    {"label": "a long task", "prompt_id": "p1", "actions": [
        prompt("Run the whole test suite"), begins("Bash", "cargo test --workspace", 1),
        {"sleep": 2}, {"spawn": True}]},
    {"label": "a loop", "prompt_id": "p2", "actions": [
        {"kill": True}, ends("Bash", "cargo test --workspace", 1, "p1"),
        stop("p1", "The suite passes."), prompt("Make the flaky test pass"), *loop]},
    {"label": "quiet", "prompt_id": "p3", "actions": [
        stop("p2", "I could not make it pass."), prompt(flaky),
        begins("Read", "src/scheduler.rs", 21), ends("Read", "src/scheduler.rs", 21),
        {"say": "Reading src/scheduler.rs"}, {"say": f"export API_TOKEN={token}"}]},
    {"label": "the next event", "prompt_id": "p3", "actions": [stop("p3", "The test waits on a lock.")]},
    {"label": "cannot tell", "prompt_id": "p5", "actions": [
        prompt("Tidy the imports"), begins("Edit", "src/lib.rs", 51), ends("Edit", "src/lib.rs", 51)]},
    {"label": "act", "prompt_id": "p6", "actions": [
        stop("p5", "Tidied."), prompt("Update the changelog"),
        begins("Read", "CHANGELOG.md", 61), ends("Read", "CHANGELOG.md", 61)]},
    {"label": "off", "prompt_id": "p7", "actions": [
        stop("p6", "Updated."), prompt("Rename the module"),
        begins("Read", "src/lib.rs", 71), ends("Read", "src/lib.rs", 71)]},
]
json.dump(steps, open(path, "w"), indent=1)
PY
  # The recorded answers. A quiet turn is asked at each check it reaches; a row answers once
  # unless it repeats.
  cat >"$E2E_PROFILE/system_one/replay.jsonl" <<'JSONL'
{"set": "stall_kind/1", "match": "prompt: Fix the flaky test", "answers": {"kind": {"type": "choice", "choice": "frozen", "confidence": 0.91, "probabilities": {"frozen": 0.91}}, "repeating": {"type": "noul", "noul": 0.1}, "progress": {"type": "noul", "noul": 0.05}}}
{"set": "stall_kind/1", "match": "prompt: Tidy the imports", "repeat": true, "answers": {"kind": {"type": "choice", "choice": "cannot_tell", "confidence": 0.7, "probabilities": {"cannot_tell": 0.7}}, "repeating": {"type": "noul", "noul": 0.5}, "progress": {"type": "noul", "noul": 0.5}}}
{"set": "stall_kind/1", "match": "prompt: Update the changelog", "answers": {"kind": {"type": "choice", "choice": "stuck", "confidence": 0.84, "probabilities": {"stuck": 0.84}}, "repeating": {"type": "noul", "noul": 0.1}, "progress": {"type": "noul", "noul": 0.1}}}
JSONL
  sed -e "s|@HOOK@|$HOOK|" -e "s|@STEPS@|$E2E_WORK/steps.json|" -e "s|@LOG@|$E2E_WORK/plugin.log|" \
    -e "s|@READ@|$E2E_WORK/stdin.log|" -e "s|@BURNER@|$E2E_WORK/burner.pid|" \
    >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code (#566's): `claude plugin ...` logs its arguments. Otherwise, at each line
# it reads, it acts out the next step: each event through the plugin's hook, in the step's session
# and prompt, writing each answer's sequence to its terminal; `sleep`, `say` (a line on its
# terminal), `spawn` (a child that burns a core for at most 90 seconds) and `kill`. Every line it
# reads is logged, so the scenario can show nothing but its own Enters reached it.
import json
import os
import subprocess
import sys
import time

if sys.argv[1:2] == ["plugin"]:
    with open("@LOG@", "a", encoding="utf-8") as log:
        log.write(" ".join(sys.argv[1:]) + "\n")
    sys.exit(0)

STEPS = json.load(open("@STEPS@", encoding="utf-8"))
COMMON = {"transcript_path": "/tmp/e2e-transcript.jsonl", "cwd": os.getcwd(),
          "permission_mode": "default", "session_id": "s1"}
BURN = "import time\nend = time.monotonic() + 90\nwhile time.monotonic() < end:\n    pass\n"
burners = []


def read_line():
    line = sys.stdin.readline()
    with open("@READ@", "a", encoding="utf-8") as log:
        log.write(json.dumps(line) + "\n")
    return line


print("Claude Code (stand-in): press Enter for each step", flush=True)
for number, step in enumerate(STEPS, 1):
    if not read_line():
        break
    for action in step["actions"]:
        if "sleep" in action:
            time.sleep(action["sleep"])
        elif "say" in action:
            print(action["say"], flush=True)
        elif action.get("spawn"):
            burner = subprocess.Popen([sys.executable, "-c", BURN])
            burners.append(burner)
            with open("@BURNER@", "a", encoding="utf-8") as pids:
                pids.write(f"{burner.pid}\n")
        elif action.get("kill"):
            for burner in burners:
                burner.kill()
                burner.wait()
            burners.clear()
        else:
            payload = {**COMMON, "prompt_id": step["prompt_id"], **action}
            answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps(payload),
                                    capture_output=True, text=True, check=False).stdout
            sequence = json.loads(answer or "{}").get("terminalSequence")
            if sequence:
                sys.stdout.write(sequence)
                sys.stdout.flush()
    print(f"step {number}: {step['label']}", flush=True)
while read_line():
    pass
FAKE
  chmod +x "$bin/claude"
  # The `claude` Marley runs for the plugin's commands: the stand-in, never the real one.
  export MARLEY_CLAUDE=$bin/claude
  git init -q -b main "$E2E_WORK/repo"
  marley_setting "{\"stall_check_after_seconds\": 10, \"system_one\": {\"enabled\": true, \"provider\": \"replay\", \"projects\": [\"$E2E_WORK/repo\"], \"uses\": {\"stall_kind\": \"act\"}}}"
  open_path "$E2E_WORK/repo"
}

teardown() {
  local pid
  if [[ -f $E2E_WORK/burner.pid ]]; then
    while read -r pid; do kill "$pid" 2>/dev/null; done <"$E2E_WORK/burner.pid"
  fi
  [[ -f $E2E_WORK/notifications.pid ]] && kill "$(cat "$E2E_WORK/notifications.pid")" 2>/dev/null
  [[ -f $E2E_WORK/bus.pid ]] && kill "$(head -1 "$E2E_WORK/bus.pid")" 2>/dev/null
  return 0
}

write_notification_server() {
  cat >"$E2E_WORK/notifications.py" <<'PY'
# A desktop notification server on the scenario's private bus (535's): logs each banner and
# answers as a real server does.
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

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# The next step of the stand-in, and time for its events to land.
next_step() {
  press "" Return
  settle 3
}

away() { sway_msg workspace 2 >/dev/null; settle 1; }
back() { sway_msg workspace 1 >/dev/null; settle 1; }

# The stand-in agent's reading of the seats' labels, kept as $E2E_WORK/<label>.txt.
seat_labels() {
  mcp_agent fleet-labels | tee "$E2E_WORK/$1.txt"
}

# Today's rows of the stall kind: its calls, and every outcome.
stall_calls() {
  cat "$E2E_PROFILE"/system_one/calls-*.jsonl 2>/dev/null | grep '"row":"call"' |
    grep '"use":"stall_kind"' || true
}
outcomes() {
  cat "$E2E_PROFILE"/system_one/calls-*.jsonl 2>/dev/null | grep '"row":"outcome"' || true
}

# Whether an outcome row says `$1`, a regular expression.
outcome_says() {
  outcomes | grep -qE -- "$1"
}

# Whether the labels kept as $E2E_WORK/<label>.txt hold no flag.
no_flag() {
  ! grep -qF "flag '" "$E2E_WORK/$1.txt"
}

# The last stall call row, kept as $E2E_WORK/<label>.json, with its provider, reading and state
# printed.
last_call() {
  stall_calls | tail -n 1 >"$E2E_WORK/$1.json"
  python3 - "$E2E_WORK/$1.json" <<'PY'
import json, sys

row = json.load(open(sys.argv[1]))
print(f"  {row['use']} {row['mode']} {row['provider']} {row['set']}: {row['reading']}")
for line in (row.get("state") or "").splitlines():
    print(f"    {line}")
PY
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "claude"
  press "" Return
  settle 3

  echo "== a long task"
  next_step
  settle 22
  shot 569-01-long-task
  seat_labels long-task
  expect "a tool that burns CPU carries no flag" no_flag long-task
  expect "and nothing was asked, two checks on" test "$(stall_calls | wc -l)" -eq 0

  echo "== a loop"
  stall_mode suggest rules
  next_step
  shot 569-02-looping
  seat_labels looping
  expect "the loop is flagged from the rule" holds "$E2E_WORK/looping.txt" \
    "flag 'looping'" "flag_source 'rules'"
  last_call loop
  expect "one rules row, no model called" holds "$E2E_WORK/loop.json" '"provider":"rules"' \
    '"set":"stall_kind/1"' 'tool: Bash' 'repeats: ended 3 times in a row' 'repeating: yes'
  expect "the loop's row is the only stall call" test "$(stall_calls | wc -l)" -eq 1

  echo "== quiet, stalled"
  stall_mode suggest replay
  next_step
  expect "the loop's end is its outcome" outcome_says 'the loop ended .* later: Stop'
  settle 12
  shot 569-03-stalled
  seat_labels stalled
  expect "the seat carries the flag, its source and its confidence" holds "$E2E_WORK/stalled.txt" \
    "flag 'stalled:frozen'" "flag_source 'model'" "flag_confidence '0.91'"
  last_call stalled
  expect "the check asked the set with the state" holds "$E2E_WORK/stalled.json" \
    '"set":"stall_kind/1"' '"provider":"replay"' 'prompt: Fix the flaky test' 'tools use the CPU: no' \
    'kind: frozen 0.91'
  expect "the prompt was cut" python3 -c '
import json, sys
state = json.load(open(sys.argv[1]))["state"]
line = next(line for line in state.splitlines() if line.startswith("prompt: "))
sys.exit(0 if line.endswith("…") and len(line) <= 310 else 1)' "$E2E_WORK/stalled.json"
  expect "the token on the terminal was masked" holds "$E2E_WORK/stalled.json" '[redacted: secret]'
  expect "and not as it was written" bash -c "! grep -q 'FakeFake' '$E2E_WORK/stalled.json'"
  expect "no transcript path" bash -c "! grep -q 'e2e-transcript' '$E2E_WORK/stalled.json'"
  expect "one ask for the check" test "$(stall_calls | wc -l)" -eq 2

  echo "== the tooltip"
  pointer_to "$MARK_X" "$MARK_Y"
  settle 2
  shot 569-04-tooltip
  pointer_to 5 5

  echo "== the next event"
  next_step
  shot 569-07-cleared
  seat_labels cleared
  expect "the next event takes the flag off" no_flag cleared
  expect "and logs the call's outcome" outcome_says 'the next event .* later: Stop'

  echo "== cannot tell"
  next_step
  settle 12
  shot 569-05-cannot-tell
  seat_labels cannot-tell
  expect "cannot_tell flags nothing" no_flag cannot-tell
  last_call cannot-tell
  expect "the reading was logged" holds "$E2E_WORK/cannot-tell.json" 'prompt: Tidy the imports'

  echo "== act"
  stall_mode act
  press "" Return
  settle 1
  away
  settle 26
  back
  shot 569-06-act
  cat "$E2E_WORK/banners.log"
  expect "one banner for the episode" test "$(grep -c 'repo: Claude Code may be stuck' "$E2E_WORK/banners.log")" -eq 1
  expect "it names the kind and the reading" holds "$E2E_WORK/banners.log" 'stuck (0.84)'

  echo "== off"
  stall_mode off
  local before
  before=$(stall_calls | wc -l)
  next_step
  settle 12
  shot 569-08-off
  seat_labels off
  expect "off shows no flag" no_flag off
  expect "and makes no call" test "$(stall_calls | wc -l)" -eq "$before"

  echo "== what the stand-in read"
  cat "$E2E_WORK/stdin.log"
  expect "nothing but the scenario's Enters reached the agent" python3 -c '
import json, sys
lines = [json.loads(line) for line in open(sys.argv[1])]
sys.exit(0 if lines and all(line == "\n" for line in lines) else 1)' "$E2E_WORK/stdin.log"

  echo "== the settings"
  palette "marley: open settings"
  settle 4
  pointer_to "$SETTINGS_X" "$SETTINGS_Y"
  settle 1
  scroll 6
  settle 2
  shot 569-09a-agents
  scroll 24
  settle 2
  shot 569-09-settings
}
