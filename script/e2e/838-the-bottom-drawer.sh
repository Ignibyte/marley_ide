# shellcheck shell=bash
# #838's visual check: the bottom drawer with Problems and Debug. A Cargo crate whose src/main.rs
# has a syntax error (rust-analyzer reports it; its checks build in the scenario's own target), a C
# program built with `gcc -g`, and `.zed/debug.json` launching it under GDB, stopped at `main`.
# - main.rs open, the bottom row's diagnostics count: the drawer on Problems with its count, the
#   error listed, no Problems center tab (`838-01-problems`, REQ-001, REQ-002).
# - The drawer's Debug tab: the debugger panel under the same tabs (`838-02-debug`, REQ-002).
# - `debugger: start`, the GDB config: the program paused at `main` (`838-03-session`, REQ-003).
# - Marley Day, the Problems tab (`838-04-day`, REQ-001, REQ-002).
compositor sway

AWAY_X=${AWAY_X:-700}
AWAY_Y=${AWAY_Y:-300}
# From the probe run's shots: the bottom row's diagnostics count and the drawer's tabs.
COUNT_X=${COUNT_X:-452}
COUNT_Y=${COUNT_Y:-985}
PROBLEMS_X=${PROBLEMS_X:-307}
PROBLEMS_Y=${PROBLEMS_Y:-685}
DEBUG_X=${DEBUG_X:-376}
DEBUG_Y=${DEBUG_Y:-685}
STOP_AT_PROBLEMS=${STOP_AT_PROBLEMS:-0}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo/src" "$repo/.zed"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  export CARGO_TARGET_DIR=$E2E_WORK/target
  cat >"$repo/Cargo.toml" <<'TOML'
[package]
name = "drawer"
version = "0.1.0"
edition = "2021"

[dependencies]
TOML
  cat >"$repo/src/main.rs" <<'RS'
fn main() {
    let names = ["rail", "thread", "terminal"];
    let count = ;
    println!("{names:?} {count}");
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
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
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
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== main.rs, then the bottom row's count"
  palette "file finder: toggle" 3
  type_text "main.rs"
  settle 3
  press "" Return
  # rust-analyzer starts and reports the error.
  settle 25
  click "$COUNT_X" "$COUNT_Y"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 838-01-problems
  if [[ $STOP_AT_PROBLEMS == 1 ]]; then
    return
  fi

  echo "== the Debug tab"
  click "$DEBUG_X" "$DEBUG_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 838-02-debug

  echo "== a GDB session"
  palette "debugger: start" 3
  type_text "Debug hello"
  settle 2
  press "" Return
  settle 10
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 838-03-session

  echo "== Marley Day, Problems"
  click "$PROBLEMS_X" "$PROBLEMS_Y"
  settle 2
  profile_setting theme '"Marley Day"'
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 4
  shot 838-04-day
}
