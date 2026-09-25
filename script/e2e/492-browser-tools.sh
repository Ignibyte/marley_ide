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

# The plugin's bridge, the file Marley's marketplace ships.
BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge

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
  write_agent_client
  git init -q -b browser "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

# The stand-in agent: one call through the bridge, its answer printed.
agent_mcp() {
  BRIDGE=$BRIDGE MARLEY_MCP_ENDPOINT=$E2E_PROFILE/mcp-endpoint.json \
    python3 "$E2E_WORK/agent-mcp.py" "$@"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  echo "== navigate, with no Browser tab open"
  agent_mcp navigate "$SITE/index.html"
  settle 3
  shot 492-01-opened-and-navigated
  echo "== look"
  agent_mcp look "$(shot_file 492-look.jpg)"
  echo "== snapshot"
  agent_mcp snapshot
  echo "== console"
  agent_mcp console
  echo "== network"
  agent_mcp network
  echo "== type and click by ref"
  agent_mcp type-into textbox "Email" "agent@example.com"
  agent_mcp type-into textbox "the frame's field" "typed by the agent"
  agent_mcp click-on button "Sign in"
  settle 1
  shot 492-02-typed-and-clicked
  settle 6
  shot 492-03-chip-gone
  echo "== refused schemes"
  agent_mcp navigate "file:///etc/passwd"
  agent_mcp navigate "javascript:alert(1)"
  echo "== scroll, then look again"
  agent_mcp scroll 300
  agent_mcp look "$E2E_WORK/after-scroll.jpg"
  echo "== tools"
  agent_mcp tools
}

write_agent_client() {
  cat >"$E2E_WORK/agent-mcp.py" <<'PY'
# A stand-in agent for #492's e2e test: it runs the plugin's bridge, as Claude Code does, and
# calls Marley's browser tools, printing what comes back.
import base64
import json
import os
import queue
import re
import subprocess
import sys
import threading
import time


class Client:
    def __init__(self):
        self.bridge = subprocess.Popen(
            [os.environ["BRIDGE"]],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
        )
        self.messages = queue.Queue()
        self.next_id = 0
        threading.Thread(target=self.pump, daemon=True).start()
        self.call(
            "initialize",
            {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "e2e", "version": "0"}},
        )
        self.send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def pump(self):
        for line in self.bridge.stdout:
            if line.strip():
                self.messages.put(json.loads(line))
        self.messages.put(None)

    def send(self, message):
        self.bridge.stdin.write(json.dumps(message) + "\n")
        self.bridge.stdin.flush()

    def call(self, method, params=None, seconds=60):
        self.next_id += 1
        self.send({"jsonrpc": "2.0", "id": self.next_id, "method": method, "params": params or {}})
        deadline = time.monotonic() + seconds
        while (left := deadline - time.monotonic()) > 0:
            try:
                message = self.messages.get(timeout=left)
            except queue.Empty:
                break
            if message is None:
                sys.exit(f"{method}: the bridge closed")
            if message.get("id") == self.next_id:
                return message
        sys.exit(f"{method}: no answer")

    def tool(self, name, arguments=None):
        """The tool's result, or its error printed; None when it failed."""
        reply = self.call("tools/call", {"name": name, "arguments": arguments or {}})
        result = reply.get("result") or {}
        if "error" in reply:
            print(f"  {name} refused: {reply['error'].get('message')}")
            return None
        if result.get("isError"):
            print(f"  {name} refused: {result['content'][0]['text']}")
            return None
        return result

    def close(self):
        self.bridge.stdin.close()
        self.bridge.wait(timeout=10)


def find_ref(snapshot, role, name):
    for line in snapshot.splitlines():
        match = re.match(r'\s*- (\S+)(?: "(.*?)")?.*\[ref=(e\d+)\]', line)
        if match and match.group(1) == role and (match.group(2) or "").startswith(name):
            return match.group(3)
    sys.exit(f"no {role} {name!r} in the snapshot")


def main():
    command, *rest = sys.argv[1:]
    client = Client()
    if command == "tools":
        tools = client.call("tools/list")["result"]["tools"]
        names = [tool["name"] for tool in tools]
        print(f"  {len(names)} tools: {', '.join(names)}")
        evaluating = [name for name in names if "eval" in name or "script" in name]
        print(f"  tools that evaluate script: {evaluating or 'none'}")
    elif command == "navigate":
        result = client.tool("browser_navigate", {"url": rest[0]})
        if result:
            print(f"  {json.dumps(result['structuredContent'])}")
    elif command == "look":
        result = client.tool("browser_look")
        if result:
            print(f"  {json.dumps(result['structuredContent'])}")
            images = [block for block in result["content"] if block["type"] == "image"]
            with open(rest[0], "wb") as file:
                file.write(base64.b64decode(images[0]["data"]))
            print(f"  the frame: {images[0]['mimeType']}, saved as {os.path.basename(rest[0])}")
    elif command == "snapshot":
        result = client.tool("browser_snapshot")
        if result:
            print(result["content"][0]["text"], end="")
    elif command in ("console", "network"):
        result = client.tool(f"browser_{command}")
        for entry in (result or {}).get("structuredContent", {}).get("entries", []):
            if command == "console":
                print(f"  {entry['level']}: {entry['text']} ({entry.get('source')}:{entry.get('line')})")
            else:
                print(f"  {entry['method']} {entry.get('status')} {entry.get('kind')} {entry['url']}")
    elif command in ("type-into", "click-on"):
        role, name = rest[0], rest[1]
        snapshot = client.tool("browser_snapshot")["content"][0]["text"]
        reference = find_ref(snapshot, role, name)
        if command == "type-into":
            result = client.tool("browser_type", {"ref": reference, "text": rest[2]})
        else:
            result = client.tool("browser_click", {"ref": reference})
        if result:
            print(f"  {reference}: {result['structuredContent']['did']}")
    elif command == "scroll":
        result = client.tool("browser_scroll", {"dy": float(rest[0])})
        if result:
            print(f"  {result['structuredContent']['did']}")
    client.close()


main()
PY
}
