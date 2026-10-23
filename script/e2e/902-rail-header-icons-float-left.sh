# shellcheck shell=bash
# #902's visual check: the rail header's six icons together from the left. A scratch project.
# - `902-01-night`: Projects, Threads, Sessions, Files, Add Project and Shrink from the left
#   (REQ-001).
# - `902-02-day`: the same in Marley Day (REQ-001).
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

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  pointer_to 900 600
  settle 1
  shot 902-01-night
  profile_setting theme '"Marley Day"'
  settle 4
  shot 902-02-day
}
