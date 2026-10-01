# shellcheck shell=bash
# #503's e2e test: where a URL clicked in a terminal opens, and the dev server URL a terminal
# offers. A fake `xdg-open`, first on the PATH Marley starts with and checked before the launch,
# logs what it is asked to open, so the system browser is never reached. On the terminal's PATH:
# `links` prints a docs URL and a URL nothing listens on; `devserver` listens on 127.0.0.1 and
# prints its URL as Vite does; `osc8` prints an OSC 8 link to the site's page; `ssh` is a stand-in
# SSH client that prints a local URL. Each fake waits after printing, so the URL clicked is the
# row above the cursor's, or the row above that. Checks: the run's `xdg-open` log, the Browser
# tabs a stand-in agent lists through Marley's MCP server, and the clipboard under the sway.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

SITE=
SITE_PORT=

# Where things are, in the headless output's pixels, measured from the first run's shots. The
# terminal's rows are counted up from the cursor's, the text starts at TEXT_X and each cell is
# CELL_W wide.
TEXT_X=${TEXT_X:-269}
CELL_W=${CELL_W:-9}
ROW_H=${ROW_H:-19.5}
# The cursor's row while a fake waits, with the offer's strip under the terminal and without.
CURSOR_Y_STRIP=${CURSOR_Y_STRIP:-930}
CURSOR_Y=${CURSOR_Y:-956}
# The rail's row of the first terminal.
TERMINAL_ROW_X=${TERMINAL_ROW_X:-100}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-136}
# The offer's label and its arrow, in the strip under a terminal.
OFFER_X=${OFFER_X:-1282}
OFFER_ARROW_X=${OFFER_ARROW_X:-1342}
OFFER_Y=${OFFER_Y:-954}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin sysbin=$E2E_WORK/sysbin
  mkdir -p "$home" "$bin" "$sysbin" "$E2E_WORK/site"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  # The system browser: logs its argument and always succeeds, so neither `open`'s later
  # commands nor gpui's desktop-portal fallback, which would reach the user's own browser, runs.
  cat >"$sysbin/xdg-open" <<SH
#!/bin/sh
printf '%s\n' "\$*" >>"$E2E_WORK/xdg-open.log" || true
exit 0
SH
  chmod +x "$sysbin/xdg-open"
  : >"$E2E_WORK/xdg-open.log"
  export PATH="$sysbin:$PATH"
  [[ $(command -v xdg-open) == "$sysbin/xdg-open" ]] || {
    echo "the fake xdg-open is not first on the PATH; not launching" >&2
    return 1
  }
  printf '<!doctype html><title>Site page</title><p>The site.</p>\n' >"$E2E_WORK/site/index.html"
  printf '<!doctype html><title>OSC 8 page</title><p>Reached by an OSC 8 link.</p>\n' \
    >"$E2E_WORK/site/osc8.html"
  offline_chromium
  SITE_PORT=$(serve_site site)
  SITE=http://127.0.0.1:$SITE_PORT
  write_fakes
  git init -q -b links "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

write_fakes() {
  local bin=$E2E_WORK/bin
  cat >"$bin/links" <<'SH'
#!/bin/sh
echo "docs: https://example.com/docs"
echo "dead: http://localhost:9/nothing"
read -r _
SH
  cat >"$bin/devserver" <<PY
#!/usr/bin/env python3
# A stand-in dev server: one page on 127.0.0.1 at a free port, its URL printed as Vite prints
# it, served until Ctrl+C.
import http.server

class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        body = b"<!doctype html><title>Dev page</title><h1>Dev page</h1>"
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_):
        pass

server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
port = server.server_address[1]
with open("$E2E_WORK/devserver.port", "w") as out:
    out.write(str(port))
print(f"\n  VITE v5.4.0  ready in 120 ms\n\n  ➜  Local:   http://localhost:{port}/", flush=True)
try:
    server.serve_forever()
except KeyboardInterrupt:
    pass
PY
  cat >"$bin/osc8" <<SH
