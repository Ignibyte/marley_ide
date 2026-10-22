# shellcheck shell=bash
# #862's visual check: the panel buttons as one labeled group at the bottom row's right. A scratch
# repository with a changed file.
# - At start: Home, then every dock's panels, each with its icon and name, and no other panel
#   buttons on the row (`862-01-group`, REQ-001).
# - `git panel: toggle focus`, which opens Changes as a tab: its button's dot (`862-02-open`,
#   REQ-002).
# - Marley Day (`862-03-day`, REQ-001).
compositor sway

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
  pointer_to 900 600
  settle 1
  shot 862-01-group

  echo "== Changes as a tab"
  palette "git panel: toggle focus" 3
  pointer_to 900 600
  settle 1
  shot 862-02-open

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 862-03-day
}
