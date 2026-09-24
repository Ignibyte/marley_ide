# shellcheck shell=bash
# #487's e2e test: the headless sway backend. Marley runs in a sway of the run's own; a click on
# the agent bar's Rich Input button (a path only a click reached before) opens the editor, text
# typed there reaches the stand-in agent, and the wheel scrolls the terminal back through the
# numbers printed before the agent started. The stand-in is #481's: the shell `exec`s a
# `claude` that prints each line it reads.
compositor sway

# The Rich Input button on the agent bar, and a point over the terminal's grid, in the
# 1600x1000 window (read from 487-01's shot).
RICH_INPUT_X=402
RICH_INPUT_Y=954
GRID_X=900
GRID_Y=400

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
stand_in() {
  exec -a claude bash -c 'printf "ready\n"; while IFS= read -r line; do printf "claude got: %s\n" "$line"; done'
}
RC
  terminal_env HOME "$home"
  git init -q -b sway-pointer "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "seq -f 'line %g' 1 200"
  press "" Return
  settle 1
  type_text "stand_in"
  press "" Return
  settle 3
  shot 487-01-marley
  click "$RICH_INPUT_X" "$RICH_INPUT_Y"
  settle 1
  shot 487-02-rich-input-clicked
  type_text "typed under sway"
  press "" Return
  settle 2
  shot 487-03-sent
  pointer_to "$GRID_X" "$GRID_Y"
  scroll -10
  settle 1
  shot 487-04-scrolled-back
}
