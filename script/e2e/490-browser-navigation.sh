# shellcheck shell=bash
# #490's e2e test: the Browser tab's toolbar. A loopback site serves page A (a link to B), page
# B (it counts its loads in sessionStorage and says how each one came), a dialogs page (alert,
# confirm and prompt, each printing its answer) and /slow. The scenario goes to A by URL,
# follows the link, goes back with Alt+Left and forward with the button, types a host with no
# scheme, reloads with Ctrl+R and F5, leaves the address bar with Escape, stops a slow load,
# searches, answers three dialogs, and lets a stand-in agent navigate. Chromium resolves no
# host here, so the search fails in the page and nothing leaves the machine.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The page's top left in the 1600x1000 window: right of the rail, under the tab bar and the
# toolbar. A target at (x, y) in the page is at (PAGE_X + x, PAGE_Y + y) in the window.
PAGE_X=260
PAGE_Y=109
# The toolbar's buttons, in the window.
TOOLBAR_Y=88
FORWARD_X=303
RELOAD_X=329

setup() {
  offline_chromium
  mkdir -p "$E2E_WORK/site"
  cat >"$E2E_WORK/site/a.html" <<'HTML'
<!doctype html><html><head><title>Page A</title><style>
body { margin: 0; font: 20px sans-serif; background: #eef }
#link { position: absolute; left: 0; top: 0; width: 600px; height: 300px; display: flex;
  align-items: center; justify-content: center; background: #cde; font-size: 28px }
</style></head><body>
<a id="link" href="b.html">Page A: go to page B</a>
</body></html>
HTML
  cat >"$E2E_WORK/site/b.html" <<'HTML'
<!doctype html><html><head><title>Page B</title><style>
body { margin: 0; font: 20px sans-serif; background: #efe }
#loads { position: absolute; left: 20px; top: 20px; margin: 0; font-size: 24px }
#field { position: absolute; left: 20px; top: 120px; width: 400px; height: 60px;
  font-size: 22px; box-sizing: border-box }
</style></head><body>
<p id="loads"></p>
<input id="field">
<script>
const loads = Number(sessionStorage.getItem('b-loads') || 0) + 1;
sessionStorage.setItem('b-loads', loads);
const how = performance.getEntriesByType('navigation')[0]?.type ?? 'unknown';
const show = (came) => document.getElementById('loads').textContent =
  'Page B, load ' + loads + ': ' + came;
show(how);
addEventListener('pageshow', (event) => { if (event.persisted) show('back-forward cache'); });
</script>
</body></html>
HTML
  cat >"$E2E_WORK/site/dialogs.html" <<'HTML'
<!doctype html><html><head><title>Dialogs</title><style>
body { margin: 0; font: 20px sans-serif }
button { position: absolute; left: 20px; width: 300px; height: 80px; font-size: 22px }
#log { position: absolute; left: 360px; top: 20px; white-space: pre; font: 20px monospace }
</style></head><body>
<button id="alert" style="top:20px">alert</button>
<button id="confirm" style="top:140px">confirm</button>
<button id="prompt" style="top:260px">prompt</button>
<div id="log"></div>
<script>
const log = (line) => { document.getElementById('log').textContent += line + '\n'; };
document.getElementById('alert').onclick = () => {
  alert('Hello from the page.');
  log('alert: answered');
};
document.getElementById('confirm').onclick = () => log('confirm: ' + confirm('Keep going?'));
document.getElementById('prompt').onclick = () =>
  log('prompt: ' + prompt('What should the page print?', 'the default'));
</script>
</body></html>
HTML
  PORT=$(serve_site site)
  SITE=http://127.0.0.1:$PORT
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

# Types `text` in the address bar after Ctrl+L, and Enter.
go_to() {
  press CTRL l
  settle 1
  type_text "$1"
  press "" Return
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
  shot 490-00-address-selected
  type_text "$SITE/a.html"
  press "" Return
  settle 3
  shot 490-01-typed-url
  click_page 300 150
  settle 3
  shot 490-02-link
  press ALT Left
  settle 3
  shot 490-03-back
  click "$FORWARD_X" "$TOOLBAR_Y"
  settle 3
  shot 490-03b-forward
  go_to "127.0.0.1:$PORT/b.html"
  settle 3
  shot 490-04a-host
  press CTRL r
  settle 3
  press "" F5
  settle 3
  shot 490-04-forward-reload
  # Escape leaves the address bar with the page's URL, and the page has the focus again: the
  # field takes the rest of the typing.
  click_page 220 150
  settle 1
  type_text "before"
  press CTRL l
  settle 1
  type_text "not a destination"
  settle 1
  press "" Escape
  settle 1
  type_text " after"
  settle 1
  shot 490-06b-restored
  go_to "$SITE/slow"
  settle 1
  shot 490-05-loading
  click "$RELOAD_X" "$TOOLBAR_Y"
  settle 2
  shot 490-05b-stopped
  go_to "marley browser test"
  settle 3
  shot 490-06-search
  go_to "$SITE/dialogs.html"
  settle 3
  click_page 170 60
  settle 2
  shot 490-07-alert
  press "" Return
  settle 1
  click_page 170 180
  settle 2
  press "" Escape
  settle 1
  click_page 170 300
  settle 2
  shot 490-08a-prompt
  type_text "typed in Marley"
  press "" Return
  settle 2
  shot 490-08-prompt-answered
  agent navigate "$SITE/a.html"
  settle 2
  shot 490-09-agent-navigated
}
