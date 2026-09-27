# shellcheck shell=bash
# #583's e2e test: a project's Chromium listens on no TCP port; it speaks DevTools on its pipe to
# Marley's relay, a hidden mode of Marley's executable in the same unit. Before Marley starts,
# the repository's project unit runs the way an earlier build started it, on a port: Marley
# closes it and starts the relay in its place (REQ-008). Chromium's processes listen on no port
# and the relay on one on 127.0.0.1 (REQ-001); the tab shows the page and the agent tools work
# through the relay (REQ-002). The relay's WebSocket refuses a client without the token, with a
# wrong one, and with a web page's Origin (REQ-003). One client cannot call on another's session
# (REQ-005). Marley quits and starts again, and the unit lives on and the tab comes back
# (REQ-006). A client's Browser.close ends Chromium, the relay and the unit, and the relay's
# files go (REQ-009). Chromium runs offline.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

setup() {
  offline_chromium
  mkdir -p "$E2E_WORK/site"
  cat >"$E2E_WORK/site/index.html" <<'HTML'
<!doctype html><html><head><title>Behind the relay</title></head>
<body style="margin:0;font:32px sans-serif;background:#eef4ee"><h1 style="margin:40px">A page behind Marley's relay</h1></body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
  write_mcp_agent
  write_ws_probe
  git init -q -b main "$E2E_WORK/repo"
  # The repository's project unit, started the way a build before #583 started it: Chromium
  # on a port, no relay.
  local profile unit
  profile=$(browser_profile "$E2E_WORK/repo")
  unit=$(browser_unit "$E2E_WORK/repo")
  mkdir -p "$profile"
  chmod 700 "$(dirname "$profile")" "$profile"
  systemd-run --user --quiet --collect --unit="$unit" "--description=Marley's browser" \
    --service-type=exec --property=KillMode=mixed --property=TimeoutStopSec=10 -- \
    "$MARLEY_CHROMIUM" --headless --remote-debugging-port=0 --user-data-dir="$profile" \
    --no-first-run --no-default-browser-check --password-store=basic --no-startup-window
  for _ in $(seq 100); do
    [[ -s $profile/DevToolsActivePort ]] && break
    sleep 0.1
  done
  echo "an earlier build's unit: $(systemctl --user is-active "$unit"), on port $(head -1 "$profile/DevToolsActivePort")"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

# A WebSocket upgrade to the relay's port, printing the status and never the token: `none`,
# `wrong` or `origin` (the right token from a web page).
write_ws_probe() {
  cat >"$E2E_WORK/ws-probe.py" <<'PY'
import json, socket, sys, urllib.parse
entry = json.load(open(sys.argv[1]))
url = urllib.parse.urlsplit(entry["url"])
case = sys.argv[2]
headers = [f"GET {url.path} HTTP/1.1", f"Host: {url.hostname}:{url.port}", "Upgrade: websocket",
           "Connection: Upgrade", "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==", "Sec-WebSocket-Version: 13"]
if case == "wrong":
    headers.append("Authorization: Bearer " + "0" * len(entry["token"]))
if case == "origin":
    headers.append("Authorization: Bearer " + entry["token"])
    headers.append("Origin: http://example.com")
with socket.create_connection((url.hostname, url.port), timeout=10) as connection:
    connection.sendall(("\r\n".join(headers) + "\r\n\r\n").encode())
    reply = connection.recv(4096).decode("utf-8", "replace")
print(f"  {case}: {reply.split(chr(13))[0]}")
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

steps() {
  local profile unit relay_file
  profile=$(browser_profile "$E2E_WORK/repo")
  unit=$(browser_unit "$E2E_WORK/repo")
  relay_file=$(dirname "$profile")/relay.json
  settle 12
  # Trusts the repository.
  press "" Return
  settle 2

  echo "== the earlier build's unit gives way to the relay"
  palette "marley: open browser"
  settle 10
  systemctl --user show -p ExecStart --value "$unit" | grep -oE -- '--[a-z-]+' | sort -u | tr '\n' ' ' | tee "$E2E_WORK/exec.txt"
  echo
  expect "the unit runs the relay" holds "$E2E_WORK/exec.txt" "--browser-relay" "--remote-debugging-pipe"
  expect "the unit's Chromium takes no port" bash -c "! grep -q -- '--remote-debugging-port' '$E2E_WORK/exec.txt'"
  expect "the earlier build's endpoint file is gone" test ! -e "$profile/DevToolsActivePort"
  expect "the relay's file is its owner's alone" test "$(stat -c '%a' "$relay_file")" = 600
  expect "the relay's socket is its owner's alone" test "$(stat -c '%a' "$(dirname "$profile")/relay.sock")" = 600

  echo "== the page in the tab"
  press CTRL l
  settle 1
  type_text "$SITE/index.html"
  press "" Return
  settle 4
  shot 583-01-tab
  mcp_agent tabs | tee "$E2E_WORK/tabs.txt"
  expect "the agent tools reach the page through the relay" holds "$E2E_WORK/tabs.txt" "Behind the relay"

  echo "== what listens"
  local main cgroup pid listening chromium_ports=0
  main=$(systemctl --user show -p MainPID --value "$unit")
  cgroup=$(systemctl --user show -p ControlGroup --value "$unit")
  local count
  while read -r pid; do
    [[ $pid == "$main" ]] && continue
    # grep -c counts none as a failure, which is what a process with no port gives.
    count=$(ss -ltnpH | grep -c "pid=$pid," || true)
    chromium_ports=$((chromium_ports + count))
  done <"/sys/fs/cgroup$cgroup/cgroup.procs"
  listening=$(ss -ltnpH | { grep "pid=$main," || true; } | awk '{print $4}' | tr '\n' ' ')
  echo "  the relay (pid $main) listens on: $listening; Chromium's processes on $chromium_ports ports"
  expect "Chromium listens on no TCP port" test "$chromium_ports" -eq 0
  expect "the relay listens on one port, on 127.0.0.1" bash -c "[[ '$listening' =~ ^127\.0\.0\.1:[0-9]+\ $ ]]"

  echo "== who the relay's WebSocket admits"
  for case in none wrong origin; do
    python3 "$E2E_WORK/ws-probe.py" "$relay_file" "$case" | tee -a "$E2E_WORK/probe.txt"
  done
  expect "no token: 401" holds "$E2E_WORK/probe.txt" "none: HTTP/1.1 401"
  expect "a wrong token: 401" holds "$E2E_WORK/probe.txt" "wrong: HTTP/1.1 401"
  expect "a web page's Origin: 403" holds "$E2E_WORK/probe.txt" "origin: HTTP/1.1 403"

  echo "== one client's session is not another's"
  agent hold-session 8 >"$E2E_WORK/hold.txt" &
  local holder=$!
  settle 3
  cat "$E2E_WORK/hold.txt"
  local session
  session=$(grep -oE 'holding session [A-Za-z0-9]+' "$E2E_WORK/hold.txt" | awk '{print $3}')
  agent use-session "$session" | tee "$E2E_WORK/use.txt"
  expect "a call on another client's session is refused" holds "$E2E_WORK/use.txt" "agent: refused"
  wait "$holder"

  echo "== Marley quits and comes back"
  quit_marley
  settle 2
  echo "  the unit after the quit: $(systemctl --user is-active "$unit")"
  expect "the unit lives on" systemctl --user is-active --quiet "$unit"
  launch_marley
  settle 12
  shot 583-03-restored
  mcp_agent tabs | tee "$E2E_WORK/tabs-after.txt"
  expect "the tab came back on its page" holds "$E2E_WORK/tabs-after.txt" "Behind the relay"

  echo "== Chromium closed by a client"
  agent close-browser
  for _ in $(seq 100); do
    systemctl --user is-active --quiet "$unit" || break
    sleep 0.1
  done
  echo "  the unit: $(systemctl --user is-active "$unit" || true)"
  expect "the unit stopped" bash -c "! systemctl --user is-active --quiet '$unit'"
  expect "the relay's socket is gone" test ! -e "$(dirname "$profile")/relay.sock"
  expect "the relay's file is gone" test ! -e "$relay_file"
  settle 2
  shot 583-04-closed
}
