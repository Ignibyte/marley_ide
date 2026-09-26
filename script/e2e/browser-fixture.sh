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
#   CDP client can, through the `DevToolsActivePort` in Marley's profile, and drives the browser's
#   first page: `navigate <url>`, and `highlight <selector> <seconds>`, which keeps the
#   highlight, drawn for its own session, for that long; `close <text>` closes the page whose URL
#   holds the text.
# - `mcp_agent <command> ...` runs a stand-in agent that reaches Marley's MCP server through the
#   Claude Code plugin's bridge, as Claude Code in a terminal does, and calls the browser tools
#   (#492): `tools`, `tabs`, `navigate <url>`, `look [<image file>]`, `snapshot [full]`, `console`,
#   `network`, `type-into <role> <name> <text>`, `click-on <role> <name>` and `scroll <dy>`.
#   `--tab <id>` names the tab a tool acts on, and `--new-tab` has `navigate` open one (#493).
#   `picks` lists the user's picks and `pick <id> [<image file>]` reads one, saving its crop
#   (#496). `annotate <role> <name> <note>` draws the agent's box around an element,
#   `annotations` lists a tab's boxes, and `annotate-clear` removes the agent's (#498).
#   `recordings` lists the saved recordings, and `recording <id> [<frame> <image file>]` prints
#   one's timeline and saves a frame (#499). `terminal-read <text>` reads the newest block, in
#   any terminal, whose command holds the text: its command, the redaction counts and its
#   output (#516). `terminals` lists every terminal: its id, title, project and folder (#513).
# - `browser_profile` and `browser_unit` name the run's Chromium profile and its user unit;
#   `browser_teardown`, for the scenario's `teardown`, stops the unit and the servers.

browser_profile() {
  printf '%s/browser/profile' "$(realpath "$E2E_PROFILE")"
}

# The unit's name, as `marley_browser::service::unit_name` makes it.
browser_unit() {
  printf 'marley-browser-%s' "$(printf '%s' "$(browser_profile)" | sha256sum | cut -c1-12)"
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
  node "$E2E_WORK/agent.mjs" "$(browser_profile)" "$@"
}

mcp_agent() {
  [[ -f $E2E_WORK/mcp-agent.py ]] || write_mcp_agent
  BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge \
    MARLEY_MCP_ENDPOINT=$E2E_PROFILE/mcp-endpoint.json \
    python3 "$E2E_WORK/mcp-agent.py" "$@"
}

browser_teardown() {
  systemctl --user stop "$(browser_unit)" 2>/dev/null || true
  if [[ -f $E2E_WORK/servers ]]; then
    xargs kill <"$E2E_WORK/servers" 2>/dev/null || true
  fi
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
// A stand-in agent for the browser's e2e scenarios: a CDP client of the run's Chromium.
import { readFileSync } from 'node:fs';

const [, , profile, command, ...args] = process.argv;
const [port, path] = readFileSync(`${profile}/DevToolsActivePort`, 'utf8').trim().split('\n');
const socket = new WebSocket(`ws://127.0.0.1:${port}${path}`);
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
            print(f"  tab {tab['id']}: {tab['title']!r} at {tab['url']}{focused}")
    elif command == "navigate":
        result = client.tool("browser_navigate", {"url": rest[0], **options})
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
    elif command == "terminals":
        result = client.tool("terminal_list")
        for terminal in (result or {}).get("structuredContent", {}).get("terminals", []):
            print(f"  terminal {terminal['id']}: {terminal['title']!r}, project {terminal['project']}, in {terminal['cwd']}")
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
