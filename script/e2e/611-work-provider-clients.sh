# shellcheck shell=bash
# #611's e2e test: the marley.work/v1 clients. `fleet-stub-provider.py` serves the contract's
# fixtures as two stores: over HTTP on a port of its own, which asks for a bearer token, and over
# MCP on stdio, which Marley starts. Marley reads the token from FLEET_TOKEN.
#
# Both headers read ready with their agents listed (`ready`, REQ-001). With the HTTP store killed,
# its header reads unreachable with the reason and its agents offline, while the MCP store's list
# is as it was (`unreachable`, REQ-002, REQ-006). Started again answering marley.work/v9, it reads
# incompatible and names that contract (`incompatible`, REQ-003). Answering again, then silent,
# it reads stale and keeps its last list (`stale`, REQ-004). `log.txt` holds Marley's log lines
# about the bearer: the variable's name, and whether the token itself appears (REQ-005).
compositor sway

TOKEN=fleet-token-611-not-for-logs
HTTP_PID=""
HTTP_PORT=""
STUB=""

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  STUB=$PWD/script/e2e/fleet-stub-provider.py
  echo ok >"$E2E_WORK/http-control"
  echo ok >"$E2E_WORK/mcp-control"
  HTTP_PORT=$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
  start_http
  export FLEET_TOKEN=$TOKEN
  python3 - "$E2E_PROFILE/config/settings.json" "$HTTP_PORT" "$STUB" "$E2E_WORK/mcp-control" <<'PY'
import json, re, sys
path, port, stub, control = sys.argv[1:5]
providers = [
    {"kind": "http", "name": "store over HTTP", "url": f"http://127.0.0.1:{port}",
     "bearer_env": "FLEET_TOKEN"},
    {"kind": "mcp", "name": "store over MCP", "command": "python3",
     "args": [stub, "mcp", "--control", control]},
]
entry = '"fleet": ' + json.dumps({"providers": providers}) + ","
text = open(path).read()
match = re.search(r'"marley"\s*:\s*\{', text)
if match:
    text = text[:match.end()] + "\n    " + entry + text[match.end():]
else:
    at = text.index("{") + 1
    text = text[:at] + '\n  "marley": {' + entry + '},' + text[at:]
open(path, "w").write(text)
PY
  open_path "$E2E_WORK/repo"
}

teardown() {
  if [[ -n $HTTP_PID ]]; then
    kill "$HTTP_PID" 2>/dev/null || true
  fi
}

# The HTTP store, on the scenario's port, with the token it asks for.
start_http() {
  STUB_TOKEN=$TOKEN python3 "$STUB" http "$HTTP_PORT" --control "$E2E_WORK/http-control" \
    >"$E2E_WORK/http-stub.log" 2>&1 &
  HTTP_PID=$!
  for _ in $(seq 50); do
    ss -ltnH "sport = :$HTTP_PORT" | grep -q . && break
    sleep 0.1
  done
  echo "the HTTP store: pid $HTTP_PID on 127.0.0.1:$HTTP_PORT"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  press "CTRL SHIFT" p
  settle 1
  type_text "marley: toggle fleet"
  settle 1
  press "" Return

  echo "== both stores ready"
  settle 8
  shot ready

  echo "== the HTTP store killed"
  kill "$HTTP_PID"
  HTTP_PID=""
  settle 6
  shot unreachable

  echo "== the HTTP store back, answering marley.work/v9"
  echo v9 >"$E2E_WORK/http-control"
  start_http
  # The backoff after three failures is 8 s.
  settle 14
  shot incompatible

  echo "== answering, then silent"
  echo ok >"$E2E_WORK/http-control"
  # The backoff after the fourth failure in a row is 16 s.
  settle 20
  shot answering
  echo silent >"$E2E_WORK/http-control"
  settle 14
  shot stale

  echo "== the bearer in the log"
  {
    grep -h 'bearer' "$E2E_PROFILE/logs/Marley.log" || echo "no line about the bearer"
    if grep -q "$TOKEN" "$E2E_PROFILE/logs/Marley.log"; then
      echo "the token itself is in the log"
    else
      echo "the token itself is not in the log"
    fi
  } | tee "$(shot_file log.txt)"
}
