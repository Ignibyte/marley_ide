# shellcheck shell=bash
# #922's visual check: the bottom drawer as the BottomPanel board draws it. A Cargo crate with a
# syntax error in src/main.rs and one in src/parts.rs (rust-analyzer reports them; its checks build
# in the scenario's own target), #838's C program with `.zed/debug.json` for GDB, and #899's
# stand-in Claude Code started in the window's terminal, idle through Marley's plugin hook, so it
# is the window's one agent.
# - `922-01-problems`: `marley: toggle problems`: the inset card with its grab handle, the tabs
#   with their pills (Problems red), Move, Maximize and Close, the filter and a row per problem
#   (REQ-001, REQ-005, REQ-009).
# - `922-02-filtered`: "parts" typed in the filter: only parts.rs's row (REQ-002).
# - `922-03-opened`: main.rs's row clicked: main.rs open with the caret on its line (REQ-003).
# - `922-04-asked`: a row's Ask an agent to fix clicked: the stand-in's terminal holds "Fix this:",
#   the place and the message (REQ-004).
# - `922-05-paused`: `debugger: start`, the GDB config: the Debug tab with its "paused" pill
#   (REQ-005).
# - `922-06-maximized`: Problems, Maximize: Problems fills the window with its own Restore
#   (REQ-006).
# - `922-07-side`: Restore, then Move to the Side: the drawer in the right dock with its strip, the
#   button reads Move to the Bottom (REQ-006, REQ-008).
# - `922-08-bottom`: Move to the Bottom: the drawer back at the bottom (REQ-008).
# - `922-09-closed`: Close: the drawer gone (REQ-007).
# - `922-10-day`: Marley Day, Problems again (REQ-001, REQ-005, REQ-009).
compositor sway
HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)

AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-300}
STOP_AT=${STOP_AT:-}
# From the probe run's shots.
MAIN_ROW_X=${MAIN_ROW_X:-380}
MAIN_ROW_Y=${MAIN_ROW_Y:-776}
ASK_X=${ASK_X:-1525}
ASK_Y=${ASK_Y:-776}
DEBUG_TAB_X=${DEBUG_TAB_X:-397}
DEBUG_TAB_Y=${DEBUG_TAB_Y:-697}
PROBLEMS_TAB_X=${PROBLEMS_TAB_X:-312}
PROBLEMS_TAB_Y=${PROBLEMS_TAB_Y:-697}
MOVE_X=${MOVE_X:-1442}
MOVE_Y=${MOVE_Y:-697}
ZOOM_X=${ZOOM_X:-1525}
ZOOM_Y=${ZOOM_Y:-697}
SIDE_MOVE_X=${SIDE_MOVE_X:-1365}
SIDE_MOVE_Y=${SIDE_MOVE_Y:-76}
RESTORE_X=${RESTORE_X:-1566}
RESTORE_Y=${RESTORE_Y:-59}
CLOSE_X=${CLOSE_X:-1570}
CLOSE_Y=${CLOSE_Y:-697}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$config/plugins" "$repo/src" "$repo/.zed"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/$SHIPPED", "version": "$SHIPPED"}]}}
JSON
  export CLAUDE_CONFIG_DIR=$config MARLEY_CLAUDE=$bin/claude
  export CARGO_TARGET_DIR=$E2E_WORK/target
  printf '[package]\nname = "drawer"\nversion = "0.1.0"\nedition = "2021"\n' >"$repo/Cargo.toml"
  cat >"$repo/src/main.rs" <<'RS'
mod parts;

fn main() {
    let names = ["rail", "thread", "terminal"];
    let count = ;
    println!("{names:?} {count} {}", parts::part());
}
RS
  cat >"$repo/src/parts.rs" <<'RS'
pub fn part() -> u32 {
    let total = 1 +;
    total
}
RS
  cat >"$repo/hello.c" <<'C'
#include <stdio.h>

int main(void) {
    const char *names[] = {"rail", "thread", "terminal"};
    for (int index = 0; index < 3; index++) {
        printf("%s\n", names[index]);
    }
    return 0;
}
C
  gcc -g -O0 -o "$repo/hello" "$repo/hello.c"
  cat >"$repo/.zed/debug.json" <<JSON
[
  {
    "label": "Debug hello",
    "adapter": "GDB",
    "request": "launch",
    "program": "$repo/hello",
    "cwd": "$repo",
    "stopAtBeginningOfMainSubprogram": true
  }
]
JSON
  git init -q -b main "$repo"
  sed -e "s|@HOOK@|$HOOK|" >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
import json
import os
import subprocess
import sys

if sys.argv[1:2] == ["plugin"] or sys.argv[1:2] == ["--version"]:
    sys.exit(0)
print("stand-in Claude Code, idle", flush=True)
event = {"hook_event_name": "SessionStart", "source": "startup", "cwd": os.getcwd(),
         "session_id": "s-" + str(os.getpid()), "permission_mode": "default"}
answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps(event),
                        capture_output=True, text=True, check=False).stdout
sequence = json.loads(answer or "{}").get("terminalSequence")
if sequence:
    sys.stdout.write(sequence)
    sys.stdout.flush()
for line in sys.stdin:
    print("got: " + line.rstrip("\n"), flush=True)
FAKE
  chmod +x "$bin/claude"
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 15
  # Trusts the scratch crate.
  press "" Return
  settle 3
  type_text "claude"
  press "" Return
  settle 3
  press CTRL p
  settle 2
  type_text "main.rs"
  settle 2
  press "" Return
  # rust-analyzer reads the crate and reports both errors.
  settle 40

  echo "== Problems"
  palette "marley: toggle problems" 3
  away
  take 922-01-problems || return 0

  echo "== the filter"
  type_text "parts"
  settle 2
  away
  take 922-02-filtered || return 0
  press CTRL a
  press "" BackSpace
  settle 2

  echo "== a row opened"
  click "$MAIN_ROW_X" "$MAIN_ROW_Y"
  settle 3
  away
  take 922-03-opened || return 0

  echo "== Ask an agent to fix"
  click "$ASK_X" "$ASK_Y"
  settle 3
  away
  take 922-04-asked || return 0

  echo "== a GDB session"
  palette "debugger: start" 3
  type_text "Debug hello"
  settle 2
  press "" Return
  settle 10
  away
  take 922-05-paused || return 0

  echo "== Maximize Problems"
  click "$PROBLEMS_TAB_X" "$PROBLEMS_TAB_Y"
  settle 2
  click "$ZOOM_X" "$ZOOM_Y"
  settle 2
  away
  take 922-06-maximized || return 0
  click "$RESTORE_X" "$RESTORE_Y"
  settle 2

  echo "== Move to the side"
  click "$MOVE_X" "$MOVE_Y"
  settle 3
  away
  take 922-07-side || return 0

  echo "== Move to the bottom"
  click "$SIDE_MOVE_X" "$SIDE_MOVE_Y"
  settle 3
  away
  take 922-08-bottom || return 0

  echo "== Close"
  click "$CLOSE_X" "$CLOSE_Y"
  settle 2
  away
  take 922-09-closed || return 0

  echo "== Marley Day"
  palette "marley: toggle problems" 3
  profile_setting theme '"Marley Day"'
  settle 4
  away
  shot 922-10-day
}
