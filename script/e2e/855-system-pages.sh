# shellcheck shell=bash
# #855's visual check: the System tab with This build and Where data lives, as the SystemPages
# board draws its pages. A scratch repository.
# - `marley: open system pages`: the page list and This build (`855-01-build`, REQ-001).
# - Where data lives picked: each folder with its path and size (`855-02-data`, REQ-002).
# - Marley Day (`855-03-day`, REQ-001, REQ-002).
compositor sway

STOP_AT=${STOP_AT:-0}
# Where data lives in the tab's page list, from the first run's shot.
DATA_X=${DATA_X:-340}
DATA_Y=${DATA_Y:-150}

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

  echo "== the System tab, This build"
  palette "marley: open system pages" 3
  pointer_to 1500 950
  settle 1
  shot 855-01-build
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== Where data lives"
  click "$DATA_X" "$DATA_Y"
  settle 4
  pointer_to 1500 950
  settle 1
  shot 855-02-data

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 855-03-day
}
