# shellcheck shell=bash
# #550's e2e test: a close or a quit asks before it ends a working agent, and a working terminal
# closed from its tab is held for undo. A stand-in `claude`, first on the terminal's PATH, runs the
# plugin's real `event.py` at each Return (the first step makes its seat working, the second idle),
# prints a tick every second, and writes its pid, so the scenario can tell whether it still runs.
# The profile holds a closed terminal for 8 seconds. Checks: the quit asks and Cancel keeps
# Marley; the tab's close asks, Close holds the agent alive and Ctrl-Shift-T brings the terminal
# back; an idle agent closes at once and ends; an unclaimed hold ends the agent; with the question
# off, a close still holds; and a quit with no working agent goes through.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py

# Gives the profile's `marley` settings block the key, or adds the block.
marley_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'PY'
import sys

path, key, value = sys.argv[1:]
text = open(path).read()
entry = f'"{key}": {value},'
if '"marley": {' in text:
    text = text.replace('"marley": {', '"marley": {\n    ' + entry, 1)
else:
    lines = text.split("\n")
    at = next(i for i, line in enumerate(lines) if line.startswith("{"))
    lines.insert(at + 1, '  "marley": {' + entry + '},')
    text = "\n".join(lines)
open(path, "w").write(text)
PY
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  cat >"$E2E_WORK/steps.json" <<'JSON'
[
  {"label": "working", "events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "Refactor the parser"},
    {"hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": "cargo build"}, "tool_use_id": "t1"}]},
  {"label": "idle", "events": [
    {"hook_event_name": "PostToolUse", "tool_name": "Bash", "tool_input": {"command": "cargo build"}, "tool_use_id": "t1"},
    {"hook_event_name": "Stop", "last_assistant_message": "The parser is refactored."}]}
]
JSON
  sed -e "s|@HOOK@|$HOOK|" -e "s|@STEPS@|$E2E_WORK/steps.json|" -e "s|@PID@|$E2E_WORK/claude.pid|" \
    >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code: a step of the plugin's hook events at each Return, a tick every second,
# and its pid in a file.
import json
import os
import subprocess
import sys
import threading
import time

STEPS = json.load(open("@STEPS@", encoding="utf-8"))
COMMON = {"session_id": f"e2e-{os.getpid()}", "transcript_path": "/tmp/e2e-transcript.jsonl",
          "cwd": os.getcwd(), "permission_mode": "default"}
with open("@PID@", "w", encoding="utf-8") as pid:
    pid.write(str(os.getpid()))


def tick():
    count = 0
    while True:
        count += 1
        print(f"tick {count}", flush=True)
        time.sleep(1)


threading.Thread(target=tick, daemon=True).start()
print("Claude Code (stand-in): Return for each step", flush=True)
for number, step in enumerate(STEPS, 1):
    if not sys.stdin.readline():
        break
    for event in step["events"]:
        answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps({**COMMON, **event}),
                                capture_output=True, text=True, check=False).stdout
        sequence = json.loads(answer or "{}").get("terminalSequence")
        if sequence:
            sys.stdout.write(sequence)
            sys.stdout.flush()
    print(f"step {number}: {step['label']}", flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$bin/claude"
  marley_setting undo_close_seconds 8
  git init -q -b close "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

# Whether the newest stand-in still runs.
alive() {
  local pid
  pid=$(cat "$E2E_WORK/claude.pid")
  kill -0 "$pid" 2>/dev/null
}
gone() { ! alive; }

# Whether Marley's MCP server lists a terminal: the tab is there.
listed() {
  mcp_agent terminals | tee "$E2E_WORK/terminals.txt"
  holds "$E2E_WORK/terminals.txt" "  terminal "
}
unlisted() { ! listed; }

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# The stand-in in the focused terminal, taken to its first step: working.
start_agent() {
  type_text "claude"
  press "" Return
  settle 3
  press "" Return
  settle 2
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  start_agent
  echo "== the quit asks"
  palette "zed: quit"
  settle 2
  shot 550-01-quit-asks
  press "" Escape
  settle 2
  shot 550-02-still-running
  expect "Cancel keeps Marley running" test -n "$(marley_pid)"
  echo "== the tab's close asks, and Close holds the agent"
  press "CTRL SHIFT" w
  settle 2
  shot 550-03-close-asks
  press "" Return
  settle 2
  shot 550-04-held
  expect "the tab is gone" unlisted
  expect "the held agent still runs" alive
  echo "== Ctrl-Shift-T brings it back"
  press "CTRL SHIFT" t
  settle 3
  shot 550-05-restored
  expect "the terminal is back" listed
  expect "its agent still runs" alive
  echo "== an idle agent closes at once"
  press "" Return
  settle 2
  press "CTRL SHIFT" w
  settle 2
  shot 550-06-idle-closes
  expect "an idle agent's terminal closes, and its agent ends" gone
  echo "== a hold nobody claims ends the agent"
  palette "workspace: new terminal"
  settle 3
  start_agent
  press "CTRL SHIFT" w
  settle 2
  press "" Return
  settle 2
  expect "held" alive
  settle 9
  press "CTRL SHIFT" t
  settle 2
  shot 550-07-expired
  expect "the hold ended the agent" gone
  expect "nothing came back" unlisted
  echo "== with the question off, a close still holds"
  marley_setting ask_before_ending_a_working_agent false
  settle 2
  palette "workspace: new terminal"
  settle 3
  start_agent
  press "CTRL SHIFT" w
  settle 2
  shot 550-08-no-ask
  expect "no question: the tab is gone" unlisted
  expect "and the agent is held" alive
  settle 9
  echo "== a quit with no working agent"
  quit_marley
}
