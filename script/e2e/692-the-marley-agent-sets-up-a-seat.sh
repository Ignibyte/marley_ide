# shellcheck shell=bash
# #692's visual check: the Marley agent sets up a harness seat. #691's stand-in `rh` answers `seat
# add` and `seat start` from this script (its start opens a real actor under the seat's name) and
# passes everything else to the harness's built `rh`. The stand-in MCP client, through the Claude
# Code plugin's bridge, calls `seat_add` for `builder` on Codex as the manager: the question shows
# (`692-01-card`, REQ-001); Apply answers `starting`, the stand-in ran add then start, and the rail
# lists builder (`692-02-started`, REQ-002); a seat with the role foreman comes back with the
# harness's `seat_role_reserved` (REQ-003).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The question's Apply, as #682's run found it.
APPLY_X=${APPLY_X:-958}
APPLY_Y=${APPLY_Y:-901}

RH=${RH:-/srv/stacks/rustal-harness/bin/rh}

# The harness's CLI on the scratch root, outside any tmux.
rh() { env -u TMUX -u TMUX_PANE "$RH" --state "$ROOT" "$@"; }

# `$1` as the JSON array of its bytes, for an actor's `output` step.
bytes() { python3 -c 'import json, sys; print(json.dumps(list(sys.argv[1].encode())))' "$1"; }

setup() {
  [[ -x /srv/stacks/rustal-harness/target/debug/rh ]] || {
    echo "no built rh: build the harness first (its scripts/setup.sh)" >&2
    return 1
  }
  # A short root: the harness's sockets live under it.
  ROOT=$XDG_RUNTIME_DIR/rh692-$$
  echo "$ROOT" >"$E2E_WORK/root"
  mkdir -m 0700 "$ROOT"
  (rh serve >"$ROOT/serve.log" 2>&1 &)
  local tries=50
  until grep -q '"ready":true' "$ROOT/serve.log" 2>/dev/null || ((tries-- == 0)); do
    sleep 0.2
  done
  cat >"$ROOT/seat.json" <<JSON
{"version": 1, "steps": [
  {"step": "output", "bytes": $(bytes $'the seat is up\n')},
  {"step": "publish", "state": "running", "value": {}},
  {"step": "sleep", "millis": 60000}, {"step": "sleep", "millis": 60000},
  {"step": "sleep", "millis": 60000}]}
JSON
  mkdir -p "$E2E_WORK/bin"
  cat >"$E2E_WORK/bin/rh" <<SH
#!/bin/bash
# The stand-in: \`seat\` from the script, everything else to the harness's own rh.
real() { env -u TMUX -u TMUX_PANE "$RH" "\$@"; }
if [[ \$3 == seat ]]; then
  printf '%s\n' "\$*" >>"$E2E_WORK/seat.log"
  case \$4 in
    add)
      if [[ " \$* " == *" --role foreman "* ]]; then
        echo "rh: seat_role_reserved: the foreman's seat waits for TICKET-113" >&2
        exit 1
      fi
      printf '{"seat": "%s", "profile": "%s/profiles/%s.json", "kind": "codex", "cwd": "%s", "binary": "/stand-in/codex"}\n' "\$5" "\$2" "\$5" "\$2"
      ;;
    start)
      real --state "\$2" actor new "\$5" --script "\$2/seat.json" --cwd "\$2" >/dev/null
      for _ in 1 2 3 4 5 6 7 8 9 10; do
        id=\$(real --state "\$2" fleet | python3 -c 'import json, sys
fleet = json.load(sys.stdin)
for seat in fleet.get("seats", []):
    if seat.get("title") == sys.argv[1]:
        print(seat["id"])' "\$5")
        [[ -n \$id ]] && break
        sleep 0.5
      done
      printf '{"id": "%s", "title": "%s", "profile": "%s", "opened": true, "state": "working", "views": []}\n' "\$id" "\$5" "\$5"
      ;;
  esac
  exit 0
fi
exec env -u TMUX -u TMUX_PANE "$RH" "\$@"
SH
  chmod +x "$E2E_WORK/bin/rh"
  profile_setting marley.harness "{\"command\": \"$E2E_WORK/bin/rh\", \"args\": [\"--state\", \"$ROOT\", \"mcp\", \"--grant\", \"write\"]}"
  profile_setting marley.harness_writes true
  export MCP_CLIENT_NAME="Stand-in agent"
  mkdir -p "$E2E_WORK/repo" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

teardown() {
  [[ -f $E2E_WORK/root ]] || return 0
  ROOT=$(cat "$E2E_WORK/root")
  chmod 0700 "$ROOT" 2>/dev/null
  local ws
  for ws in $(rh list 2>/dev/null | python3 -c 'import json, sys
print(" ".join(w["id"] for w in json.load(sys.stdin).get("workspaces", []) if w.get("state") == "running"))'); do
    rh stop "$ws" >/dev/null 2>&1
  done
  rh shutdown --stop-backend >/dev/null 2>&1
  rm -rf "$ROOT"
  return 0
}

# Whether the stand-in ran `seat add builder` on Codex as the manager, then `seat start builder`.
seat_commands_ran() {
  local log=$E2E_WORK/seat.log root
  root=$(cat "$E2E_WORK/root")
  cat "$log"
  holds "$log" "--state $root seat add builder --agent codex --cwd $E2E_WORK/repo --role manager" \
    "--state $root seat start builder"
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== seat_add asks the user"
  mcp_agent tool seat_add \
    "{\"name\": \"builder\", \"agent\": \"codex\", \"cwd\": \"$E2E_WORK/repo\", \"role\": \"manager\"}" \
    >"$E2E_WORK/started.txt" 2>&1 &
  local proposal=$!
  settle 3
  shot 692-01-card

  echo "== Apply adds and starts it"
  click "$APPLY_X" "$APPLY_Y"
  wait "$proposal" || true
  cat "$E2E_WORK/started.txt"
  expect "the answer is starting" holds "$E2E_WORK/started.txt" '"result": "starting"'
  settle 6
  expect "the stand-in ran seat add, then seat start" seat_commands_ran
  pointer_to 900 400
  settle 1
  shot 692-02-started

  echo "== a refused seat"
  mcp_agent tool seat_add \
    "{\"name\": \"chief\", \"agent\": \"claude\", \"cwd\": \"$E2E_WORK/repo\", \"role\": \"foreman\"}" \
    >"$E2E_WORK/refused.txt" 2>&1 &
  proposal=$!
  settle 3
  click "$APPLY_X" "$APPLY_Y"
  wait "$proposal" || true
  cat "$E2E_WORK/refused.txt"
  expect "the refusal carries the harness's code" holds "$E2E_WORK/refused.txt" "seat_role_reserved"
}
