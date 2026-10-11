# shellcheck shell=bash
# #750's visual check: Codex resumes after a restart, and a turn the restart cut off offers
# Continue. A scratch repo with `sub`. #540's stand-in `claude` prints the frames Claude Code's
# hooks would (`SessionStart`; on `work` a `UserPromptSubmit` and no `Stop`; on `done` a `Stop`)
# and logs every line typed into it; #650's stand-in Codex (`codex-fixture.sh`, 0.155.1, in the
# tested range) runs on Marley's App Server, with `marley.codex_app_server` on.
# - Claude Code in `sub` after `work` (`working`), Marley quits and starts again: resumed in `sub`
#   with Interrupted · Continue on its row (`chip`, REQ-004).
# - Continue clicked: the stand-in reads "Continue where you left off.", the chip goes
#   (`continued`, REQ-006).
# - `work`, `done`, a relaunch: resumed, no chip (`finished`, REQ-005).
# - A Codex from New Agent in the repo, `work` (`codex-working`), a relaunch: `codex resume
#   --remote unix://… --cd <repo> <thread>` typed, the row with the chip (`codex`, REQ-001,
#   REQ-002, REQ-004); Continue clicked: its TUI reads the prompt (`codex-continue`, REQ-006).
# - The switch off, a new Codex, `work` (`codex-no-server`), a relaunch: its terminal a plain shell,
#   and every resume the first thread's (`codex-off`, REQ-003).
compositor sway

# shellcheck source=script/e2e/codex-fixture.sh
. script/e2e/codex-fixture.sh

