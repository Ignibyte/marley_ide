# shellcheck shell=bash
# #853's visual check: the focused pane outlined in the accent, as the Splits board draws it. A
# scratch repository with two files.
# - `rail.rs` opened and split right, the right pane focused: its accent outline, none on the left
#   (`853-01-split`, REQ-001).
# - The left pane focused: the outline moved (`853-02-other`, REQ-001).
# - Marley Day (`853-03-day`, REQ-001).
compositor sway

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf 'fn walk() {\n    let rows = mixed_rows();\n}\n' >"$repo/rail.rs"
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

  echo "== rail.rs, split right"
  press CTRL p
  settle 2
  type_text "rail.rs"
  settle 2
  press "" Return
  settle 3
  palette "pane: split right" 3
  pointer_to 1500 950
  settle 1
  shot 853-01-split

  echo "== the left pane focused"
  palette "workspace: activate pane left" 2
  pointer_to 1500 950
  settle 1
  shot 853-02-other

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 853-03-day
}
