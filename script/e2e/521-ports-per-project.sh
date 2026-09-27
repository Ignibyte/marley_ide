# shellcheck shell=bash
# #521's e2e test: the ports a project's processes listen on, as rows under the project in the
# rail. Before Marley starts, the scenario starts its servers: in repo/web one bound to 0.0.0.0,
# whose row's URL is on 127.0.0.1 (REQ-008); in repo/tools a stand-in on 127.0.0.1 and ::1 on one
# port, one row (REQ-008); in repo/web one that names itself marley and one whose command line
# names the profile's browser/ folder, and one in a folder outside the project, which show nowhere
# (REQ-003). The rail shows the project's rows with the port, the process's name and the URL
# (REQ-001), and a row's tooltip names the process (REQ-002). A server typed into the project's
# terminal gets its row within five seconds (REQ-004); ports_list gives each listener with its
# project and folder and no command line (REQ-009); Copy puts a row's URL on the clipboard
# (REQ-006); Stop ends the typed server and takes its row away (REQ-007); Open shows a row's
# server in a Browser tab (REQ-005). Before the typed server, with the rail closed Marley stops
# reading the tables, and a server started meanwhile has its row once the rail opens again (D3).
# The ports rise in the order web, tools, typed, late, so the rows keep one order. Chromium runs
# offline.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# In the window's logical pixels, from the first run's shots: a port row's title, the rows of the
# web server, the stand-in and the typed server, the row's Open, Copy and Stop buttons, and the
# project's terminal.
ROW_X=${ROW_X:-110}
WEB_Y=${WEB_Y:-182}
TOOLS_Y=${TOOLS_Y:-228}
TYPED_Y=${TYPED_Y:-274}
OPEN_X=${OPEN_X:-186}
COPY_X=${COPY_X:-210}
STOP_X=${STOP_X:-234}
TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo/web" "$E2E_WORK/repo/tools" "$E2E_WORK/elsewhere"
  # shellcheck disable=SC2016 # the prompt is the terminal's to expand
  printf 'PS1=%s\n' "'\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  echo '<!doctype html><title>Web</title><h1>The web folder</h1>' >"$E2E_WORK/repo/web/index.html"
  offline_chromium
  write_mcp_agent
  write_stand_ins
  git init -q -b main "$E2E_WORK/repo"
  read -r WEB_PORT TOOLS_PORT TYPED_PORT LATE_PORT < <(python3 -c '
import socket
held = [socket.socket() for _ in range(4)]
for held_socket in held:
    held_socket.bind(("127.0.0.1", 0))
print(*sorted(held_socket.getsockname()[1] for held_socket in held))
')
  echo "the ports: web $WEB_PORT, tools $TOOLS_PORT, typed $TYPED_PORT, late $LATE_PORT"
  start_server "$E2E_WORK/repo/web" web python3 -u -m http.server --bind 0.0.0.0 "$WEB_PORT"
  start_server "$E2E_WORK/repo/tools" tools python3 -u "$E2E_WORK/dual.py" "$TOOLS_PORT"
  start_server "$E2E_WORK/elsewhere" elsewhere python3 -u -m http.server --bind 127.0.0.1 0
  start_server "$E2E_WORK/repo/web" named python3 -u "$E2E_WORK/named.py"
  start_server "$E2E_WORK/repo/web" profile python3 -u -m http.server --bind 127.0.0.1 \
    --directory "$E2E_PROFILE/browser" 0
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

# Starts `command…` in `folder` in the background, its output in `$E2E_WORK/<name>.log` and its
# pid in `$E2E_WORK/<name>.pid` and in the servers the teardown stops.
start_server() {
  local folder=$1 name=$2
  shift 2
  (cd "$folder" && exec "$@" >"$E2E_WORK/$name.log" 2>&1) &
  echo "$!" >"$E2E_WORK/$name.pid"
  echo "$!" >>"$E2E_WORK/servers"
}

# The stand-ins: one listening on 127.0.0.1 and ::1 on the port it is given, and a server that
# names itself marley, as Marley's own processes are named.
write_stand_ins() {
  cat >"$E2E_WORK/dual.py" <<'PY'
import socket, sys, time
port = int(sys.argv[1])
four = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
four.bind(("127.0.0.1", port))
four.listen()
six = socket.socket(socket.AF_INET6, socket.SOCK_STREAM)
six.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_V6ONLY, 1)
six.bind(("::1", port))
six.listen()
print(f"listening on 127.0.0.1 and ::1 port {port}", flush=True)
time.sleep(3600)
PY
  cat >"$E2E_WORK/named.py" <<'PY'
import http.server
with open("/proc/self/comm", "w") as comm:
    comm.write("marley")
server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), http.server.SimpleHTTPRequestHandler)
print(f"named marley, port {server.server_address[1]}", flush=True)
server.serve_forever()
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Keeps `command`'s output in `$E2E_WORK/out.txt` as the log shows it.
out() {
  "$@" | tee "$E2E_WORK/out.txt"
}

