# shellcheck shell=bash
# #563's visual check: a note the first time one of Marley's keys takes a key from a terminal's
# program. With no agent CLI running, Ctrl-G reaches the program and shows no note (REQ-001);
# under a stand-in agent it opens Rich Input and shows the note once (REQ-002, REQ-003); the block
# keys have a note of their own (REQ-005); after a relaunch on the same data directory neither
# note comes back (REQ-003), while New Agent's first use from a terminal shows its own.
compositor sway

TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
# A toast's close button at the workspace's bottom right, from the shots.
TOAST_CLOSE_X=${TOAST_CLOSE_X:-1563}
TOAST_CLOSE_Y=${TOAST_CLOSE_Y:-879}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  # A stand-in agent CLI that shows the keys it gets.
  printf '#!/usr/bin/env bash\nexec -a claude cat -v\n' >"$bin/claude"
  chmod +x "$bin/claude"
  git init -q -b main "$repo"
  open_path "$repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1

  echo "== no agent: Ctrl-G reaches the program, no note"
  type_text "cat -v"
  press "" Return
  settle 1
  press "CTRL" g
  settle 2
  shot 563-01-passes
  press "CTRL" c
  settle 1

  echo "== under an agent: Rich Input, and the note"
  type_text "claude"
  press "" Return
  settle 3
  press "CTRL" g
  settle 2
  shot 563-02-note
  press "" Escape
  settle 1
  click "$TOAST_CLOSE_X" "$TOAST_CLOSE_Y"
  settle 1
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1

  echo "== again: no note"
  press "CTRL" g
  settle 2
  shot 563-03-once
  press "" Escape
  settle 1
  click "$TERMINAL_X" "$TERMINAL_Y"
  press "CTRL" c
  settle 1

  echo "== the block keys: a note of their own"
  type_text "echo one"
  press "" Return
  settle 1
  press "CTRL" Up
  settle 2
  shot 563-04-blocks
  press "" Escape
  settle 1

  echo "== after a relaunch: remembered"
  # The store's write lands off the main thread; give it a moment before Marley quits.
  settle 2
  quit_marley
  launch_marley
  settle 12
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  type_text "echo two"
  press "" Return
  settle 1
  press "CTRL" Up
  settle 1
  press "" Escape
  type_text "claude"
  press "" Return
  settle 3
  press "CTRL" g
  settle 2
  shot 563-05-remembered
  press "" Escape
  settle 1

  echo "== New Agent from a terminal: its own note"
  click "$TERMINAL_X" "$TERMINAL_Y"
  press "CTRL ALT" n
  settle 2
  shot 563-06-new-agent
  press "" Escape
}
