# shellcheck shell=bash
# #700's visual check: every window has its Home group from the start, and its Rusty group while
# Rusty is on, listed Home then Rusty above the projects. The run's profile starts with no database,
# as a fresh install does, and Rusty off; `marley_rusty`'s stand-in `rusty-mcp` answers once it is
# on, never the user's Rusty (R-D8).
#
# `700-01-fresh`: Home above the scratch project, no Rusty (REQ-001). `700-02-rusty-on`: Rusty
# turned on while Marley runs: Home, Rusty, the project (REQ-002). `700-03-restarted`: a quit and a
# relaunch with no path, Home shown: one Home, one Rusty, the project (REQ-003). `700-04-no-folder`: the
# database removed and Marley started with no path: Home holds the start workspace, Rusty under it,
# no project (REQ-004).
compositor sway

# The rail's Home header, from the first run's shots.
HOME_X=${HOME_X:-100}
HOME_Y=${HOME_Y:-89}
AWAY_X=${AWAY_X:-1000}
AWAY_Y=${AWAY_Y:-900}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  # A fresh install: no workspaces, no groups.
  rm -rf "$E2E_PROFILE/db"
  profile_setting marley.rusty.enabled false
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
  away
  shot 700-01-fresh

  echo "== Rusty turned on"
  profile_setting marley.rusty.enabled true
  settle 6
  away
  shot 700-02-rusty-on

  echo "== a quit and a relaunch with no path, Home shown"
  # Home shown at the quit, so the relaunch restores it as the window's active workspace, which
  # the rail must adopt rather than claim or duplicate; the palette also leaves the terminal.
  click "$HOME_X" "$HOME_Y"
  settle 2
  quit_marley
  # With no path, as a launch from the menu: a path is an open request (L-601).
  open_path ""
  launch_marley
  settle 15
  away
  shot 700-03-restarted

  echo "== a fresh start with no folder"
  # A restored folderless workspace starts with nothing focused, so the quit's keys need the
  # pane focused first.
  click "$AWAY_X" 250
  settle 1
  quit_marley
  rm -rf "$E2E_PROFILE/db"
  launch_marley
  settle 15
  away
  shot 700-04-no-folder
}
