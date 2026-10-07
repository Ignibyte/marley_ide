# shellcheck shell=bash
# #675's visual check: every Rusty screen and page opens in the window's Rusty group.
# `marley_rusty`'s stand-in `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, over a scratch state folder
# (`RUSTY_STAND_IN_STATE`) whose vault holds one page, never the user's Rusty (R-D8).
#
# `675-01-before`: a project and its terminal, no Rusty group. `675-02-graph`: the header's Graph
# clicked: a Rusty group with its icon, Graph its row and in front (REQ-001, REQ-002).
# `675-03-project`: the project's terminal clicked: in front again, Graph still under Rusty
# (REQ-003). `675-04-page`: `rusty: open page` for the page while the project shows: the page under
# Rusty, in front (REQ-001). `675-05-menu`: the Rusty group's right-click menu, no Rename and no
# Remove (REQ-004). `675-06-off`: Rusty turned off while the group shows: no Rusty group, and the
# window back on the project (REQ-005).
compositor sway

# In the window's logical pixels, from the first run's shots: the header's Graph button, the
# project's terminal row, the Rusty group's header, and a point clear of the rail.
HEADER_Y=${HEADER_Y:-21}
GRAPH_X=${GRAPH_X:-106}
TERMINAL_X=${TERMINAL_X:-140}
TERMINAL_Y=${TERMINAL_Y:-184}
RUSTY_X=${RUSTY_X:-90}
RUSTY_Y=${RUSTY_Y:-262}
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-600}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault/notes"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '# A note\n\nA page in the scratch vault.\n' >"$E2E_WORK/rusty/vault/notes/a-note.md"
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
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  away
  shot 675-01-before

  echo "== Graph from the header"
  click "$GRAPH_X" "$HEADER_Y"
  settle 5
  away
  shot 675-02-graph
  expect "the graph was read" bash -c "grep -q ' tools/call brain_graph ' '$E2E_WORK/rusty/calls'"

  echo "== the project's terminal"
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 2
  away
  shot 675-03-project

  echo "== a page from the page picker"
  palette "rusty: open page"
  type_text "a note"
  settle 2
  press "" Return
  settle 4
  away
  shot 675-04-page

  echo "== the Rusty group's menu"
  click "$RUSTY_X" "$RUSTY_Y" right
  settle 2
  shot 675-05-menu
  press "" Escape
  settle 1

  echo "== Rusty off"
  profile_setting marley.rusty.enabled false
  settle 4
  away
  shot 675-06-off
}
