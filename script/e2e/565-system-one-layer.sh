# shellcheck shell=bash
# #565's e2e test: the System One layer, off by default and turned on for this run. A fake
# `/v1/systemone` server on a free loopback port logs each request's headers and body, and answers
# by the state's text: `slow` sleeps three seconds, `fail` answers 500, and anything else answers
# `command_failed` at 0.92, or at 0.08 when the state says the command exited 0, with 1,200 input
# tokens. The profile's settings turn the layer on, on the `compatible` provider at the fake's URL
# with the scratch repository listed, and the key comes from MARLEY_SYSTEM_ONE_KEY, exported
# before Marley starts. The run never presses Set Key or Forget Key: they write the keyring, and
# the headless sway reaches the user's own. Each check is `marley: system one check` from the
# palette after a command in the terminal: the answer in a toast and in System One calls, with the key's
# source (REQ-002, REQ-009); the project taken off the list (REQ-003); a secret masked (REQ-004);
# a metadata-only project (REQ-005); a slow answer, five failures and the breaker (REQ-006); a
# budget of 0 (REQ-007); the replay provider (REQ-008); the layer off (REQ-001). Every change is
# made while Marley runs (REQ-011). Last, the Settings window's System One section (REQ-010).
compositor sway

# In the window's logical pixels, from the first run's shots: the rail's terminal row, the first
# row of System One calls, the rows of the masked and the metadata-only checks once every call is listed,
# and a point in the Settings window's page to scroll at.
TERMINAL_ROW_X=${TERMINAL_ROW_X:-110}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-135}
DECISION_X=${DECISION_X:-700}
DECISION_Y=${DECISION_Y:-235}
MASKED_ROW_Y=${MASKED_ROW_Y:-698}
FACTS_ROW_Y=${FACTS_ROW_Y:-659}
SETTINGS_X=${SETTINGS_X:-1200}
SETTINGS_Y=${SETTINGS_Y:-500}

# The run's key: not a real one, and never printed.
KEY=e2e-not-a-real-key

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  # shellcheck disable=SC2016 # the prompt is the terminal's to expand
  printf 'PS1=%s\n' "'\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  git init -q -b main "$E2E_WORK/repo"
  write_fake
  (exec python3 -u "$E2E_WORK/systemone.py" "$E2E_WORK/systemone.log" >"$E2E_WORK/fake.out" 2>&1) &
  echo "$!" >>"$E2E_WORK/servers"
  local port=
  for _ in $(seq 50); do
    # sed, not grep: a line not printed yet is no failure under `set -e`.
    port=$(sed -n 's/^port \([0-9]*\)$/\1/p' "$E2E_WORK/fake.out")
    [[ -n $port ]] && break
    sleep 0.1
  done
  [[ -n $port ]] || { echo "setup: the fake /v1/systemone server did not start" >&2; return 1; }
  : >"$E2E_WORK/systemone.log"
  export MARLEY_SYSTEM_ONE_KEY=$KEY
  system_one_setting "{\"enabled\": true, \"provider\": \"compatible\", \"endpoint\": \"http://127.0.0.1:$port/v1/systemone\", \"projects\": [\"$E2E_WORK/repo\"]}"
  open_path "$E2E_WORK/repo"
}

teardown() {
  local pid
  if [[ -f $E2E_WORK/servers ]]; then
    while read -r pid; do
      kill "$pid" 2>/dev/null || true
    done <"$E2E_WORK/servers"
  fi
}

write_fake() {
  cat >"$E2E_WORK/systemone.py" <<'PY'
# A fake /v1/systemone on a free loopback port: logs each request's path, headers and body, one
# JSON line each, and answers by the state's text.
import http.server
import json
import sys
import time

LOG = sys.argv[1]


class Handler(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length") or 0)
        raw = self.rfile.read(length).decode("utf-8", "replace")
        with open(LOG, "a", encoding="utf-8") as log:
            log.write(json.dumps({"path": self.path, "headers": dict(self.headers.items()),
                                  "body": raw}) + "\n")
        try:
            body = json.loads(raw)
        except ValueError:
            body = {}
        state = str(body.get("state", ""))
        if "slow" in state:
            time.sleep(3)
        if "fail" in state:
            status, payload = 500, {"error": "e2e: failing on purpose"}
        else:
            noul = 0.08 if "exit code: 0" in state else 0.92
            status, payload = 200, {
                "model": body.get("model"),
                "answers": {"command_failed": {"type": "noul", "noul": noul}},
                "usage": {"input_tokens": 1200, "output_tokens": 0},
            }
        data = json.dumps(payload).encode()
        try:
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)
        except (BrokenPipeError, ConnectionResetError):
            pass

    def log_message(self, *args):
        pass


server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
print(f"port {server.server_address[1]}", flush=True)
server.serve_forever()
PY
}

