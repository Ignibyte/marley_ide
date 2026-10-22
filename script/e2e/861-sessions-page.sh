# shellcheck shell=bash
# #861's visual check: the Sessions page. The run's own tmux server (#880) with two sessions,
# `build` in the scratch repository and `notes` in the home folder; `marley.tmux_hosts` names this
# machine.
# - The rail's Sessions view: Open as a Page under the hosts (`861-00-rail`, REQ-003).
# - Open as a Page: the header's counts, a row per session, the placing line and the
#   hosts (`861-01-page`, REQ-001).
# - Close… on `notes`, confirmed: its row gone (`861-02-closed`, REQ-002).
# - Marley Day (`861-03-day`, REQ-001).
compositor sway

STOP_AT=${STOP_AT:-0}
# The rail's Sessions view button, and Open as a Page under its hosts, from the first run's shots.
VIEW_X=${VIEW_X:-63}
VIEW_Y=${VIEW_Y:-17}
LINK_X=${LINK_X:-60}
LINK_Y=${LINK_Y:-315}
# Close… on the second row, from the first run's shot.
CLOSE_X=${CLOSE_X:-1529}
CLOSE_Y=${CLOSE_Y:-202}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  tmux new-session -d -s build -c "$repo"
  tmux new-session -d -s notes -c "$home"
  profile_setting marley.tmux_hosts '["local"]'
  open_path "$repo"
}

teardown() {
  tmux kill-server 2>/dev/null || true
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 12

  echo "== the rail's Sessions view"
  click "$VIEW_X" "$VIEW_Y"
  settle 3
  pointer_to 1500 950
  settle 1
  shot 861-00-rail
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== the Sessions page, from the rail"
  click "$LINK_X" "$LINK_Y"
  settle 4
  pointer_to 1500 950
  settle 1
  shot 861-01-page
  if [[ $STOP_AT == 2 ]]; then
    return
  fi

  echo "== Close… on notes, confirmed"
  click "$CLOSE_X" "$CLOSE_Y"
  settle 2
  press "" Return
  settle 14
  pointer_to 1500 950
  settle 1
  shot 861-02-closed
  tmux list-sessions || true

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 861-03-day
}
