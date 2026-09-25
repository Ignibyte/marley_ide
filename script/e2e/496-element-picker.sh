# shellcheck shell=bash
# #496's e2e test: pick an element in the Browser tab and send it to the agent. A loopback page
# has a card with a Save button (a test id, its label in a span, a click listener in app.js)
# and, below it, a button whose middle sits under a transparent layer. The terminal is used
# first. The pick button turns pick mode on, and Chromium's inspect highlight follows the
# pointer over the label; a click on the label stages a pick of the button, and pick mode ends;
# a caption and Enter type the pick's line into the terminal; the stand-in agent lists the picks
# and reads the button's bundle and crop through Marley's MCP server; Ctrl+Shift+C and a click
# on the covered button's uncovered edge pick it, and its bundle names what covers it; pick mode
# then Escape leaves no pick, and the next click reaches the page. Chromium runs offline.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The page's top left in the 1600x1000 window, as #490 measured it, and how far one row of the
# pick tray moves it down.
PAGE_X=260
PAGE_Y=109
TRAY_ROW=36
# The toolbar's pick button, and the Browser tab's tab.
PICK_X=1342
PICK_Y=87
TAB_X=515
TAB_Y=50

setup() {
  local home=$E2E_WORK/home
  offline_chromium
  mkdir -p "$home" "$E2E_WORK/site"
  printf '%s\n' "PS1='\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$E2E_WORK/site/card.html" <<'HTML'
<!doctype html><html><head><title>Card</title><style>
body { margin: 0; font: 18px sans-serif; background: #f4f1ea }
.card { position: absolute; left: 40px; top: 40px; width: 360px; height: 140px; background: #fff; border: 1px solid #ccc; border-radius: 8px }
.card h2 { position: absolute; left: 20px; top: 14px; margin: 0; font-size: 24px }
#save { position: absolute; left: 20px; top: 70px; width: 200px; height: 44px; font-size: 18px }
#covered { position: absolute; left: 40px; top: 240px; width: 200px; height: 44px; font-size: 18px }
#veil { position: absolute; left: 110px; top: 230px; width: 60px; height: 64px }
#log { position: absolute; left: 460px; top: 40px; margin: 0; font: 16px monospace }
</style><script src="app.js" defer></script></head><body>
<div class="card">
  <h2>Checkout</h2>
  <button id="save" data-testid="save-button"><span class="label">Save changes</span></button>
</div>
<button id="covered">Covered</button>
<div id="veil"></div>
<pre id="log"></pre>
</body></html>
HTML
  cat >"$E2E_WORK/site/app.js" <<'JS'
const log = (line) => { document.getElementById('log').textContent += line + '\n'; };
document.getElementById('save').addEventListener('click', () => log('saved'));
document.getElementById('covered').addEventListener('click', () => log('covered clicked'));
JS
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

# The window's point for (x, y) in the page, with `rows` rows in the pick tray above it.
page_point() {
  local rows=${3:-0} tray=0
  ((rows > 0)) && tray=$((rows * TRAY_ROW + 1))
  echo "$((PAGE_X + $1)) $((PAGE_Y + tray + $2))"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  echo "== the terminal is used first"
  click 800 500
  settle 1
  palette "marley: open browser"
  settle 5
  press CTRL l
  settle 1
  type_text "$SITE/card.html"
  press "" Return
  settle 4
  echo "== the agent's tools include the pick tools"
  mcp_agent tools
  echo "== the pick button, then the pointer over the Save button's label"
  click "$PICK_X" "$PICK_Y"
  settle 1
  # shellcheck disable=SC2046
  pointer_to $(page_point 160 132)
  settle 2
  shot 496-01-hover
  echo "== a click on the label picks the button, and pick mode ends"
  # shellcheck disable=SC2046
  click $(page_point 160 132)
  settle 3
  shot 496-02-staged
  echo "== a caption, and Enter types the pick's line into the terminal"
  type_text "Make this button green"
  press "" Return
  settle 2
  shot 496-03-sent
  echo "== the agent lists the picks and reads the first"
  mcp_agent picks
  mcp_agent pick 1 "$(shot_file 496-pick-1.jpg)"
  echo "== back in the page, Ctrl+Shift+C and a click on the covered button's uncovered edge"
  click "$TAB_X" "$TAB_Y"
  settle 2
  press "CTRL SHIFT" c
  settle 1
  # shellcheck disable=SC2046
  click $(page_point 60 262 1)
  settle 3
  mcp_agent pick 2 "$(shot_file 496-pick-2.jpg)"
  echo "== pick mode, then Escape: no pick, and a click on Save reaches the page"
  press "CTRL SHIFT" c
  settle 1
  shot 496-04a-picking
  press "" Escape
  settle 1
  # shellcheck disable=SC2046
  click $(page_point 160 132 2)
  settle 2
  shot 496-04-cancelled
  mcp_agent picks
}
