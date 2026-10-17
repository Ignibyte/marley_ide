# shellcheck shell=bash
# #796's visual check: a terminal's rail row is two lines tall whatever it shows under its title,
# so a line that comes or goes moves neither the title nor the rows under it. A scratch repository
# with a folder `sub`, three terminals in it, listed in window order (`rail_order: window`) so
# attention moves no row: a shell started in `sub` (the first shell's `.bashrc` goes there, so no
# command has run), a stand-in Claude Code that plays the plugin's real `event.py` at an Enter (as
# #519's does), and a shell at the root with nothing run.
# - `796-01-before`: the shell shows `sub` over a blank, the agent its status over a blank, the idle
#   shell two blanks; the three rows are one height (REQ-001).
# - `796-02-command`: the shell runs `sleep 30`: `sub` over `sleep 30 · running`, the rows where
#   `796-01-before` has them (REQ-002).
# - `796-03-activity`: the agent starts Bash: its status over the tool, the rows where
#   `796-02-command` has them (REQ-003).
# - `796-04-root`: the shell stops and goes to the root: `cd .. · done` over a blank, the rows where
#   `796-03-activity` has them (REQ-004).
compositor sway

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo/sub"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
# The first shell starts in sub, the others at the root.
if mkdir "$E2E_WORK/first-shell" 2>/dev/null; then cd "$E2E_WORK/repo/sub" || return; fi
RC
  terminal_env HOME "$home"
  sed -e "s|@HOOK@|$HOOK|" >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code: at the first line it reads it runs the plugin's hook with a prompt and a
# tool's start, as Claude Code runs a hook, and writes each answer's sequence to its terminal.
import json
import os
import subprocess
import sys

COMMON = {"session_id": "e2e-session", "transcript_path": "/tmp/e2e-transcript.jsonl",
          "cwd": os.getcwd(), "permission_mode": "default"}
EVENTS = [
    {"hook_event_name": "SessionStart", "source": "startup"},
    {"hook_event_name": "UserPromptSubmit", "prompt": "List the files"},
    {"hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": "ls -la"},
     "tool_use_id": "t1"},
]

print("Claude Code (stand-in): press Enter to start a tool", flush=True)
if sys.stdin.readline():
    for event in EVENTS:
        answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps({**COMMON, **event}),
                                capture_output=True, text=True, check=False).stdout
        sequence = json.loads(answer or "{}").get("terminalSequence")
        if sequence:
            sys.stdout.write(sequence)
            sys.stdout.flush()
    print("Bash running", flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$bin/claude"
  profile_setting marley.rail_order '"window"'
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  palette "workspace: new terminal"
  settle 3
  palette "workspace: new terminal"
  settle 3

  echo "== the agent before its first event, and the rows before any line comes"
  press ALT 2
  settle 1
  type_text "claude"
  press "" Return
  press ALT 3
  # Past the quiet timer's 2 seconds, with no event yet.
  settle 4
  shot 796-01-before

  echo "== the shell in sub runs a command"
  press ALT 1
  settle 1
  type_text "sleep 30"
  press "" Return
  press ALT 3
  settle 3
  shot 796-02-command

  echo "== the agent starts a tool"
  press ALT 2
  settle 1
  press "" Return
  press ALT 3
  settle 3
  shot 796-03-activity

  echo "== the shell stops and goes to the root"
  press ALT 1
  settle 1
  press CTRL c
  settle 1
  type_text "cd .."
  press "" Return
  press ALT 3
  settle 3
  shot 796-04-root
}
