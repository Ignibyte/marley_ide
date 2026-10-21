# shellcheck shell=bash
# #870's visual check: prompts as the Dialogs board draws them. Project `b` runs #833's stand-in
# Claude Code (the plugin's real `event.py` behind it), which asks to write a file and waits; its
# README is edited and left unsaved. `marley.tmux_hosts` is `[]`.
# - Closing the README: Zed's save prompt as a card, Cancel quiet at the left, Don't Save plain,
#   Save primary at the right (`870-01-save`, REQ-001, REQ-002).
# - Closing the waiting agent's terminal: the close guard's card, its detail the agent's line,
#   Cancel, Show, Close as a danger (`870-02-close`, REQ-001, REQ-002).
# - Marley Day, the close guard's card (`870-03-day`, REQ-001).
compositor sway

HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}

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
  write_steps
  write_stand_in "$bin/claude"
  mkfifo "$E2E_WORK/b.fifo"
  mkdir -p "$E2E_WORK/b"
  git init -q -b main "$E2E_WORK/b"
  printf '# b\n' >"$E2E_WORK/b/README.md"
  touch "$E2E_WORK/start-b"
  profile_setting marley.tmux_hosts '[]'
  open_path "$E2E_WORK/b"
}

# The stand-in's one step: it asks to write a file and waits on the user.
write_steps() {
  cat >"$E2E_WORK/steps.json" <<'JSON'
{
  "waiting": [
    {"hook_event_name": "SessionStart", "source": "startup"},
    {"hook_event_name": "UserPromptSubmit", "prompt": "Write the README"},
    {"hook_event_name": "PreToolUse", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# readme"}, "tool_use_id": "t2"},
    {"hook_event_name": "PermissionRequest", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# readme"}}
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


# Stand-in `$1` acts out step `$2`; a stand-in that is not reading fails the step, not the run.
step_of() {
  timeout 5 bash -c "echo $2 >'$E2E_WORK/$1.fifo'" ||
    echo "step_of: stand-in $1 is not reading" >&2
}

# Runs `$1` from the command palette.
palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 12
  # Trusts b.
  press "" Return
  settle 4
  echo "== b waits"
  step_of b waiting
  settle 5

  echo "== the README, edited, closed"
  palette "file finder: toggle" 3
  type_text "README.md"
  settle 3
  press "" Return
  settle 3
  type_text "edited "
  settle 1
  palette "pane: close active item" 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 870-01-save
  press "" Escape
  settle 2

  echo "== the waiting agent's terminal, closed"
  palette "pane: activate previous item" 2
  palette "pane: close active item" 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 870-02-close

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 870-03-day
}
