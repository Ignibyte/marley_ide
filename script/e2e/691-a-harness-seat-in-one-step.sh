# shellcheck shell=bash
# #691's visual check: a harness seat in one step. `marley.harness` names a stand-in `rh` that
# answers `seat add` and `seat start` from this script, logging each, and passes everything else to
# the harness's built `rh` on a scratch root. Its `seat start` opens a real actor under the seat's
# name, so the tab Marley opens shows a real session, and no Claude Code or Codex is started. A
# role of `foreman` is refused as harness TICKET-109 refuses it.
#
# `marley: new harness seat` shows the form, filled in for `builder` on Codex (`691-01-form`,
# REQ-001); Enter adds and starts it and opens its tab (`691-02-opened`, REQ-002, with the log); a
# second seat with the role foreman keeps the form open with the refusal (`691-03-refused`,
# REQ-003).
compositor sway

RH=${RH:-/srv/stacks/rustal-harness/bin/rh}
# The form's Codex button and Role field, from the first run's shot; 0 stops after it.
CODEX_X=${CODEX_X:-590}
CODEX_Y=${CODEX_Y:-399}
ROLE_X=${ROLE_X:-800}
ROLE_Y=${ROLE_Y:-579}

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
  ROOT=$XDG_RUNTIME_DIR/rh691-$$
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

# Whether the stand-in ran `seat add builder` on Codex in the project's folder, then
# `seat start builder`, through the followed command's `--state <root>`.
seat_commands_ran() {
  local log=$E2E_WORK/seat.log root
  root=$(cat "$E2E_WORK/root")
  cat "$log"
  holds "$log" "--state $root seat add builder --agent codex --cwd $E2E_WORK/repo" \
    "--state $root seat start builder"
}

new_seat_form() {
  press "CTRL SHIFT" p
  settle 1
  type_text "marley: new harness seat"
  settle 1
  press "" Return
  settle 2
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the form"
  new_seat_form
  type_text "builder"
  if ((CODEX_X == 0)); then
    shot 691-01-form
    echo "the form's places are not set yet: the form shot only"
    return 0
  fi
  click "$CODEX_X" "$CODEX_Y"
  settle 1
  shot 691-01-form

  echo "== Create adds and starts the seat"
  press "" Return
  settle 8
  shot 691-02-opened
  expect "the seat commands ran through the followed command" seat_commands_ran

  echo "== a refused seat"
  new_seat_form
  type_text "chief"
  click "$ROLE_X" "$ROLE_Y"
  settle 1
  type_text "foreman"
  press "" Return
  settle 3
  shot 691-03-refused
  press "" Escape
}
