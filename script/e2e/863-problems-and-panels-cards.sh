# shellcheck shell=bash
# #863's visual check: the problems and panels cards above the bottom row. A scratch crate
# rust-analyzer reads, with an error and a warning in `src/main.rs`.
# - The row's problems count clicked: the card, errors and warnings, Open in the drawer
#   (`863-01-problems`, REQ-001); Open in the drawer clicked: the Problems drawer
#   (`863-02-drawer`, REQ-001).
# - The panel group's last button: the Panels card, each panel, where it opens and its key
#   (`863-03-panels`, REQ-002), and in Marley Day (`863-04-day`, REQ-002).
compositor sway

STOP_AT=${STOP_AT:-}
# The row's problems count, the card's Open in the drawer and the group's last button, from the
# first run's shots.
COUNT_X=${COUNT_X:-458}
DRAWER_X=${DRAWER_X:-480}
DRAWER_Y=${DRAWER_Y:-958}
PANELS_X=${PANELS_X:-1578}
ROW_Y=985

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo/src"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '[package]\nname = "rail"\nversion = "0.1.0"\nedition = "2021"\n' >"$repo/Cargo.toml"
  printf 'fn main() {\n    let unused = 1;\n    missing();\n}\n' >"$repo/src/main.rs"
  git init -q -b main "$repo"
  open_path "$repo"
}

steps() {
  settle 15
  # Trusts the scratch crate.
  press "" Return
  settle 4
  press CTRL p
  settle 2
  type_text "main.rs"
  settle 2
  press "" Return
  # rust-analyzer reads the crate and std.
  settle 35

  echo "== the problems card"
  click "$COUNT_X" "$ROW_Y"
  settle 2
  # Off the button, so its tooltip leaves the card in view.
  pointer_to 900 600
  settle 2
  take 863-01-problems || return 0
  if ((DRAWER_X > 0)); then
    click "$DRAWER_X" "$DRAWER_Y"
    settle 3
  fi
  pointer_to 900 600
  settle 1
  take 863-02-drawer || return 0

  echo "== the panels card"
  click "$PANELS_X" "$ROW_Y"
  settle 2
  pointer_to 900 600
  settle 2
  take 863-03-panels || return 0
  press "" Escape
  settle 1

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  click "$PANELS_X" "$ROW_Y"
  settle 2
  pointer_to 900 600
  settle 2
  take 863-04-day || return 0
  press "" Escape
}
