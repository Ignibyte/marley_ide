# shellcheck shell=bash
# #631's e2e test: gaps between blocks and the density setting. In bash with a one-row prompt in a
# git repository, the density at its default (comfortable): three blocks stand half a row apart,
# each header two lines (the folder and branch over the command), and the prompt the shell waits
# at stays on the last row. The mouse lands on the rows as drawn: a double click on a word under a
# gap selects that word, a drag over two blocks highlights their rows with the gap clear, and a
# right click picks the block under the pointer. Compact draws #630's look: no gaps, one-row
# headers. A full screen keeps the prompt on the last row; scrolled back, the rows start at the top
# edge under the pinned header, and the wheel brings the live screen back.
compositor sway

# Where the calibration run drew the words the pointer aims at: the first block's output `one`,
# the second's `two` and `three`.
ONE_X=275
ONE_Y=737
TWO_X=285
TWO_Y=805
THREE_X=290
THREE_Y=824
GRID_X=800
GRID_Y=500

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  git init -q -b main "$E2E_WORK/repo"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  # The terminal takes the focus, and the prompt editor docks.
  click "$GRID_X" "$GRID_Y"
  settle 1.5

  echo "== three blocks, comfortable"
  type_text "echo one"
  press "" Return
  settle 1.5
  type_text "printf 'two\\nthree\\n'"
  press "" Return
  settle 1.5
  type_text "echo four"
  press "" Return
  settle 2
  shot 631-01-comfortable

  echo "== the mouse on the rows as drawn"
  click "$THREE_X" "$THREE_Y"
  click "$THREE_X" "$THREE_Y"
  settle 1
  shot 631-02-word
  pointer_to "$ONE_X" "$ONE_Y"
  pointer_down
  sleep 0.2
  pointer_to $((ONE_X + 40)) $((ONE_Y + 30))
  sleep 0.2
  pointer_to "$TWO_X" "$TWO_Y"
  sleep 0.2
  pointer_up
  settle 1
  shot 631-03-drag
  click "$TWO_X" "$TWO_Y" right
  settle 1
  shot 631-04-menu
  press "" Escape
  settle 1

  echo "== compact, and back"
  profile_setting marley.block_density '"compact"'
  settle 3
  shot 631-05-compact
  profile_setting marley.block_density '"comfortable"'
  settle 3

  echo "== a full screen"
  click "$GRID_X" "$GRID_Y"
  settle 1
  type_text "seq 1 60"
  press "" Return
  settle 2
  shot 631-06-full

  echo "== scrolled back, and back"
  pointer_to "$GRID_X" "$GRID_Y"
  scroll -4
  settle 1
  shot 631-07-scrolled
  scroll 30
  settle 1
  shot 631-08-back
}