# The bytes the process `pid` reads in the next `seconds`, as its `/proc/<pid>/io` counts them.
bytes_read() {
  local pid=$1 seconds=$2 before after
  before=$(awk '/^rchar:/ {print $2}' "/proc/$pid/io")
  settle "$seconds"
  after=$(awk '/^rchar:/ {print $2}' "/proc/$pid/io")
  echo $((after - before))
}

# The port each server listens on, from `ss`, by its pid.
listening() {
  local name
  for name in web tools elsewhere named profile; do
    local pid
    pid=$(cat "$E2E_WORK/$name.pid")
    echo "  $name (pid $pid): $(ss -ltnpH | grep "pid=$pid," | awk '{print $4}' | tr '\n' ' ')"
  done
}

steps() {
  settle 12
  # Trusts the repository.
  press "" Return
  settle 5

  echo "== what listens"
  listening | tee "$E2E_WORK/listening.txt"
  shot 521-01-rows
  pointer_to "$ROW_X" "$WEB_Y"
  settle 2
  shot 521-02-tooltip

  echo "== what ports_list says"
  out mcp_agent ports
  cp "$E2E_WORK/out.txt" "$E2E_WORK/ports-before.txt"
  expect "the web server is the project's, from repo/web" holds "$E2E_WORK/ports-before.txt" \
    "port $WEB_PORT: python3" "at http://127.0.0.1:$WEB_PORT/, project repo, folder $E2E_WORK/repo"
  expect "the stand-in is the project's, once, on 127.0.0.1" bash -c \
    "test \$(grep -c 'port $TOOLS_PORT:' '$E2E_WORK/ports-before.txt') = 1 && grep -q 'at http://127.0.0.1:$TOOLS_PORT/' '$E2E_WORK/ports-before.txt'"
  local elsewhere named profile
  elsewhere=$(cat "$E2E_WORK/elsewhere.pid")
  named=$(cat "$E2E_WORK/named.pid")
  profile=$(cat "$E2E_WORK/profile.pid")
  expect "no server outside the project, named marley, or naming the profile's browser folder" \
    bash -c "! grep -qE 'pid ($elsewhere|$named|$profile) ' '$E2E_WORK/ports-before.txt'"
  expect "no entry carries the command line" bash -c "! grep -q 'command' '$E2E_WORK/ports-before.txt'"

  echo "== the scan while the rail is closed"
  # Each round reads both tables whole, tens of kilobytes on this machine; a closed rail's Marley
  # reads next to nothing. #503 reads the same tables every two seconds while a terminal holds a
  # printed URL, so this comes before the typed server prints one.
  local marley open_read closed_read
  marley=$(marley_pid)
  open_read=$(bytes_read "$marley" 9)
  press "CTRL ALT" j
  # The round under way when the rail closes still runs.
  settle 4
  closed_read=$(bytes_read "$marley" 9)
  echo "  Marley read $open_read bytes in nine seconds with the rail open, $closed_read with it closed"
  expect "the scan stops while the rail is closed" test $((closed_read * 4)) -lt "$open_read"
  start_server "$E2E_WORK/repo/tools" late python3 -u -m http.server --bind 127.0.0.1 "$LATE_PORT"
  settle 2
  press "CTRL ALT" j
  settle 5
  shot 521-07-reopened

  echo "== a server typed into the project's terminal"
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  type_text "python3 -m http.server $TYPED_PORT"
  press "" Return
  settle 5
  shot 521-03-appeared
  out mcp_agent ports
  local typed
  typed=$(grep -oE "port $TYPED_PORT: python3 pid [0-9]+" "$E2E_WORK/out.txt" | awk '{print $5}')
  echo "  the typed server: pid ${typed:-none}"
  expect "the typed server's row came, in the project" test -n "$typed"

  echo "== Copy on the stand-in's row"
  pointer_to "$ROW_X" "$TOOLS_Y"
  settle 1
  click "$COPY_X" "$TOOLS_Y"
  settle 1
  shot 521-05-copied
  WAYLAND_DISPLAY=$SWAY_DISPLAY wl-paste --no-newline >"$E2E_WORK/clipboard.txt" || true
  echo "  the clipboard: $(cat "$E2E_WORK/clipboard.txt")"
  expect "Copy put the stand-in's URL on the clipboard" \
    test "$(cat "$E2E_WORK/clipboard.txt")" = "http://127.0.0.1:$TOOLS_PORT/"

  echo "== Stop on the typed server's row"
  pointer_to "$ROW_X" "$TYPED_Y"
  settle 1
  click "$STOP_X" "$TYPED_Y"
  settle 5
  shot 521-06-stopped
  expect "the typed server ended" bash -c "! kill -0 '$typed' 2>/dev/null"
  out mcp_agent ports
  expect "its row went" bash -c "! grep -q 'port $TYPED_PORT:' '$E2E_WORK/out.txt'"

  echo "== Open on the web server's row"
  pointer_to "$ROW_X" "$WEB_Y"
  settle 1
  click "$OPEN_X" "$WEB_Y"
  settle 8
  shot 521-04-opened
  out mcp_agent tabs
  expect "a Browser tab shows the web server" holds "$E2E_WORK/out.txt" "at http://127.0.0.1:$WEB_PORT/"
}
