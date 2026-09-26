# shellcheck shell=bash
# #519's e2e test: Claude Code's hook events in the rail. A stand-in `claude`, a Python script
# first on the terminal's PATH (the rail names a Python script by its file), acts out a session:
# at each Enter it runs the plugin's real `hooks/event.py` with the next step's recorded payloads
# on stdin, as Claude Code runs a hook, and writes each answer's `terminalSequence` to its
# terminal, as Claude Code does. The agent's row shows what the events say: working with the
# prompt and the tool, waiting on a permission while another tool finishes, a subagent counted,
# idle with the last message, the user's prompt kept through injected ones, failed with the
# error. No frame marks a bell or posts a desktop notification. Setup checks the hook itself:
# `{}` outside a Marley terminal, and a sequence under 4,096 bytes for a 100 KB payload.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py

# Whether the hook answers `{}` outside a Marley terminal.
answers_nothing_outside() {
  local answer
  answer=$(printf '{"hook_event_name": "Stop"}' | env -u TERM_PROGRAM python3 "$HOOK")
  echo "outside: $answer"
  [[ $answer == "{}" ]]
}

# Whether, for 100 KB prompts, tool inputs and messages and 5,000-byte paths, the sequence stays
# under 4,096 bytes and still carries the event and what its row shows.
stays_under_the_cap() {
  python3 - "$HOOK" <<'PY'
import base64, json, os, subprocess, sys

hook = sys.argv[1]
big = "x" * 100_000
common = {"session_id": "s", "permission_mode": "default",
          "cwd": "/" + "c" * 5_000, "transcript_path": "/" + "t" * 5_000}
# Each payload, and the fields of the summary its row shows.
payloads = [
    ({"hook_event_name": "UserPromptSubmit", "prompt": big}, ["prompt"]),
    ({"hook_event_name": "PreToolUse", "tool_name": "Write", "tool_use_id": "t1",
      "tool_input": {"file_path": "/" + "d/" * 3_000 + "f", "content": big}}, ["tool", "preview"]),
    ({"hook_event_name": "Stop", "last_assistant_message": big}, ["message"]),
]
env = dict(os.environ, TERM_PROGRAM="zed")
fits = True
for payload, shown in payloads:
    answer = subprocess.run([sys.executable, hook], input=json.dumps({**common, **payload}),
                            capture_output=True, text=True, env=env, check=False).stdout
    sequence = json.loads(answer).get("terminalSequence", "")
    body = sequence.removeprefix("\x1b]777;notify;marley-event;").removesuffix("\x07")
    summary = json.loads(base64.b64decode(body)) if sequence else {}
    size = len(sequence.encode())
    kept = [key for key in shown if key in summary]
    print(f"{payload['hook_event_name']}: a {size}-byte sequence carrying "
          f"{summary.get('event')}, with {', '.join(kept) or 'nothing it shows'}")
    fits = (fits and 0 < size < 4_096 and summary.get("event") == payload["hook_event_name"]
            and kept == shown)
sys.exit(0 if fits else 1)
PY
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin"
  expect "the hook answers {} outside a Marley terminal" answers_nothing_outside
  expect "the hook's sequence stays under 4,096 bytes" stays_under_the_cap
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  write_steps "$E2E_WORK/steps.json"
  sed -e "s|@HOOK@|$HOOK|" -e "s|@STEPS@|$E2E_WORK/steps.json|" >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code. At each line it reads it runs the plugin's hook with the next step's
# payloads, as Claude Code runs a hook, and writes each answer's sequence to its terminal.
import json
import os
import subprocess
import sys

STEPS = json.load(open("@STEPS@", encoding="utf-8"))
COMMON = {"session_id": "e2e-session", "transcript_path": "/tmp/e2e-transcript.jsonl",
          "cwd": os.getcwd(), "permission_mode": "default"}

print("Claude Code (stand-in): press Enter for each step", flush=True)
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
  # The desktop's notification service, watched for a Notify from a frame.
  stdbuf -oL busctl --user monitor org.freedesktop.Notifications >"$E2E_WORK/notifications.log" 2>&1 &
  echo "$!" >"$E2E_WORK/monitor.pid"
  git init -q -b events "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  if [[ -f $E2E_WORK/monitor.pid ]]; then
    kill "$(cat "$E2E_WORK/monitor.pid")" 2>/dev/null || true
  fi
}

