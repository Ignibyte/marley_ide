# shellcheck shell=bash
# #602's e2e test: drag to reorder the rail. A scratch project `repo` with three terminals in its
# folders `one`, `two` and `three`, and the groups Scratch and Web made from the rail's empty space
# (#600). Web's header dragged over `repo`'s block shows its preview and a line above the block,
# with the rows in place (`line`, REQ-001, REQ-007), and dropped there lists Web first
# (`projects`, REQ-002). `three` dragged onto `one` lists three, one, two (`rows`, REQ-003); `one`
# dropped on Scratch's header stays where it was (`refused`, REQ-004). Move Project Down moves
# `repo` below Scratch (`move`, REQ-009), and a click on `two` shows it (`click`, REQ-005). Marley
# quits and starts again with no path: Web, Scratch and `repo`, with three, one, two (`restart`,
# REQ-006).
compositor sway

EMPTY_X=${EMPTY_X:-130}
EMPTY_Y=${EMPTY_Y:-760}
ROW_X=${ROW_X:-110}
PLUS_X=${PLUS_X:-236}
# The headers and rows as the first run's shots found them. At the start: `repo` at 96 with `one`,
# `two` and `three` under it, then Scratch and Web at 364.
REPO_Y=${REPO_Y:-96}
WEB_Y=${WEB_Y:-364}
WEB_TO_Y=${WEB_TO_Y:-100}
# Once Web is first: `repo` at 143, `one` 189, `two` 247, `three` 305, Scratch 364.
MOVED_REPO_Y=${MOVED_REPO_Y:-143}
ONE_Y=${ONE_Y:-189}
THREE_Y=${THREE_Y:-305}
SCRATCH_Y=${SCRATCH_Y:-364}
# Once `three` is first, `one` is at 247.
MOVED_ONE_Y=${MOVED_ONE_Y:-247}
# Once `repo` is last: `three` 236, `one` 294, `two` 352.
TWO_Y=${TWO_Y:-352}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo/one" "$E2E_WORK/repo/two" "$E2E_WORK/repo/three"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

# Picks the `$1`th entry of an open context menu with the keys: the first is selected as it opens.
menu_entry() {
  local step
  for ((step = 1; step < $1; step++)); do
    press "" Down
  done
  press "" Return
}

# Makes a group named `$1` from the empty space's menu.
new_group() {
  click "$EMPTY_X" "$EMPTY_Y" right
  settle 1
  menu_entry 1
  settle 1
  type_text "$1"
  press "" Return
  settle 4
}

# A terminal in `repo` from its +, moved into the folder `$1`.
repo_terminal() {
  click "$PLUS_X" "$REPO_Y"
  settle 1
  menu_entry 1
  settle 4
  type_text "cd $1"
  press "" Return
  settle 2
}

# Presses at `$1`,`$2` and moves to `$3`,`$4` in steps, the button still down.
drag_to() {
  local x=$1 y=$2 to_x=$3 to_y=$4 step
  pointer_to "$x" "$y"
  settle 0.3
  pointer_down
  settle 0.3
  for step in 1 2 3 4 5 6; do
    pointer_to "$((x + (to_x - x) * step / 6))" "$((y + (to_y - y) * step / 6))"
    sleep 0.1
  done
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== repo's first terminal into one, then two more"
  click "$ROW_X" "$((REPO_Y + 46))"
  settle 1
  type_text "cd one"
  press "" Return
  settle 2
  repo_terminal two
  repo_terminal three

  echo "== the groups Scratch and Web"
  new_group "Scratch"
  new_group "Web"
  shot layout

  echo "== Web's header over repo's block, then dropped"
  drag_to "$ROW_X" "$WEB_Y" "$ROW_X" "$WEB_TO_Y"
  shot line
  pointer_up
  settle 3
  shot projects

  echo "== three onto one"
  drag_to "$ROW_X" "$THREE_Y" "$ROW_X" "$ONE_Y"
  shot rows-line
  pointer_up
  settle 3
  shot rows

  echo "== one onto Scratch's header"
  drag_to "$ROW_X" "$MOVED_ONE_Y" "$ROW_X" "$SCRATCH_Y"
  shot refused-line
  pointer_up
  settle 3
  shot refused

  echo "== Move Project Down on repo"
  click "$ROW_X" "$MOVED_REPO_Y" right
  settle 1
  menu_entry 2
  settle 3
  shot move

  echo "== a click on two"
  click "$ROW_X" "$TWO_Y"
  settle 3
  shot click

  echo "== quit, and start again with no path"
  quit_marley
  open_path ""
  launch_marley
  settle 15
  shot restart
}
