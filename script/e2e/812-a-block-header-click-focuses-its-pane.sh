# shellcheck shell=bash
# #812's visual check: a click on a block header focuses its terminal. The repository's terminal
# and a terminal in a split each run a command, so each pane draws a block header; each sets `P` to
# its side, and `where` logs the side of the terminal it runs in.
# - A click in the left pane's body focuses the left terminal (`812-01-left-focused`).
# - A click on the right pane's block header, then `where`: the command runs in the right terminal
#   and the log says so (`812-02-header-clicked`, REQ-001, REQ-002).
# - With the left pane focused, the right terminal wheeled back into `seq 1 400`, its command
#   pinned over the top row (#529, `812-03-pinned`); a click on the pinned header: the right pane
#   jumps to the block's start (`812-04-pinned-clicked`), and `where` runs there (`812-05-typed`,
#   REQ-004).
compositor sway

# A point in the left pane's body, and one on the right pane's header of `P=right; echo right`,
# measured from the first run's shots.
LEFT_X=${LEFT_X:-500}
LEFT_Y=${LEFT_Y:-300}
HEADER_X=${HEADER_X:-1000}
HEADER_Y=${HEADER_Y:-871}
# The pinned header's row, over the terminal's top row, as #529's scenario measured it.
PINNED_Y=${PINNED_Y:-77}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
where() {
  echo "typed in \$P" | tee -a "$E2E_WORK/where.log"
}
RC
  terminal_env HOME "$home"
  : >"$E2E_WORK/where.log"
  git init -q -b header "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Whether the newest `where` line names side `$1`.
typed_in() {
  tail -1 "$E2E_WORK/where.log" | grep -qx "typed in $1"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "P=left; echo left"
  press "" Return
  settle 1
  palette "pane: split right"
  settle 4
  # A split opens Home since #785, so the new pane gets its terminal from the palette.
  palette "workspace: new terminal"
  settle 4
  type_text "P=right; echo right"
  press "" Return
  settle 2
  click "$LEFT_X" "$LEFT_Y"
  settle 1
  shot 812-01-left-focused
  click "$HEADER_X" "$HEADER_Y"
  settle 1
  type_text "where"
  press "" Return
  settle 2
  shot 812-02-header-clicked
  cat "$E2E_WORK/where.log"
  expect "the command typed after the header's click ran in the right terminal" typed_in right

  echo "== the pinned command header"
  type_text "seq 1 400"
  press "" Return
  settle 2
  click "$LEFT_X" "$LEFT_Y"
  settle 1
  pointer_to "$HEADER_X" "$LEFT_Y"
  scroll -15
  settle 1
  shot 812-03-pinned
  click "$HEADER_X" "$PINNED_Y"
  settle 1
  shot 812-04-pinned-clicked
  type_text "where"
  press "" Return
  settle 2
  shot 812-05-typed
  cat "$E2E_WORK/where.log"
  expect "the command typed after the pinned header's click ran in the right terminal" typed_in right
}