#!/bin/sh
printf '\033]8;;%s\033\\\\%s\033]8;;\033\\\\\n' "$SITE/osc8.html" "Open the OSC 8 page"
read -r _
SH
  # Named ssh and run by python3, so the terminal reads its foreground program as `ssh`. Marley's
  # `ssh` asks `ssh -G` for the host's config first (#526); refused, it runs the plain ssh.
  cat >"$bin/ssh" <<PY
#!/usr/bin/env python3
import sys, time
if "-G" in sys.argv[1:]:
    sys.exit(255)
print("Welcome to e2e-host")
print("Local dev: http://localhost:$SITE_PORT/", flush=True)
time.sleep(600)
PY
  chmod +x "$bin/links" "$bin/devserver" "$bin/osc8" "$bin/ssh"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# The y of the row `$1` rows above the cursor's, with the strip (`strip`) or without (`plain`).
row_y() {
  local base=$CURSOR_Y
  [[ $2 == strip ]] && base=$CURSOR_Y_STRIP
  awk -v base="$base" -v rows="$1" -v height="$ROW_H" 'BEGIN { printf "%d", base - rows * height }'
}

# The x of column `$1` (from 0) of a row.
column_x() {
  awk -v x="$TEXT_X" -v column="$1" -v width="$CELL_W" 'BEGIN { printf "%d", x + (column + 0.5) * width }'
}

# Brings the first terminal to the front from the rail.
first_terminal() {
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y"
  settle 1
}

tabs() {
  mcp_agent tabs >"$E2E_WORK/tabs.txt"
  cat "$E2E_WORK/tabs.txt"
}

opened() {
  grep -cxF -- "$1" "$E2E_WORK/xdg-open.log" || true
}

tab_count() {
  grep -cF -- " at $1, " "$E2E_WORK/tabs.txt" || true
}

# `marley.<key>` set to the JSON value in the profile copy's settings, as #516's scenario sets
# its settings.
marley_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'PY'
import pathlib, re, sys
path, key, value = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3]
text = path.read_text()
existing = re.search(r'"%s"\s*:\s*(true|false|"[^"\n]*"|-?[0-9.]+)' % re.escape(key), text)
if existing:
    text = text[: existing.start(1)] + value + text[existing.end(1) :]
else:
    block = re.search(r'"marley"\s*:\s*\{', text)
    if block:
        text = text[: block.end()] + f'\n    "{key}": {value},' + text[block.end() :]
    else:
        brace = text.index("{")
        text = text[: brace + 1] + f'\n  "marley": {{ "{key}": {value} }},' + text[brace + 1 :]
path.write_text(text)
PY
}

