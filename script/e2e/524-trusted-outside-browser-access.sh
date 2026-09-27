# shellcheck shell=bash
# #524's e2e test: programs outside Marley that the user allowed by name reach its browser tools
# through tokens of their own. In Browser Clients the user allows `reader` to read pages and
# `driver` to act in them: each gets an endpoint file at 0600 in a 0700 folder (REQ-001). Each
# client lists only its grant's tools (REQ-002); `reader`'s click is refused and the page stays as
# it was (REQ-003); `driver` types and clicks, and the tab names it with Cut Off (REQ-004). A
# client reaches no terminal or fleet tool, no resource and no stream (REQ-010). Cut Off refuses
# `driver`'s old token and deletes its file (REQ-005), and the bridge says the token was refused
# (REQ-011). Marley's own bearer keeps every tool, on 127.0.0.1 only (REQ-007). A client's fifth
# session ends its first (REQ-008). An over-long header line gets 431 and a silent connection is
# closed within 10 seconds (REQ-009). After a restart the old token is refused and the new file
# works (REQ-006). The tokens are never printed. Chromium runs offline.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The Browser Clients modal's checkbox while no client is listed, and the toolbar's Cut Off, in
# the window's logical pixels (from the first run's shots).
CHECKBOX_X=902
FORM_Y=219
TOOLBAR_Y=87
CUT_OFF_X=1330

