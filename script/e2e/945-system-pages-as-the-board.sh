# shellcheck shell=bash
# #945's visual check: the System tab's pages as the System board draws them. A scratch repository;
# a page on loopback that sets a cookie, served by the scenario; `install.json` in the run's data
# folder naming the debug binary and another commit.
# - Install and update: the four steps and "Running Marley · still on …" (`945-01-install`,
#   REQ-001, REQ-002).
# - Marley's MCP endpoint: Endpoint, Bridge, Tools, Last refusal, Limits and the two buttons
#   (`945-02-mcp`, REQ-003, REQ-004); Open Agent Activity clicked: the Agent Activity tab
#   (`945-03-activity`, REQ-004).
# - The page opened in a Browser tab, then Marley's Chromium: the version, the project's profile
#   with its pages and cookie sites, the limits row (`945-04-chromium`, REQ-005).
# - Shell integration, Fleet providers and For developers with their buttons (`945-05a-shell`,
#   `945-05b-fleet`, `945-05c-developers`, REQ-006).
# - Marley Day, Install and update (`945-06-day`, REQ-001, REQ-002).
compositor sway

STOP_AT=${STOP_AT:-0}
# The page list's rows, from #855's shots: This build at 113, then one every 29 pixels.
LIST_X=${LIST_X:-340}
list_y() { echo $((113 + 29 * $1)); }
# The MCP page's first button, from the first run's shot.
ACTIVITY_X=${ACTIVITY_X:-587}
ACTIVITY_Y=${ACTIVITY_Y:-496}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo binary
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  binary=$(realpath "${CARGO_TARGET_DIR:-$PWD/target}/debug/marley")
  cat >"$E2E_PROFILE/install.json" <<JSON
{
  "commit": "0000000000000000000000000000000000000945",
  "dirty": false,
  "ended": "2026-10-28T12:00:00-0500",
  "binary": "$binary",
  "steps": [
    {"name": "toolchain", "value": "rustc 1.98.1 (stand-in)", "result": "ok"},
    {"name": "build", "value": "412 s", "result": "ok"},
    {"name": "remote server", "value": "linux-x86_64", "result": "ok"},
    {"name": "desktop entry", "value": "$E2E_WORK/marley.desktop", "result": "ok"}
  ]
}
JSON
  write_site
  python3 -u "$E2E_WORK/site.py" "$E2E_WORK/site.port" >"$E2E_WORK/site.out" 2>&1 &
  echo "$!" >"$E2E_WORK/site.pid"
  local waited=0
  until [[ -s $E2E_WORK/site.port ]] || ((waited > 50)); do
    sleep 0.1
    waited=$((waited + 1))
  done
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

# A page on loopback that sets one cookie.
write_site() {
  cat >"$E2E_WORK/site.py" <<'PY'
import http.server
import sys


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        body = b"<!doctype html><title>Cookie page</title><h1>A page with a cookie</h1>"
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.send_header("Set-Cookie", "tour=945; Path=/")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
with open(sys.argv[1], "w") as port:
    port.write(str(server.server_address[1]))
server.serve_forever()
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

# Shows the page at `$1` in the list and shoots it as `$2`.
page_shot() {
  click "$LIST_X" "$(list_y "$1")"
  settle 3
  pointer_to 1500 950
  settle 1
  shot "$2"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the cookie page in a Browser tab"
  palette "marley: open browser" 5
  press CTRL l
  settle 1
  type_text "http://127.0.0.1:$(cat "$E2E_WORK/site.port")/"
  press "" Return
  settle 5

  echo "== the System tab"
  palette "marley: open system pages" 5
  page_shot 2 945-01-install
  page_shot 5 945-02-mcp
  if [[ $STOP_AT == 1 ]]; then
    return
  fi
  page_shot 6 945-04-chromium
  page_shot 4 945-05a-shell
  page_shot 7 945-05b-fleet
  page_shot 8 945-05c-developers

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  page_shot 2 945-06-day

  echo "== Open Agent Activity"
  profile_setting theme '"Marley Night"'
  settle 3
  page_shot 5 945-02-mcp-again
  click "$ACTIVITY_X" "$ACTIVITY_Y"
  settle 3
  pointer_to 1500 950
  settle 1
  shot 945-03-activity
  kill "$(cat "$E2E_WORK/site.pid")" 2>/dev/null || true
}
