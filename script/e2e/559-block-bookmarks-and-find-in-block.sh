# shellcheck shell=bash
# #559's visual check: bookmarks on blocks and find within a block. Before any bookmark, `cat -v`
# prints the bytes Alt+Up and Alt+Down send, so the keys reach the program (REQ-005). Six blocks,
# more than a screen: Ctrl+Shift+B marks the newest block in view, its bookmark before the pill and
# a tick at the right edge (REQ-001, REQ-003); a block revealed with Ctrl+Up is marked from its
# hover Bookmark button (REQ-002); Alt+Down and Alt+Up scroll between the marks (REQ-004);
# Ctrl+Shift+B again removes the newest's mark. A block's Find button holds the search bar to it,
# outlined, with its matches alone counted (REQ-006); after Escape, Ctrl+Shift+F searches the
# whole terminal (REQ-007); with a block selected, Ctrl+Shift+F holds the search to it (REQ-009).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
# The hover buttons of a finished block: Bookmark, then Find, then Save as Workflow, Filter, Copy
# and Rerun; and the rows the steps hover, from the shots: the top row with a block revealed, and
# the printf block's first row with the terminal at its bottom.
BOOKMARK_X=${BOOKMARK_X:-1198}
FIND_X=${FIND_X:-1219}
TOP_ROW_Y=${TOP_ROW_Y:-77}
PRINTF_ROW_Y=${PRINTF_ROW_Y:-292}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
RC
  terminal_env HOME "$home"
  git init -q -b main "$repo"
  open_path "$repo"
}

run() {
  type_text "$1"
  press "" Return
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1

  echo "== with no bookmark, Alt+Up and Alt+Down reach the program"
  run "cat -v"
  press ALT Up
  press ALT Down
  press "" Return
  settle 1
  press CTRL d
  settle 1
  shot 559-00-keys-pass
  mcp_agent terminal-screen repo | tee "$E2E_WORK/keys.txt"
  expect "cat -v printed the keys' sequences" holds "$E2E_WORK/keys.txt" '^[[1;3A^[[1;3B'

  echo "== six blocks; Ctrl+Shift+B marks the newest"
  run "seq 1 30"
  run "echo needle one"
  run "seq 31 60"
  run "printf 'needle %s\n' two three"
  run "seq 61 90"
  settle 1
  press "CTRL SHIFT" b
  settle 1
  shot 559-01-marked

  echo "== a revealed block marked from its Bookmark button"
  # The newest block, then back to echo's.
  press CTRL Up
  press CTRL Up
  press CTRL Up
  press CTRL Up
  settle 1
  shot 559-01a-revealed
  press "" Escape
  settle 1
  pointer_to "$BOOKMARK_X" "$TOP_ROW_Y"
  settle 1
  shot 559-01b-hover
  click "$BOOKMARK_X" "$TOP_ROW_Y"
  settle 1
  pointer_to "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  shot 559-02-two-marks

  echo "== Alt+Down and Alt+Up scroll between the marks"
  press ALT Down
  settle 1
  shot 559-03-jumped-down
  press ALT Up
  settle 1
  shot 559-04-jumped-up

  echo "== Ctrl+Shift+B again removes the newest's mark"
  press ALT Down
  settle 1
  press "CTRL SHIFT" b
  settle 1
  shot 559-05-unmarked

  echo "== a block's Find button holds the search to it"
  pointer_to "$FIND_X" "$PRINTF_ROW_Y"
  settle 1
  shot 559-05a-hover
  click "$FIND_X" "$PRINTF_ROW_Y"
  settle 2
  type_text "needle"
  settle 2
  pointer_to "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  shot 559-06-find-in-block

  echo "== Escape ends it; Ctrl+Shift+F searches the whole terminal"
  press "" Escape
  settle 1
  press "CTRL SHIFT" f
  settle 1
  type_text "needle"
  settle 2
  shot 559-07-whole-terminal

  echo "== with a block selected, Ctrl+Shift+F holds the search to it"
  press "" Escape
  settle 1
  press CTRL Up
  press CTRL Up
  settle 1
  press "CTRL SHIFT" f
  settle 1
  type_text "needle"
  settle 2
  shot 559-08-selected
  press "" Escape
  settle 1
}
