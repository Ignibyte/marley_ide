# shellcheck shell=bash
# #540's e2e test: a Claude Code session resumed after a Marley restart. A stand-in `claude`, a
# Python script first on the terminal's PATH (the rail names a Python script by its file), prints
# the `SessionStart` frame #519's plugin would, for the session id in a file, with its folder and
# its source, logs its arguments and folder, and on a line `exit` prints `SessionEnd`
# (`prompt_input_exit`) and ends. Started in `sub`, the session comes back after a relaunch as
# `claude --resume <id>` in `sub`, and again after a second; once the user exits it, a relaunch
# gives a plain shell; with `marley.resume_agents` off, nothing is resumed.
compositor sway

GRID_X=800
GRID_Y=500
SESSION=11111111-2222-3333-4444-555555555555

# The log's lines, one per run of the stand-in: its arguments, then its folder.
runs() {
  cat "$E2E_WORK/claude.log"
  [[ $(wc -l <"$E2E_WORK/claude.log") -eq $1 ]]
}

# Whether the log's line `n` is a resume of the session in `sub`.
resumed_in_sub() {
  local line
  line=$(sed -n "${1}p" "$E2E_WORK/claude.log")
  echo "$line"
  [[ $line == "--resume $SESSION @ $E2E_WORK/repo/sub" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo/sub"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  printf '%s\n' "$SESSION" >"$E2E_WORK/session"
  : >"$E2E_WORK/claude.log"
  sed -e "s|@LOG@|$E2E_WORK/claude.log|" -e "s|@SESSION@|$E2E_WORK/session|" >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code: the frames its hooks would print, a line saying which session runs,
# and a log of how it was started.
import base64
import json
import os
import sys

args = sys.argv[1:]
folder = os.getcwd()
with open("@LOG@", "a", encoding="utf-8") as log:
    log.write(f"{' '.join(args)} @ {folder}\n")
resumed = "--resume" in args
session = args[args.index("--resume") + 1] if resumed else open("@SESSION@").read().strip()


def frame(summary):
    body = base64.b64encode(json.dumps(summary).encode()).decode()
    sys.stdout.write(f"\x1b]777;notify;marley-event;{body}\x07")
    sys.stdout.flush()


frame({"v": 1, "event": "SessionStart", "session_id": session, "cwd": folder,
       "source": "resume" if resumed else "startup"})
print(f"{'resumed' if resumed else 'session'} {session} in {folder}", flush=True)
for line in sys.stdin:
    if line.strip() == "exit":
        frame({"v": 1, "event": "SessionEnd", "session_id": session, "cwd": folder,
               "reason": "prompt_input_exit"})
        print("bye", flush=True)
        break
FAKE
  chmod +x "$bin/claude"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

relaunch() {
  quit_marley
  launch_marley
  settle 14
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  click "$GRID_X" "$GRID_Y"
  settle 1.5

  echo "== Claude Code in sub"
  type_text "cd sub"
  press "" Return
  settle 1
  type_text "claude"
  press "" Return
  settle 2
  shot 540-01-running

  echo "== relaunched: resumed in sub"
  relaunch
  shot 540-02-resumed
  expect "the restored terminal resumed the session in sub" resumed_in_sub 2

  echo "== relaunched again: resumed again"
  relaunch
  shot 540-03-again
  expect "the session was resumed after a second relaunch" resumed_in_sub 3

  echo "== the user exits Claude Code: a plain shell after the relaunch"
  click "$GRID_X" "$GRID_Y"
  settle 1
  type_text "exit"
  press "" Return
  settle 2
  relaunch
  shot 540-04-exited
  expect "nothing was resumed after the exit" runs 3

  echo "== resume_agents off"
  click "$GRID_X" "$GRID_Y"
  settle 1
  type_text "claude"
  press "" Return
  settle 2
  quit_marley
  profile_setting marley.resume_agents false
  launch_marley
  settle 14
  shot 540-05-off
  expect "nothing was resumed with the setting off" runs 4
}