# The session the stand-in acts out, a step per Enter.
write_steps() {
  cat >"$1" <<'JSON'
[
  {"label": "a prompt, and Bash running", "events": [
    {"hook_event_name": "SessionStart", "source": "startup"},
    {"hook_event_name": "UserPromptSubmit", "prompt": "Add a README to the project"},
    {"hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": "ls -la"}, "tool_use_id": "t1"}
  ]},
  {"label": "Write asks for permission while Read runs", "events": [
    {"hook_event_name": "PostToolUse", "tool_name": "Bash", "tool_input": {"command": "ls -la"}, "tool_use_id": "t1"},
    {"hook_event_name": "PreToolUse", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# repo"}, "tool_use_id": "t2"},
    {"hook_event_name": "PreToolUse", "tool_name": "Read", "tool_input": {"file_path": "src/lib.txt"}, "tool_use_id": "t3"},
    {"hook_event_name": "PermissionRequest", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# repo"}},
    {"hook_event_name": "PostToolUse", "tool_name": "Read", "tool_input": {"file_path": "src/lib.txt"}, "tool_use_id": "t3"}
  ]},
  {"label": "the Write done; a subagent searching", "events": [
    {"hook_event_name": "PostToolUse", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# repo"}, "tool_use_id": "t2"},
    {"hook_event_name": "PreToolUse", "tool_name": "Task", "tool_input": {"description": "Find the TODOs"}, "tool_use_id": "t4"},
    {"hook_event_name": "SubagentStart", "agent_id": "sub-1", "agent_type": "general-purpose"},
    {"hook_event_name": "PreToolUse", "tool_name": "Grep", "tool_input": {"pattern": "TODO"}, "tool_use_id": "t5", "agent_id": "sub-1"}
  ]},
  {"label": "the turn ends", "events": [
    {"hook_event_name": "PostToolUse", "tool_name": "Grep", "tool_input": {"pattern": "TODO"}, "tool_use_id": "t5", "agent_id": "sub-1"},
    {"hook_event_name": "SubagentStop", "agent_id": "sub-1", "last_assistant_message": "No TODOs."},
    {"hook_event_name": "PostToolUse", "tool_name": "Task", "tool_input": {"description": "Find the TODOs"}, "tool_use_id": "t4"},
    {"hook_event_name": "Stop", "last_assistant_message": "I added README.md with a short description of the project."}
  ]},
  {"label": "a harness's prompt, and the continuation after a compact", "events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "<task-notification>\nThe build finished.\n</task-notification>"},
    {"hook_event_name": "UserPromptSubmit", "prompt": "This session is being continued from a previous conversation that ran out of context."},
    {"hook_event_name": "Stop", "last_assistant_message": "Noted."}
  ]},
  {"label": "a new prompt fails", "events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "Run the tests"},
    {"hook_event_name": "StopFailure", "error": "rate_limit"}
  ]}
]
JSON
}

# The next step: Enter to the stand-in, then time for the rail to redraw.
next_step() {
  press "" Return
  settle 2
  shot "$1"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "claude"
  press "" Return
  # Past the quiet timer's 2 seconds, with no event yet.
  settle 4
  shot 519-00-before-events
  next_step 519-01-working
  next_step 519-02-waiting
  next_step 519-03-subagent
  next_step 519-04-idle
  next_step 519-05-injected
  next_step 519-06-failed
  mcp_agent terminal-read claude | tee "$E2E_WORK/stand-in.txt"
  expect "the stand-in acted out every step" \
    holds "$E2E_WORK/stand-in.txt" "step 1: a prompt" "step 6: a new prompt fails"
  # A call to the service, so the log shows the monitor saw calls, then no Notify from a frame.
  busctl --user call org.freedesktop.Notifications /org/freedesktop/Notifications \
    org.freedesktop.Notifications GetServerInformation >/dev/null
  settle 1
  expect "the monitor watched the notification service" \
    holds "$E2E_WORK/notifications.log" "Member=GetServerInformation"
  expect "no frame posted a desktop notification" \
    bash -c "! grep -q marley-event '$E2E_WORK/notifications.log'"
}