setup() {
  offline_chromium
  mkdir -p "$E2E_WORK/site"
  cat >"$E2E_WORK/site/form.html" <<'HTML'
<!doctype html><html><head><title>A form</title><style>
body { margin: 0; padding: 24px 40px; font: 24px sans-serif; background: #f3f6ee }
input, button { font-size: 24px }
</style></head><body>
<h1>A form an outside client fills</h1>
<p><label>Name <input id="name" style="width:360px"></label></p>
<p><button id="save">Save</button></p>
<p id="saved">Nothing saved.</p>
<script>
document.getElementById('save').addEventListener('click', () => {
  document.getElementById('saved').textContent = 'Saved: ' + document.getElementById('name').value;
});
</script></body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
  write_mcp_agent
  write_mcp_http
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
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

steps() {
  local clients=$E2E_PROFILE/mcp/clients
  local reader=$clients/reader.json driver=$clients/driver.json
  settle 12
  # Trusts the repository.
  press "" Return
  settle 2

  echo "== Browser Clients: reader allowed to read, driver to act"
  palette "marley: browser clients"
  settle 2
  shot 524-00-empty
  # driver first, while no client is listed and the checkbox sits where the empty list puts it.
  type_text "driver"
  click "$CHECKBOX_X" "$FORM_Y"
  settle 1
  press "" Return
  settle 2
  type_text "reader"
  press "" Return
  settle 2
  shot 524-01-allowed
  for file in "$clients" "$reader" "$driver" "$E2E_PROFILE/mcp/clients.json"; do
    echo "  $(stat -c '%a' "$file") $(basename "$file")"
  done
  expect "the clients' folder is 0700" test "$(stat -c '%a' "$clients")" = 700
  expect "reader's file is 0600" test "$(stat -c '%a' "$reader")" = 600
  expect "driver's file is 0600" test "$(stat -c '%a' "$driver")" = 600
  expect "the registry is 0600" test "$(stat -c '%a' "$E2E_PROFILE/mcp/clients.json")" = 600
  echo "  reader's file holds: $(jq -c 'keys' "$reader"), its header: $(jq -c '.headers | keys' "$reader")"
  expect "the registry names both, driver to act" \
    test "$(jq -c '[.clients[] | [.name, .write]]' "$E2E_PROFILE/mcp/clients.json")" = '[["driver",true],["reader",false]]'
  expect "no token in the registry" bash -c "! grep -qi 'bearer' '$E2E_PROFILE/mcp/clients.json'"
  press "" Escape
  settle 1

  echo "== the browser on the form"
  palette "marley: open browser"
  settle 6
  press CTRL l
  settle 1
  type_text "$SITE/form.html"
  press "" Return
  settle 4

  echo "== each client's tools"
  out mcp_agent --endpoint "$reader" tools
  expect "reader lists the ten reading tools" holds "$E2E_WORK/out.txt" "10 tools:"
  expect "reader lists no tool off its list" bash -c \
    "! grep -qE 'terminal_|fleet_|session_|draft_test|open_url|browser_click' '$E2E_WORK/out.txt'"
  out mcp_agent --endpoint "$driver" tools
  expect "driver lists eighteen" holds "$E2E_WORK/out.txt" "18 tools:" "browser_click" "browser_check_pick"
  expect "driver lists no tool off its list" bash -c \
    "! grep -qE 'terminal_|fleet_|session_|draft_test|open_url' '$E2E_WORK/out.txt'"

  echo "== reader may not click"
  out mcp_agent --endpoint "$reader" click-on button Save
  expect "reader's click is refused for its grant" holds "$E2E_WORK/out.txt" \
    "browser_click refused" "may only read"
  out mcp_agent --endpoint "$reader" snapshot full
  expect "the page is as it was" holds "$E2E_WORK/out.txt" "Nothing saved."

  echo "== driver types and saves"
  mcp_agent --endpoint "$driver" type-into textbox Name "from the driver"
  mcp_agent --endpoint "$driver" click-on button Save
  settle 1
  shot 524-02-driven
  out mcp_agent --endpoint "$driver" snapshot full
  expect "driver's save went through" holds "$E2E_WORK/out.txt" "Saved: from the driver"

  echo "== what no client reaches"
  out mcp_http "$reader" call terminal_read '{"terminal": 1}'
  expect "terminal_read is refused" holds "$E2E_WORK/out.txt" "terminal_read: refused" "not open to outside clients"
  out mcp_http "$reader" call fleet_snapshot
  expect "fleet_snapshot is refused" holds "$E2E_WORK/out.txt" "fleet_snapshot: refused"
  out mcp_http "$reader" resources-read
  expect "resources/read is an error" holds "$E2E_WORK/out.txt" "resources/read: error -32601"
  out mcp_http "$reader" get
  expect "a standing stream is refused" holds "$E2E_WORK/out.txt" "GET: 405"

  echo "== Cut Off"
  cp "$driver" "$E2E_WORK/driver-old.json"
  settle 5
  shot 524-02b-before-cut-off
  click "$CUT_OFF_X" "$TOOLBAR_Y"
  settle 2
  shot 524-03-cut-off
  out mcp_http "$E2E_WORK/driver-old.json" initialize
  expect "driver's old token is refused" holds "$E2E_WORK/out.txt" "initialize: 403"
  expect "driver's file is gone" test ! -e "$driver"
  expect "the registry names reader alone" \
    test "$(jq -c '[.clients[].name]' "$E2E_PROFILE/mcp/clients.json")" = '["reader"]'
  out mcp_agent --endpoint "$E2E_WORK/driver-old.json" tabs
  expect "the bridge says the token was refused" holds "$E2E_WORK/out.txt" "refused this endpoint file's token"
  expect "the bridge does not say Marley is not running" bash -c "! grep -q 'not running' '$E2E_WORK/out.txt'"

  echo "== Marley's own bearer"
  out mcp_agent tools
  expect "Marley's own bridge lists every tool" holds "$E2E_WORK/out.txt" \
    "terminal_read" "browser_draft_test" "browser_open_url"
  local port listening
  port=$(jq -r '.url' "$E2E_PROFILE/mcp-endpoint.json" | sed -E 's|.*:([0-9]+)/mcp|\1|')
  listening=$(ss -ltnH "sport = :$port" | awk '{print $4}' | tr '\n' ' ')
  echo "  the server listens on: $listening"
  expect "the server listens on 127.0.0.1 only" test "$listening" = "127.0.0.1:$port "

  echo "== a client's fifth session"
  out mcp_http "$reader" sessions 5
  expect "the fifth ended the first" holds "$E2E_WORK/out.txt" "session 1 used again: 404" "session 5 used again: 200"
  out mcp_agent tabs
  expect "Marley's own session still opens" holds "$E2E_WORK/out.txt" "form.html"

  echo "== what the server reads before it knows who asks"
  out mcp_http "$reader" long-header
  expect "a 16 KiB header line gets 431" holds "$E2E_WORK/out.txt" "a 16 KiB header line: 431"
  out mcp_http "$reader" silent
  expect "a silent connection is closed within ten seconds" bash -c \
    "grep -qE 'closed after (9|10|11) s' '$E2E_WORK/out.txt'"
  out mcp_agent --endpoint "$reader" tools
  expect "the server answers after both" holds "$E2E_WORK/out.txt" "10 tools:"

  echo "== a restart"
  cp "$reader" "$E2E_WORK/reader-old.json"
  quit_marley
  launch_marley
  settle 12
  expect "reader's file was written again" test -e "$reader"
  # The new server listens on a port of its own: the old token goes to it, with the new URL.
  jq -s '.[0] * {headers: .[1].headers}' "$reader" "$E2E_WORK/reader-old.json" \
    >"$E2E_WORK/reader-stale.json"
  out mcp_http "$E2E_WORK/reader-stale.json" initialize
  expect "reader's token from the last run is refused" holds "$E2E_WORK/out.txt" "initialize: 403"
  expect "driver has no file" test ! -e "$driver"
  out mcp_agent --endpoint "$reader" tools
  expect "reader's new file works" holds "$E2E_WORK/out.txt" "10 tools:"
}
