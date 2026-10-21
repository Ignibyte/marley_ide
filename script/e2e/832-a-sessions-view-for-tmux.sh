# shellcheck shell=bash
# #832's visual check: the rail's Sessions view. A tmux server of the run's own (`TMUX_TMPDIR`)
# has `build`, started in `repo`, and `loose`, started in the run's home; `marley.tmux_hosts` is
# unset, so this machine is looked on. `forge` is a saved ssh host behind a stand-in ssh that is
# refused, not looked on.
# - SESSIONS clicked: this machine with 2, `build` under repo and `loose` under Home, both
#   detached, and HOSTS TO LOOK ON with this machine ticked and `forge` not (`832-01-sessions`,
#   REQ-001, REQ-002, REQ-005).
# - `build` clicked: it attaches in a tab and reads `attached here` (`832-02-attached`, REQ-003).
# - `loose`'s menu, Close Session…: the question (`832-03-close-asks`), End Session: `loose` gone
#   from the list and from tmux (`832-04-closed`, REQ-004).
# - `forge` ticked: `marley.tmux_hosts` printed, and `forge`'s heading with its failure
#   (`832-05-host-added`, REQ-005).
# - The rail shrunk: the Sessions tile with its count (`832-06-strip`); the tile clicked: the rail
#   open on Sessions (`832-07-from-strip`, REQ-006).
# - Marley Day: the view (`832-08-day`, REQ-001).
compositor sway

AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the probe run's shots.
SESSIONS_X=${SESSIONS_X:-145}
SESSIONS_Y=${SESSIONS_Y:-17}
ROW_X=${ROW_X:-130}
BUILD_Y=${BUILD_Y:-96}
LOOSE_Y=${LOOSE_Y:-142}
CLOSE_ENTRY_DY=${CLOSE_ENTRY_DY:-40}
FORGE_X=${FORGE_X:-26}
FORGE_Y=${FORGE_Y:-190}
STRIP_SESSIONS_X=${STRIP_SESSIONS_X:-30}
STRIP_SESSIONS_Y=${STRIP_SESSIONS_Y:-181}

# tmux's socket path must stay under 108 bytes, which a run's work folder does not (#808).
SOCKETS=${XDG_RUNTIME_DIR:-/tmp}/marley-e2e-832
LOCAL_TMUX=$SOCKETS/local

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo" "$LOCAL_TMUX"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  terminal_env TMUX_TMPDIR "$LOCAL_TMUX"
  export TMUX_TMPDIR=$LOCAL_TMUX
  cat >"$bin/ssh" <<'SSH'
#!/bin/bash
echo "ssh: connect to host forge port 22: Connection refused" >&2
exit 255
SSH
  chmod +x "$bin/ssh"
  export MARLEY_SSH=$bin/ssh
  TMUX_TMPDIR=$LOCAL_TMUX HOME=$home tmux -f /dev/null new-session -d -s build -c "$E2E_WORK/repo" \
    -x 100 -y 30
  TMUX_TMPDIR=$LOCAL_TMUX HOME=$home tmux -f /dev/null new-session -d -s loose -c "$home" \
    -x 100 -y 30
  profile_setting ssh_connections '[{"host": "forge"}]'
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  TMUX_TMPDIR=$LOCAL_TMUX tmux kill-server 2>/dev/null || true
  rm -rf "$SOCKETS"
}

# `marley.tmux_hosts` in the run's copy of the settings, as Marley wrote it.
hosts_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY'
import json, pathlib, re, sys

text = pathlib.Path(sys.argv[1]).read_text()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
print(json.loads(text).get("marley", {}).get("tmux_hosts"))
PY
}

steps() {
  settle 12
  # Trusts repo.
  press "" Return
  settle 4

  echo "== Sessions"
  click "$SESSIONS_X" "$SESSIONS_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 832-01-sessions

  echo "== build attached"
  click "$ROW_X" "$BUILD_Y"
  settle 5
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 832-02-attached

  echo "== loose closed"
  click "$ROW_X" "$LOOSE_Y" right
  settle 2
  shot 832-03-menu
  click "$((ROW_X + 40))" "$((LOOSE_Y + CLOSE_ENTRY_DY))"
  settle 2
  shot 832-03-close-asks
  press "" Return
  settle 4
  echo "tmux ls: $(TMUX_TMPDIR=$LOCAL_TMUX tmux ls -F '#{session_name}' 2>&1 | paste -sd ' ')"
  shot 832-04-closed

  echo "== forge ticked"
  click "$FORGE_X" "$FORGE_Y"
  settle 6
  echo "tmux_hosts: $(hosts_setting)"
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 832-05-host-added

  echo "== the strip"
  profile_setting marley.rail_shrunk true
  settle 3
  shot 832-06-strip
  click "$STRIP_SESSIONS_X" "$STRIP_SESSIONS_Y"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 832-07-from-strip

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 832-08-day
}
