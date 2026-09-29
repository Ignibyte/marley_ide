# shellcheck shell=bash
# #536's visual check: copy and paste that know an agent CLI runs. A stand-in Claude Code, bash
# and then `cat -v` both run under the name `claude`, prints an indented reply, then shows every
# byte it is sent; it never turns bracketed paste on. A copy of its reply loses the shared indent
# (REQ-001); a multi-line paste arrives bracketed (REQ-002); a dropped PNG goes in raw inside a
# bracketed paste of its own and a text file quoted (REQ-003); an ESC inside a paste is removed
# (REQ-004); rich input's two lines arrive as one bracketed paste and an Enter (REQ-005). In a
# plain shell running `cat -v` the same copy, paste and drop behave as in Zed (REQ-006).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
# The reply's first and last rows once the stand-in waits, above the agent bar it brings,
# column 0's x and a point past the lines' ends; the project panel's rows for the two files.
# From the shots.
REPLY_FIRST_Y=${REPLY_FIRST_Y:-867}
REPLY_LAST_Y=${REPLY_LAST_Y:-906}
# The same lines in the plain shell, which has no agent bar, so two rows lower.
PLAIN_FIRST_Y=${PLAIN_FIRST_Y:-897}
PLAIN_LAST_Y=${PLAIN_LAST_Y:-936}
COLUMN_ZERO_X=${COLUMN_ZERO_X:-270}
LINE_END_X=${LINE_END_X:-600}
PANEL_X=${PANEL_X:-1450}
NOTES_Y=${NOTES_Y:-74}
SHOT_Y=${SHOT_Y:-100}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
# A stand-in Claude Code: its reply drawn in a gutter, then every byte it gets shown by cat -v,
# with no canonical mode, no echo and no carriage return turned into a line feed.
stand_in() {
  (exec -a claude bash -c 'printf "  first line of the reply\n    a nested line\n  last line\n"; stty -icanon -echo -icrnl; exec -a claude cat -v')
}
RC
  terminal_env HOME "$home"
  base64 -d >"$repo/shot.png" <<'PNG'
iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==
PNG
  printf 'notes\n' >"$repo/notes file.txt"
  git init -q -b main "$repo"
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Selects from column 0 of the row at `$1` to past the end of the row at `$2`.
select_rows() {
  pointer_to "$COLUMN_ZERO_X" "$1"
  pointer_down
  pointer_to "$((COLUMN_ZERO_X + 40))" "$1"
  pointer_to "$LINE_END_X" "$2"
  settle 1
  pointer_up
  settle 1
}

# Drags the project panel's row at `$1` onto the terminal.
drop_file() {
  pointer_to "$PANEL_X" "$1"
  pointer_down
  pointer_to "$((PANEL_X - 40))" "$(($1 + 20))"
  pointer_to "$((TERMINAL_X + 100))" "$TERMINAL_Y"
  pointer_to "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  pointer_up
  settle 1
}

clipboard() {
  WAYLAND_DISPLAY=$SWAY_DISPLAY wl-paste -n | cat -A | tee "$E2E_WORK/$1.txt"
}

# The terminals' titles name the process Zed sees, `cat -v` for the stand-in too, so they are
# told apart by the project's name and which came last.
screen() {
  mcp_agent terminal-screen "$1" | tee "$E2E_WORK/$2.txt"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1

  echo "== the stand-in's reply copied without its gutter, pasted bracketed"
  type_text "stand_in"
  press "" Return
  settle 2
  shot 536-00-stand-in
  select_rows "$REPLY_FIRST_Y" "$REPLY_LAST_Y"
  press "CTRL SHIFT" c
  settle 1
  clipboard agent-copy
  expect "the copy lost the shared indent and kept the nested one" \
    bash -c "grep -q '^first line of the reply\\\$' '$E2E_WORK/agent-copy.txt' && grep -q '^  a nested line\\\$' '$E2E_WORK/agent-copy.txt'"
  press "CTRL SHIFT" v
  settle 1
  shot 536-01-copy-and-paste
  screen repo agent-paste
  expect "the paste arrived bracketed" holds "$E2E_WORK/agent-paste.txt" '^[[200~first line of the reply'

  echo "== a PNG dropped goes in raw and bracketed; a text file quoted"
  drop_file "$SHOT_Y"
  drop_file "$NOTES_Y"
  shot 536-02-dropped-paths
  screen repo agent-drop
  expect "the PNG's path came bracketed and the text file's quoted" \
    holds "$E2E_WORK/agent-drop.txt" '/shot.png^[[201~' "notes file.txt'"

  echo "== an ESC inside a paste is removed"
  printf 'one\n\033[201~two' | WAYLAND_DISPLAY=$SWAY_DISPLAY wl-copy
  settle 1
  click "$TERMINAL_X" "$TERMINAL_Y"
  press "CTRL SHIFT" v
  settle 1
  shot 536-03-escape-stripped
  screen repo agent-escape
  expect "the inner marker lost its ESC" holds "$E2E_WORK/agent-escape.txt" '[201~two^[[201~'

  echo "== rich input's two lines: one bracketed paste and an Enter"
  press CTRL g
  settle 2
  type_text "first prompt line"
  press SHIFT Return
  type_text "second prompt line"
  press "" Return
  settle 2
  shot 536-05-rich-input
  screen repo agent-rich
  expect "the prompt came as one paste and then Enter" \
    holds "$E2E_WORK/agent-rich.txt" '^[[200~first prompt line' 'second prompt line^[[201~^M'

  echo "== a plain shell: Zed's copy, paste and drop"
  press CTRL c
  settle 1
  palette "workspace: new terminal"
  settle 3
  type_text "printf '  first line of the reply\n    a nested line\n  last line\n'; stty -icanon -echo -icrnl; cat -v"
  press "" Return
  settle 2
  select_rows "$PLAIN_FIRST_Y" "$PLAIN_LAST_Y"
  press "CTRL SHIFT" c
  settle 1
  clipboard plain-copy
  expect "the plain copy kept its indent" \
    bash -c "head -1 '$E2E_WORK/plain-copy.txt' | grep -q '^  first line of the reply\\\$'"
  press "CTRL SHIFT" v
  settle 1
  drop_file "$SHOT_Y"
  shot 536-04-plain-shell
  # The newest terminal whose title holds the project's name: the plain shell's.
  screen repo plain
  expect "no markers in the plain shell" bash -c "! grep -qF '[200~' '$E2E_WORK/plain.txt'"
}
