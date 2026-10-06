# shellcheck shell=bash
# #548's visual check: Jev through Cloudflare Workers AI, for one project. Two scratch
# repositories, `direct` and `private`, both may send; `private` goes through `cloudflare` by
# `provider_by_project`, and `direct` through `compatible` (TypeSafe's own URL cannot be pointed
# at a fake). One fake on a free loopback port answers TypeSafe's shape at `/v1/systemone` and
# Cloudflare's REST envelope at `/client/v4/accounts/<id>/ai/run`, logs each request's path,
# headers and body, and answers 403 with Cloudflare's error envelope when the state holds
# `refuse-token`. Both keys are made up and come from the environment; the run never presses a
# keyring button, since the headless sway reaches the user's own keyring (L-565).
#
# `548-01-search` to `548-06-no-account`; the settings shots come last.
compositor sway

# In the window's logical pixels, from the first runs' shots: each project's terminal row in the
# rail (`private`, handed over second, sits first), the Settings window's search field, and its
# Provider dropdown once the search narrows the page.
PRIVATE_ROW_X=${PRIVATE_ROW_X:-110}
PRIVATE_ROW_Y=${PRIVATE_ROW_Y:-135}
DIRECT_ROW_X=${DIRECT_ROW_X:-110}
DIRECT_ROW_Y=${DIRECT_ROW_Y:-230}
SETTINGS_SEARCH_X=${SETTINGS_SEARCH_X:-912}
SETTINGS_SEARCH_Y=${SETTINGS_SEARCH_Y:-59}
PROVIDER_X=${PROVIDER_X:-1513}
PROVIDER_Y=${PROVIDER_Y:-254}

# The run's secrets: made up, and never printed.
KEY=e2e-not-a-real-key
TOKEN=e2e-not-a-real-cloudflare-token
ACCOUNT=0123456789abcdef0123456789abcdef

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  # shellcheck disable=SC2016 # the prompt is the terminal's to expand
  printf 'PS1=%s\n' "'\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  git init -q -b main "$E2E_WORK/direct"
  git init -q -b main "$E2E_WORK/private"
  write_fake
  (exec python3 -u "$E2E_WORK/fake.py" "$E2E_WORK/requests.log" >"$E2E_WORK/fake.out" 2>&1) &
  echo "$!" >>"$E2E_WORK/servers"
  local port=
  for _ in $(seq 50); do
    # sed, not grep: a line not printed yet is no failure under `set -e` (L-565).
    port=$(sed -n 's/^port \([0-9]*\)$/\1/p' "$E2E_WORK/fake.out")
    [[ -n $port ]] && break
    sleep 0.1
  done
  [[ -n $port ]] || { echo "setup: the fake server did not start" >&2; return 1; }
  : >"$E2E_WORK/requests.log"
  export MARLEY_SYSTEM_ONE_KEY=$KEY MARLEY_CLOUDFLARE_API_TOKEN=$TOKEN
  system_one_setting "{\"enabled\": true, \"provider\": \"compatible\", \"endpoint\": \"http://127.0.0.1:$port/v1/systemone\", \"projects\": [\"$E2E_WORK/direct\", \"$E2E_WORK/private\"], \"provider_by_project\": {\"$E2E_WORK/private\": \"cloudflare\"}, \"cloudflare_api\": \"http://127.0.0.1:$port/client/v4\", \"cloudflare_account_id\": \"$ACCOUNT\"}"
  open_path "$E2E_WORK/direct"
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
  cat >"$E2E_WORK/fake.py" <<'PY'
# A fake of both providers on a free loopback port: TypeSafe's /v1/systemone and Cloudflare's
# /client/v4/accounts/<id>/ai/run. Logs each request's path, headers and body, one JSON line each.
import http.server
import json
import sys

LOG = sys.argv[1]


def answer(model, state):
    noul = 0.08 if "exit code: 0" in state else 0.92
    return {
        "model": model,
        "answers": {"command_failed": {"type": "noul", "noul": noul}},
        "usage": {"input_tokens": 1200, "output_tokens": 0},
    }


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
        if self.path.startswith("/client/v4/accounts/") and self.path.endswith("/ai/run"):
            state = str((body.get("input") or {}).get("state", ""))
            if "refuse-token" in state:
                status, payload = 403, {"result": None, "success": False, "messages": [],
                                        "errors": [{"code": 10000, "message": "Authentication error"}]}
            else:
                status, payload = 200, {"result": answer("jev-1.13.0", state), "success": True,
                                        "errors": [], "messages": []}
        elif self.path == "/v1/systemone":
            status, payload = 200, answer(body.get("model"), str(body.get("state", "")))
        else:
            status, payload = 404, {"error": "no such path"}
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

