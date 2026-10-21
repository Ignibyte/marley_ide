# shellcheck shell=bash
# #873's visual check: the rail's view switch as icons. A scratch repository.
# - At start: the header's folder, thread and terminal icons before Files, the folder pressed
#   (`873-01-projects`, REQ-001).
# - The thread icon: Threads shown, its icon pressed (`873-02-threads`, REQ-001, REQ-002).
# - Marley Day, the terminal icon: Sessions shown (`873-03-day`, REQ-001, REQ-002).
compositor sway

AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-600}
# The header's icons, from the probe run's shot.
THREADS_X=${THREADS_X:-39}
SESSIONS_X=${SESSIONS_X:-63}
ICONS_Y=${ICONS_Y:-17}
STOP_AT_START=${STOP_AT_START:-0}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 873-01-projects
  if [[ $STOP_AT_START == 1 ]]; then
    return
  fi

  echo "== the thread icon"
  click "$THREADS_X" "$ICONS_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 873-02-threads

  echo "== Marley Day, the terminal icon"
  profile_setting theme '"Marley Day"'
  settle 3
  click "$SESSIONS_X" "$ICONS_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 873-03-day
}
