# shellcheck shell=bash
# #492's e2e test: an agent sees and drives the Browser tab through Marley's MCP server. A
# stand-in agent, run by the harness through the Claude Code plugin's bridge, navigates while no
# Browser tab is open (one opens), looks at the page and saves the frame it gets, takes a
# snapshot (a cross-site iframe's field included), reads the console and the network (a token in
# a query hidden), types into the sign-in form and the iframe's field by ref, clicks Sign in, is
# refused `file:` and `javascript:` URLs, scrolls, and lists the tools. The page reports each
# key and click it gets, and whether the browser marked it trusted. Chromium runs offline.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

setup() {
  local port_b
  offline_chromium
  mkdir -p "$E2E_WORK/site-a" "$E2E_WORK/site-b"
  cat >"$E2E_WORK/site-b/frame.html" <<'HTML'
<!doctype html><html><body style="margin:0;padding:10px;background:#dff0d8;font:16px sans-serif">
<input id="inner" aria-label="the frame's field" style="width:220px;font-size:16px">
<span id="got"></span>
<script>
document.getElementById('inner').addEventListener('input', (event) =>
  document.getElementById('got').textContent = 'the frame has: ' + event.target.value);
</script>
</body></html>
HTML
  port_b=$(serve_site site-b)
  cat >"$E2E_WORK/site-a/index.html" <<HTML
<!doctype html><html><head><title>Sign in</title><style>
body { margin: 0; padding: 20px; font: 18px sans-serif; height: 2000px }
input, button { font-size: 18px }
#report { position: fixed; right: 10px; top: 10px; width: 430px; white-space: pre;
  font: 14px monospace; background: #eef; padding: 6px }
</style></head><body>
<h1>Sign in</h1>
<form id="form">
  <p><label>Email <input id="email" type="email" style="width:260px"></label></p>
  <p><label>Password <input id="password" type="password" style="width:260px"></label></p>
  <p><button id="submit" type="submit">Sign in</button></p>
</form>
<p id="status">Not signed in.</p>
<iframe src="http://localhost:$port_b/frame.html"
  style="width:420px;height:80px;border:2px solid #888"></iframe>
<div id="report"></div>
<script>
let keys = 0, trustedKeys = 0;
const clicks = [];
const render = () => document.getElementById('report').textContent =
  'keys: ' + keys + ', trusted: ' + trustedKeys + '\\n' + clicks.join('\\n');
addEventListener('keydown', (event) => { keys++; if (event.isTrusted) trustedKeys++; render(); }, true);
addEventListener('click', (event) => {
  clicks.push('click on ' + (event.target.id || event.target.tagName) + ', trusted ' + event.isTrusted);
  render();
}, true);
document.getElementById('form').addEventListener('submit', (event) => {
  event.preventDefault();
  document.getElementById('status').textContent =
    'Signed in as ' + document.getElementById('email').value + '.';
});
console.log('hello from the page');
console.warn('a warning from the page');
setTimeout(() => { throw new Error('an uncaught error'); }, 0);
fetch('/api?token=abc123&page=2').catch(() => {});
render();
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

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  echo "== navigate, with no Browser tab open"
  mcp_agent navigate "$SITE/index.html"
  settle 3
  shot 492-01-opened-and-navigated
  echo "== look"
  mcp_agent look "$(shot_file 492-look.jpg)"
  echo "== snapshot"
  mcp_agent snapshot
  echo "== console"
  mcp_agent console
  echo "== network"
  mcp_agent network
  echo "== type and click by ref"
  mcp_agent type-into textbox "Email" "agent@example.com"
  mcp_agent type-into textbox "the frame's field" "typed by the agent"
  mcp_agent click-on button "Sign in"
  settle 1
  shot 492-02-typed-and-clicked
  settle 6
  shot 492-03-chip-gone
  echo "== refused schemes"
  mcp_agent navigate "file:///etc/passwd"
  mcp_agent navigate "javascript:alert(1)"
  echo "== scroll, then look again"
  mcp_agent scroll 300
  mcp_agent look "$E2E_WORK/after-scroll.jpg"
  echo "== tools"
  mcp_agent tools
}
