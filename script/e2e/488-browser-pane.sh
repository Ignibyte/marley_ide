# shellcheck shell=bash
# #488's e2e test: the Browser tab. `marley: open browser` starts the run's own Chromium as a
# transient user unit and shows its page. A stand-in agent attached to the same Chromium
# navigates it to a fixture page with a cross-site iframe and highlights its button, and the
# tab shows both. The page is laid out again when the tab widens, and a closed and reopened tab
# shows the same page from the same Chromium.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

setup() {
  local port_b
  mkdir -p "$E2E_WORK/site-a" "$E2E_WORK/site-b"
  cat >"$E2E_WORK/site-b/frame.html" <<'HTML'
<!doctype html><html><body style="margin:0;padding:8px;background:#dff0d8;font:16px sans-serif">
<p>A cross-site frame, from localhost</p></body></html>
HTML
  port_b=$(serve_site site-b)
  cat >"$E2E_WORK/site-a/index.html" <<HTML
<!doctype html><html><head><title>Marley fixture</title></head>
<body style="margin:0;padding:16px;font:18px sans-serif;background:#fff">
<h1>Marley fixture</h1>
<p>The page is <b id="size" style="font:bold 24px monospace"></b> CSS pixels.</p>
<button id="button" style="font-size:18px">A button</button>
<iframe src="http://localhost:$port_b/frame.html"
  style="display:block;margin-top:16px;width:420px;height:90px;border:2px solid #888"></iframe>
<script>
const size = () => document.getElementById('size').textContent = innerWidth + ' x ' + innerHeight;
size();
addEventListener('resize', size);
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

steps() {
  local port highlighter
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  palette "marley: open browser"
  settle 1
  shot 488-01-starting
  settle 4
  port=$(head -1 "$(browser_profile)/DevToolsActivePort")
  echo "unit $(browser_unit): $(systemctl --user is-active "$(browser_unit)")"
  systemctl --user show -p ExecStart --value "$(browser_unit)" | grep -oE 'path=[^ ;]+|--[a-z-]+(=[^ ;]*)?' | tr '\n' ' '
  echo
  echo "listening: $(ss -ltnH "sport = :$port" | awk '{print $4}' | tr '\n' ' ')"
  agent navigate "$SITE"
  settle 2
  shot 488-02-page
  agent highlight '#button' 6 &
  highlighter=$!
  settle 3
  shot 488-03-highlight
  wait "$highlighter"
  palette "workspace: toggle right dock"
  settle 2
  shot 488-04-wider
  press CTRL w
  settle 1
  palette "marley: open browser"
  settle 3
  shot 488-05-reopened
  echo "units: $(systemctl --user list-units --plain --no-legend 'marley-browser-*' | wc -l) running; this run's is $(systemctl --user is-active "$(browser_unit)")"
}