# Merges the JSON object `$1` into `marley.system_one` in the profile copy's settings, which it
# rewrites as plain JSON (the copy's comments go; its values stay).
system_one_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'PY'
import json, pathlib, sys

path, changes = pathlib.Path(sys.argv[1]), json.loads(sys.argv[2])
text = path.read_text() if path.exists() else "{}"
# JSONC to JSON, outside strings only: no comments, no comma before a closing bracket.
def skip_blank(index):
    while index < len(text):
        if text[index] in " \t\r\n":
            index += 1
        elif text.startswith("//", index):
            end = text.find("\n", index)
            index = len(text) if end < 0 else end
        elif text.startswith("/*", index):
            end = text.find("*/", index + 2)
            index = len(text) if end < 0 else end + 2
        else:
            break
    return index


out, index, in_string = [], 0, False
while index < len(text):
    character = text[index]
    if in_string:
        out.append(character)
        if character == "\\" and index + 1 < len(text):
            out.append(text[index + 1])
            index += 2
            continue
        if character == '"':
            in_string = False
        index += 1
        continue
    if character == '"':
        in_string = True
    elif text.startswith("//", index) or text.startswith("/*", index):
        index = skip_blank(index)
        continue
    elif character == ",":
        ahead = skip_blank(index + 1)
        if ahead < len(text) and text[ahead] in "}]":
            index += 1
            continue
    out.append(character)
    index += 1
cleaned = "".join(out).strip()
settings = json.loads(cleaned) if cleaned else {}
settings.setdefault("marley", {}).setdefault("system_one", {}).update(changes)
path.write_text(json.dumps(settings, indent=2) + "\n")
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Runs `command` in the project's terminal, shown and focused from its rail row, and then the
# check; `wait` is how long to let the answer come.
check_after() {
  local command=$1 wait=${2:-3}
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y"
  settle 1
  type_text "$command"
  press "" Return
  settle 2
  palette "marley: system one check"
  settle "$wait"
}

# How many requests the fake has had.
requests() {
  wc -l <"$E2E_WORK/systemone.log"
}

# How many calls today's file holds.
rows() {
  cat "$E2E_PROFILE"/system_one/calls-*.jsonl 2>/dev/null | grep -c '"row":"call"' || true
}

# The body of the fake's last request.
last_body() {
  tail -n 1 "$E2E_WORK/systemone.log" | python3 -c 'import json, sys; print(json.load(sys.stdin)["body"])'
}

