# shellcheck shell=bash
# #841's visual check: Forge as a fleet provider. A stand-in Forge (stateless JSON-RPC over HTTP at
# /mcp/forge, as forge-mcp serves it) refuses a missing or wrong bearer and logs each request; it
# lists five sessions: four running on forge, agent1, agent2 and agent3, one stopped. The token
# reaches Marley through `bearer_env`, so the run never writes the user's keyring.
# - The Fleet panel open with no forge provider: the stand-in's log stays empty (REQ-004).
# - The provider set: Forge's source, its agents under forge, agent1, agent2 and agent3, each with
#   its harness, ticket and folder; agent1's silent agent waiting; no stopped session
#   (`841-01-fleet`, REQ-001, REQ-002).
# - `marley: set forge token`: the masked field, Escape before Enter (`841-02-token`, REQ-003).
# - Marley Day, the Fleet panel (`841-03-day`, REQ-001).
compositor sway

TOKEN=forge_pk_e2eStandInTokenForTheFleetScenario0001

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  write_forge
  : >"$E2E_WORK/forge.log"
  FORGE_TOKEN=$TOKEN python3 "$E2E_WORK/forge.py" "$E2E_WORK/forge.port" "$E2E_WORK/forge.log" &
  FORGE_PID=$!
  for _ in $(seq 50); do
    [[ -s $E2E_WORK/forge.port ]] && break
    sleep 0.1
  done
  export FORGE_E2E_TOKEN=$TOKEN
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.fleet '{"providers": [], "hosts": []}'
  open_path "$repo"
}

# The stand-in Forge.
write_forge() {
  cat >"$E2E_WORK/forge.py" <<'PY'
import json
import os
import sys
import time
from datetime import datetime, timedelta, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

PORT_FILE, LOG = sys.argv[1], sys.argv[2]
TOKEN = os.environ["FORGE_TOKEN"]


def ago(seconds):
    return (datetime.now(timezone.utc) - timedelta(seconds=seconds)).isoformat()


def session(name, host, harness, ticket, folder, status, state, silent):
    return {"id": f"session-{name}", "project_id": "00000000-0000-0000-0000-000000000001",
            "agent_name": name, "folder": folder, "host": host, "harness": harness,
            "status": status, "current_ticket": ticket, "last_output_at": ago(silent),
            "last_heartbeat_at": ago(2), "state": state, "created_at": ago(9000),
            "updated_at": ago(2)}


def agents():
    return {"agents": [
        session("code-1", "forge", "claude-code", "#826", "/srv/stacks/marley_ide", "running", "working", 4),
        session("code-2", "agent1", "claude-code", "#828", "/srv/work/marley_ide", "running", "needs_attention", 420),
        session("qa-1", "agent2", "codex", "#829", "/srv/work/marley_ide", "running", "working", 20),
        session("docs-1", "agent3", "claude-code", None, "/srv/work/rusty-v3", "running", "idle", 7200),
        session("old-1", "agent3", "claude-code", "#700", "/srv/work/rusty-v3", "stopped", "idle", 90000),
    ]}


class Forge(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def note(self, line):
        with open(LOG, "a") as log:
            log.write(line + "\n")

    def answer(self, code, body=None):
        data = json.dumps(body).encode() if body is not None else b""
        self.send_response(code)
        if body is not None:
            self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        self.note(f"GET {self.path}")
        self.answer(405)

    def do_DELETE(self):
        self.answer(405)

    def do_POST(self):
        length = int(self.headers.get("Content-Length") or 0)
        message = json.loads(self.rfile.read(length) or b"{}")
        method = message.get("method")
        bearer = self.headers.get("Authorization") == f"Bearer {TOKEN}"
        self.note(f"POST {self.path} {method} bearer={'ok' if bearer else 'missing or wrong'}")
        if not bearer:
            return self.answer(401, {"error": "unauthorized"})
        if "id" not in message:
            return self.answer(202)
        if method == "initialize":
            result = {"protocolVersion": message["params"]["protocolVersion"],
                      "capabilities": {"tools": {}},
                      "serverInfo": {"name": "forge-mcp", "version": "e2e"}}
        elif method == "tools/list":
            result = {"tools": [{"name": "agent-list", "inputSchema": {"type": "object"}}]}
        elif method == "tools/call" and message["params"]["name"] == "agent-list":
            listed = agents()
            result = {"content": [{"type": "text", "text": json.dumps(listed)}],
                      "structuredContent": listed}
        else:
            return self.answer(200, {"jsonrpc": "2.0", "id": message["id"],
                                     "error": {"code": -32601, "message": "unknown"}})
        self.answer(200, {"jsonrpc": "2.0", "id": message["id"], "result": result})


server = ThreadingHTTPServer(("127.0.0.1", 0), Forge)
with open(PORT_FILE, "w") as port_file:
    port_file.write(str(server.server_address[1]))
server.serve_forever()
PY
}

# Runs `$1` from the command palette.
palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

steps() {
  local port
  port=$(cat "$E2E_WORK/forge.port")
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the Fleet panel, no forge provider"
  palette "marley: toggle fleet" 6
  echo "requests to the stand-in with no provider: $(wc -l <"$E2E_WORK/forge.log")"

  echo "== the forge provider"
  profile_setting marley.fleet "{\"providers\": [{\"kind\": \"forge\", \"url\": \"http://127.0.0.1:$port/mcp/forge\", \"bearer_env\": \"FORGE_E2E_TOKEN\", \"poll_seconds\": 3}], \"hosts\": []}"
  settle 10
  pointer_to 700 500
  settle 1
  shot 841-01-fleet

  echo "== marley: set forge token"
  palette "marley: set forge token" 2
  type_text "forge_pk_typed"
  settle 1
  shot 841-02-token
  press "" Escape
  settle 1

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 841-03-day

  echo "== the stand-in's log"
  sort "$E2E_WORK/forge.log" | uniq -c
  kill "$FORGE_PID" 2>/dev/null || true
}
