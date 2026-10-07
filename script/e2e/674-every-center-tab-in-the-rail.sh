# shellcheck shell=bash
# #674's visual check: every center tab has a row in the rail, a project's files under a Files row.
# A scratch project with `two.txt` at its root and `notes/one.md` under it.
#
# `674-01-files`: both files opened from the file finder, listed under Files 2 after the
# terminal, `one.md` with its folder (REQ-001, REQ-002). `674-02-other`: a project search opened,
# its row after the terminal and highlighted (REQ-001, REQ-005). `674-03-dirty`: `one.md` brought
# forward from its row and typed in, its dot (REQ-003). `674-04-folded`: the Files row clicked,
# folded, and highlighted for the file in front (REQ-004, REQ-005). `674-05-closed`: unfolded, and
# `two.txt` closed from its row's close button: its row gone, Files 1 (REQ-006).
compositor sway

# In the window's logical pixels, from the first run's shots: the Files row, the rows of
# `one.md` and `two.txt`, the close button at a row's end, and a point clear of the rail.
FILES_X=${FILES_X:-120}
FILES_Y=${FILES_Y:-313}
ONE_Y=${ONE_Y:-441}
TWO_Y=${TWO_Y:-372}
ROW_X=${ROW_X:-140}
CLOSE_X=${CLOSE_X:-228}
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-600}

setup() {
  mkdir -p "$E2E_WORK/home" "$E2E_WORK/repo/notes"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  printf 'the second file\n' >"$E2E_WORK/repo/two.txt"
  printf '# One\n\nThe first file, in a folder.\n' >"$E2E_WORK/repo/notes/one.md"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

# Opens `$1` with the file finder.
open_file() {
  palette "file finder: toggle"
  type_text "$1"
  settle 2
  press "" Return
  settle 2
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== two files"
  open_file two.txt
  open_file one.md
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 674-01-files

  echo "== a project search"
  palette "pane: deploy search"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 674-02-other

  echo "== one.md from its row, typed in"
  click "$ROW_X" "$ONE_Y"
  settle 2
  type_text "x"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 674-03-dirty

  echo "== Files folded"
  click "$FILES_X" "$FILES_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 674-04-folded

  echo "== unfolded, two.txt closed from its row"
  click "$FILES_X" "$FILES_Y"
  settle 2
  pointer_to "$ROW_X" "$TWO_Y"
  settle 1
  click "$CLOSE_X" "$TWO_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 674-05-closed
}
