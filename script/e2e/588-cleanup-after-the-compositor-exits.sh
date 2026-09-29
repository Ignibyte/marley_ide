# shellcheck shell=bash
# #588's check of the runner: its cleanup when the headless sway exits partway. After a first shot
# the scenario kills its own sway, as sway exited unasked in #560's golden run; the runner then
# names the compositor's exit and fails the run, still copies Marley's log beside the shots, and
# stops the run's Marley, pointer helper and key holder. This run fails on purpose, so it is not in
# the golden set.
compositor sway

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  printf "PS1='\\$ '\n" >"$home/.bashrc"
  terminal_env HOME "$home"
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 10
  shot 588-01-before
  echo "== the compositor killed"
  pkill -KILL -f -- "-c $SWAY_DIR/config"
  settle 3
}
