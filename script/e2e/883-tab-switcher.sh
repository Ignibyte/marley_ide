# shellcheck shell=bash
# #883's visual check: the Ctrl Tab switcher as the Splits board draws it. A scratch repository
# with `rail.rs` open beside the terminal; `tab switcher: toggle` from the palette, which stays
# open with no key held.
# - The switcher: each row's tile (">_", "rs") and the line saying where the tab is, the hint
#   under the rows (`883-01-switcher`, REQ-001).
# - Marley Day (`883-02-day`, REQ-001).
compositor sway

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo/crates/rail"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf 'fn walk() {}\n' >"$repo/crates/rail/rail.rs"
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

  echo "== rail.rs open beside the terminal"
  press CTRL p
  settle 2
  type_text "rail.rs"
  settle 2
  press "" Return
  settle 3

  echo "== the switcher"
  palette "tab switcher: toggle" 3
  pointer_to 1500 950
  settle 1
  shot 883-01-switcher

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 883-02-day
  press "" Escape
  settle 1
}
