# shellcheck shell=bash
# Fixtures for the browser's e2e scenarios (#488 on), sourced by a scenario at its top level.
#
# - `serve_site <dir>` serves $E2E_WORK/<dir> on 127.0.0.1 at a free port and prints the port.
#   A page served at 127.0.0.1 that embeds one served at `localhost` embeds another site, so
#   Chromium draws it as a cross-site iframe, in a process of its own. Each site also answers
#   `/slow`, a page that takes three seconds to come.
# - `offline_chromium`, called in `setup`, has Marley start a Chromium that reaches no host but
#   `localhost` and 127.0.0.1 (its resolver rules map IP literals too): a search or a typed
#   name fails in the page, and nothing leaves the machine.
# - `agent <command> ...` runs a stand-in agent that attaches to the run's Chromium the way any
#   CDP client can: through the relay's WebSocket with the token in `relay.json` beside the
#   profile of the project `$E2E_WORK/repo` (#583), or over the `DevToolsActivePort` of a
#   Chromium an earlier build started. It drives the browser's first page: `navigate <url>`, and
#   `highlight <selector> <seconds>`, which keeps the highlight, drawn for its own session, for
#   that long; `close <text>` closes the page whose URL holds the text. `hold-session <seconds>`
#   attaches and prints its page session, `use-session <id>` calls on a session it does not own,
#   and `close-browser` sends `Browser.close` (#583).
# - `mcp_agent <command> ...` runs a stand-in agent that reaches Marley's MCP server through the
#   Claude Code plugin's bridge, as Claude Code in a terminal does, and calls the browser tools
#   (#492): `tools`, `tabs`, `navigate <url>`, `look [<image file>]`, `snapshot [full]`, `console`,
#   `network`, `type-into <role> <name> <text>`, `click-on <role> <name>` and `scroll <dy>`.
#   `--tab <id>` names the tab a tool acts on, and `--new-tab` has `navigate` open one (#493);
#   `tabs` gives each tab's project and marks `default` the one a call naming no tab acts on for
#   this caller (#574).
#   `picks` lists the user's picks and `pick <id> [<image file>]` reads one, saving its crop
#   (#496), with its HTML, styles, sibling texts, selection and React component since #518;
#   `pick-json <id> <file>` saves the whole answer as JSON (#518). `check-pick <id> [<image file>
#   [<json file>]]` checks a pick (#505): what found it again, each change and the box now,
#   with the new crop and the answer saved when named. `draft-test <id> [<json file>]` drafts
#   a Playwright test from a recording (#506) and prints it, saving the answer when named. `annotate <role> <name> <note>` draws the agent's box around an element,
#   `annotations` lists a tab's boxes, and `annotate-clear` removes the agent's (#498).
#   `recordings` lists the saved recordings, and `recording <id> [<frame> <image file>]` prints
#   one's timeline and saves a frame (#499). `terminal-read <text>` reads the newest block, in
#   any terminal, whose command holds the text: its command, the redaction counts and its
#   output (#516). `terminals` lists every terminal: its id, title, project and folder (#513).
#   `blocks` lists every terminal's blocks: command, exit code, whether it runs and whether its
#   output is still kept (#546); `blocks-here` lists the blocks of the terminal the agent runs in,
#   naming none (#520), and `terminals` marks that one `(self)` and gives each `terminal_id`. `fleet` lists `fleet_snapshot`'s seats: each one's id, state and
#   the labels an agent row shows, or `no seats` (#547). `open-url <url> <directory>` asks
#   `browser_open_url` to open a URL for a program in that folder (#561). `--endpoint <file>`,
#   first, points the bridge at another endpoint file, an outside client's (#524).
# - `mcp_http <endpoint file> <command> ...` talks to Marley's MCP server directly with the file's
#   URL and token, printing statuses and messages and never the token (#524): `initialize`,
#   `call <tool> [<json arguments>]`, `resources-read`, `get` (a standing stream), `sessions <n>`
#   (opens n sessions, then uses the first and the last again), `long-header` (a request with a
#   16 KiB header line) and `silent` (a connection that sends half a request and waits).
# - `write_login_site <dir>` writes a site that keeps a login three ways into `$E2E_WORK/<dir>`
#   (#507, #581): `signin.html?as=<name>` keeps it as a cookie that outlives the browser, in
#   `localStorage` and in an IndexedDB record, then goes to `whoami.html`, which shows all three
#   and puts them in its title, `whoami: cookie=… local=… idb=…`, which `browser_tabs` reads.
# - `browser_profile [root]` and `browser_unit [root]` name the Chromium profile and the user
#   unit of the project whose one folder is `root`, `$E2E_WORK/repo` by default: a Chromium per
#   project since #507. `browser_close [root]` asks that project's Chromium to close over CDP (the
#   relay's WebSocket with its token since #583, else its port), as
#   Marley does before it stops a unit, and waits up to five seconds for the unit to stop: stopped
#   by a signal, Chromium loses the cookies it has not written yet. `browser_teardown`, for the
#   scenario's `teardown`, stops every unit of the run, the profile of earlier builds' too, and the
#   servers.

