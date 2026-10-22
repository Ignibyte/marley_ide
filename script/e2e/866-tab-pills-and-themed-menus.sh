# shellcheck shell=bash
# #866's visual check: tabs with their kind and state, the + menu's keys, and the split menu drawn.
# A scratch repository with a source file; the terminal runs `false`, so its last command failed.
# - The tab bar: the terminal's ">_" tile and its error dot, the file's "rs" tile
#   (`866-01-tabs`, REQ-001).
# - The project's + menu: New Terminal and New Agent with their keys (`866-02-plus`, REQ-002).
# - The tab bar's split menu: each split drawn as a small picture, with its key (`866-03-split`,
#   REQ-003).
# - Marley Day, the tab bar (`866-04-day`, REQ-001).
compositor sway

# The project's + and the tab bar's split button, as the probe run measured them.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-136}
SPLIT_X=${SPLIT_X:-1557}
SPLIT_Y=${SPLIT_Y:-50}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo/src"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf 'fn main() {}\n' >"$repo/src/main.rs"
  git init -q -b main "$repo"
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4
  type_text "false"
  press "" Return
  settle 2
  press CTRL p
  settle 2
  type_text "main.rs"
  settle 2
  press "" Return
  settle 3
  pointer_to 900 600
  settle 1
  shot 866-01-tabs

  echo "== the project's + menu"
  click "$PLUS_X" "$PLUS_Y"
  settle 2
  pointer_to 900 600
  settle 1
  shot 866-02-plus
  press "" Escape
  settle 1

  echo "== the split menu"
  click "$SPLIT_X" "$SPLIT_Y"
  settle 2
  pointer_to 900 600
  settle 1
  shot 866-03-split
  press "" Escape
  settle 1

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 866-04-day
}