# A second launch on this profile with `$1`, which hands it to the running Marley (#513).
hand_off() {
  env -u DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE WAYLAND_DISPLAY="$SWAY_DISPLAY" \
    SWAYSOCK="$SWAY_SOCK" "$E2E_MARLEY" --user-data-dir "$E2E_PROFILE" "$1" \
    >"$E2E_WORK/second.log" 2>&1 </dev/null
}

# Runs `$3` in the terminal whose rail row is at `$1`, `$2`, then the check.
check_in() {
  click "$1" "$2"
  settle 1
  type_text "$3"
  press "" Return
  settle 2
  palette "marley: system one check"
  settle 3
}

# How many requests the fake had at the path holding `$1`.
requests_at() {
  grep -c "\"path\": \"[^\"]*$1" "$E2E_WORK/requests.log" || true
}

# Whether the fake's last Cloudflare request went to the account's path with the token as its
# bearer and a body of Cloudflare's model and `input` holding the state and the questions, with
# no trace of the direct key.
cloudflare_request_right() {
  python3 - "$E2E_WORK/requests.log" "$ACCOUNT" "$TOKEN" "$KEY" <<'PY'
import json, sys

log, account, token, key = sys.argv[1:5]
lines = [json.loads(line) for line in open(log, encoding="utf-8") if "/ai/run" in line]
last = lines[-1]
body = json.loads(last["body"])
headers = {name.lower(): value for name, value in last["headers"].items()}
assert last["path"] == f"/client/v4/accounts/{account}/ai/run", last["path"]
assert headers.get("authorization") == f"Bearer {token}", "the bearer"
assert body["model"] == "typesafe/jev", body.get("model")
assert set(body["input"]) == {"state", "questions"}, sorted(body["input"])
assert "command_failed" in body["input"]["questions"], sorted(body["input"]["questions"])
assert key not in json.dumps(last), "the direct key"
PY
}

# How many of today's rows name the provider `$1`.
rows_of() {
  cat "$E2E_PROFILE"/system_one/calls-*.jsonl 2>/dev/null | grep '"row":"call"' |
    grep -c "\"provider\":\"$1\"" || true
}

steps() {
  settle 12
  # Trusts `direct`.
  press "" Return
  settle 3
  echo "== private in the same window"
  hand_off "$E2E_WORK/private"
  settle 6
  # Trusts `private`.
  press "" Return
  settle 3

  echo "== direct"
  check_in "$DIRECT_ROW_X" "$DIRECT_ROW_Y" false
  shot 548-02-direct
  expect "direct asked the compatible fake once" test "$(requests_at /v1/systemone)" = 1
  expect "direct asked Cloudflare nothing" test "$(requests_at /ai/run)" = 0

  echo "== private, through Cloudflare"
  check_in "$PRIVATE_ROW_X" "$PRIVATE_ROW_Y" false
  shot 548-03-cloudflare
  expect "private asked Cloudflare once" test "$(requests_at /ai/run)" = 1
  expect "Cloudflare's path, bearer, model and input, and no direct key" cloudflare_request_right

  echo "== the calls"
  palette "marley: open system one calls"
  settle 3
  shot 548-04-calls
  expect "a compatible row" test "$(rows_of compatible)" = 1
  expect "a cloudflare row" test "$(rows_of cloudflare)" = 1
  expect "the cloudflare row's model" test "$(cat "$E2E_PROFILE"/system_one/calls-*.jsonl | grep '"provider":"cloudflare"' | grep -c '"model":"typesafe/jev"')" = 1

  echo "== a refused token"
  check_in "$PRIVATE_ROW_X" "$PRIVATE_ROW_Y" "false refuse-token"
  palette "marley: open system one calls"
  settle 2
  shot 548-05-refused
  expect "the refused request reached Cloudflare" test "$(requests_at /ai/run)" = 2

  echo "== no account"
  system_one_setting '{"cloudflare_account_id": ""}'
  settle 3
  check_in "$PRIVATE_ROW_X" "$PRIVATE_ROW_Y" false
  shot 548-06-no-account
  expect "no request without an account" test "$(requests_at /ai/run)" = 2

  echo "== the providers"
  palette "marley: open settings"
  settle 4
  click "$SETTINGS_SEARCH_X" "$SETTINGS_SEARCH_Y"
  settle 1
  type_text "Cloudflare"
  settle 3
  shot 548-01-search
  click "$PROVIDER_X" "$PROVIDER_Y"
  settle 2
  shot 548-01-providers
  grep -o '"path": "[^"]*"' "$E2E_WORK/requests.log"
}
