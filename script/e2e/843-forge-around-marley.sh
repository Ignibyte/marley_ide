# shellcheck shell=bash
# #843's visual check: Forge around Marley. #842's stand-in Forge (a sprint, eleven tickets, four
# running agents) grown two bulletins; the provider set from the start.
# - The rail's FORGE section: "4 online", a row per agent with host, ticket and state, Open the
#   sprint board; the bottom row's "Forge · 4 online · #828 QA failed" (`843-01-rail`, REQ-001,
#   REQ-003).
# - Home: the FORGE card: the sprint, its progress and days left, 2 working, 1 needs you, 1 QA
#   failed, and the two bulletins (`843-02-home`, REQ-002).
# - The bottom row's Forge item clicked: the sprint board (`843-03-board`, REQ-003).
# - Marley Day, the rail and the bottom row (`843-04-day`, REQ-001).
compositor sway

TOKEN=forge_pk_e2eStandInTokenForTheFleetScenario0001
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-600}
# From the probe run's shots: the bottom row's Home button, and its Forge item while Home has the
# focus (the focus segment is then empty).
HOME_X=${HOME_X:-1447}
HOME_Y=${HOME_Y:-985}
ITEM_X=${ITEM_X:-420}
ITEM_Y=${ITEM_Y:-985}
STOP_AT=${STOP_AT:-0}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo port
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
  port=$(cat "$E2E_WORK/forge.port")
  export FORGE_E2E_TOKEN=$TOKEN
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.fleet "{\"providers\": [{\"kind\": \"forge\", \"url\": \"http://127.0.0.1:$port/mcp/forge\", \"bearer_env\": \"FORGE_E2E_TOKEN\"}], \"hosts\": []}"
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


TICKETS = []
COMMENTS = {}


def ticket(number, kind, status, phase, title, description, minutes):
    TICKETS.append({"id": f"00000000-0000-0000-0000-000000000{number}", "number": number,
                    "title": title, "description": description, "type": kind,
                    "status": status, "phase": phase, "updated_at": ago(minutes * 60)})


ticket(830, "design", "open", "plan", "Marley Day on a full screen", "", 1440)
ticket(831, "feature", "open", "plan", "A Sessions view for every tmux session", "", 120)
ticket(832, "chore", "open", "plan", "Draw GPUI's components in the theme", "", 20)
ticket(826, "feature", "in-progress", "implement", "The rail shrinks to a strip of logos", "", 0)
ticket(829, "feature", "in-progress", "implement", "The bottom row says what has the focus", "", 12)
ticket(825, "design", "in-review", "review", "Marley Night: coat and eyes", "", 60)
ticket(828, "feature", "qa-failure", "test", "A project's mark from its logo.png",
       "Look for logo.png, then logo.svg, in the project's top folder, and draw it in the rail, "
       "the strip, the title bar and Home.", 18)
ticket(827, "bug", "qa-review", "test", "Search box hides in the shrunk rail", "", 40)
ticket(824, "feature", "done", "complete", "A system prompt for the Marley agent", "", 300)
ticket(823, "feature", "done", "complete", "Any row anywhere in its group", "", 400)
ticket(821, "feature", "closed", "complete", "tmux rows under their project", "", 500)
COMMENTS["00000000-0000-0000-0000-000000000828"] = [
    {"body": "code-2: Fixing: cache the logo per host.", "created_at": ago(600)},
    {"body": "qa-1: Read on refresh; see the log.", "created_at": ago(1080)},
]
SPRINT = {"id": "00000000-0000-0000-0000-0000000000aa", "number": 14, "name": "Rail and theme",
          "github_release_branch": "release/2026-10-24", "status": "active",
          "ends_at": (datetime.now(timezone.utc) + timedelta(days=4, hours=2)).isoformat()}


def find(ticket_id):
    return next(item for item in TICKETS if item["id"] == ticket_id)


def tool(name, arguments):
    if name == "agent-list":
        return agents()
    if name == "bulletin-list":
        return {"bulletins": [
            {"id": "b1", "title": "release/2026-10-24 freezes Thursday at noon",
             "body": "Land QA fixes first.", "severity": "critical"},
            {"id": "b2", "title": "agent3 is short on disk",
             "body": "Prune target folders before long builds.", "severity": "warning"},
        ]}
    if name == "sprint-current":
        return {"sprint": SPRINT}
    if name == "ticket-list":
        return {"tickets": TICKETS, "count": len(TICKETS)}
    if name == "ticket-get":
        return {"ticket": find(arguments["id"]), "comments": COMMENTS.get(arguments["id"], [])}
    if name == "ticket-update":
        found = find(arguments["id"])
        found["status"] = arguments.get("status", found["status"])
        found["updated_at"] = ago(0)
        return {"ticket": found, "updated": True}
    if name == "ticket-claim":
        return {"ticket": find(arguments["id"]), "previous_owner_id": None}
    if name == "ticket-comment":
        COMMENTS.setdefault(arguments["ticket_id"], []).insert(
            0, {"body": arguments["body"], "created_at": ago(0)})
        return {"comment": {"body": arguments["body"]}}
    return None


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
        called = message.get("params", {}).get("name", "") if method == "tools/call" else ""
        arguments = message.get("params", {}).get("arguments", {}) if called else {}
        detail = f" {called} {json.dumps(arguments, sort_keys=True)}" if called.startswith("ticket-") and called not in ("ticket-list", "ticket-get") else (f" {called}" if called else "")
        self.note(f"POST {self.path} {method}{detail} bearer={'ok' if bearer else 'missing or wrong'}")
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
        elif method == "tools/call" and (answered := tool(called, arguments)) is not None:
            result = {"content": [{"type": "text", "text": json.dumps(answered)}],
                      "structuredContent": answered}
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

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  # The watch reads Forge as the provider is seen.
  settle 6
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 843-01-rail
  if [[ $STOP_AT == 1 ]]; then
    stop_forge
    return
  fi

  echo "== Home"
  click "$HOME_X" "$HOME_Y"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 843-02-home

  echo "== the bottom row's Forge item"
  click "$ITEM_X" "$ITEM_Y"
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 843-03-board

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 843-04-day
  stop_forge
}

# Prints the stand-in's tool calls and stops it.
stop_forge() {
  echo "== the stand-in's tool calls"
  grep -o "tools/call [a-z-]*" "$E2E_WORK/forge.log" | sort | uniq -c
  kill "$FORGE_PID" 2>/dev/null || true
}
