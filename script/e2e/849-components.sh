# shellcheck shell=bash
# #849's visual check: the Component Library as its board draws it. A scratch repository.
# - `marley: open component library`: the navigation column with Find a component, the groups
#   and their names, the cards (`849-01-library`, REQ-001).
# - "stat" typed in the find field: the column and the cards narrowed to it (`849-02-found`,
#   REQ-002).
# - Marley Day (`849-03-day`, REQ-001).
compositor sway

STOP_AT=${STOP_AT:-0}
# The navigation column's find field, from the first run's shot.
FIND_X=${FIND_X:-370}
FIND_Y=${FIND_Y:-86}

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
  settle 4

  echo "== the Component Library"
  palette "marley: open component library" 5
  pointer_to 1500 950
  settle 1
  shot 849-01-library
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== stat"
  click "$FIND_X" "$FIND_Y"
  settle 1
  type_text "stat"
  settle 2
  pointer_to 1500 950
  settle 1
  shot 849-02-found

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 849-03-day
}