# The folder of the project whose one main folder is `root`, keyed as
# `marley_browser::service::project_key` keys it (#507).
browser_project_dir() {
  local root
  root=$(realpath -m "${1:-$E2E_WORK/repo}")
  printf '%s/browser/projects/%s' "$(realpath "$E2E_PROFILE")" \
    "$(printf '%s\n' "$root" | sha256sum | cut -c1-16)"
}

browser_profile() {
  printf '%s/profile' "$(browser_project_dir "$@")"
}

# The unit of the Chromium on profile `profile`, as `marley_browser::service::unit_name` names it.
browser_unit_of() {
  printf 'marley-browser-%s' "$(printf '%s' "$1" | sha256sum | cut -c1-12)"
}

browser_unit() {
  browser_unit_of "$(browser_profile "$@")"
}

browser_close() {
  local profile unit
  profile=$(browser_profile "$@")
  unit=$(browser_unit_of "$profile")
  node --input-type=module -e '
    import { existsSync, readFileSync } from "node:fs";
    const relayFile = process.argv[1] + "/../relay.json";
    let socket;
    if (existsSync(relayFile)) {
      const { url, token } = JSON.parse(readFileSync(relayFile, "utf8"));
      socket = new WebSocket(url, { headers: { Authorization: "Bearer " + token } });
    } else {
      const [port, path] = readFileSync(process.argv[1] + "/DevToolsActivePort", "utf8").trim().split("\n");
      socket = new WebSocket("ws://127.0.0.1:" + port + path);
    }
    socket.onopen = () => socket.send(JSON.stringify({ id: 1, method: "Browser.close" }));
    socket.onclose = () => process.exit(0);
    setTimeout(() => process.exit(0), 5000);
  ' "$profile" 2>/dev/null || true
  for _ in $(seq 50); do
    systemctl --user is-active --quiet "$unit" || return 0
    sleep 0.1
  done
  return 1
}

serve_site() {
  local site=$1 port=
  [[ -f $E2E_WORK/serve.py ]] || write_server
  python3 -u "$E2E_WORK/serve.py" "$E2E_WORK/$site" >"$E2E_WORK/$site.log" 2>&1 &
  echo "$!" >>"$E2E_WORK/servers"
  for _ in $(seq 50); do
    port=$(grep -oE 'port [0-9]+' "$E2E_WORK/$site.log" | head -1 | cut -d' ' -f2)
    [[ -n $port ]] && break
    sleep 0.1
  done
  [[ -n $port ]] || { echo "serve_site: $site did not start" >&2; return 1; }
  echo "$port"
}

offline_chromium() {
  local binary=
  for binary in /usr/lib/chromium/chromium "$(command -v chromium)" "$(command -v chromium-browser)"; do
    [[ -x $binary ]] && break
  done
  [[ -x $binary ]] || { echo "offline_chromium: no Chromium" >&2; return 1; }
  cat >"$E2E_WORK/chromium" <<SH
#!/bin/sh
exec "$binary" --host-resolver-rules="MAP * ~NOTFOUND, EXCLUDE localhost, EXCLUDE 127.0.0.1" "\$@"
SH
  chmod +x "$E2E_WORK/chromium"
  export MARLEY_CHROMIUM=$E2E_WORK/chromium
}

agent() {
  [[ -f $E2E_WORK/agent.mjs ]] || write_agent
  node "$E2E_WORK/agent.mjs" "$(browser_profile "$E2E_WORK/repo")" "$@"
}

mcp_agent() {
  local endpoint=$E2E_PROFILE/mcp-endpoint.json
  if [[ ${1:-} == --endpoint ]]; then
    endpoint=$2
    shift 2
  fi
  [[ -f $E2E_WORK/mcp-agent.py ]] || write_mcp_agent
  BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge \
    MARLEY_MCP_ENDPOINT=$endpoint \
    python3 "$E2E_WORK/mcp-agent.py" "$@"
}

