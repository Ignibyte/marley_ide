# shellcheck shell=bash
# #537's visual check: git's credential prompts off in the terminals Marley opens for agent CLIs.
# A local server answers every request with 401, so a push to it needs credentials. Stand-ins
# `claude` and `codex`, first on the terminals' PATH, print the two variables, push to the server
# with no credential helper, print git's exit status, then show that a `store` helper still answers
# for another host, and wait. Claude Code from the project's + (REQ-001 to REQ-004) and Codex from
# the New Agent picker (REQ-001, REQ-002) fail the push at once; the same push typed in a New
# Terminal waits on git's Username prompt (REQ-005).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The project's +; Down steps from New Terminal in its menu to Claude Code. From the shots.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-96}
CLAUDE_STEPS=${CLAUDE_STEPS:-3}

free_port() {
  python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])'
}
PORT=$(free_port)

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo store=$E2E_WORK/store
  mkdir -p "$home" "$bin" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  # A stored credential for another host, built at run time so no file holds one.
  printf 'http://%s:%s@127.0.0.2\n' agent "stored$RANDOM" >"$store"
  sed -e "s|@PORT@|$PORT|" -e "s|@STORE@|$store|" >"$bin/claude" <<'STAND_IN'
#!/usr/bin/env bash
# A stand-in agent: what git does in the terminal Marley opened for it.
name=$(basename "$0")
echo "GIT_TERMINAL_PROMPT=${GIT_TERMINAL_PROMPT-unset} GCM_INTERACTIVE=${GCM_INTERACTIVE-unset}"
git -c credential.helper= push "http://127.0.0.1:@PORT@/fixture.git" HEAD:main
echo "push exited $?"
printf 'protocol=http\nhost=127.0.0.2\n\n' |
  git -c credential.helper='store --file=@STORE@' credential fill | grep '^username='
exec -a "$name" sleep 600
STAND_IN
  cp "$bin/claude" "$bin/codex"
  chmod +x "$bin/claude" "$bin/codex"
  export MARLEY_CLAUDE=$bin/claude
  # A server that asks every request for credentials, and logs each.
  timeout 900 python3 - "$PORT" "$E2E_WORK/server.log" >/dev/null 2>&1 <<'SERVER' &
import http.server, sys

port, log = int(sys.argv[1]), sys.argv[2]

class Ask(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        with open(log, "a") as out:
            out.write(f"{self.command} {self.path}\n")
        self.send_response(401)
        self.send_header("WWW-Authenticate", 'Basic realm="e2e"')
        self.send_header("Content-Length", "0")
        self.end_headers()

    do_POST = do_GET

    def log_message(self, *args):
        pass

http.server.HTTPServer(("127.0.0.1", port), Ask).serve_forever()
SERVER
  SERVER_PID=$!
  git init -q -b main "$repo"
  git -C "$repo" -c user.name=Scenario -c user.email=scenario@example.invalid commit -q --allow-empty -m Start
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# The newest terminal's screen: a terminal is titled by its process, so they are told apart by
# the project's name and which came last.
screen_of() {
  mcp_agent terminal-screen repo | tee "$E2E_WORK/$1.txt"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the terminal's claude and codex are the stand-ins"
  type_text "command -v claude codex > $E2E_WORK/which.txt"
  press "" Return
  settle 2
  cat "$E2E_WORK/which.txt"
  if ! grep -qx "$E2E_WORK/bin/claude" "$E2E_WORK/which.txt" ||
    ! grep -qx "$E2E_WORK/bin/codex" "$E2E_WORK/which.txt"; then
    echo "check the stand-ins come first on the terminal's PATH: FAIL (stopping before any launch)"
    kill "$SERVER_PID"
    return 1
  fi

  echo "== Claude Code from the project's +: the push fails at once"
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  local step
  for ((step = 0; step < CLAUDE_STEPS; step++)); do
    press "" Down
  done
  settle 1
  shot 537-00-menu
  press "" Return
  settle 6
  shot 537-01-rail-agent
  screen_of rail
  expect "the agent's terminal holds both variables and git gave up at once" \
    holds "$E2E_WORK/rail.txt" '$ claude' 'GIT_TERMINAL_PROMPT=0 GCM_INTERACTIVE=never' \
    'terminal prompts disabled' 'push exited 128'
  expect "a stored credential still answers" holds "$E2E_WORK/rail.txt" 'username=agent'
  expect "no variable was typed at the prompt" bash -c "! grep -q 'GIT_TERMINAL_PROMPT=0 claude' '$E2E_WORK/rail.txt'"

  echo "== Codex from the New Agent picker: the same"
  press "CTRL ALT" n
  settle 2
  type_text "Codex"
  settle 1
  press "" Return
  settle 6
  shot 537-02-picker-agent
  screen_of picker
  expect "Codex's terminal too" \
    holds "$E2E_WORK/picker.txt" '$ codex' 'GIT_TERMINAL_PROMPT=0 GCM_INTERACTIVE=never' \
    'terminal prompts disabled' 'push exited 128'

  echo "== a New Terminal: git asks for the username"
  palette "workspace: new terminal"
  settle 3
  type_text "git -c credential.helper= push http://127.0.0.1:$PORT/fixture.git HEAD:main"
  press "" Return
  settle 3
  shot 537-03-plain-terminal
  screen_of plain
  expect "git asks for the username there" holds "$E2E_WORK/plain.txt" "Username for 'http://127.0.0.1:$PORT'"
  press CTRL c
  settle 1

  echo "== the server's requests"
  cat "$E2E_WORK/server.log"
  kill "$SERVER_PID"
}
