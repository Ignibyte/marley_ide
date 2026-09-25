# shellcheck shell=bash
# #493's e2e test: one Browser tab per page of Marley's Chromium. Page A has a `_blank` link and
# a `window.open` button; the pages they open, B and C, print each key they get, so a shot shows
# which page Marley's keys reach. The scenario opens A, clicks the link (B opens beside A, with
# the focus), goes back to A and clicks the button (C opens beside A, before B, with the focus).
# A stand-in agent, through the plugin's bridge, opens page D in a new tab, which leaves the
# focus in C, lists the tabs and looks at D by its id. The mouse closes B's tab (the browser
# loses page B), a CDP client closes page D (its tab goes), and Ctrl+T opens a blank tab with
# the focus in its address bar. Chromium runs offline.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The page's top left in the 1600x1000 window, as #490 measured it.
PAGE_X=260
PAGE_Y=109
# The tab bar's middle, and page B's tab once the agent's tab sits before it: the fifth tab.
TAB_Y=51
LINKED_TAB_X=905

setup() {
  offline_chromium
  mkdir -p "$E2E_WORK/site"
  cat >"$E2E_WORK/site/index.html" <<'HTML'
<!doctype html><html><head><title>Tabs home</title><style>
body { margin: 0; font: 20px sans-serif; background: #eef }
#link { position: absolute; left: 0; top: 0; width: 500px; height: 200px; display: flex;
  align-items: center; justify-content: center; background: #cde; font-size: 26px }
#open { position: absolute; left: 0; top: 240px; width: 500px; height: 120px; font-size: 24px }
</style></head><body>
<a id="link" href="linked.html" target="_blank">Page A: open page B in a new tab</a>
<button id="open" onclick="window.open('popup.html')">Page A: window.open page C</button>
</body></html>
HTML
  for page in linked:"Linked page":B:#efe popup:"Popup page":C:#fee; do
    IFS=: read -r file title letter color <<<"$page"
    cat >"$E2E_WORK/site/$file.html" <<HTML
<!doctype html><html><head><title>$title</title><style>
body { margin: 0; padding: 20px; font: 24px sans-serif; background: $color }
#keys { font: 28px monospace }
</style></head><body>
<p>Page $letter. The keys it got:</p>
<p id="keys"></p>
<script>
addEventListener('keydown', (event) => {
  if (event.key.length === 1) document.getElementById('keys').textContent += event.key;
});
</script>
</body></html>
HTML
  done
  cat >"$E2E_WORK/site/agent.html" <<'HTML'
<!doctype html><html><head><title>Agent page</title></head>
<body style="margin:0;padding:20px;font:24px sans-serif;background:#ffd">
<p>Page D, which the agent opened.</p></body></html>
HTML
  cat >"$E2E_WORK/site/typed.html" <<'HTML'
<!doctype html><html><head><title>Typed page</title></head>
<body style="margin:0;padding:20px;font:24px sans-serif;background:#dff">
<p>Page E, typed in the new tab's address bar.</p></body></html>
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

steps() {
  local answer agent_tab
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  palette "marley: open browser"
  settle 5
  press CTRL l
  settle 1
  type_text "$SITE/index.html"
  press "" Return
  settle 3
  echo "== the link opens page B in a tab beside page A's, with the focus"
  click_page 250 100
  settle 3
  type_text "bbb"
  settle 2
  shot 493-01-new-tab
  echo "== back in page A, window.open opens page C beside it, with the focus"
  press CTRL Page_Up
  settle 2
  click_page 250 300
  settle 3
  type_text "ccc"
  settle 2
  shot 493-01b-window-open
  echo "== the agent opens page D in a new tab; the focus stays in page C"
  answer=$(mcp_agent navigate --new-tab "$SITE/agent.html")
  echo "$answer"
  agent_tab=$(sed -n 's/.*"tab": "\([0-9A-F]*\)".*/\1/p' <<<"$answer")
  echo "== the tabs"
  mcp_agent tabs
  echo "== a look at the agent's tab by its id"
  mcp_agent look --tab "$agent_tab"
  type_text "cc"
  settle 2
  shot 493-02-agent-tab
  echo "== the mouse closes page B's tab"
  click "$LINKED_TAB_X" "$TAB_Y" middle
  settle 2
  mcp_agent tabs
  shot 493-03a-tab-closed
  echo "== a CDP client closes page D"
  agent close agent.html
  settle 2
  mcp_agent tabs
  shot 493-03-closed
  echo "== Ctrl+T opens a blank tab with the focus in its address bar"
  click_page 700 600
  settle 1
  press CTRL t
  settle 3
  type_text "$SITE/typed.html"
  settle 1
  shot 493-04-ctrl-t
  press "" Return
  settle 3
  shot 493-04b-typed
  echo "== the tabs at the end"
  mcp_agent tabs
}