mcp_http() {
  [[ -f $E2E_WORK/mcp-http.py ]] || write_mcp_http
  python3 "$E2E_WORK/mcp-http.py" "$@"
}

write_mcp_http() {
  cat >"$E2E_WORK/mcp-http.py" <<'PY'
"""Talks to Marley's MCP server with an endpoint file's URL and token, printing statuses and
messages and never a token (#524)."""

import json
import socket
import sys
import time
import urllib.parse


def endpoint(path):
    with open(path) as file:
        entry = json.load(file)
    parsed = urllib.parse.urlsplit(entry["url"])
    return parsed.hostname, parsed.port, parsed.path, entry["headers"]["Authorization"]


def exchange(target, method, body=None, session=None, extra=""):
    """One request on a fresh connection: the status, the headers and the body."""
    host, port, path, authorization = target
    data = json.dumps(body).encode() if body is not None else b""
    head = (
        f"{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nAuthorization: {authorization}\r\n"
        "Accept: application/json, text/event-stream\r\nContent-Type: application/json\r\n"
        f"Content-Length: {len(data)}\r\n"
    )
    if session:
        head += f"Mcp-Session-Id: {session}\r\n"
    head += extra + "\r\n"
    with socket.create_connection((host, port), timeout=15) as connection:
        connection.sendall(head.encode() + data)
        if method == "GET":
            connection.settimeout(2)
        reply = b""
        try:
            while True:
                chunk = connection.recv(65536)
                if not chunk:
                    break
                reply += chunk
        except socket.timeout:
            pass
    text = reply.decode("utf-8", "replace")
    status_line, _, rest = text.partition("\r\n")
    headers, _, payload = rest.partition("\r\n\r\n")
    status = status_line.split(" ", 2)[1] if " " in status_line else "none"
    fields = {}
    for line in headers.split("\r\n"):
        name, _, value = line.partition(":")
        fields[name.strip().lower()] = value.strip()
    messages = []
    for line in payload.splitlines():
        if line.startswith("data: "):
            messages.append(json.loads(line[6:]))
    return status, fields, messages


INITIALIZE = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "e2e", "version": "1"}},
}


def open_session(target):
    status, fields, _ = exchange(target, "POST", INITIALIZE)
    session = fields.get("mcp-session-id")
    if session:
        exchange(target, "POST", {"jsonrpc": "2.0", "method": "notifications/initialized"}, session)
    return status, session


def main():
    target = endpoint(sys.argv[1])
    command = sys.argv[2]
    if command == "initialize":
        status, session = open_session(target)
        print(f"  initialize: {status}{', a session' if session else ''}")
    elif command == "call":
        status, session = open_session(target)
        if not session:
            print(f"  initialize: {status}")
            return
        arguments = json.loads(sys.argv[4]) if len(sys.argv) > 4 else {}
        body = {"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": sys.argv[3], "arguments": arguments}}
        status, _, messages = exchange(target, "POST", body, session)
        for message in messages:
            if "error" in message:
                print(f"  {sys.argv[3]}: error {message['error'].get('code')}: {message['error'].get('message')}")
            else:
                result = message.get("result", {})
                text = (result.get("content") or [{}])[0].get("text", "")
                print(f"  {sys.argv[3]}: {'refused' if result.get('isError') else 'answered'}: {text[:200]}")
    elif command == "resources-read":
        status, session = open_session(target)
        body = {"jsonrpc": "2.0", "id": 3, "method": "resources/read", "params": {"uri": "fleet://snapshot"}}
        status, _, messages = exchange(target, "POST", body, session)
        for message in messages:
            if "error" in message:
                print(f"  resources/read: error {message['error'].get('code')}: {message['error'].get('message')}")
            else:
                print("  resources/read: answered")
    elif command == "get":
        status, session = open_session(target)
        status, _, _ = exchange(target, "GET", session=session)
        print(f"  GET: {status}")
    elif command == "sessions":
        count = int(sys.argv[3])
        sessions = []
        for index in range(count):
            status, session = open_session(target)
            print(f"  session {index + 1}: {status}")
            sessions.append(session)
            time.sleep(0.05)
        body = {"jsonrpc": "2.0", "id": 4, "method": "tools/list"}
        for index in (0, count - 1):
            status, _, _ = exchange(target, "POST", body, sessions[index])
            print(f"  session {index + 1} used again: {status}")
    elif command == "long-header":
        status, _, _ = exchange(target, "POST", INITIALIZE, extra="X-Filler: " + "a" * 16384 + "\r\n")
        print(f"  a 16 KiB header line: {status}")
    elif command == "silent":
        host, port, _, _ = target
        started = time.monotonic()
        with socket.create_connection((host, port), timeout=30) as connection:
            connection.sendall(b"POST /mcp HTTP/1.1\r\n")
            try:
                closed = connection.recv(1) == b""
            except (ConnectionResetError, socket.timeout):
                closed = True
        print(f"  a connection that sent half a request: {'closed' if closed else 'open'} after {round(time.monotonic() - started)} s")


