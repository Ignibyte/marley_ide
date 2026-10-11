# shellcheck shell=bash
# #715's visual check: the usage meter on agent rows, and a limited Codex continued at its reset.
# #650's stand-in Codex on its App Server (`codex-fixture.sh`, `marley.codex_app_server` on), cued
# through its `tui.fifo`, sends the account's windows (`limits`, REQ-001, REQ-003) and fails turns
# at the limit (`limited +80s`, REQ-004 to REQ-008; `limited-unknown`, REQ-009). A stand-in `claude`
# reports its windows as the shared plugin does, `"$MARLEY_BIN" report idle --quota …` (REQ-002).
# Continue at Reset is clicked once and the reset passes (`continued`); armed again, a turn the
# user starts drops it (`dropped`); `marley.continue_at_reset` on arms a third without a click
# (`setting`, REQ-010, REQ-011), and Cancel holds against it (`cancelled`). The server logs each `turn/start` with its client and text, and
# every method Marley sends (REQ-013).
compositor sway
# shellcheck source=script/e2e/codex-fixture.sh
. script/e2e/codex-fixture.sh

# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
CODEX_ROW_X=${CODEX_ROW_X:-140}
CODEX_ROW_Y=${CODEX_ROW_Y:-228}
CONTINUE_X=${CONTINUE_X:-193}
CONTINUE_Y=${CONTINUE_Y:-151}
CANCEL_X=${CANCEL_X:-223}
CANCEL_Y=${CANCEL_Y:-151}
SEARCH_X=${SEARCH_X:-912}
SEARCH_Y=${SEARCH_Y:-59}
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

# Writes a cue to the stand-in Codex's TUI.
tui() {
  echo "$1" >"$E2E_WORK/tui.fifo"
  settle "${2:-2}"
}

# How many turns Marley started on the server.
marley_turns() {
  grep -c -- "- turn/start by marley" "$E2E_WORK/server.log" || true
}

continued_once() {
  grep -- "- turn/start by marley" "$E2E_WORK/server.log"
  [[ $(marley_turns) -eq 1 ]] &&
    grep -q -- "- turn/start by marley: Continue where you left off.$" "$E2E_WORK/server.log"
}

write_claude_stand_in() {
  sed -e "s|@WORK@|$E2E_WORK|g" >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code: a session start, then the shared plugin's report of its usage windows.
import base64
import json
import os
import subprocess
import sys
import time

WORK = "@WORK@"
SESSION = "71571571-5715-4715-8715-715715715715"
program = os.environ.get("MARLEY_BIN", "")
folder = os.getcwd()


def frame(summary):
    summary = {"v": 1, "cwd": folder, **summary}
    body = base64.b64encode(json.dumps(summary).encode()).decode()
    sys.stdout.write(f"\x1b]777;notify;marley-event;{body}\x07")
    sys.stdout.flush()


print("Claude Code (stand-in): reports its usage", flush=True)
frame({"event": "SessionStart", "session_id": SESSION, "source": "startup"})
time.sleep(0.5)
now = int(time.time() * 1000)
done = subprocess.run(
    [program, "report", "--source", "mod:claude-code", "--seq", str(now), "idle",
     "--session-id", SESSION,
     "--quota", f"five_hour:41:{now + (2 * 3600 + 10 * 60) * 1000}",
     "--quota", f"seven_day:12:{now + 4 * 86400 * 1000}"],
    capture_output=True, text=True, check=False)
with open(f"{WORK}/claude.log", "a", encoding="utf-8") as log:
    log.write(f"report: exit {done.returncode} {done.stderr.strip()}\n")
print(f"reported: exit {done.returncode}", flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$1"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin versions=$E2E_WORK/versions repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$versions" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  git init -q -b main "$repo"
  printf 'usage\n' >"$repo/README.md"
  write_codex_stand_in "$versions/0.162.0"
  ln -sfn "$versions/0.162.0" "$bin/codex"
  write_claude_stand_in "$bin/claude"
  export MARLEY_CODEX=$bin/codex
  profile_setting marley.codex_app_server true
  mkfifo "$E2E_WORK/tui.fifo"
  open_path "$repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== Codex on its App Server, and its windows"
  press "CTRL ALT" n
  settle 2
  type_text "Codex"
  settle 1
  press "" Return
  settle 2
  # New Agent asks where Codex starts; the project comes first.
  press "" Return
  settle 10
  tui work
  tui "limits five_hour:62:+95m seven_day:31:+3d"
  tui "done" 3
  echo "== Claude Code's windows, from its report"
  palette "workspace: new terminal" 3
  type_text "claude"
  press "" Return
  settle 5
  cat "$E2E_WORK/claude.log"
  take 715-01-meters || return 0

  echo "== the tooltip"
  pointer_to "$CODEX_ROW_X" "$CODEX_ROW_Y"
  settle 2
  take 715-02-tooltip || return 0
  pointer_to 800 500

  echo "== limited"
  tui work
  tui "limited +80s" 3
  take 715-03-limited || return 0

  echo "== Continue at Reset"
  click "$CONTINUE_X" "$CONTINUE_Y"
  settle 2
  take 715-04-armed || return 0
  settle 85
  # The Codex row, the entry gone, shows its terminal: the TUI draws Marley's prompt.
  click "$CODEX_ROW_X" "$CODEX_ROW_Y"
  pointer_to 800 500
  settle 2
  take 715-05-continued || return 0
  expect "Marley started one turn, with the resume prompt" continued_once

  echo "== armed, then a turn the user starts"
  tui "done" 2
  tui work
  tui "limited +80s" 3
  click "$CONTINUE_X" "$CONTINUE_Y"
  settle 2
  tui work 3
  settle 85
  take 715-06-dropped || return 0
  expect "the user's turn dropped the arm: still one turn from Marley" test "$(marley_turns)" -eq 1

  echo "== no reset"
  tui "done" 2
  tui work
  tui limited-unknown 3
  take 715-07-no-reset || return 0

  echo "== the setting"
  palette "marley: open settings" 3
  click "$SEARCH_X" "$SEARCH_Y"
  settle 1
  type_text "Continue Limited"
  settle 2
  take 715-08-setting-off || return 0
  profile_setting marley.continue_at_reset true
  settle 3
  take 715-08-setting || return 0
  sway_msg '[title="Settings"] kill' >/dev/null
  settle 2
  tui work
  tui "limited +80s" 3
  take 715-09-armed-by-setting || return 0

  echo "== Cancel holds with the setting on"
  click "$CANCEL_X" "$CANCEL_Y"
  settle 2
  # An update folds the seat again; the cancelled limit is not armed again.
  tui "limits seven_day:35:+3d" 3
  take 715-10-cancelled || return 0
  expect "Marley sent no account/rateLimits/read" bash -c "! grep -q 'marley account/rateLimits/read' '$E2E_WORK/server.log'"
}
