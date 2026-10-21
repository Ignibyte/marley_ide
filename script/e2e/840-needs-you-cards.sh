# shellcheck shell=bash
# #840's visual check: the rail's Needs you entries as cards. Projects `b` and `c`, handed over one
# after the other, each run #833's stand-in Claude Code (the plugin's real `event.py` behind it),
# which asks to write a file and waits; `b` asks first. `marley.tmux_hosts` is `[]`.
# - Both waiting: NEEDS YOU 2 · longest first, b's card over c's, each with its tile, who · where,
#   waited, the ask and Open (`840-01-night`, REQ-001, REQ-002).
# - Marley Day (`840-02-day`, REQ-001, REQ-002).
compositor sway

HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}

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
  write_steps
  write_stand_in "$bin/claude"
  for name in b c; do
    mkfifo "$E2E_WORK/$name.fifo"
    mkdir -p "$E2E_WORK/$name"
    git init -q -b main "$E2E_WORK/$name"
  done
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
  # Trusts b.
  press "" Return
  settle 4
  echo "== b waits"
  step_of b waiting
  settle 65

  echo "== c, handed over, waits"
  touch "$E2E_WORK/start-c"
  hand_over "$E2E_WORK/c"
  settle 6
  press "" Return
  settle 4
  step_of c waiting
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 840-01-night

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 840-02-day
}
