# shellcheck shell=bash
# #821's visual check: tmux sessions as rows under their project, and the question when an
# attached tab closes. A tmux server of the run's own (`TMUX_TMPDIR`) has `build`, started in
# `repo`, and `loose`, started in the run's home; `marley.tmux_hosts` is unset, so this machine is
# listed. A fake ssh (`MARLEY_SSH`) refuses the saved host `gone`.
# - `build` under `repo`, `loose` under Default (Home), no TMUX section (`821-01-placed`, REQ-001,
#   REQ-002, REQ-007).
# - A click on `build`: a tab attached to it, and `build` only as that tab's terminal row
#   (`821-02-attached`, REQ-003, REQ-004).
# - The tab closed: the question, Close Session, Hide, Cancel (`821-03-asked`, REQ-005).
# - Hide: the tab gone, `build` still running and its row back (`821-04-hidden`, REQ-006).
# - `build` attached again and closed with Close Session: the session ended, its row gone
#   (`821-05-closed`, REQ-005).
# - `loose` attached and its close cancelled: the tab still there (`821-06-cancel`, REQ-005).
# - `tmux_hosts` naming `gone` too: its error row under Default (`821-07-host-down`, REQ-008).
compositor sway

# The rows and the question's buttons, as the first run's shots found them.
ROW_X=${ROW_X:-130}
BUILD_Y=${BUILD_Y:-280}
LOOSE_Y=${LOOSE_Y:-130}
# The question's buttons stand in a column in the middle of the window.
BUTTONS_X=${BUTTONS_X:-800}
CLOSE_SESSION_Y=${CLOSE_SESSION_Y:-532}
HIDE_Y=${HIDE_Y:-558}
CANCEL_Y=${CANCEL_Y:-584}

# tmux's socket path must stay under 108 bytes, which a run's work folder does not (#808).
SOCKETS=${XDG_RUNTIME_DIR:-/tmp}/marley-e2e-821
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
echo "ssh: connect to host gone port 22: Connection refused" >&2
exit 255
SSH
  chmod +x "$bin/ssh"
  export MARLEY_SSH=$bin/ssh
  TMUX_TMPDIR=$LOCAL_TMUX HOME=$home tmux -f /dev/null new-session -d -s build -c "$E2E_WORK/repo" \
    -x 100 -y 30
  TMUX_TMPDIR=$LOCAL_TMUX HOME=$home tmux -f /dev/null new-session -d -s loose -c "$home" \
    -x 100 -y 30
  profile_setting ssh_connections '[{"host": "gone"}]'
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  TMUX_TMPDIR=$LOCAL_TMUX tmux kill-server 2>/dev/null || true
  rm -rf "$SOCKETS"
}

# Runs `$1` from the command palette, as #798's scenario does.
palette() {
  press "CTRL SHIFT" p
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "$1" -s 800 -k Return
}

# Whether the run's tmux has the session `$1`.
has_session() {
  TMUX_TMPDIR=$LOCAL_TMUX tmux has-session -t "=$1" 2>/dev/null
}

lacks_session() {
  ! has_session "$1"
}

# Whether a tmux client is attached to the session `$1`.
attached_to() {
  TMUX_TMPDIR=$LOCAL_TMUX tmux list-clients -F '#{session_name}' | grep -qx "$1"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 6
  shot 821-01-placed

  echo "== a click on build attaches it in repo"
  click "$ROW_X" "$BUILD_Y"
  settle 4
  expect "a client attached build" attached_to build
  shot 821-02-attached

  echo "== the tab closed: the question"
  palette "pane: close active item"
  settle 2
  shot 821-03-asked

  echo "== Hide"
  click "$BUTTONS_X" "$HIDE_Y"
  settle 12
  expect "build still runs" has_session build
  shot 821-04-hidden

  echo "== build again, closed with Close Session"
  click "$ROW_X" "$BUILD_Y"
  settle 4
  palette "pane: close active item"
  settle 2
  click "$BUTTONS_X" "$CLOSE_SESSION_Y"
  settle 4
  expect "build ended" lacks_session build
  settle 8
  shot 821-05-closed

  echo "== loose attached, its close cancelled"
  click "$ROW_X" "$LOOSE_Y"
  settle 4
  palette "pane: close active item"
  settle 2
  click "$BUTTONS_X" "$CANCEL_Y"
  settle 2
  expect "loose still attached" attached_to loose
  shot 821-06-cancel

  echo "== tmux_hosts names gone too"
  profile_setting marley.tmux_hosts '["local", "gone"]'
  settle 8
  shot 821-07-host-down
}