# Places in the window, as the first run's shots found them; `STOP_AT=<shot>` ends the run after
# that shot.
GRID_X=${GRID_X:-800}
GRID_Y=${GRID_Y:-500}
CLAUDE_CHIP_X=${CLAUDE_CHIP_X:-110}
CLAUDE_CHIP_Y=${CLAUDE_CHIP_Y:-200}
CODEX_CHIP_X=${CODEX_CHIP_X:-110}
CODEX_CHIP_Y=${CODEX_CHIP_Y:-248}
CLAUDE_ROW_X=${CLAUDE_ROW_X:-40}
CLAUDE_ROW_Y=${CLAUDE_ROW_Y:-176}
STOP_AT=${STOP_AT:-}
SESSION=11111111-2222-3333-4444-555555555555

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin versions=$E2E_WORK/versions
  mkdir -p "$home" "$bin" "$versions" "$E2E_WORK/repo/sub"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  printf '%s\n' "$SESSION" >"$E2E_WORK/session"
  : >"$E2E_WORK/claude.log"
  sed -e "s|@LOG@|$E2E_WORK/claude.log|" -e "s|@SESSION@|$E2E_WORK/session|" \
    -e "s|@INPUT@|$E2E_WORK/claude-input.log|" >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code (#540's): the frames its hooks would print, a log of how it was started
# and of every line typed into it. `work` starts a turn and leaves it open; `done` ends it.
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
    word = line.strip()
    with open("@INPUT@", "a", encoding="utf-8") as typed:
        typed.write(word + "\n")
    if word == "work":
        frame({"v": 1, "event": "UserPromptSubmit", "session_id": session, "cwd": folder,
               "prompt": "work"})
        print("working", flush=True)
    elif word == "done":
        frame({"v": 1, "event": "Stop", "session_id": session, "cwd": folder})
        print("done", flush=True)
    elif word == "exit":
        frame({"v": 1, "event": "SessionEnd", "session_id": session, "cwd": folder,
               "reason": "prompt_input_exit"})
        break
FAKE
  chmod +x "$bin/claude"
  write_codex_stand_in "$versions/0.155.1"
  ln -sfn "$versions/0.155.1" "$bin/codex"
  export MARLEY_CODEX=$bin/codex
  profile_setting marley.codex_app_server true
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

launches() { cat "$E2E_WORK/launches.log" 2>/dev/null; }

# Types `$1` and Enter into the terminal in front.
cue() {
  type_text "$1"
  press "" Return
  settle "${2:-2}"
}

# A Codex from the New Agent picker, in the repo: the terminal in front is in `sub`, so the
# picker asks where Codex starts, the project first.
picker_codex() {
  press "CTRL ALT" n
  settle 2
  type_text "Codex"
  settle 1
  press "" Return
  settle 2
  press "" Return
  settle 8
}

# Quits Marley as `quit_marley` does, confirming the question a working agent's terminal asks at
# a quit (#550: "Quit Marley?", Quit first).
quit_working() {
  local pid
  pid=$(marley_pid)
  press "CTRL SHIFT" p
  settle 1
  type_text "zed: quit"
  settle 1
  press "" Return
  settle 2
  if kill -0 "$pid" 2>/dev/null; then
    press "" Return
  fi
  for _ in $(seq 30); do
    kill -0 "$pid" 2>/dev/null || return 0
    sleep 1
  done
  echo "quit_working: Marley still runs 30 seconds after the quit" >&2
  return 1
}

relaunch() {
  quit_working
  launch_marley
  settle 16
}

# Whether the stand-in's log line `$1` is a resume of the session in `sub`.
claude_resumed() {
  local line
  line=$(sed -n "${1}p" "$E2E_WORK/claude.log")
  echo "claude run $1: $line"
  [[ $line == "--resume $SESSION @ $E2E_WORK/repo/sub" ]]
}

# Whether the last line typed into `$1`'s log is the resume prompt.
typed_continue() {
  local last
  last=$(tail -n 1 "$E2E_WORK/$1" 2>/dev/null)
  echo "last line typed: $last"
  [[ $last == "Continue where you left off." ]]
}

# Whether a Codex resume joined an App Server, in the repo, of the thread the first server made.
# The stand-in logs its arguments as it got them, unquoted.
codex_resumed() {
  local thread resumes
  thread=$(grep -m1 -o "thread [0-9a-f-]* ephemeral=False" "$E2E_WORK/server.log" | cut -d' ' -f2)
  resumes=$(launches | grep -F "resume --remote")
  echo "the first thread: $thread"
  echo "resumes: $resumes"
  [[ -n $thread ]] && grep -q -- "--cd $E2E_WORK/repo $thread\$" <<<"$resumes"
}

# Whether every Codex resume is of the thread the first server made: the Codex started with the
# switch off had no thread of Marley's to resume.
only_the_first_thread_resumed() {
  local thread resumes
  thread=$(grep -m1 -o "thread [0-9a-f-]* ephemeral=False" "$E2E_WORK/server.log" | cut -d' ' -f2)
  resumes=$(launches | grep "codex: resume")
  echo "resumes: $resumes"
  [[ -n $thread ]] && ! grep -v -- " $thread\$" <<<"$resumes"
}

# Shoots `$1`, and ends the run there when `STOP_AT` names it.
take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  click "$GRID_X" "$GRID_Y"
  settle 1.5

  echo "== Claude Code in sub, its turn left open"
  cue "cd sub" 1
  cue "claude" 2
  cue "work" 2
  take 750-01-working || return 0

  echo "== relaunched: resumed with the chip"
  relaunch
  take 750-02-chip || return 0
  expect "the restored terminal resumed the session in sub" claude_resumed 2

  echo "== Continue"
  click "$CLAUDE_CHIP_X" "$CLAUDE_CHIP_Y"
  settle 2
  take 750-03-continued || return 0
  expect "Claude Code read the resume prompt" typed_continue claude-input.log

  echo "== a turn that ended: no chip"
  click "$CLAUDE_ROW_X" "$CLAUDE_ROW_Y"
  settle 1
  cue "work" 1
  cue "done" 2
  relaunch
  take 750-04-finished || return 0
  expect "resumed again" claude_resumed 3

  echo "== Codex on Marley's App Server, its turn left open"
  picker_codex
  cue "work" 4
  take 750-05-codex-working || return 0
  relaunch
  take 750-06-codex || return 0
  expect "Codex resumed on a new App Server in the repo" codex_resumed

  echo "== Continue on Codex's row"
  click "$CODEX_CHIP_X" "$CODEX_CHIP_Y"
  settle 2
  take 750-07-codex-continue || return 0
  expect "Codex read the resume prompt" typed_continue tui.log

  echo "== the switch off: a new Codex is not resumed"
  profile_setting marley.codex_app_server false
  settle 3
  picker_codex
  cue "work" 2
  take 750-08-codex-no-server || return 0
  relaunch
  take 750-09-codex-off || return 0
  expect "only the App Server's thread was resumed" only_the_first_thread_resumed
}
