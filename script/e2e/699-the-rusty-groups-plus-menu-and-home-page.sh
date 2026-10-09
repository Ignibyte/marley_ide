# shellcheck shell=bash
# #699's visual check: the Rusty group's + menu and its home page. `marley_rusty`'s stand-in
# `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, over a scratch state folder (`RUSTY_STAND_IN_STATE`): a
# page under `notes` and `tasks.json` with two made-up lists, never the user's Rusty (R-D8).
#
# The rail's Rusty button opens the home page in the Rusty group; its pane's + lists Rusty's links
# above Zed's (`699-01-plus-menu`, REQ-001), and Tasks from it opens the Tasks tab there
# (`699-02-from-menu`, REQ-002). Closing every tab leaves the home page, not Zed's Welcome page
# (`699-03-closed-all`, REQ-003). The project's + is Zed's (`699-05-project-plus`, REQ-005). After a
# quit and a relaunch, the Rusty group comes back with no tab (`699-04a-restored`) and shows its
# home page when the rail shows it (`699-04-shown-empty`, REQ-004).
compositor sway

# In the window's logical pixels, from the first run's shots: the rail's Rusty button, the Rusty
# group's + and the project's, the rail's project header and Rusty group header, and a point clear
# of all of them.
HEADER_Y=${HEADER_Y:-21}
RUSTY_X=${RUSTY_X:-19}
PLUS_X=${PLUS_X:-1530}
PLUS_Y=${PLUS_Y:-50}
PROJECT_PLUS_X=${PROJECT_PLUS_X:-1290}
PROJECT_PLUS_Y=${PROJECT_PLUS_Y:-50}
PROJECT_X=${PROJECT_X:-100}
PROJECT_Y=${PROJECT_Y:-89}
GROUP_X=${GROUP_X:-100}
GROUP_Y=${GROUP_Y:-182}
AWAY_X=${AWAY_X:-1000}
AWAY_Y=${AWAY_Y:-900}
# How many steps down the menu Tasks sits: Home, Brain, Today's Note, Graph, Tasks.
TASKS_STEPS=${TASKS_STEPS:-4}

setup() {
  local bin=$E2E_WORK/bin vault=$E2E_WORK/rusty/vault
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$vault/notes"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '# A note\n\nA page in the scratch vault.\n' >"$vault/notes/a-note.md"
  cat >"$E2E_WORK/rusty/tasks.json" <<'JSON'
{"groups": [
  {"id": 1, "name": "Home", "tasks": [
    {"id": 11, "title": "Water the plants"},
    {"id": 12, "title": "Return the library books"}]},
  {"id": 2, "name": "Work", "tasks": [
    {"id": 21, "title": "Draft the release notes"}]}
]}
JSON
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  local step
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the Rusty button, then the group's +"
  click "$RUSTY_X" "$HEADER_Y"
  settle 4
  away
  shot 699-00-home
  click "$PLUS_X" "$PLUS_Y"
  settle 2
  shot 699-01-plus-menu

  echo "== Tasks from the menu"
  # A menu opens with nothing chosen (Zed #64365); Home chooses its first entry.
  press "" Home
  for ((step = 0; step < TASKS_STEPS; step++)); do
    press "" Down
  done
  settle 1
  press "" Return
  settle 4
  away
  shot 699-02-from-menu

  echo "== every tab closed"
  palette "pane: close all items"
  settle 3
  away
  shot 699-03-closed-all

  echo "== the project's +"
  click "$PROJECT_X" "$PROJECT_Y"
  settle 3
  click "$PROJECT_PLUS_X" "$PROJECT_PLUS_Y"
  settle 2
  shot 699-05-project-plus
  press "" Escape
  settle 1

  echo "== a quit and a relaunch, then the Rusty group from the rail"
  quit_marley
  # With no path, as a launch from the menu: a path is an open request, which Zed answers
  # instead of restoring the last session (#601's scenario).
  open_path ""
  launch_marley
  settle 12
  away
  shot 699-04a-restored
  click "$GROUP_X" "$GROUP_Y"
  settle 4
  away
  shot 699-04-shown-empty
}
