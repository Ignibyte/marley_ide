# shellcheck shell=bash
# #661's visual check: with `marley.rusty.enabled` off, nothing of Rusty shows but its switch. The
# palette lists no `rusty:` command and no `marley: toggle brain view`; the Settings window's Marley
# page keeps the Rusty section's header and switch alone. Turned on from the settings file while
# the Settings window is open, the section's other items and the commands come back; off again,
# the commands go. `marley_rusty`'s stand-in, named by `MARLEY_RUSTY_MCP`, serves a scratch vault
# while Rusty is on, never the user's Rusty (R-D8).
#
# `661-01-palette-off` to `661-06-off-again`, one per step below.
compositor sway

# Where things sit, from the first run's shots: the Settings window's search field (L-659).
SETTINGS_SEARCH_X=${SETTINGS_SEARCH_X:-912}
SETTINGS_SEARCH_Y=${SETTINGS_SEARCH_Y:-59}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled false
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

# Opens the palette and types `$1`, leaving it open for the shot.
palette_shows() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 2
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the palette, Rusty off"
  palette_shows "rusty"
  shot 661-01-palette-off
  press "" Escape
  settle 1
  palette_shows "brain view"
  shot 661-02-brain-off
  press "" Escape
  settle 1

  echo "== the settings page, Rusty off"
  palette "marley: open settings"
  settle 4
  click "$SETTINGS_SEARCH_X" "$SETTINGS_SEARCH_Y"
  settle 1
  type_text "Rusty"
  settle 3
  shot 661-03-settings-off

  echo "== turned on, the window still open"
  profile_setting marley.rusty.enabled true
  settle 5
  shot 661-04-settings-on

  echo "== the palette, Rusty on"
  press CTRL w
  settle 2
  palette_shows "rusty"
  shot 661-05-palette-on
  press "" Escape
  settle 1

  echo "== off again"
  profile_setting marley.rusty.enabled false
  settle 4
  palette_shows "rusty"
  shot 661-06-off-again
  press "" Escape
}
