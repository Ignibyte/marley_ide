# shellcheck shell=bash
# #740's visual check: one harness per host. Two real runtimes of the harness's built `rh`, each on a
# scratch root with one session: root A is `marley.harness`, followed as before; root B is
# `marley.harnesses`' `box-2`, reached through a fake `ssh` first on the PATH that logs its
# arguments, drops everything up to the destination and runs the rest in a shell, as sshd does. A
# stand-in `rh` answers `seat` commands as #710's does and passes everything else to the real one.
# - The rail lists `alpha` under HARNESS and `beta` under HARNESS · box-2 (`two`, REQ-001).
# - The seat form, Harness box-2, Create: `seat add builder` reached root B through the fake ssh,
#   and `builder` is listed under box-2 (`seat`, REQ-002).
# - box-2's link drops and its host refuses new ones: its header says why, its rows are muted, and
#   HARNESS stays connected (`one-down`, REQ-003).
compositor sway

# The seat form's box-2 button, as the first run's shot found it.
BOX2_X=${BOX2_X:-625}
BOX2_Y=${BOX2_Y:-205}

RH=${RH:-/srv/stacks/rustal-harness/bin/rh}

# The harness's CLI on root `$1`, outside any tmux.
rh_on() {
  local root=$1
  shift
  env -u TMUX -u TMUX_PANE "$RH" --state "$root" "$@"
}

# Starts a runtime on root `$1` and opens a session `$2` in it.
start_root() {
  local root=$1 title=$2 tries=50
  mkdir -m 0700 "$root"
  (rh_on "$root" serve >"$root/serve.log" 2>&1 &)
  until grep -q '"ready":true' "$root/serve.log" 2>/dev/null || ((tries-- == 0)); do
    sleep 0.2
  done
  cat >"$root/seat.json" <<JSON
{"version": 1, "steps": [
  {"step": "output", "bytes": $(bytes $'the seat is up\n')},
  {"step": "publish", "state": "running", "value": {}},
  {"step": "sleep", "millis": 60000}, {"step": "sleep", "millis": 60000},
  {"step": "sleep", "millis": 60000}]}
JSON
  rh_on "$root" actor new "$title" --script "$root/seat.json" --cwd "$root" >/dev/null
}

bytes() { python3 -c 'import json, sys; print(json.dumps(list(sys.argv[1].encode())))' "$1"; }

setup() {
  [[ -x /srv/stacks/rustal-harness/target/debug/rh ]] || {
    echo "no built rh: build the harness first (its scripts/setup.sh)" >&2
    return 1
  }
  # Short roots: the harness's sockets live under them.
  ROOT_A=$XDG_RUNTIME_DIR/rh740a-$$
  ROOT_B=$XDG_RUNTIME_DIR/rh740b-$$
  printf '%s\n%s\n' "$ROOT_A" "$ROOT_B" >"$E2E_WORK/roots"
  start_root "$ROOT_A" alpha
  start_root "$ROOT_B" beta
  mkdir -p "$E2E_WORK/bin"
  cat >"$E2E_WORK/bin/rh" <<SH
#!/bin/bash
# The stand-in: \`seat\` from the script, everything else to the harness's own rh.
real() { env -u TMUX -u TMUX_PANE "$RH" "\$@"; }
if [[ \$3 == seat ]]; then
  printf '%s\n' "\$*" >>"$E2E_WORK/seat.log"
  case \$4 in
    add)
      printf '{"seat": "%s", "profile": "%s/profiles/%s.json", "kind": "claude", "cwd": "%s", "binary": "/stand-in/claude"}\n' "\$5" "\$2" "\$5" "\$2"
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
  # The fake ssh: logs its arguments, then runs the words after the destination as the host's
  # shell would, unless the host is marked down.
  cat >"$E2E_WORK/bin/ssh" <<SH
#!/bin/bash
printf '%q ' "\$@" >>"$E2E_WORK/ssh.log"
echo >>"$E2E_WORK/ssh.log"
while [[ \$# -gt 0 && \$1 != -- ]]; do shift; done
shift
host=\$1
shift
if [[ -f $E2E_WORK/\$host-down ]]; then
  echo "ssh: connect to host \$host port 22: Connection refused" >&2
  exit 255
fi
exec bash -c "\$*"
SH
  chmod +x "$E2E_WORK/bin/rh" "$E2E_WORK/bin/ssh"
  : >"$E2E_WORK/ssh.log"
  # Marley's own search path finds the fake ssh first.
  export PATH="$E2E_WORK/bin:$PATH"
  profile_setting marley.harness "{\"command\": \"$E2E_WORK/bin/rh\", \"args\": [\"--state\", \"$ROOT_A\", \"mcp\", \"--grant\", \"write\"]}"
  profile_setting marley.harnesses "[{\"name\": \"box-2\", \"ssh\": \"box-2\", \"state\": \"$ROOT_B\", \"rh\": \"$E2E_WORK/bin/rh\"}]"
  profile_setting marley.harness_writes true
  mkdir -p "$E2E_WORK/repo" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

teardown() {
  [[ -f $E2E_WORK/roots ]] || return 0
  local root ws
  while read -r root; do
    [[ -d $root ]] || continue
    chmod 0700 "$root" 2>/dev/null
    for ws in $(rh_on "$root" list 2>/dev/null | python3 -c 'import json, sys
print(" ".join(w["id"] for w in json.load(sys.stdin).get("workspaces", []) if w.get("state") == "running"))'); do
      rh_on "$root" stop "$ws" >/dev/null 2>&1
    done
    rh_on "$root" shutdown --stop-backend >/dev/null 2>&1
    rm -rf "$root"
  done <"$E2E_WORK/roots"
  return 0
}

# Whether the seat commands went to root B through the fake ssh to box-2.
seat_went_to_box_2() {
  local root_b
  root_b=$(sed -n 2p "$E2E_WORK/roots")
  cat "$E2E_WORK/ssh.log"
  cat "$E2E_WORK/seat.log"
  holds "$E2E_WORK/ssh.log" "-- box-2 $E2E_WORK/bin/rh --state $root_b seat add builder" &&
    holds "$E2E_WORK/seat.log" "--state $root_b seat start builder"
}

steps() {
  local root_b
  root_b=$(sed -n 2p "$E2E_WORK/roots")
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4
  expect "box-2 is followed through ssh" holds "$E2E_WORK/ssh.log" "-- box-2 $E2E_WORK/bin/rh --state $root_b mcp --grant write"
  shot 740-01-two

  echo "== a seat on box-2"
  press "CTRL SHIFT" p
  settle 1
  type_text "marley: new harness seat"
  settle 1
  press "" Return
  settle 2
  type_text "builder"
  click "$BOX2_X" "$BOX2_Y"
  settle 1
  shot 740-02a-form
  press "" Return
  settle 8
  shot 740-02-seat
  expect "the seat commands went to box-2's root through ssh" seat_went_to_box_2

  echo "== box-2's link drops"
  touch "$E2E_WORK/box-2-down"
  pkill -f -- "--state $root_b mcp" || true
  settle 5
  shot 740-03-one-down
}
