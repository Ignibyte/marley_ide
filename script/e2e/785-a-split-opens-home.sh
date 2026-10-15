# shellcheck shell=bash
# #785's e2e test: in the Marley layout a split of a center terminal opens Home in the new pane,
# not a second shell; Home splits into another Home; and a tab's right-click menu moves the tab
# into a new pane on the side picked, a pane's only tab leaving Home on the other side.
compositor sway

# The terminals, read through Marley's MCP server (#513).
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# Whether Marley runs exactly one terminal.
one_terminal() {
  mcp_agent terminals >"$E2E_WORK/terminals.txt" || return 1
  cat "$E2E_WORK/terminals.txt"
  [[ $(grep -c '^  terminal ' "$E2E_WORK/terminals.txt") -eq 1 ]]
}

# Picks the entry of an open menu that is `back` entries above its last.
menu_from_end() {
  local back=$1
  press "" End
  for ((; back > 0; back--)); do
    press "" Up
  done
  press "" Return
}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo"
  printf "PS1='\$ '\n" >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  # The terminal takes the focus.
  click 700 500
  settle 1.5

  echo "== the split button's Split Right on a terminal"
  click 1557 50
  settle 1
  press "" Home
  press "" Return
  settle 2
  shot 785-01-split-right
  expect "the split started no terminal" one_terminal

  echo "== pane: split down on the terminal"
  click 500 500
  settle 1
  press "CTRL SHIFT" p
  settle 1
  type_text "pane: split down"
  settle 1
  press "" Return
  settle 2
  shot 785-02-split-down
  expect "the split started no terminal" one_terminal

  echo "== the split button on Home"
  click 1048 50
  settle 1
  click 1557 50
  settle 1
  press "" Home
  press "" Return
  settle 2
  shot 785-03-split-home

  echo "== Split Down in the menu of a tab with a neighbour"
  click 500 300
  settle 1
  press CTRL p
  settle 1
  type_text "README"
  settle 1
  press "" Return
  settle 2
  shot 785-04-before
  click 530 50 right
  settle 1
  shot 785-04-menu
  menu_from_end 0
  settle 2
  shot 785-04-tab-menu

  echo "== Split Right in the menu of a pane's only tab"
  click 420 362 right
  settle 1
  shot 785-05-menu
  # Split Right is three entries above the last, Split Down.
  menu_from_end 3
  settle 2
  shot 785-05-split-only-tab

  echo "== the split button on a file"
  click 570 362
  settle 1
  click 665 362
  settle 1
  press "" Home
  press "" Return
  settle 2
  shot 785-06-file-split
}
