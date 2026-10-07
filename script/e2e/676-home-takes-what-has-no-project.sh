# shellcheck shell=bash
# #676's visual check: the screens that belong to no project open in the window's Home group.
# System One calls stands for the three (the Agent tab and a harness session's tab take the same
# path, `groups::in_group`, and would need a fleet and a harness of their own here).
#
# `676-01-calls`: `marley: open system one calls` from the project: a Home group, System One calls
# its row and in front (REQ-001). `676-02-project`: the project's terminal clicked: in front, the
# calls still under Home (REQ-002). `676-03-again`: the action again from the project: Home shown
# with the same one tab, and one Home group (REQ-003).
compositor sway

# In the window's logical pixels, from the first run's shots: the project's terminal row, and a
# point clear of the rail.
TERMINAL_X=${TERMINAL_X:-140}
TERMINAL_Y=${TERMINAL_Y:-184}
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-600}

setup() {
  mkdir -p "$E2E_WORK/repo" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 3
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== System One calls from the project"
  palette "marley: open system one calls"
  away
  shot 676-01-calls

  echo "== the project's terminal"
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 2
  away
  shot 676-02-project

  echo "== System One calls again"
  palette "marley: open system one calls"
  away
  shot 676-03-again
}