if __name__ == "__main__":
    main()
PY
}

browser_teardown() {
  local data project
  data=$(realpath "$E2E_PROFILE")
  for project in "$data"/browser/projects/*/; do
    [[ -d $project ]] || continue
    systemctl --user stop "$(browser_unit_of "${project%/}/profile")" 2>/dev/null || true
  done
  systemctl --user stop "$(browser_unit_of "$data/browser/profile")" 2>/dev/null || true
  if [[ -f $E2E_WORK/servers ]]; then
    xargs kill <"$E2E_WORK/servers" 2>/dev/null || true
  fi
}

write_login_site() {
  cat >"$E2E_WORK/$1/signin.html" <<'HTML'
<!doctype html><html><head><title>Signing in</title></head>
<body style="font:28px sans-serif;margin:40px">Signing in…
<script>
const name = new URLSearchParams(location.search).get('as') || 'nobody';
document.cookie = 'login=' + name + '; Max-Age=86400; path=/; SameSite=Lax';
localStorage.setItem('login', name);
const opening = indexedDB.open('marley-507', 1);
opening.onupgradeneeded = () => opening.result.createObjectStore('login');
opening.onsuccess = () => {
  const transaction = opening.result.transaction('login', 'readwrite');
  transaction.objectStore('login').put(name, 'name');
  transaction.oncomplete = () => location.replace('whoami.html');
};
</script></body></html>
HTML
  cat >"$E2E_WORK/$1/whoami.html" <<'HTML'
<!doctype html><html><head><title>whoami</title></head>
<body style="margin:0;font:34px sans-serif;background:#f4f1ea">
<h1 style="margin:40px 40px 24px">Who is signed in</h1>
<p style="margin:0 40px 12px">cookie: <b id="cookie">…</b></p>
<p style="margin:0 40px 12px">localStorage: <b id="local">…</b></p>
<p style="margin:0 40px 12px">IndexedDB: <b id="idb">…</b></p>
<script>
const cookie = (document.cookie.match(/(?:^|; )login=([^;]*)/) || [])[1] || 'none';
const local = localStorage.getItem('login') || 'none';
const show = (idb) => {
  document.getElementById('cookie').textContent = cookie;
  document.getElementById('local').textContent = local;
  document.getElementById('idb').textContent = idb;
  document.title = 'whoami: cookie=' + cookie + ' local=' + local + ' idb=' + idb;
};
const opening = indexedDB.open('marley-507', 1);
opening.onupgradeneeded = () => opening.result.createObjectStore('login');
opening.onerror = () => show('error');
opening.onsuccess = () => {
  const reading = opening.result.transaction('login').objectStore('login').get('name');
  reading.onsuccess = () => show(reading.result || 'none');
  reading.onerror = () => show('error');
};
</script></body></html>
HTML
}

write_server() {
  cat >"$E2E_WORK/serve.py" <<'PY'
# A loopback server for the browser's e2e scenarios: a directory's files, and /slow, a page that
# answers after three seconds.
import http.server
import sys
import time

directory = sys.argv[1]


class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=directory, **kwargs)

    def do_GET(self):
        if self.path.split('?')[0] != '/slow':
            return super().do_GET()
        time.sleep(3)
        body = b'<!doctype html><title>Slow page</title><p>This page took three seconds.</p>'
        try:
            self.send_response(200)
            self.send_header('Content-Type', 'text/html')
            self.send_header('Content-Length', str(len(body)))
            self.end_headers()
            self.wfile.write(body)
        except (BrokenPipeError, ConnectionResetError):
            # The browser stopped waiting.
            pass


server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
print(f'port {server.server_address[1]}', flush=True)
server.serve_forever()
PY
}

write_agent() {
  cat >"$E2E_WORK/agent.mjs" <<'JS'
// A stand-in agent for the browser's e2e scenarios: a CDP client of the run's Chromium, through
// its relay's WebSocket with the relay's token (#583), or over the port a Chromium an earlier
// build started listens on.
import { existsSync, readFileSync } from 'node:fs';

const [, , profile, command, ...args] = process.argv;
const relayFile = `${profile}/../relay.json`;
let socket;
if (existsSync(relayFile)) {
  const { url, token } = JSON.parse(readFileSync(relayFile, 'utf8'));
  socket = new WebSocket(url, { headers: { Authorization: `Bearer ${token}` } });
} else {
  const [port, path] = readFileSync(`${profile}/DevToolsActivePort`, 'utf8').trim().split('\n');
  socket = new WebSocket(`ws://127.0.0.1:${port}${path}`);
}
let nextId = 0;
const waiting = new Map();
const listeners = [];
socket.onmessage = (message) => {
  const data = JSON.parse(message.data);
  if (data.id !== undefined && waiting.has(data.id)) {
    const { resolve, reject } = waiting.get(data.id);
    waiting.delete(data.id);
    if (data.error) reject(new Error(`${data.error.message}`));
    else resolve(data.result);
  } else if (data.method) {
    for (const listener of listeners) listener(data);
  }
};
const send = (method, params = {}, sessionId) =>
  new Promise((resolve, reject) => {
    const id = ++nextId;
    waiting.set(id, { resolve, reject });
    socket.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
  });
const once = (method) => new Promise((resolve) => listeners.push((data) => data.method === method && resolve(data)));
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
await new Promise((resolve, reject) => {
  socket.onopen = resolve;
  socket.onerror = () => reject(new Error('no DevTools socket'));
});

if (command === 'close-browser') {
  // Chromium may end the connection before it answers.
  await Promise.race([send('Browser.close').catch(() => {}), sleep(3000)]);
  console.log('agent: asked the browser to close');
  process.exit(0);
}
if (command === 'use-session') {
  try {
    await send('Runtime.evaluate', { expression: '1 + 1', returnByValue: true }, args[0]);
    console.log('agent: the other session answered');
  } catch (error) {
    console.log(`agent: refused: ${error.message}`);
  }
  socket.close();
  process.exit(0);
}
const { targetInfos } = await send('Target.getTargets');
if (command === 'close') {
  const closing = targetInfos.find((target) => target.type === 'page' && target.url.includes(args[0]));
  if (!closing) throw new Error(`agent: no page at ${args[0]}`);
  await send('Target.closeTarget', { targetId: closing.targetId });
  console.log(`agent: closed the page at ${closing.url}`);
  socket.close();
  process.exit(0);
}
const page = targetInfos.find((target) => target.type === 'page');
if (!page) throw new Error('the browser has no page');
const { sessionId } = await send('Target.attachToTarget', { targetId: page.targetId, flatten: true });

if (command === 'navigate') {
  await send('Page.enable', {}, sessionId);
  const loaded = once('Page.loadEventFired');
  await send('Page.navigate', { url: args[0] }, sessionId);
  await loaded;
  console.log(`agent: navigated to ${args[0]}`);
} else if (command === 'hold-session') {
  console.log(`agent: holding session ${sessionId}`);
  await sleep(Number(args[0] ?? 10) * 1000);
} else if (command === 'highlight') {
  await send('DOM.enable', {}, sessionId);
  await send('Overlay.enable', {}, sessionId);
  const { root } = await send('DOM.getDocument', {}, sessionId);
  const { nodeId } = await send('DOM.querySelector', { nodeId: root.nodeId, selector: args[0] }, sessionId);
  await send('Overlay.highlightNode', {
    nodeId,
    highlightConfig: {
      showInfo: true,
      contentColor: { r: 111, g: 168, b: 220, a: 0.66 },
      paddingColor: { r: 147, g: 196, b: 125, a: 0.55 },
      borderColor: { r: 255, g: 229, b: 153, a: 0.66 },
      marginColor: { r: 246, g: 178, b: 107, a: 0.66 },
    },
  }, sessionId);
  console.log(`agent: highlighted ${args[0]}`);
  await sleep(Number(args[1] ?? 5) * 1000);
} else {
  throw new Error(`agent: no command ${command}`);
}
socket.close();
JS
}

write_mcp_agent() {
  cat >"$E2E_WORK/mcp-agent.py" <<'PY'
# A stand-in agent for the browser's e2e scenarios (#492 on): it runs the plugin's bridge, as
# Claude Code does, and calls Marley's browser tools, printing what comes back.
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
    options = {}
    words = []
    given = iter(sys.argv[1:])
    for word in given:
        if word == "--tab":
            options["tab"] = next(given)
        elif word == "--new-tab":
            options["new_tab"] = True
        else:
            words.append(word)
    command, *rest = words
    client = Client()
    if command == "tools":
        tools = client.call("tools/list")["result"]["tools"]
        names = [tool["name"] for tool in tools]
        print(f"  {len(names)} tools: {', '.join(names)}")
        evaluating = [name for name in names if "eval" in name or "script" in name]
        print(f"  tools that evaluate script: {evaluating or 'none'}")
    elif command == "tabs":
        result = client.tool("browser_tabs")
        for tab in (result or {}).get("structuredContent", {}).get("tabs", []):
            focused = ", focused" if tab["focused"] else ""
            default = ", default" if tab.get("default") else ""
            print(f"  tab {tab['id']}: {tab['title']!r} at {tab['url']}, project {tab.get('project')}{focused}{default}")
    elif command == "navigate":
        result = client.tool("browser_navigate", {"url": rest[0], **options})
        if result:
            print(f"  {json.dumps(result['structuredContent'])}")
    elif command == "open-url":
        result = client.tool("browser_open_url", {"url": rest[0], "directory": rest[1]})
        if result:
            print(f"  {json.dumps(result['structuredContent'])}")
    elif command == "look":
        result = client.tool("browser_look", options)
        if result:
            print(f"  {json.dumps(result['structuredContent'])}")
            if rest:
                images = [block for block in result["content"] if block["type"] == "image"]
                with open(rest[0], "wb") as file:
                    file.write(base64.b64decode(images[0]["data"]))
                print(f"  the frame: {images[0]['mimeType']}, saved as {os.path.basename(rest[0])}")
    elif command == "snapshot":
        result = client.tool("browser_snapshot", {**options, "full": bool(rest and rest[0] == "full")})
        if result:
            print(result["content"][0]["text"], end="")
    elif command in ("console", "network"):
        result = client.tool(f"browser_{command}", options)
        for entry in (result or {}).get("structuredContent", {}).get("entries", []):
            if command == "console":
                print(f"  {entry['level']}: {entry['text']} ({entry.get('source')}:{entry.get('line')})")
            else:
                print(f"  {entry['method']} {entry.get('status')} {entry.get('kind')} {entry['url']}")
    elif command in ("type-into", "click-on"):
        role, name = rest[0], rest[1]
        snapshot = client.tool("browser_snapshot", options)["content"][0]["text"]
        reference = find_ref(snapshot, role, name)
        if command == "type-into":
            result = client.tool("browser_type", {"ref": reference, "text": rest[2], **options})
        else:
            result = client.tool("browser_click", {"ref": reference, **options})
        if result:
            print(f"  {reference}: {result['structuredContent']['did']}")
    elif command == "scroll":
        result = client.tool("browser_scroll", {"dy": float(rest[0]), **options})
        if result:
            print(f"  {result['structuredContent']['did']}")
    elif command == "annotate":
        role, name, note = rest[0], rest[1], rest[2]
        snapshot = client.tool("browser_snapshot", {"full": True, **options})["content"][0]["text"]
        reference = find_ref(snapshot, role, name)
        result = client.tool("browser_annotate", {"ref": reference, "note": note, **options})
        if result:
            answer = result["structuredContent"]
            box = answer["box"]
            print(f"  {reference}: {answer['did']}, annotation {answer['id']} at {box['x']:.0f},{box['y']:.0f} {box['width']:.0f}x{box['height']:.0f}")
    elif command == "blocks":
        listed = client.tool("terminal_list")
        for terminal in (listed or {}).get("structuredContent", {}).get("terminals", []):
            answer = (client.tool("terminal_blocks", {"terminal": terminal["id"]}) or {}).get("structuredContent", {})
            for block in answer.get("blocks", []):
                print(f"  block {block['index']}: {block['command']!r}, exit {block['exit_code']}, running {block['running']}, kept {block['output_kept']}")
    elif command == "fleet":
        answer = (client.tool("fleet_snapshot") or {}).get("structuredContent") or {}
        seats = answer.get("seats", [])
        for seat in seats:
            labels = seat.get("labels", {})
            shown = "".join(f", {key} {labels[key]!r}" for key in ("prompt", "tool", "message", "error") if key in labels)
            print(f"  seat {seat['id']}: {seat['state']}{shown}")
        if not seats:
            print("  no seats")
    elif command == "terminals":
        result = client.tool("terminal_list")
        for terminal in (result or {}).get("structuredContent", {}).get("terminals", []):
            own = " (self)" if terminal.get("self") else ""
            print(f"  terminal {terminal['id']}{own}: {terminal['title']!r}, project {terminal['project']}, "
                  f"id {terminal.get('terminal_id')}, in {terminal['cwd']}")
    elif command == "blocks-here":
        answer = (client.tool("terminal_blocks") or {}).get("structuredContent", {})
        for block in answer.get("blocks", []):
            print(f"  block {block['index']}: {block['command']!r}, exit {block['exit_code']}")
    elif command == "terminal-read":
        listed = client.tool("terminal_list")
        found = None
        for terminal in (listed or {}).get("structuredContent", {}).get("terminals", []):
            blocks = client.tool("terminal_blocks", {"terminal": terminal["id"]})
            answer = (blocks or {}).get("structuredContent", {})
            matching = [block for block in answer.get("blocks", []) if rest[0] in block["command"]]
            if matching:
                found = (terminal["id"], matching[-1], answer["redacted"])
        if found is None:
            sys.exit(f"no block's command holds {rest[0]!r}")
        terminal, block, listed_redacted = found
        result = client.tool("terminal_read", {"terminal": terminal, "block": block["index"]})
        if result:
            read = result["structuredContent"]
            print(f"  terminal {terminal}, block {block['index']}: {read['command']!r}")
            print(f"  redacted: {read['redacted']} in the read, {listed_redacted} in the list")
            print(read["output"])
    elif command == "recordings":
        result = client.tool("browser_recordings")
        recordings = (result or {}).get("structuredContent", {}).get("recordings", [])
        if result and not recordings:
            print("  no recordings")
        for recording in recordings:
            print(f"  recording {recording['id']}: {recording['title']!r} at {recording['url']}, {recording['seconds']:.1f} s, {recording['frames']} frames, {recording['entries']} entries")
    elif command == "recording":
        arguments = {"id": rest[0]}
        if len(rest) > 1:
            arguments["frame"] = int(rest[1])
        result = client.tool("browser_recording", arguments)
        if result:
            recording = result["structuredContent"]
            print(f"  recording {recording['id']}: {recording['seconds']:.1f} s, {recording['frames']} frames")
            frame_times = []
            for entry in recording["entries"]:
                kind, at = entry["kind"], entry["at_ms"]
                if kind == "frame":
                    frame_times.append(at)
                    continue
                details = {key: value for key, value in entry.items() if key not in ("kind", "at_ms")}
                if kind == "snapshot":
                    details = {"lines": len(details.get("text", "").splitlines())}
                print(f"  {at:>6} ms {kind}: {json.dumps(details)}")
            gaps = [later - earlier for earlier, later in zip(frame_times, frame_times[1:])]
            print(f"  frames: {len(frame_times)}, the least gap between two: {min(gaps) if gaps else 'none'} ms")
            print(f"  the snapshot at the save: {len(recording['snapshot'].splitlines())} lines")
            images = [block for block in result["content"] if block["type"] == "image"]
            if images and len(rest) > 2:
                with open(rest[2], "wb") as file:
                    file.write(base64.b64decode(images[0]["data"]))
                print(f"  frame {rest[1]}: {images[0]['mimeType']}, saved as {os.path.basename(rest[2])}")
    elif command == "draft-test":
        result = client.tool("browser_draft_test", {"id": rest[0]})
        if result:
            draft = result["structuredContent"]
            print(f"  draft of recording {draft['id']}: {draft['path']}")
            print(f"  starts at {draft['start']}; reads {', '.join(draft['env']) or 'no variables'}")
            for skipped in draft["skipped"]:
                print(f"  skipped: {skipped}")
            if draft.get("note"):
                print(f"  note: {draft['note']}")
            print(f"  run: {draft['run']}")
            for line in draft["test"].splitlines():
                print(f"  | {line}")
            if len(rest) > 1:
                with open(rest[1], "w") as file:
                    json.dump(draft, file, indent=1)
                print(f"  the draft: saved as {os.path.basename(rest[1])}")
    elif command == "annotate-clear":
        result = client.tool("browser_annotate", {"clear": True, **options})
        if result:
            print(f"  {result['structuredContent']['did']}")
    elif command == "annotations":
        result = client.tool("browser_annotations", options)
        annotations = (result or {}).get("structuredContent", {}).get("annotations", [])
        if result and not annotations:
            print("  no annotations")
        for annotation in annotations:
            box = annotation["box"]
            print(f"  annotation {annotation['id']} by the {annotation['maker']}: {annotation['note']!r} at {box['x']:.0f},{box['y']:.0f} {box['width']:.0f}x{box['height']:.0f}")
    elif command == "picks":
        result = client.tool("browser_picks")
        picks = (result or {}).get("structuredContent", {}).get("picks", [])
        if result and not picks:
            print("  no picks")
        for pick in picks:
            sent = f"sent with {pick['caption']!r}" if pick["sent"] else "not sent"
            print(f"  pick {pick['id']}: {pick['summary']} at {pick['url']} in tab {pick['tab']}, {sent}")
    elif command == "check-pick":
        result = client.tool("browser_check_pick", {"id": int(rest[0])})
        if result:
            check = result["structuredContent"]
            found = f"found by {check['found_by']}" if check["found"] else "not found"
            print(f"  check of pick {check['id']}: {found}")
            for change in check["changes"]:
                print(f"  change: {change}")
            bundle = check.get("bundle")
            if bundle:
                box = bundle["page_box"]
                print(f"  box now: {box['x']:.0f},{box['y']:.0f} {box['width']:.0f}x{box['height']:.0f}")
            images = [block for block in result["content"] if block["type"] == "image"]
            if not images:
                print("  no crop")
            elif len(rest) > 1:
                with open(rest[1], "wb") as file:
                    file.write(base64.b64decode(images[0]["data"]))
                print(f"  the crop: {images[0]['mimeType']}, saved as {os.path.basename(rest[1])}")
            if len(rest) > 2:
                with open(rest[2], "w") as file:
                    json.dump(check, file, indent=1)
                print(f"  the check: saved as {os.path.basename(rest[2])}")
    elif command == "pick-json":
        result = client.tool("browser_pick", {"id": int(rest[0])})
        if result:
            with open(rest[1], "w") as file:
                json.dump(result["structuredContent"], file, indent=1)
            print(f"  pick {rest[0]}: saved as {os.path.basename(rest[1])}")
    elif command == "pick":
        result = client.tool("browser_pick", {"id": int(rest[0])})
        if result:
            pick = result["structuredContent"]
            bundle = pick["bundle"]
            print(f"  pick {pick['id']}: {pick['summary']} at {pick['url']} ({pick['title']!r})")
            print(f"  tag {bundle['tag']}, role {bundle['role']}, name {bundle['name']!r}, text {bundle['text']!r}")
            for locator in bundle["locators"]:
                print(f"  locator {locator['kind']}: {locator['value']} (unique: {locator.get('unique')})")
            for listener in bundle["listeners"]:
                place = f"{listener.get('script')}:{listener['line']}:{listener['column']}"
                original = listener.get("original")
                if original:
                    place += f" -> {original.get('file')}:{original['line']}:{original['column']} (source {original['source']})"
                print(f"  listener {listener['event']} on {listener['on']}: {place}")
            print(f"  blockers: {', '.join(bundle['blockers']) or 'none'}")
            box = bundle["page_box"]
            print(f"  box in the page: {box['x']:.0f},{box['y']:.0f} {box['width']:.0f}x{box['height']:.0f}")
            html = bundle.get("html", "")
            print(f"  html: {len(html)} characters: {html[:300]!r}")
            styles = bundle.get("styles", {})
            print(f"  styles: {len(styles)}: " + "; ".join(f"{name} {value}" for name, value in styles.items()))
            nearby = bundle.get("nearby_text", [])
            print(f"  nearby texts: {len(nearby)}: {nearby[:4]!r}")
            print(f"  selected text: {bundle.get('selected_text')!r}")
            component = bundle.get("component")
            if component:
                print(f"  component chain: {' '.join('<' + name + '>' for name in component['chain'])}")
                source = component.get("source")
                if source:
                    print(f"  component source ({source['from']}): {source['source']} line {source['line']} column {source.get('column')}, file {source.get('file')}")
                else:
                    print("  component source: none")
            else:
                print("  component: none")
            check = pick.get("check")
            if check:
                found = f"found by {check['found_by']}" if check["found_by"] else "not found"
                print(f"  latest check: {found}, {len(check['changes'])} change(s)")
            images = [block for block in result["content"] if block["type"] == "image"]
            if not images:
                print("  no crop")
            elif len(rest) > 1:
                with open(rest[1], "wb") as file:
                    file.write(base64.b64decode(images[0]["data"]))
                print(f"  the crop: {images[0]['mimeType']}, saved as {os.path.basename(rest[1])}")
    client.close()


main()
PY
}
