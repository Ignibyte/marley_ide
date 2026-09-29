# shellcheck shell=bash
# #486's visual check: a launch's first terminal opens at the last session's size. #485's prompt:
# two lines, the second wider than the 100 columns Zed's terminal starts a PTY at, and a `bind`
# while bash starts, so readline lays the prompt out for the width the PTY has then. The first
# launch keeps no size yet, so its first terminal is misdrawn as before (REQ-002); after a quit,
# which keeps the size, a launch on the same data directory opens its first terminal at it, and
# the typed line lands whole on the prompt (REQ-001). It runs on Hyprland's hidden workspace, as
# #485's does: in the headless sway the first view lays out before bash starts readline, and the
# first launch draws the prompt right with or without a kept size.

setup() {
  mkdir -p "$E2E_WORK/home"
  cat >"$E2E_WORK/home/.bashrc" <<'RC'
bind 'set show-all-if-ambiguous on'
dashes=$(printf '%.0s-' {1..124})
PS1="\n\[\e[1;36m\]$dashes\[\e[0m\] \[\e[3;36m\]probe\[\e[0m\] \[\e[1;36m\]❯\[\e[0m\] "
RC
  terminal_env HOME "$E2E_WORK/home"
  git init -q -b long-prompt "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "echo first"
  settle 1
  shot 486-01-first-launch

  echo "== a quit, which keeps the size, and a launch on the same data directory"
  quit_marley
  launch_marley
  settle 12
  type_text "echo again"
  settle 1
  shot 486-02-relaunched
}