steps() {
  local dev url
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  echo "== a URL that is not local goes to the system browser"
  type_text "links"
  press "" Return
  settle 3
  shot 503-00-links
  click_with CTRL "$(column_x 8)" "$(row_y 2 plain)"
  settle 2
  expect "Ctrl+click on a docs URL opened the system browser" \
    test "$(opened "https://example.com/docs")" -eq 1
  press "" Return
  settle 1
  echo "== a dev server's URL is offered while its port listens"
  type_text "devserver"
  press "" Return
  settle 5
  dev=$(cat "$E2E_WORK/devserver.port")
  url=http://localhost:$dev/
  shot 503-01-offer
  echo "== Ctrl+click opens it in a Browser tab of the project"
  click_with CTRL "$(column_x 16)" "$(row_y 1 strip)"
  settle 4
  shot 503-02-browser-tab
  tabs
  expect "a Browser tab of the project on the dev server, focused" \
    holds "$E2E_WORK/tabs.txt" "'Dev page' at $url, project repo, focused"
  echo "== again: the same tab comes forward"
  first_terminal
  click_with CTRL "$(column_x 16)" "$(row_y 1 strip)"
  settle 3
  shot 503-03-same-tab
  tabs
  expect "still one tab on the dev server" test "$(tab_count "$url")" -eq 1
  # Closes the Browser tab, which is in front with the focus.
  press CTRL w
  settle 2
  echo "== Shift+Ctrl+click opens it in the system browser"
  first_terminal
  click_with "SHIFT CTRL" "$(column_x 16)" "$(row_y 1 strip)"
  settle 2
  shot 503-04-inverse
  tabs
  expect "Shift+Ctrl+click opened the system browser" test "$(opened "$url")" -eq 1
  expect "and no tab" test "$(tab_count "$url")" -eq 0
  echo "== the offer's label opens it as Ctrl+click does"
  click "$OFFER_X" "$OFFER_Y"
  settle 4
  shot 503-07-offer-opens
  tabs
  expect "the offer opened a Browser tab on the dev server" test "$(tab_count "$url")" -eq 1
  echo "== the offer's menu"
  first_terminal
  click "$OFFER_ARROW_X" "$OFFER_Y"
  settle 1
  shot 503-08-offer-menu
  press "" Down
  press "" Down
  press "" Return
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wl-paste --no-newline >"$E2E_WORK/clipboard.txt" || true
  echo "clipboard: $(cat "$E2E_WORK/clipboard.txt")"
  expect "Copy URL put the URL on the clipboard" \
    test "$(cat "$E2E_WORK/clipboard.txt")" = "$url"
  click "$OFFER_ARROW_X" "$OFFER_Y"
  settle 1
  press "" Down
  press "" Return
  settle 2
  expect "Open in System Browser opened it there" test "$(opened "$url")" -eq 2
  echo "== the offer goes when the server stops"
  press CTRL c
  settle 5
  shot 503-09-offer-gone
  echo "== a server on 0.0.0.0 opens at 127.0.0.1"
  type_text "python3 -m http.server 0 --bind 0.0.0.0"
  press "" Return
  settle 5
  shot 503-05-unspecified-offer
  click_with CTRL "$(column_x 40)" "$(row_y 1 strip)"
  settle 4
  shot 503-06-unspecified-tab
  tabs
  expect "the 0.0.0.0 URL opened at 127.0.0.1" \
    bash -c "grep -qE \"at http://127\\.0\\.0\\.1:[0-9]+/, project repo, focused\" '$E2E_WORK/tabs.txt'"
  first_terminal
  press CTRL c
  settle 3
  echo "== an OSC 8 link, clicked, opens its target"
  type_text "osc8"
  press "" Return
  settle 3
  # Nothing listens any more, so no strip.
  click "$(column_x 4)" "$(row_y 1 plain)"
  settle 4
  shot 503-10-osc8
  tabs
  expect "the OSC 8 link's target opened in a Browser tab" \
    holds "$E2E_WORK/tabs.txt" "'OSC 8 page' at $SITE/osc8.html, project repo, focused"
  first_terminal
  press "" Return
  settle 1
  echo "== an agent's terminal carries the offer in its bar"
  palette "workspace: new terminal"
  settle 3
  type_text "exec -a claude bash -c 'echo \"Dev server: http://localhost:$SITE_PORT/\"; while :; do sleep 1; done'"
  press "" Return
  settle 5
  shot 503-11-agent-bar
  echo "== over ssh, nothing is offered and every URL goes to the system browser"
  palette "workspace: new terminal"
  settle 3
  type_text "ssh e2e-host"
  press "" Return
  settle 5
  shot 503-12-ssh
  click_with CTRL "$(column_x 14)" "$(row_y 1 plain)"
  settle 2
  tabs
  expect "a URL clicked over ssh opened the system browser" \
    test "$(opened "http://localhost:$SITE_PORT/")" -eq 1
  expect "and no tab" test "$(tab_count "http://localhost:$SITE_PORT/")" -eq 0
  echo "== marley.terminal_links set to system_browser"
  marley_setting terminal_links '"system_browser"'
  settle 3
  first_terminal
  type_text "devserver"
  press "" Return
  settle 5
  dev=$(cat "$E2E_WORK/devserver.port")
  url=http://localhost:$dev/
  click_with CTRL "$(column_x 16)" "$(row_y 1 strip)"
  settle 2
  expect "Ctrl+click opened the system browser" test "$(opened "$url")" -eq 1
  first_terminal
  click_with "SHIFT CTRL" "$(column_x 16)" "$(row_y 1 strip)"
  settle 4
  shot 503-14-system-default
  tabs
  expect "Shift+Ctrl+click opened a Browser tab" test "$(tab_count "$url")" -eq 1
  echo "== the Marley page's Terminal section"
  palette "marley: open settings"
  settle 4
  shot 503-13-settings
}
