# shellcheck shell=bash
# #760's visual check: Marley calls itself Marley. Under sway, on a scratch project:
# - sway's tree gives the window's app id, `com.ignibyte.Marley` (`760-01-window`, REQ-001);
# - `zed: about` names the app Marley (`760-02-about`, REQ-002);
# - the run's profile holds `marley-dev.sock` and no `zed-dev.sock`, and a second launch on a new
#   folder hands it to the running Marley (`760-03-socket`, REQ-003);
# - quit and started again, Marley restores both folders from the same database (`760-04-restarted`,
#   REQ-004).
compositor sway

setup() {
  mkdir -p "$E2E_WORK/first" "$E2E_WORK/second"
  printf '# first\n' >"$E2E_WORK/first/README.md"
  printf '# second\n' >"$E2E_WORK/second/NOTES.md"
  open_path "$E2E_WORK/first"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# The app ids of the sway's windows, one per line.
app_ids() {
  sway_msg -t get_tree | jq -r '.. | objects | select(.pid? != null) | .app_id // empty'
}

window_is_marleys() { app_ids | grep -qx 'com.ignibyte.Marley'; }
no_old_class() { ! app_ids | grep -qx 'dev.zed.Zed-Dev'; }
new_socket() { [[ -S $E2E_PROFILE/marley-dev.sock ]]; }
no_old_socket() { [[ ! -e $E2E_PROFILE/zed-dev.sock ]]; }

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 2

  echo "== the window's app id"
  app_ids
  shot 760-01-window
  expect "the window's app id is com.ignibyte.Marley" window_is_marleys
  expect "no window has the old class" no_old_class

  echo "== the About window"
  palette "zed: about"
  settle 3
  shot 760-02-about
  app_ids
  press "" Escape
  settle 2

  echo "== the socket, and a second launch"
  for socket in "$E2E_PROFILE"/*.sock; do
    ls -l "$socket"
  done
  expect "the profile holds marley-dev.sock" new_socket
  expect "and no zed-dev.sock" no_old_socket
  launch_marley_with "$E2E_WORK/second"
  settle 6
  # Trusts the second folder.
  press "" Return
  settle 2
  shot 760-03-socket
  expect "the second launch handed its folder over" grep -q "Marley is already running" "$E2E_LOG"

  echo "== quit, and started again"
  quit_marley
  settle 2
  launch_marley_with
  settle 14
  shot 760-04-restarted
  expect "the database stayed in db/0-dev" test -d "$E2E_PROFILE/db/0-dev"
}
