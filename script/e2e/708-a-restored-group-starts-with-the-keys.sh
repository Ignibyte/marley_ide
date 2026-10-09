# shellcheck shell=bash
# #708's visual check: after a relaunch the window's keys work with no click first. The run's
# profile starts with no database and Rusty on, its stand-in `rusty-mcp` answering, never the
# user's Rusty (R-D8), so the window has Home, Rusty and a scratch project. Home is shown at the
# quit (`708-01-before-quit`), so the relaunch restores Home and reopens Rusty and the project
# behind it. With no click, Ctrl+Shift+P opens the palette (`708-02-palette`, REQ-001).
compositor sway

# The rail's Home header, from #700's shots.
HOME_X=${HOME_X:-100}
HOME_Y=${HOME_Y:-89}
AWAY_X=${AWAY_X:-1000}
AWAY_Y=${AWAY_Y:-900}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  rm -rf "$E2E_PROFILE/db"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== Home shown, then a quit"
  click "$HOME_X" "$HOME_Y"
  settle 3
  away
  shot 708-01-before-quit
  quit_marley

  echo "== a relaunch with no path, then the palette's key and no click"
  # With no path, as a launch from the menu: a path is an open request (L-601).
  open_path ""
  launch_marley
  settle 15
  press "CTRL SHIFT" p
  settle 2
  shot 708-02-palette
  press "" Escape
}
