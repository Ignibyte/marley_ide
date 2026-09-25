# shellcheck shell=bash
# #497's e2e test: a pick's listener, through its script's source map, opened in the editor. The
# repository holds `src/app.ts`; a loopback site serves what a build would have made of it,
# `app.js` line for line with a hand-written `app.js.map` that names `../src/app.ts`, and
# `plain.js`, which has no map. A pick of the Save button, whose click listener `app.js` adds,
# shows the listener as `src/app.ts:2` in the tray; a click on that opens `src/app.ts` in the
# editor on line 2. A pick of the Plain button shows its listener as `plain.js:2`, which opens
# nothing. The stand-in agent's `browser_pick` gives both. Chromium runs offline.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The page's top left in the 1600x1000 window, as #490 measured it, and how far one row of the
# pick tray moves it down.
PAGE_X=260
PAGE_Y=109
TRAY_ROW=36
# The Browser tab's tab, and the place of the newest pick's listener in the tray.
TAB_X=515
TAB_Y=50
PLACE_X=470
PLACE_Y=127

setup() {
  local home=$E2E_WORK/home
  offline_chromium
  mkdir -p "$home" "$E2E_WORK/site" "$E2E_WORK/repo/src"
  printf '%s\n' "PS1='\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  git init -q -b browser "$E2E_WORK/repo"
  cat >"$E2E_WORK/repo/src/app.ts" <<'TS'
const save = document.getElementById('save') as HTMLButtonElement;
save.addEventListener('click', () => {
  document.getElementById('log')!.textContent += 'saved\n';
});
TS
  cat >"$E2E_WORK/site/app.js" <<'JS'
const save = document.getElementById('save');
save.addEventListener('click', () => {
  document.getElementById('log').textContent += 'saved\n';
});
//# sourceMappingURL=app.js.map
JS
  # Line for line: each generated line's first column is the same line of the source.
  printf '%s\n' '{"version":3,"file":"app.js","sources":["../src/app.ts"],"names":[],"mappings":"AAAA;AACA;AACA;AACA"}' \
    >"$E2E_WORK/site/app.js.map"
  cat >"$E2E_WORK/site/plain.js" <<'JS'
// This script has no source map.
document.getElementById('plain').addEventListener('click', () => {
  document.getElementById('log').textContent += 'plain\n';
});
JS
  cat >"$E2E_WORK/site/index.html" <<'HTML'
<!doctype html><html><head><title>Sources</title><style>
body { margin: 0; font: 18px sans-serif; background: #eef4f1 }
#save { position: absolute; left: 40px; top: 40px; width: 200px; height: 44px; font-size: 18px }
#plain { position: absolute; left: 40px; top: 120px; width: 200px; height: 44px; font-size: 18px }
#log { position: absolute; left: 300px; top: 40px; margin: 0; font: 16px monospace }
</style></head><body>
<button id="save">Save</button>
<button id="plain">Plain</button>
<pre id="log"></pre>
<script src="app.js"></script>
<script src="plain.js"></script>
</body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
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
  palette "marley: open browser"
  settle 5
  press CTRL l
  settle 1
  type_text "$SITE/index.html"
  press "" Return
  settle 4
  echo "== a pick of the Save button: its listener is src/app.ts:2"
  press "CTRL SHIFT" c
  settle 1
  # shellcheck disable=SC2046
  click $(page_point 140 62)
  settle 3
  shot 497-01-tray
  echo "== a click on the listener opens src/app.ts on line 2"
  click "$PLACE_X" "$PLACE_Y"
  settle 3
  shot 497-02-opened
  echo "== back in the page, a pick of the Plain button: plain.js:2, which opens nothing"
  click "$TAB_X" "$TAB_Y"
  settle 2
  press "CTRL SHIFT" c
  settle 1
  # shellcheck disable=SC2046
  click $(page_point 140 142 1)
  settle 3
  click "$PLACE_X" "$PLACE_Y"
  settle 2
  shot 497-03-no-map
  echo "== the agent reads both picks"
  mcp_agent pick 1
  mcp_agent pick 2
}
