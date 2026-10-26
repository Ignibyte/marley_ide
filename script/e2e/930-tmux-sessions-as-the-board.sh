# shellcheck shell=bash
# #930's visual check: tmux sessions as the TmuxSessions board draws them. A tmux server of the
# run's own (`TMUX_TMPDIR`, L-808) has `build` started in `repo`, `loose` in the run's home, and
# `deploy` with a client attached from outside Marley (a control-mode `tmux -C attach` held open);
# `marley.tmux_hosts` is `["local"]`. `forge` is a saved ssh host behind #832's refused stand-in
# ssh (`MARLEY_SSH`), not looked on. No agent CLI runs.
# - `930-01-rail`: the Sessions view after `build` was attached: `build` at full strength,
#   `deploy` and `loose` faded, `forge` with a grey dot and "Not looked on yet. Add to tmux
#   hosts", no checkbox block; the bottom row's "tmux: 3 · 1 detached" (REQ-001, REQ-002,
#   REQ-008).
# - `930-02-page`: the tmux chip clicked: the Sessions page, its column headers, the three chips,
#   Show on `build` and Attach on the others, "3 on 1 host · 1 attached here · 1 attached
#   elsewhere · 1 detached", and the host checkboxes (REQ-003 to REQ-007, REQ-009).
# - `930-03-show`: Show on `build`: its terminal tab in front (REQ-005).
# - `930-04-host-added`: Add to tmux hosts on `forge`: the setting printed, `forge`'s heading with
#   its failure (REQ-002).
# - `930-05-unticked`: `forge` unticked on the page: the setting printed, "not listed" on the page
#   (REQ-007).
# - `930-06-day`: the page in Marley Day (REQ-003 to REQ-007).
compositor sway

STOP_AT=${STOP_AT:-}
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the probe runs' shots.
SESSIONS_X=${SESSIONS_X:-62}
SESSIONS_Y=${SESSIONS_Y:-17}
ROW_X=${ROW_X:-130}
BUILD_Y=${BUILD_Y:-96}
TMUX_CHIP_X=${TMUX_CHIP_X:-929}
TMUX_CHIP_Y=${TMUX_CHIP_Y:-985}
SHOW_X=${SHOW_X:-1456}
SHOW_Y=${SHOW_Y:-184}
ADD_X=${ADD_X:-147}
ADD_Y=${ADD_Y:-252}
FORGE_TICK_X=${FORGE_TICK_X:-293}
FORGE_TICK_Y=${FORGE_TICK_Y:-424}

# tmux's socket path must stay under 108 bytes, which a run's work folder does not (#808).
SOCKETS=${XDG_RUNTIME_DIR:-/tmp}/marley-e2e-930
LOCAL_TMUX=$SOCKETS/local

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

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
  local session folder
  for session in build loose deploy; do
    folder=$home
    [[ $session == build ]] && folder=$E2E_WORK/repo
    TMUX_TMPDIR=$LOCAL_TMUX HOME=$home tmux -f /dev/null new-session -d -s "$session" -c "$folder" \
      -x 100 -y 30
  done
  # A client attached from outside Marley, held open on a FIFO.
  mkfifo "$E2E_WORK/deploy.fifo"
  (TMUX_TMPDIR=$LOCAL_TMUX tmux -C attach -t deploy <"$E2E_WORK/deploy.fifo" >/dev/null 2>&1 &)
  exec 7>"$E2E_WORK/deploy.fifo"
  profile_setting ssh_connections '[{"host": "forge"}]'
  profile_setting marley.tmux_hosts '["local"]'
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  exec 7>&- 2>/dev/null || true
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

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 12
  # Trusts repo.
  press "" Return
  settle 4
  echo "tmux ls: $(TMUX_TMPDIR=$LOCAL_TMUX tmux ls -F '#{session_name} #{session_attached}' 2>&1 | paste -sd ' ')"

  echo "== Sessions, build attached"
  click "$SESSIONS_X" "$SESSIONS_Y"
  settle 2
  away
  take 930-00-sessions || return 0
  click "$ROW_X" "$BUILD_Y"
  settle 5
  away
  take 930-01-rail || return 0

  echo "== the tmux chip: the Sessions page"
  click "$TMUX_CHIP_X" "$TMUX_CHIP_Y"
  settle 3
  away
  take 930-02-page || return 0

  echo "== Show on build"
  click "$SHOW_X" "$SHOW_Y"
  settle 3
  away
  take 930-03-show || return 0

  echo "== Add to tmux hosts on forge"
  click "$ADD_X" "$ADD_Y"
  settle 6
  echo "tmux_hosts: $(hosts_setting)"
  away
  take 930-04-host-added || return 0

  echo "== forge unticked on the page"
  press "CTRL SHIFT" p
  settle 2
  type_text "marley: open sessions page"
  settle 3
  press "" Return
  settle 3
  click "$FORGE_TICK_X" "$FORGE_TICK_Y"
  settle 4
  echo "tmux_hosts: $(hosts_setting)"
  away
  take 930-05-unticked || return 0

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  away
  shot 930-06-day
}
