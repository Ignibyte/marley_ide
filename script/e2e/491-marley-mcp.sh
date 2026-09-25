# shellcheck shell=bash
# #491's e2e test: Marley's MCP server, reached as Claude Code reaches it, through the plugin's
# bridge. The terminal runs four commands, then a stand-in client that speaks JSON-RPC to the
# bridge: it lists the tools and the terminal's blocks, and reads one block's output. The run log
# gets the endpoint file's mode and its URL's host (never the bearer). Then a second client,
# started from the harness, waits through Marley's quit for the bridge to say the tools changed,
# and the bridge, run once more with no Marley, answers with no tools and no error.
compositor sway

# The plugin's bridge, the file Marley's marketplace ships.
BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  write_client
  cat >"$home/.bashrc" <<RC
PS1='\$ '
mcp() { python3 "$E2E_WORK/mcp-client.py" "\$@"; }
RC
  terminal_env HOME "$home"
  terminal_env BRIDGE "$BRIDGE"
  terminal_env BRIDGE_LOG "$E2E_WORK/bridge.log"
  terminal_env MARLEY_MCP_ENDPOINT "$E2E_PROFILE/mcp-endpoint.json"
  git init -q -b mcp "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

# The stand-in client, run from the harness rather than from Marley's terminal.
client() {
  BRIDGE=$BRIDGE BRIDGE_LOG=$E2E_WORK/bridge.log MARLEY_MCP_ENDPOINT=$E2E_PROFILE/mcp-endpoint.json \
    python3 "$E2E_WORK/mcp-client.py" "$@"
}

run() {
  type_text "$1"
  press "" Return
  settle "${2:-1}"
}

steps() {
  local watcher
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  run "echo hi"
  run "false"
  run "seq 3"
  run "sleep 1" 2
  run "mcp tools" 3
  shot 491-01-tools
  run "mcp blocks" 3
  shot 491-02-blocks
  run "mcp read 'seq 3'" 3
  shot 491-03-read
  client endpoint
  client watch >"$E2E_WORK/watch.log" 2>&1 &
  watcher=$!
  settle 4
  # Ctrl+Q in a terminal goes to the shell, so Marley quits through the palette.
  press "CTRL SHIFT" p
  settle 1
  type_text "zed: quit"
  settle 1
  press "" Return
  settle 6
  wait "$watcher" || echo "the watching client failed"
  cat "$E2E_WORK/watch.log"
  if [[ -e $E2E_PROFILE/mcp-endpoint.json ]]; then
    echo "endpoint file after the quit: still there"
  else
    echo "endpoint file after the quit: removed"
  fi
  echo "with no Marley:"
  client tools
  echo "the bridges' log:"
  cat "$E2E_WORK/bridge.log"
}

write_client() {
  cat >"$E2E_WORK/mcp-client.py" <<'PY'
# A stand-in MCP client for #491's e2e test: it runs the plugin's bridge, as Claude Code does,
# and speaks JSON-RPC to it over the bridge's stdin and stdout.
import json
import os
import queue
import subprocess
import sys
import threading
import time
import urllib.parse


class Client:
    def __init__(self):
        log = open(os.environ.get("BRIDGE_LOG", os.devnull), "a")
        self.bridge = subprocess.Popen(
            [os.environ["BRIDGE"]],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=log,
            text=True,
        )
        self.messages = queue.Queue()
        self.notifications = []
        self.next_id = 0
        threading.Thread(target=self.pump, daemon=True).start()

    def pump(self):
        for line in self.bridge.stdout:
            if line.strip():
                self.messages.put(json.loads(line))
        self.messages.put(None)

    def send(self, message):
        self.bridge.stdin.write(json.dumps(message) + "\n")
        self.bridge.stdin.flush()

    def call(self, method, params=None, seconds=35):
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
            self.notifications.append(message)
        sys.exit(f"{method}: no answer")

    def wait_for(self, method, seconds):
        deadline = time.monotonic() + seconds
        while not any(note.get("method") == method for note in self.notifications):
            left = deadline - time.monotonic()
            if left <= 0:
                return False
            try:
                message = self.messages.get(timeout=left)
            except queue.Empty:
                return False
            if message is None:
                return False
            self.notifications.append(message)
        return True

    def initialize(self):
        reply = self.call(
            "initialize",
            {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "e2e", "version": "0"},
            },
        )
        self.send({"jsonrpc": "2.0", "method": "notifications/initialized"})
        return reply["result"]

    def tool(self, name, arguments=None):
        reply = self.call("tools/call", {"name": name, "arguments": arguments or {}})
        result = reply.get("result") or {}
        if result.get("isError") or "error" in reply:
            sys.exit(f"{name}: {json.dumps(reply)}")
        return result["structuredContent"]

    def own_terminal(self):
        """The terminal this client runs in: the one whose running command is `mcp ...`."""
        terminals = self.tool("terminal_list")["terminals"]
        for terminal in terminals:
            if (terminal.get("running") or "").startswith("mcp "):
                return terminal
        return terminals[0]

    def close(self):
        self.bridge.stdin.close()
        self.bridge.wait(timeout=10)


def main():
    command = sys.argv[1]
    if command == "endpoint":
        path = os.environ["MARLEY_MCP_ENDPOINT"]
        mode = os.stat(path).st_mode & 0o777
        with open(path, encoding="utf-8") as file:
            entry = json.load(file)
        url = urllib.parse.urlsplit(entry["url"])
        print(f"endpoint file: mode {mode:o}, type {entry['type']}, host {url.hostname}, path {url.path}")
        return
    client = Client()
    info = client.initialize()
    if command == "tools":
        tools = client.call("tools/list")["result"]["tools"]
        print(f"via the plugin's {os.environ['BRIDGE'].split('claude_plugin/')[-1]}")
        print(f"{info['serverInfo']['name']} lists {len(tools)} tools:")
        for tool in tools:
            print(f"  {tool['name']}")
    elif command == "blocks":
        terminal = client.own_terminal()
        print(f"terminal {terminal['id']}, {terminal['title']!r}, project {terminal['project']}:")
        print(f"  {terminal['blocks']} blocks, running {terminal['running']!r}, in {terminal['cwd']}")
        for block in client.tool("terminal_blocks", {"terminal": terminal["id"]})["blocks"]:
            state = "running" if block["running"] else f"exit {block['exit_code']}"
            where = os.path.basename(block["cwd"] or "")
            print(
                f"  #{block['index']} {block['command']!r}: {state}, {block['duration_ms']} ms,"
                f" in {where}, reported by the shell: {block['verified']}"
            )
    elif command == "read":
        terminal = client.own_terminal()
        blocks = client.tool("terminal_blocks", {"terminal": terminal["id"]})["blocks"]
        block = next(block for block in reversed(blocks) if block["command"] == sys.argv[2])
        read = client.tool("terminal_read", {"terminal": terminal["id"], "block": block["index"]})
        print(f"output of #{block['index']} {block['command']!r}, truncated {read['truncated']}:")
        print(read["output"])
    elif command == "watch":
        before = client.call("tools/list")["result"]["tools"]
        print(f"watch: {len(before)} tools while Marley runs")
        changed = client.wait_for("notifications/tools/list_changed", 30)
        print(f"watch: notifications/tools/list_changed {'came' if changed else 'did not come'}")
        after = client.call("tools/list")["result"]["tools"]
        print(f"watch: {len(after)} tools after the quit")
    client.close()


main()
PY
}
