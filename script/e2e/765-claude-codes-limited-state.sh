# shellcheck shell=bash
# #765's visual check: a Claude Code stopped at its usage limit, and continued at its reset. A
# stand-in `claude`, cued through `$E2E_WORK/claude.fifo`, prints #519's hook frames and reports
# as the shared plugin does (`"$MARLEY_BIN" report working` at a prompt, `report error --quota …`
# at a failed turn's end). `limited +80s` is a prompt, then a `StopFailure` with `rate_limit` and
# the five-hour window full for 80 seconds (REQ-001, REQ-002); `limited-unknown` the same with the
# window at 20 % (REQ-006); `done` a `Stop`. What is typed into it is logged to `typed.log` and
# starts a turn. Continue at Reset is clicked and the reset passes (`continued`, REQ-003, REQ-004);
# armed again, a prompt the user types drops it (`dropped`, REQ-005).
compositor sway

# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
CONTINUE_X=${CONTINUE_X:-193}
CONTINUE_Y=${CONTINUE_Y:-151}
TERMINAL_X=${TERMINAL_X:-900}
TERMINAL_Y=${TERMINAL_Y:-600}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle "${2:-2}"
}

# Writes a cue to the stand-in Claude Code.
cue() {
  echo "$1" >"$E2E_WORK/claude.fifo"
  settle "${2:-3}"
}

# How many lines typed into the stand-in read `$1`.
typed() {
  grep -cxF -- "$1" "$E2E_WORK/typed.log" || true
}

continued_once() {
  cat "$E2E_WORK/typed.log"
  [[ $(typed "Continue where you left off.") -eq 1 ]]
}

write_claude_stand_in() {
  sed -e "s|@WORK@|$E2E_WORK|g" >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code: #519's hook frames and the shared plugin's reports, cued through
# claude.fifo; a typed line is a prompt.
import base64
import json
import os
import subprocess
import sys
import threading
import time

WORK = "@WORK@"
SESSION = "76576576-5765-4765-8765-765765765765"
program = os.environ.get("MARLEY_BIN", "")
folder = os.getcwd()
lock = threading.Lock()
seq = [int(time.time() * 1000)]


def frame(summary):
    summary = {"v": 1, "cwd": folder, "session_id": SESSION, **summary}
    body = base64.b64encode(json.dumps(summary).encode()).decode()
    with lock:
        sys.stdout.write(f"\x1b]777;notify;marley-event;{body}\x07")
        sys.stdout.flush()
    time.sleep(0.3)


def report(state, *rest):
    seq[0] = max(seq[0] + 1, int(time.time() * 1000))
    done = subprocess.run(
        [program, "report", "--source", "mod:claude-code", "--seq", str(seq[0]), state,
         "--session-id", SESSION, *rest],
        capture_output=True, text=True, check=False)
    with open(f"{WORK}/claude.log", "a", encoding="utf-8") as log:
        log.write(f"report {state}: exit {done.returncode} {done.stderr.strip()}\n")


def say(line):
    with lock:
        print(line, flush=True)


def prompt(text):
    frame({"event": "UserPromptSubmit", "prompt": text})
    report("working")


def limited(window):
    now = int(time.time() * 1000)
    frame({"event": "StopFailure", "error": "rate_limit",
           "message": "You've hit your limit · resets soon"})
    report("error", "--quota", window(now), "--quota", f"seven_day:40:{now + 3 * 86400 * 1000}")
    say("You've hit your limit")


def play(line):
    word, _, rest = line.strip().partition(" ")
    if word == "limited":
        seconds = int(rest.removeprefix("+").removesuffix("s"))
        prompt("Refactor the parser")
        limited(lambda now: f"five_hour:100:{now + seconds * 1000}")
    elif word == "limited-unknown":
        prompt("Refactor the parser")
        limited(lambda now: f"five_hour:20:{now + 2 * 3600 * 1000}")
    elif word == "done":
        frame({"event": "Stop", "message": "Done."})
        report("idle")
        say("done")


def cues():
    path = f"{WORK}/claude.fifo"
    while True:
        with open(path, encoding="utf-8") as fifo:
            for line in fifo:
                play(line)


say("Claude Code (stand-in): cued through claude.fifo")
frame({"event": "SessionStart", "source": "startup"})
report("idle")
threading.Thread(target=cues, daemon=True).start()
for line in sys.stdin:
    text = line.strip()
    if not text:
        continue
    with open(f"{WORK}/typed.log", "a", encoding="utf-8") as log:
        log.write(text + "\n")
    say(f"user: {text}")
    prompt(text)
FAKE
  chmod +x "$1"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  git init -q -b main "$repo"
  printf 'limits\n' >"$repo/README.md"
  write_claude_stand_in "$bin/claude"
  mkfifo "$E2E_WORK/claude.fifo"
  : >"$E2E_WORK/typed.log"
  open_path "$repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== Claude Code, limited"
  palette "workspace: new terminal" 3
  type_text "claude"
  press "" Return
  settle 4
  cue "limited +80s" 4
  cat "$E2E_WORK/claude.log"
  take 765-01-limited || return 0

  echo "== Continue at Reset"
  click "$CONTINUE_X" "$CONTINUE_Y"
  settle 2
  take 765-02-armed || return 0
  settle 85
  take 765-03-continued || return 0
  expect "Marley typed the continuation once" continued_once

  echo "== armed, then a prompt the user types"
  cue "done"
  cue "limited +80s" 4
  click "$CONTINUE_X" "$CONTINUE_Y"
  settle 2
  click "$TERMINAL_X" "$TERMINAL_Y"
  type_text "Fix the tests"
  press "" Return
  settle 85
  take 765-04-dropped || return 0
  expect "the user's prompt dropped the arm: still one continuation" continued_once

  echo "== no reset"
  cue "done"
  cue limited-unknown 4
  take 765-05-no-reset || return 0
}
