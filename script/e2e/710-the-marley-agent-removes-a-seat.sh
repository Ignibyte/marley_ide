# shellcheck shell=bash
# #710's visual check: the Marley agent stops and removes a harness seat. #692's set-up: a real
# runtime, and a stand-in `rh` that answers `seat` commands from this script and passes everything
# else to the harness's built `rh`. Its `seat stop` and `seat remove` answer harness TICKET-114's
# documented object, and refuse `ghost` as `seat_unknown`. The stand-in MCP client:
# - calls `seat_stop builder`; the question shows (`710-01-stop-asked`, REQ-001), and Apply answers
#   `"result": "stopped"` with `stopped` and `supervision_ended` (REQ-002);
# - calls `seat_remove builder`; the question names the profile's removal (`710-02-remove-asked`),
#   and Apply answers `"result": "removed"` with `removed` (REQ-002);
# - calls `seat_stop ghost`, which comes back as `seat_unknown` (REQ-003).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The question's Apply, as the first run's shot found it.
APPLY_X=${APPLY_X:-1172}
APPLY_Y=${APPLY_Y:-933}

RH=${RH:-/srv/stacks/rustal-harness/bin/rh}

# The harness's CLI on the scratch root, outside any tmux.
rh() { env -u TMUX -u TMUX_PANE "$RH" --state "$ROOT" "$@"; }

setup() {
  [[ -x /srv/stacks/rustal-harness/target/debug/rh ]] || {
    echo "no built rh: build the harness first (its scripts/setup.sh)" >&2
    return 1
  }
  # A short root: the harness's sockets live under it.
  ROOT=$XDG_RUNTIME_DIR/rh710-$$
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
    stop | remove)
      if [[ \$5 == ghost ]]; then
        echo "rh: seat_unknown: the root has no profile ghost" >&2
        exit 1
      fi
      printf '{"seat": "%s", "stopped": [{"id": "agent/0b6c4f2e-7d1a-4c55-8f3b-2a6e7d1c0b9a", "state": "done"}], "supervision_ended": ["agent/0b6c4f2e-7d1a-4c55-8f3b-2a6e7d1c0b9a"]' "\$5"
      [[ \$4 == remove ]] && printf ', "removed": "%s/profiles/%s.json"' "\$2" "\$5"
      printf '}\n'
      ;;
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
  local proposal
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== seat_stop asks, then stops"
  mcp_agent tool seat_stop '{"name": "builder"}' >"$E2E_WORK/stopped.txt" 2>&1 &
  proposal=$!
  settle 3
  shot 710-01-stop-asked
  click "$APPLY_X" "$APPLY_Y"
  wait "$proposal" || true
  cat "$E2E_WORK/stopped.txt"
  expect "the answer is stopped" holds "$E2E_WORK/stopped.txt" '"result": "stopped"'
  expect "the answer lists the ended supervision" holds "$E2E_WORK/stopped.txt" '"supervision_ended"'

  echo "== seat_remove asks, then removes"
  mcp_agent tool seat_remove '{"name": "builder"}' >"$E2E_WORK/removed.txt" 2>&1 &
  proposal=$!
  settle 3
  shot 710-02-remove-asked
  click "$APPLY_X" "$APPLY_Y"
  wait "$proposal" || true
  cat "$E2E_WORK/removed.txt"
  expect "the answer is removed" holds "$E2E_WORK/removed.txt" '"result": "removed"'
  expect "the answer names the removed profile" holds "$E2E_WORK/removed.txt" '/profiles/builder.json'
  expect "the stand-in ran seat stop, then seat remove" \
    holds "$E2E_WORK/seat.log" "seat stop builder" "seat remove builder"

  echo "== a seat the harness doesn't know"
  mcp_agent tool seat_stop '{"name": "ghost"}' >"$E2E_WORK/unknown.txt" 2>&1 &
  proposal=$!
  settle 3
  click "$APPLY_X" "$APPLY_Y"
  wait "$proposal" || true
  cat "$E2E_WORK/unknown.txt"
  expect "the refusal carries the harness's code" holds "$E2E_WORK/unknown.txt" "seat_unknown"
}
