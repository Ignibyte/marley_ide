# shellcheck shell=bash
# #880's visual check: a run keeps off the user's tmux. No `marley.tmux_hosts` setting, so Marley
# lists the sessions of the tmux server its environment names, which is the run's own and empty,
# while the user's server has sessions. A scratch repository.
# - The rail: the scratch project and no "this machine" tmux row (`880-01-rail`, REQ-001).
# - The run's `tmux list-sessions` finds no server (REQ-001).
compositor sway

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  open_path "$repo"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 6

  echo "== the rail lists no tmux session of the user's"
  echo "TMUX_TMPDIR=$TMUX_TMPDIR"
  pointer_to 1500 950
  settle 1
  shot 880-01-rail
  expect "the run's tmux finds no server" bash -c '! tmux list-sessions 2>/dev/null'
}
