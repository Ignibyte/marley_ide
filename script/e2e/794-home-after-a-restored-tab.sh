# shellcheck shell=bash
# #794's visual check: Home stays first in the Default group when a relaunch restores the group's
# saved tabs. A scratch repository; a HOME whose `.bashrc` sets a plain prompt.
# - The Default group shown from the rail, a terminal opened in it beside Home (`794-01-before`).
# - A quit and a launch: the Default group restored with its terminal, Home first in its tabs and
#   listed in the rail (`794-02-restored`, REQ-001).
# - Home chosen and closed by hand: it stays closed while the terminal is left (`794-03-closed`,
#   REQ-002).
compositor sway

STOP_AT=${STOP_AT:-0}
# The rail's Default header, from #945's shots.
DEFAULT_X=${DEFAULT_X:-80}
DEFAULT_Y=${DEFAULT_Y:-89}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle "${2:-2}"
}

away() {
  pointer_to 1500 950
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== the Default group, with a terminal beside Home"
  click "$DEFAULT_X" "$DEFAULT_Y"
  settle 3
  palette "workspace: new terminal" 4
  type_text "echo kept"
  press "" Return
  settle 2
  away
  shot 794-01-before
  if [[ $STOP_AT == 1 ]]; then
    return
  fi
  echo "== a quit and a launch"
  quit_marley
  launch_marley_with
  settle 16
  away
  shot 794-02-restored
  echo "== Home closed by hand"
  palette "pane: activate previous item"
  palette "pane: close active item" 3
  away
  shot 794-03-closed
}
