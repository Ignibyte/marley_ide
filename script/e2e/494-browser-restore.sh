# shellcheck shell=bash
# #494's e2e test: the Browser tabs come back after a relaunch. Two tabs show two pages of a
# loopback site; the first page's field holds typed text, and the page prints when it loaded.
# Marley quits through its palette and starts again on the same profile, its Chromium still
# running: both tabs return on the same pages, the text still in the field, under the same ids.
# Marley quits again, its Chromium is stopped, and Marley starts once more: both tabs return at
# their URLs on new pages, the field empty, and no third tab. After each state the run prints
# the tabs and the profile database's rows for them: the workspace's items (kind and id only)
# and the tabs' own (page, URL and title). Chromium runs offline.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The page's top left in the 1600x1000 window, as #490 measured it.
PAGE_X=260
PAGE_Y=109
# The tab bar's middle, and the first Browser tab, after the terminal's.
TAB_Y=51
FIRST_TAB_X=530

setup() {
  offline_chromium
  mkdir -p "$E2E_WORK/site"
  cat >"$E2E_WORK/site/first.html" <<'HTML'
<!doctype html><html><head><title>First page</title><style>
body { margin: 0; font: 24px sans-serif; background: #eef }
p { position: absolute; left: 20px; margin: 0 }
#field { position: absolute; left: 20px; top: 100px; width: 500px; height: 50px; font-size: 24px }
</style></head><body>
<p style="top: 30px">The first page. Its field keeps what was typed until the page loads again.</p>
<input id="field">
<p id="loaded" style="top: 180px"></p>
<script>
document.getElementById('loaded').textContent =
  'Loaded ' + new Date().toISOString().slice(11, 19) + ' UTC';
</script>
</body></html>
HTML
  cat >"$E2E_WORK/site/second.html" <<'HTML'
<!doctype html><html><head><title>Second page</title></head>
<body style="margin:0;padding:20px;font:24px sans-serif;background:#efe">
<p>The second page.</p></body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
  git init -q -b browser "$E2E_WORK/repo"
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

# Clicks the page at (x, y) in its own pixels.
click_page() {
  click $((PAGE_X + $1)) $((PAGE_Y + $2)) "${3:-left}"
}

# The tabs as the agent tools list them, and the profile database's rows for them. The profile is
# a copy of the user's, with a database per channel, so each is read, and only this run's
# workspaces' rows.
report() {
  local database
  echo "== $1"
  mcp_agent tabs
  for database in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 -readonly "$database" \
      "SELECT '  item ' || i.item_id || ': kind ' || i.kind FROM items i
       JOIN workspaces w ON w.workspace_id = i.workspace_id
       WHERE i.kind = 'MarleyBrowserTab' AND w.paths LIKE '%$E2E_WORK%' ORDER BY i.position;
       SELECT '  saved ' || t.item_id || ': page ' || t.target_id || ', ' || t.url || ', ' ||
         quote(t.title)
       FROM marley_browser_tabs t JOIN workspaces w ON w.workspace_id = t.workspace_id
       WHERE w.paths LIKE '%$E2E_WORK%' ORDER BY t.item_id" 2>/dev/null || true
  done
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  palette "marley: open browser"
  settle 5
  press CTRL l
  settle 1
  type_text "$SITE/first.html"
  press "" Return
  settle 3
  click_page 250 125
  type_text "typed before the relaunch"
  settle 1
  press CTRL t
  settle 3
  type_text "$SITE/second.html"
  press "" Return
  settle 3
  shot 494-01-before
  report "before the relaunch"
  echo "== Marley quits and starts again; its Chromium runs on"
  quit_marley
  launch_marley
  settle 12
  click "$FIRST_TAB_X" "$TAB_Y"
  settle 2
  shot 494-02-reattached
  report "after the relaunch"
  echo "== Marley quits, its Chromium stops, and Marley starts again"
  quit_marley
  systemctl --user stop "$(browser_unit "$E2E_WORK/repo")"
  echo "the unit: $(systemctl --user is-active "$(browser_unit "$E2E_WORK/repo")" || true)"
  launch_marley
  settle 15
  click "$FIRST_TAB_X" "$TAB_Y"
  settle 2
  shot 494-03-reopened
  report "after the relaunch with a new Chromium"
}