steps() {
  settle 12
  # Trusts the repository.
  press "" Return
  settle 5

  echo "== a check that answers, and a failed command's"
  check_after "echo hello"
  shot 565-02a-answered
  check_after "false"
  shot 565-02b-failed
  echo "  requests $(requests), rows $(rows)"
  expect "two requests reached the fake" test "$(requests)" -eq 2
  expect "each carried the key as a bearer header" \
    bash -c "grep -c 'Bearer $KEY' '$E2E_WORK/systemone.log' | grep -qx 2"
  expect "each asked the pinned model and the check's question" \
    bash -c "grep -c 'jev-1.13.0' '$E2E_WORK/systemone.log' | grep -qx 2 && grep -q 'command_failed' '$E2E_WORK/systemone.log'"
  last_body >"$E2E_WORK/body-failed.json"
  expect "the state holds the facts and the command" holds "$E2E_WORK/body-failed.json" \
    'project: repo' 'exit code: 1' 'program: false' 'last command: false'

  echo "== System One calls"
  palette "marley: open system one calls"
  settle 3
  shot 565-02-decisions-view
  click "$DECISION_X" "$DECISION_Y"
  settle 1
  shot 565-02c-expanded

  echo "== the project taken off the list"
  system_one_setting '{"projects": []}'
  settle 2
  check_after "echo again"
  shot 565-03-refused-not-listed
  expect "no request went out for an unlisted project" test "$(requests)" -eq 2

  echo "== a secret in the command"
  system_one_setting "{\"projects\": [\"$E2E_WORK/repo\"]}"
  settle 2
  local token
  token=$(printf 'gh%s_%s' p "$(printf 'Fake%.0s' {1..9})")
  check_after "echo GITHUB_TOKEN=$token"
  shot 565-04-masked
  last_body >"$E2E_WORK/body-masked.json"
  expect "the secret went out masked" holds "$E2E_WORK/body-masked.json" '[redacted: secret]'
  expect "and not as it was typed" bash -c "! grep -q 'FakeFake' '$E2E_WORK/body-masked.json'"

  echo "== a metadata-only project"
  system_one_setting "{\"metadata_only_projects\": [\"$E2E_WORK/repo\"]}"
  settle 2
  check_after "echo hidden"
  shot 565-05-metadata-only
  last_body >"$E2E_WORK/body-facts.json"
  expect "a metadata-only project sends its facts" holds "$E2E_WORK/body-facts.json" \
    'exit code: 0' 'program: echo'
  expect "and none of its text" bash -c "! grep -q 'hidden' '$E2E_WORK/body-facts.json'"
  system_one_setting '{"metadata_only_projects": []}'
  settle 2

  echo "== a slow answer, five failures and the breaker"
  check_after "echo slow" 4
  shot 565-06a-slow
  check_after "echo ok"
  local attempt
  for attempt in 1 2 3 4 5; do
    check_after "echo fail $attempt"
  done
  shot 565-06b-failed
  local before_breaker
  before_breaker=$(requests)
  check_after "echo fail 6"
  shot 565-06c-breaker-open
  echo "  requests $(requests)"
  expect "the slow, the ok and five failing checks reached the fake" test "$before_breaker" -eq 11
  expect "the open breaker sent nothing" test "$(requests)" -eq 11

  echo "== a budget of 0"
  system_one_setting '{"daily_budget_cents": 0}'
  settle 2
  check_after "echo budget"
  shot 565-07-budget
  expect "a spent budget sent nothing" test "$(requests)" -eq 11

  echo "== the replay provider"
  printf '%s\n' '{"set": "check/1", "match": "replayed", "answers": {"command_failed": {"type": "noul", "noul": 0.77}}}' \
    >"$E2E_PROFILE/system_one/replay.jsonl"
  system_one_setting '{"provider": "replay", "daily_budget_cents": 50}'
  settle 3
  check_after "echo replayed"
  shot 565-08-replay
  expect "the replay answered with no request" test "$(requests)" -eq 11
  expect "and its row names the replay provider" \
    bash -c "grep '\"row\":\"call\"' '$E2E_PROFILE'/system_one/calls-*.jsonl | tail -n 1 | grep -q '\"provider\":\"replay\"'"

  echo "== every call in System One calls"
  palette "marley: open system one calls"
  settle 3
  shot 565-08b-decisions-all
  local rows_on
  rows_on=$(rows)
  echo "  rows $rows_on"
  expect "one row for each check, the refused and the unavailable ones too" test "$rows_on" -eq 15
  # The masked check's row, then the metadata-only one's, each opened to its state as sent.
  click "$DECISION_X" "$MASKED_ROW_Y"
  settle 1
  shot 565-04b-masked-row
  click "$DECISION_X" "$MASKED_ROW_Y"
  settle 1
  click "$DECISION_X" "$FACTS_ROW_Y"
  settle 1
  shot 565-05b-metadata-only-row

  echo "== the layer off"
  system_one_setting '{"enabled": false}'
  settle 2
  check_after "echo off"
  shot 565-09-off
  expect "the layer off sent nothing" test "$(requests)" -eq 11
  expect "and wrote no row" test "$(rows)" -eq "$rows_on"

  echo "== the key's value"
  expect "the key is nowhere under the profile, Marley's log included" \
    bash -c "! grep -rqF '$KEY' '$E2E_PROFILE'"

  echo "== the Settings window's System One section"
  # Back on, as the layer is run, for the page to show.
  system_one_setting '{"enabled": true, "provider": "compatible"}'
  settle 2
  palette "marley: open settings"
  settle 4
  shot 565-01a-marley-page
  pointer_to "$SETTINGS_X" "$SETTINGS_Y"
  settle 1
  scroll 20
  settle 2
  shot 565-01-settings-section
}
