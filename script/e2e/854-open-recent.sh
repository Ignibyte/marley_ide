# shellcheck shell=bash
# #854's visual check: Open Recent as the SearchMore board draws it. A scratch repository; the
# run's copy of the database lists the recent projects. The scenario only looks: Escape closes the
# picker, so no project opens.
# - `projects: open recent`: each project's tile, its name over its folder, and where it is
#   (`854-01-recent`, REQ-001).
# - Marley Day (`854-02-day`, REQ-001).
compositor sway

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

  echo "== Open Recent"
  palette "projects: open recent" 3
  pointer_to 1500 950
  settle 1
  shot 854-01-recent
  press "" Escape
  settle 1

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  palette "projects: open recent" 3
  pointer_to 1500 950
  settle 1
  shot 854-02-day
  press "" Escape
  settle 1
}
