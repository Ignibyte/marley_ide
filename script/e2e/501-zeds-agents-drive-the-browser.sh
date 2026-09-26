# shellcheck shell=bash
# #501's e2e test: Zed's own agents drive the browser through Marley's server. Marley registers
# the context server `marley`, and Zed's context server store runs it, its bridge, for the
# project, which is where the Zed Agent's Write profile takes its tools from. A stand-in
# external agent (just enough ACP, in Python, set as
# a custom agent server in the run's settings) logs the MCP servers Zed hands its new session and,
# prompted "open <url>", calls browser_navigate through the `marley` server among them: the page
# opens in a Browser tab while the thread shows its answer. Chromium runs offline.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The project's + in the rail, measured from #500's run, and how many steps down the agents'
# submenu the stand-in sits.
PLUS_X=236
PLUS_Y=96
STAND_IN_STEPS=2

setup() {
  offline_chromium
  mkdir -p "$E2E_WORK/site"
  cat >"$E2E_WORK/site/index.html" <<'HTML'
<!doctype html><html><head><title>Driven by an agent</title></head>
<body style="margin:0;font:20px sans-serif;background:#f3eefb"><h2 style="margin:40px">An external agent opened this page</h2></body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
  write_stand_in_agent
  # The stand-in joins the run's agent servers, inside the copied settings' own block when they
  # have one, so it does not shadow the user's.
  python3 - "$E2E_PROFILE/config/settings.json" "$E2E_WORK/stand-in-agent.py" "$E2E_WORK/stand-in.log" <<'PY'
import json, re, sys
path, script, log = sys.argv[1:]
entry = '"Stand-in": %s,' % json.dumps({
    "type": "custom", "command": "python3", "args": [script], "env": {"STAND_IN_LOG": log},
})
text = open(path).read()
match = re.search(r'"agent_servers"\s*:\s*\{', text)
if match:
    text = text[:match.end()] + "\n    " + entry + text[match.end():]
else:
    at = text.index("{") + 1
    text = text[:at] + '\n  "agent_servers": {' + entry + '},' + text[at:]
open(path, "w").write(text)
PY
  git init -q -b browser "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

write_stand_in_agent() {
  cat >"$E2E_WORK/stand-in-agent.py" <<'PY'
# A stand-in external agent for #501's e2e test: just enough ACP for Zed to start a session and
# send it a prompt. It logs the MCP servers Zed hands the session, and a prompt "open <url>"
# makes it call browser_navigate through the `marley` server among them.
import json
import os
import subprocess
import sys

log = open(os.environ["STAND_IN_LOG"], "a", buffering=1)
state = {"servers": []}


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def answer(request_id, result):
    send({"jsonrpc": "2.0", "id": request_id, "result": result})


def navigate(url):
    server = next((server for server in state["servers"] if server.get("name") == "marley"), None)
    if server is None:
        return "no marley server was handed to the session"
    env = dict(os.environ)
    env.update({pair["name"]: pair["value"] for pair in server.get("env", [])})
    process = subprocess.Popen(
        [server["command"], *server.get("args", [])],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, env=env,
    )

    def call(request_id, method, params):
        process.stdin.write(json.dumps({"jsonrpc": "2.0", "id": request_id, "method": method, "params": params}) + "\n")
        process.stdin.flush()
        for line in process.stdout:
            if line.strip() and json.loads(line).get("id") == request_id:
                return json.loads(line)
        return {}

    call(1, "initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "stand-in", "version": "0"}})
    process.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
    process.stdin.flush()
    reply = call(2, "tools/call", {"name": "browser_navigate", "arguments": {"url": url}})
    process.stdin.close()
    process.wait(timeout=10)
    result = reply.get("result") or {}
    return json.dumps(result.get("structuredContent") or reply)


for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method == "initialize":
        answer(request_id, {
            "protocolVersion": 1,
            "agentCapabilities": {"loadSession": False, "promptCapabilities": {}, "mcpCapabilities": {"http": False, "sse": False}},
            "authMethods": [],
            "agentInfo": {"name": "stand-in", "version": "0"},
        })
    elif method == "session/new":
        state["servers"] = params.get("mcpServers", [])
        shown = [{key: value for key, value in server.items() if key != "env"} for server in state["servers"]]
        log.write("session/new mcpServers: " + json.dumps(shown) + "\n")
        for server in state["servers"]:
            log.write(f"  {server.get('name')}'s env names: {[pair['name'] for pair in server.get('env', [])]}\n")
        answer(request_id, {"sessionId": "stand-in-1"})
    elif method == "session/prompt":
        text = " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        url = text.split()[-1] if text.split() else ""
        outcome = navigate(url)
        log.write("browser_navigate: " + outcome + "\n")
        send({"jsonrpc": "2.0", "method": "session/update", "params": {
            "sessionId": params.get("sessionId"),
            "update": {"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": f"I opened {url} in Marley's Browser tab."}},
        }})
        answer(request_id, {"stopReason": "end_turn"})
    elif request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "the stand-in does not do " + str(method)}})
PY
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== Zed's context server store runs Marley's server for the project"
  grep -h "context server marley" "$E2E_PROFILE/logs/Marley.log" | sed "s|$E2E_PROFILE|<profile>|; s/^.*INFO /  /"
  pgrep -af "$E2E_PROFILE/mcp/marley-mcp-bridge" | sed "s|$E2E_PROFILE|<profile>|; s/^[0-9]* /  running: /"
  echo "== a thread of the stand-in external agent, from the rail's +"
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  press "" Down
  press "" Down
  press "" Right
  settle 1
  for ((step = 0; step < STAND_IN_STEPS; step++)); do
    press "" Down
  done
  settle 1
  shot 501-01-agents
  press "" Return
  settle 6
  echo "== the prompt: the agent opens the page through Marley's server"
  type_text "open $SITE/index.html"
  press "" Return
  settle 8
  shot 501-02-driven
  echo "== what the stand-in was handed, and what the tool answered"
  sed 's/^/  /' "$E2E_WORK/stand-in.log"
}
