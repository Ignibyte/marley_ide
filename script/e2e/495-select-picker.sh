# shellcheck shell=bash
# #495's e2e test: Marley draws the page's <select> lists. A loopback page has a select in two
# groups, one option disabled, and prints its value and each input and change it gets; a
# cross-site iframe below it has a select of its own. A click on the select opens Marley's list
# under it, the current option marked; a click on an option chooses it; the list opened again
# closes on Escape and the value stays; Alt+Down on the focused select opens the list, and
# ArrowUp and Enter choose from it; a click on the iframe's select opens its list under it; and
# a stand-in agent's click on the select, through the plugin's bridge, opens no list. Chromium
# runs offline.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The page's top left in the 1600x1000 window, as #490 measured it.
PAGE_X=260
PAGE_Y=109
# "Plum" in the page select's list, measured from 495-01-open.
PLUM_X=327
PLUM_Y=333

setup() {
  local port_b
  offline_chromium
  mkdir -p "$E2E_WORK/site-a" "$E2E_WORK/site-b"
  cat >"$E2E_WORK/site-b/frame.html" <<'HTML'
<!doctype html><html><body style="margin:0;background:#dff0d8;font:16px sans-serif">
<select id="size" style="position:absolute;left:20px;top:20px;width:180px;height:30px;font-size:16px">
  <option>Small</option><option selected>Medium</option><option>Large</option>
</select>
</body></html>
HTML
  port_b=$(serve_site site-b)
  cat >"$E2E_WORK/site-a/index.html" <<HTML
<!doctype html><html><head><title>Select</title><style>
body { margin: 0; font: 18px sans-serif; background: #eef }
#fruit { position: absolute; left: 20px; top: 20px; width: 220px; height: 34px; font-size: 18px }
#value { position: absolute; left: 20px; top: 70px; margin: 0 }
#log { position: absolute; left: 320px; top: 20px; margin: 0; font: 16px monospace }
iframe { position: absolute; left: 20px; top: 260px; width: 400px; height: 120px; border: 2px solid #888 }
</style></head><body>
<select id="fruit">
  <optgroup label="Sweet"><option value="apple">Apple</option><option value="pear" selected>Pear</option></optgroup>
  <optgroup label="Sour"><option value="lemon" disabled>Lemon</option><option value="lime">Lime</option></optgroup>
  <option value="plum">Plum</option>
</select>
<p id="value"></p>
<pre id="log"></pre>
<iframe src="http://localhost:$port_b/frame.html"></iframe>
<script>
const select = document.getElementById('fruit');
const show = () => document.getElementById('value').textContent = 'The value: ' + select.value;
const log = (line) => document.getElementById('log').textContent += line + '\\n';
select.addEventListener('input', () => log('input ' + select.value));
select.addEventListener('change', () => { log('change ' + select.value); show(); });
show();
</script>
</body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site-a)
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
  echo "== a click on the select opens Marley's list"
  click_page 130 37
  settle 2
  shot 495-01-open
  echo "== a click on Plum chooses it"
  click "$PLUM_X" "$PLUM_Y"
  settle 2
  shot 495-02-chosen
  echo "== the list opened again closes on Escape"
  click_page 130 37
  settle 2
  press "" Escape
  settle 2
  shot 495-03-escaped
  echo "== Alt+Down on the focused select opens the list; ArrowUp and Enter choose"
  press ALT Down
  settle 2
  shot 495-04a-keys-open
  press "" Up
  settle 1
  press "" Return
  settle 2
  shot 495-04-keys
  echo "== a click on the cross-site iframe's select opens its list under it"
  click_page 132 299
  settle 2
  shot 495-05-frame
  press "" Escape
  settle 1
  echo "== an agent's click on the select opens no list"
  mcp_agent click-on combobox ""
  settle 2
  shot 495-06-agent
}
