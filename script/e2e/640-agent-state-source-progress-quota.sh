# shellcheck shell=bash
# #640's visual check: a harness session's state source, progress and quota on its rail row. A
# stand-in `rh mcp` (a stdio MCP server this scenario writes) serves `fleet_snapshot` and
# `fleet_events` from a fixture of seven sessions, with the labels rustal-harness publishes
# (its MREQ-005 to MREQ-007, D173, D174), and `marley.harness` names it. A reset written as `+N`
# in the fixture is the stand-in's load time plus N milliseconds, so the countdowns read the same
# however long Marley takes to start.
#
# The rows (`640-01-rows`, REQ-001 to REQ-004, REQ-006 to REQ-009): watcher and asker weaker with
# `detected`, odd with `guessed`, build, review, plain and restart drawn as #534 draws them, build
# with its progress and quota lines. The inbox holds review's question and not asker's
# (`640-02-inbox`, REQ-004). The pointer on build's row shows the tooltip (`640-03-tooltip`,
# REQ-010). The fixture rewritten moves build's numbers and makes asker reported
# (`640-04-moved`, REQ-005, REQ-011).
compositor sway

# Where build's row sits in the rail, from the first shot; 0 takes the group shot only.
BUILD_Y=${BUILD_Y:-460}

# Writes the fixture: the seven sessions, with build's numbers and asker's source as `$1` says.
write_fleet() {
  local moved=$1
  python3 - "$E2E_WORK/fleet.json" "$moved" <<'FLEET'
import json, sys

path, moved = sys.argv[1], sys.argv[2] == "moved"
build = {
    "state.source": "protocol",
    "progress.percent": "80" if moved else "40",
    "progress.activity": "Writing the summary" if moved else "Running the tests",
    "quota.five_hour.percent_used": "70.0" if moved else "62.4",
    "quota.five_hour.resets_at_ms": "+5730000",
    "quota.seven_day.percent_used": "31.0",
    "quota.seven_day.resets_at_ms": "+273630000",
    "quota.account": "work",
    "usage.input_tokens": "1200",
}
seats = [
    {"id": "build", "title": "build", "state": "working", "labels": build},
    {"id": "review", "title": "review", "state": "waiting",
     "question": {"prompt": "Merge the branch?", "options": ["yes", "no"], "context_refs": []},
     "labels": {"state.source": "reported", "progress.percent": "75",
                "progress.activity": "Waiting for a merge decision"}},
    {"id": "watcher", "title": "watcher", "state": "working",
     "labels": {"state.source": "detected"}},
    {"id": "asker", "title": "asker", "state": "waiting",
     "question": {"prompt": "Allow the edit?", "options": ["allow", "deny"], "context_refs": []},
     "labels": {"state.source": "reported" if moved else "detected"}},
    {"id": "plain", "title": "plain", "state": "idle", "labels": {}},
    {"id": "odd", "title": "odd", "state": "working",
     "labels": {"state.source": "guessed", "progress.percent": "140",
                "progress.activity": "Reading the logs\nsecond line",
                "quota.five_hour.percent_used": "lots",
                "quota.account": "someone@example.com"}},
    {"id": "restart", "title": "restart", "state": "starting",
     "labels": {"state.source": "runtime"}},
]
with open(path + ".new", "w") as out:
    json.dump({"seats": seats}, out)
import os
os.replace(path + ".new", path)
FLEET
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  write_fleet first
  cat >"$bin/rh-stand-in" <<'PY'
#!/usr/bin/env python3
# A stand-in for `rh mcp`: the read tools Marley calls, from a fixture file it watches.
import json, os, sys, time

FLEET = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "fleet.json")

def now_ms():
    return int(time.time() * 1000)

def load():
    stamp = now_ms()
    seats = json.load(open(FLEET))["seats"]
    for seat in seats:
        seat["last_event_ms"] = stamp
        seat["transport"] = "tmux"
        for key, value in seat["labels"].items():
            if value.startswith("+"):
                seat["labels"][key] = str(stamp + int(value[1:]))
    return seats

served = None
cursor = 1

def snapshot():
    global served
    served = os.stat(FLEET).st_mtime_ns
    return {"instance_id": "stand-in", "cursor": cursor, "seats": load()}

def events(after):
    global served, cursor
    stamp = os.stat(FLEET).st_mtime_ns
    if stamp == served:
        return {"events": [], "next_cursor": after}
    served = stamp
    cursor = after + 1
    out = []
    for seat in load():
        out.append({"event": {"kind": "upsert", "id": seat["id"], "ts_ms": seat["last_event_ms"],
                              "title": seat["title"], "state": seat["state"],
                              "labels": seat["labels"], "transport": "tmux"}})
        if "question" in seat:
            out.append({"event": {"kind": "question_raised", "id": seat["id"],
                                  "ts_ms": seat["last_event_ms"], "question": seat["question"]}})
    return {"events": out, "next_cursor": cursor}

for line in sys.stdin:
    message = json.loads(line)
    if "id" not in message:
        continue
    method = message.get("method")
    if method == "initialize":
        result = {"protocolVersion": "2025-06-18", "capabilities": {"tools": {}},
                  "serverInfo": {"name": "rh-stand-in", "version": "0"}}
    elif method == "tools/list":
        result = {"tools": []}
    elif method == "tools/call":
        params = message.get("params", {})
        name, arguments = params.get("name"), params.get("arguments") or {}
        if name == "fleet_snapshot":
            value = snapshot()
        elif name == "fleet_events":
            value = events(arguments.get("after", 0))
        else:
            value = {"result": "accepted", "value": {"lines": ["stand-in"]}}
        result = {"content": [{"type": "text", "text": json.dumps(value)}],
                  "structuredContent": value}
    else:
        result = {}
    print(json.dumps({"jsonrpc": "2.0", "id": message["id"], "result": result}), flush=True)
PY
  chmod +x "$bin/rh-stand-in"
  profile_setting marley.harness "{\"command\": \"$bin/rh-stand-in\", \"args\": []}"
  profile_setting marley.no_update_after_minutes 0
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 5
  pointer_to 900 400
  settle 1
  shot 640-01-rows
  shot 640-02-inbox
  if ((BUILD_Y == 0)); then
    echo "build's row is not placed yet: the group shots only"
    return 0
  fi

  echo "== build's tooltip"
  pointer_to 120 "$BUILD_Y"
  settle 3
  shot 640-03-tooltip
  pointer_to 900 400
  settle 1

  echo "== the fixture moved on"
  write_fleet moved
  settle 5
  shot 640-04-moved
}
