# shellcheck shell=bash
# #489's e2e test: typing and clicking in the Browser tab. A fixture page, every target at a
# fixed place, reports what it gets in a status box: its scroll, the keys it saw, what its
# form submitted and each click with its button and count, with a red dot where each press
# landed. The scenario types and edits in a field, composes an é, clicks, double-clicks and
# right-clicks a button, types into a cross-site iframe, pastes and copies through the system
# clipboard, sends a Super chord the page must not see, scrolls, and reports how long input
# took to reach the screen.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The page's top left in the 1600x1000 window: right of the rail, under the tab bar (488's
# shots). A target at (x, y) in the page is at (PAGE_X + x, PAGE_Y + y) in the window.
PAGE_X=260
PAGE_Y=67

setup() {
  local port_b
  export ZED_LOG=marley_browser=debug
  mkdir -p "$E2E_WORK/site-a" "$E2E_WORK/site-b"
  cat >"$E2E_WORK/site-b/frame.html" <<'HTML'
<!doctype html><html><body style="margin:0;padding:8px;background:#dff0d8;font:16px sans-serif">
<input id="inner" placeholder="the frame's field" style="width:180px;font-size:16px">
</body></html>
HTML
  port_b=$(serve_site site-b)
  cat >"$E2E_WORK/site-a/index.html" <<HTML
<!doctype html><html><head><title>Input fixture</title><style>
body { margin: 0; font: 16px sans-serif; height: 3000px }
.at { position: absolute }
#dots i { position: absolute; width: 8px; height: 8px; margin: -4px 0 0 -4px;
  background: red; border-radius: 4px; pointer-events: none }
#status { position: fixed; right: 10px; top: 10px; width: 360px; white-space: pre;
  font: 14px monospace; background: #eef; padding: 6px }
</style></head><body>
<form class="at" style="left:20px;top:20px" id="form">
  <input id="field" style="width:300px;height:30px;font-size:18px;box-sizing:border-box">
</form>
<textarea id="notes" class="at"
  style="left:20px;top:70px;width:300px;height:60px;font-size:18px;box-sizing:border-box"></textarea>
<button id="button" class="at"
  style="left:20px;top:150px;width:120px;height:40px;font-size:16px">Click me</button>
<p id="words" class="at" style="left:20px;top:200px;margin:0;font-size:18px">copy these words</p>
<iframe class="at" src="http://localhost:$port_b/frame.html"
  style="left:20px;top:250px;width:340px;height:80px;border:2px solid #888"></iframe>
<div id="status"></div>
<div id="dots"></div>
<script>
const lines = [];
const keys = [];
const status = document.getElementById('status');
const render = () => status.textContent =
  'scrollY ' + scrollY + '\\nkeys ' + keys.join(' ') + '\\n' + lines.join('\\n');
const log = (line) => { lines.push(line); if (lines.length > 8) lines.shift(); render(); };
addEventListener('keydown', (event) => {
  keys.push(event.key === ' ' ? 'Space' : event.key);
  if (keys.length > 12) keys.shift();
  render();
}, true);
addEventListener('scroll', render);
addEventListener('mousedown', (event) => {
  const dot = document.createElement('i');
  dot.style.left = event.pageX + 'px';
  dot.style.top = event.pageY + 'px';
  document.getElementById('dots').appendChild(dot);
});
document.getElementById('form').addEventListener('submit', (event) => {
  event.preventDefault();
  log('submitted: ' + document.getElementById('field').value);
});
const button = document.getElementById('button');
button.addEventListener('click', (event) => log('click: button ' + event.button + ', count ' + event.detail));
button.addEventListener('contextmenu', (event) => { event.preventDefault(); log('contextmenu: button ' + event.button); });
render();
</script>
</body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site-a)/index.html
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
  agent navigate "$SITE"
  settle 2
  # The field.
  click_page 170 35
  settle 1
  type_text "hello world"
  settle 1
  shot 489-01-typed
  for _ in 1 2 3 4 5; do
    press "" BackSpace
  done
  press CTRL a
  type_text "replaced"
  press "" Home
  type_text ">"
  press "" Return
  settle 1
  shot 489-02-edited
  # A compose sequence in the textarea: the Compose key, then ' and e.
  click_page 170 100
  settle 1
  press_keys Multi_key apostrophe e
  settle 1
  shot 489-03-composed
  # A click, a double click and a right click on the button.
  click_page 80 170
  settle 1
  click_page 80 170
  click_page 80 170
  settle 1
  click_page 80 170 right
  settle 1
  shot 489-04-clicks
  # The cross-site iframe's field.
  click_page 100 269
  settle 1
  type_text "in the frame"
  settle 1
  shot 489-06-iframe
  # The clipboard, both ways.
  local clipboard
  WAYLAND_DISPLAY=$SWAY_DISPLAY wl-copy --foreground "from the clipboard" &
  clipboard=$!
  click_page 170 100
  settle 1
  press "" End
  press CTRL v
  settle 1
  shot 489-07-pasted
  click_page 35 212
  click_page 35 212
  settle 1
  press CTRL c
  settle 2
  echo "the clipboard after Ctrl+C: $(WAYLAND_DISPLAY=$SWAY_DISPLAY wl-paste -n)"
  # wl-copy ends when another owner takes the clipboard; this stops it if none did.
  kill "$clipboard" 2>/dev/null || true
  # A Super chord the page must not see: the key log still ends with Ctrl+C's c.
  press SUPER x
  settle 1
  shot 489-08-super-filtered
  # Five wheel detents down over the page.
  pointer_to 800 600
  settle 1
  scroll 5
  settle 1
  shot 489-05-scrolled
  grep -oE 'input to frame: [0-9.]+' "$E2E_PROFILE/logs/Marley.log" | awk '{ print $4 }' | sort -n \
    | awk '{ ms[NR] = $1 } END {
        if (NR == 0) { print "latency: no input reached a frame"; exit }
        p95 = int(NR * 0.95 + 0.5)
        if (p95 < 1) p95 = 1
        printf "latency: %d inputs to the next frame, median %.1f ms, 95th percentile %.1f ms\n",
          NR, ms[int((NR + 1) / 2)], ms[p95] }'
}
