# shellcheck shell=bash
# #741's visual check: a new agent on a remote host. A real runtime of the harness's built `rh` on a
# scratch root is `marley.harnesses`' `box-2`, reached through #740's fake `ssh`, which logs its
# arguments, drops everything up to the destination and runs the rest in a shell. A stand-in `rh`
# answers `seat add`, and `seat start` by opening an actor session of the seat's name and answering
# with its views, `attach ws-NAME` among them; it answers `attach WS` by printing `attached: WS`.
# A fake `claude` on the PATH makes Claude Code a CLI choice.
# - Ctrl+Alt+N → Claude Code: Where lists On box-2… after Browse… (`where`, REQ-001).
# - On box-2… → `/srv/other` → Enter: `seat add claude-other` and `seat start claude-other` reached
#   box-2 through ssh (REQ-002), and a new terminal runs `ssh -t … box-2 … attach ws-claude-other`
#   and shows `attached: ws-claude-other` (`attached`, REQ-003).
# - The seat's row under HARNESS · box-2 → its tab → Views → Open: the view runs through
#   `ssh -t … box-2` (`view`, REQ-004).
compositor sway

# The seat's row in the rail, the tab's Views, and the first view's Open, as the first run's shots
# found them.
SEAT_ROW_X=${SEAT_ROW_X:-90}
SEAT_ROW_Y=${SEAT_ROW_Y:-306}
VIEWS_X=${VIEWS_X:-1568}
VIEWS_Y=${VIEWS_Y:-90}
OPEN_X=${OPEN_X:-327}
OPEN_Y=${OPEN_Y:-119}

RH=${RH:-/srv/stacks/rustal-harness/bin/rh}

# The harness's CLI on the scratch root, outside any tmux.
rh() { env -u TMUX -u TMUX_PANE "$RH" --state "$ROOT" "$@"; }

bytes() { python3 -c 'import json, sys; print(json.dumps(list(sys.argv[1].encode())))' "$1"; }

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin tries=50
  [[ -x /srv/stacks/rustal-harness/target/debug/rh ]] || {
    echo "no built rh: build the harness first (its scripts/setup.sh)" >&2
    return 1
  }
  # A short root: the harness's sockets live under it.
  ROOT=$XDG_RUNTIME_DIR/rh741-$$
  echo "$ROOT" >"$E2E_WORK/root"
  mkdir -m 0700 "$ROOT"
  (rh serve >"$ROOT/serve.log" 2>&1 &)
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
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
  cat >"$bin/rh" <<SH
#!/bin/bash
# The stand-in: \`seat\` and \`attach\` from the script, everything else to the harness's own rh.
real() { env -u TMUX -u TMUX_PANE "$RH" "\$@"; }
if [[ \$3 == attach ]]; then
  printf 'attached: %s\n' "\$4"
  exec sleep 600
fi
if [[ \$3 == seat ]]; then
  printf '%s\n' "\$*" >>"$E2E_WORK/seat.log"
  case \$4 in
    add)
      printf '{"seat": "%s", "profile": "%s/profiles/%s.json", "kind": "claude", "cwd": "%s", "binary": "/stand-in/claude"}\n' "\$5" "\$2" "\$5" "\$9"
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
      printf '{"id": "%s", "title": "%s", "profile": "%s", "opened": true, "state": "working", "views": [' "\$id" "\$5" "\$5"
      printf '{"kind": "native", "argv": ["%s", "--state", "%s", "view", "ws-%s"], "input": "after a claim"}, ' "\$0" "\$2" "\$5"
      printf '{"kind": "tmux", "argv": ["%s", "--state", "%s", "attach", "ws-%s"], "input": "direct"}]}\n' "\$0" "\$2" "\$5"
      ;;
  esac
  exit 0
fi
exec env -u TMUX -u TMUX_PANE "$RH" "\$@"
SH
  # The fake ssh: logs its arguments, then runs the words after the destination as the host's
  # shell would.
  cat >"$bin/ssh" <<SH
#!/bin/bash
printf '%s\n' "\$*" >>"$E2E_WORK/ssh.log"
while [[ \$# -gt 0 && \$1 != -- ]]; do shift; done
shift
shift
exec bash -c "\$*"
SH
  cat >"$bin/claude" <<'SH'
#!/bin/sh
exec sleep 600
SH
  chmod +x "$bin/rh" "$bin/ssh" "$bin/claude"
  : >"$E2E_WORK/ssh.log"
  # Marley's own search path, and its terminals', find the fakes first.
  export PATH="$bin:$PATH"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  echo "export PATH=\"$bin:\$PATH\"" >>"$home/.bashrc"
  terminal_env HOME "$home"
  profile_setting marley.harnesses "[{\"name\": \"box-2\", \"ssh\": \"box-2\", \"state\": \"$ROOT\", \"rh\": \"$bin/rh\"}]"
  profile_setting marley.harness_writes true
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

# Whether the seat went to box-2's root through ssh, named for the agent and the folder.
seat_went_to_box_2() {
  local root
  root=$(cat "$E2E_WORK/root")
  cat "$E2E_WORK/ssh.log"
  holds "$E2E_WORK/ssh.log" \
    "-- box-2 $E2E_WORK/bin/rh --state $root seat add claude-other --agent claude --cwd /srv/other" \
    "-- box-2 $E2E_WORK/bin/rh --state $root seat start claude-other"
}

# Whether the terminal's attach went through `ssh -t` to box-2.
attach_went_through_ssh() {
  grep -E -- "^-t .*-- box-2 .*attach ws-claude-other" "$E2E_WORK/ssh.log"
}

# Whether a view of the seat's tab went through `ssh -t` to box-2.
view_went_through_ssh() {
  grep -E -- "^-t .*-- box-2 .* view " "$E2E_WORK/ssh.log"
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== Ctrl+Alt+N → Claude Code: Where"
  press "CTRL ALT" n
  settle 2
  type_text "Claude Code"
  settle 1
  press "" Return
  settle 2
  shot 741-01-where

  echo "== On box-2… → /srv/other"
  type_text "box-2"
  settle 1
  press "" Return
  settle 2
  type_text "/srv/other"
  settle 1
  shot 741-02a-folder
  press "" Return
  settle 10
  shot 741-02-attached
  expect "the seat went to box-2 through ssh" seat_went_to_box_2
  expect "the terminal attached through ssh -t" attach_went_through_ssh

  echo "== the seat's tab → Views → Open"
  click "$SEAT_ROW_X" "$SEAT_ROW_Y"
  settle 3
  click "$VIEWS_X" "$VIEWS_Y"
  settle 3
  shot 741-03a-views
  click "$OPEN_X" "$OPEN_Y"
  settle 4
  shot 741-03-view
  expect "the view ran through ssh -t" view_went_through_ssh
}
