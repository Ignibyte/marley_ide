# shellcheck shell=bash
# #901's visual check: the bottom row while debugging, and Ask an Agent about a language server.
# #838's C program, built with `gcc -g`, launches under GDB's own DAP from `.zed/debug.json`,
# stopped at `main`. `lsp.rust-analyzer.binary.path` names a stand-in language server that starts and
# reports its health as an error with the reason. A stand-in `claude` (#557's) runs in a
# terminal of the window and logs what it reads.
# - `901-01-paused`: the session paused at main: the row's pill, controls, frame, line and
#   breakpoint count (REQ-001).
# - `901-02-stepped`: Step over clicked: the row's line moved on (REQ-001).
# - `901-05-day`: the paused row in Marley Day (REQ-001).
# - `901-03-ask`: the language servers menu, rust-analyzer's submenu with Ask an Agent (REQ-002).
# - `901-04-asked`: it clicked: the stand-in's prompt holds the server's name and message
#   (REQ-002).
compositor sway

AWAY_X=${AWAY_X:-700}
AWAY_Y=${AWAY_Y:-300}
# From the probe run's shots: the row's Step over, the language servers button, rust-analyzer's
# row in its menu, Ask an Agent in the submenu, and the terminal's tab.
STEP_X=${STEP_X:-466}
STEP_Y=${STEP_Y:-984}
SERVERS_X=${SERVERS_X:-430}
SERVERS_Y=${SERVERS_Y:-985}
SERVER_X=${SERVER_X:-480}
SERVER_Y=${SERVER_Y:-876}
ASK_X=${ASK_X:-660}
ASK_Y=${ASK_Y:-852}
TERMINAL_TAB_X=${TERMINAL_TAB_X:-430}
TERMINAL_TAB_Y=${TERMINAL_TAB_Y:-51}
# A first run that stops after the paused row's shot (1) or the menu's (2).
STOP_AT=${STOP_AT:-}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo/src" "$repo/.zed"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  # A stand-in Claude Code: it logs its arguments, then each line it reads.
  cat >"$bin/claude" <<SH
#!/usr/bin/env bash
exec -a claude python3 -u -c '
import sys
log = open(sys.argv[1], "a")
print("stand-in agent, argv:", " ".join(sys.argv[2:]), flush=True)
for line in sys.stdin:
    print("got:", line.rstrip("\\n"), file=log, flush=True)
    print("got:", line.rstrip("\\n"), flush=True)
' "$E2E_WORK/agent.log" "\$@"
SH
  chmod +x "$bin/claude"
  : >"$E2E_WORK/agent.log"
  # A stand-in rust-analyzer: a language server that starts, then reports its health as an
  # error with the reason, as rust-analyzer's `experimental/serverStatus` does.
  cat >"$bin/fake-rust-analyzer" <<'PY'
#!/usr/bin/env python3
import json
import sys


def read():
    length = 0
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            sys.exit(0)
        if line in (b"\r\n", b"\n"):
            break
        name, _, value = line.decode().partition(":")
        if name.lower() == "content-length":
            length = int(value.strip())
    return json.loads(sys.stdin.buffer.read(length))


def send(message):
    body = json.dumps(message).encode()
    sys.stdout.buffer.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
    sys.stdout.buffer.flush()


while True:
    message = read()
    method = message.get("method")
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": message["id"], "result": {"capabilities": {}}})
    elif method == "initialized":
        send({"jsonrpc": "2.0", "method": "experimental/serverStatus",
              "params": {"health": "error", "quiescent": True,
                         "message": "No Cargo.toml above src/main.rs."}})
    elif method == "exit":
        sys.exit(0)
    elif "id" in message:
        send({"jsonrpc": "2.0", "id": message["id"], "result": None})
PY
  chmod +x "$bin/fake-rust-analyzer"
  printf 'fn main() {\n    println!("drawer");\n}\n' >"$repo/src/main.rs"
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
  profile_setting lsp "{\"rust-analyzer\": {\"binary\": {\"path\": \"$bin/fake-rust-analyzer\"}}}"
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

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the stand-in claude in the project's terminal"
  type_text "claude"
  press "" Return
  settle 3

  echo "== a breakpoint on hello.c's line 6, then a GDB session paused at main"
  palette "file finder: toggle" 3
  type_text "hello.c"
  settle 3
  press "" Return
  settle 3
  press "CTRL" g
  settle 1
  type_text "6"
  press "" Return
  settle 1
  palette "editor: toggle breakpoint" 2
  palette "debugger: start" 3
  type_text "Debug hello"
  settle 2
  press "" Return
  settle 12
  away
  shot 901-01-paused
  [[ $STOP_AT == 1 ]] && return 0

  echo "== Step over"
  click "$STEP_X" "$STEP_Y"
  settle 4
  away
  shot 901-02-stepped

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 901-05-day
  profile_setting theme '"Marley Night"'
  settle 2

  echo "== main.rs, rust-analyzer failing, its submenu"
  palette "debugger: stop" 3
  palette "file finder: toggle" 3
  type_text "main.rs"
  settle 3
  press "" Return
  settle 10
  click "$SERVERS_X" "$SERVERS_Y"
  settle 2
  pointer_to "$SERVER_X" "$SERVER_Y"
  settle 2
  shot 901-03-ask
  [[ $STOP_AT == 2 ]] && return 0

  echo "== Ask an Agent"
  click "$ASK_X" "$ASK_Y"
  settle 3
  click "$TERMINAL_TAB_X" "$TERMINAL_TAB_Y"
  settle 2
  away
  shot 901-04-asked
  cat "$E2E_WORK/agent.log"
}
