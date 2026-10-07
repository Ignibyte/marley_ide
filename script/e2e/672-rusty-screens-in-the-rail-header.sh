# shellcheck shell=bash
# #672's visual check: Rusty's screens in the rail's header, and the row under the header at the
# pane's tab bar's height. `marley_rusty`'s stand-in `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, over
# a scratch state folder (`RUSTY_STAND_IN_STATE`), never the user's Rusty (R-D8); the copy keeps the
# user's UI size, and the rail starts at its default width.
#
# `672-01-projects`: the header with Projects, Brain, the screens that fit and `…`, the folder `+`;
# the filter row's line on the tab bar's (REQ-001, REQ-003, REQ-005). `672-02-filtering`: text in
# the filter and its clear button, the same height (REQ-005). `672-03-brain`: the Brain view, no row
# of screen buttons, the search row's line on the tab bar's (REQ-001, REQ-004, REQ-005).
# `672-04-graph`: the header's Graph button, clicked in the Projects view: the Graph tab in front
# (REQ-002). `672-05-overflow`: `…` open with the screens that did not fit (REQ-003).
# `672-06-wide`: the rail dragged wider: every screen in the header, no `…` (REQ-003).
compositor sway

# In the window's logical pixels, from the first run's shots: the header's buttons, the filter, the
# `…` button and its menu's last entry, the rail's edge, and a point in the terminal clear of the
# rail.
HEADER_Y=${HEADER_Y:-21}
PROJECTS_X=${PROJECTS_X:-19}
BRAIN_X=${BRAIN_X:-48}
GRAPH_X=${GRAPH_X:-106}
MORE_X=${MORE_X:-199}
MENU_X=${MENU_X:-250}
MENU_Y=${MENU_Y:-126}
FILTER_X=${FILTER_X:-120}
FILTER_Y=${FILTER_Y:-66}
EDGE_X=${EDGE_X:-260}
EDGE_Y=${EDGE_Y:-500}
WIDE_X=${WIDE_X:-400}
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-600}

# Whether shot `$1` draws its horizontal lines at the same rows in the rail (x 150) and over the
# tab bar (x 700) in the first 120: the line under the header and the title bar, and the line under
# the filter or search row and the tab bar. A line is a row in the copied theme's border colour.
lines_meet() {
  local file rail tab
  file=$(shot_file "$1.png")
  rail=$(magick "$file" -crop 1x120+150+0 txt:- | awk -F'[,: ]+' 'NR > 1 && /#464B57/ {print $2}' | paste -sd,)
  tab=$(magick "$file" -crop 1x120+700+0 txt:- | awk -F'[,: ]+' 'NR > 1 && /#464B57/ {print $2}' | paste -sd,)
  echo "rail lines: $rail; tab bar lines: $tab"
  [[ -n $rail && $rail == "$tab" ]]
}

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

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1

  echo "== the Projects view"
  shot 672-01-projects
  expect "the rail's lines meet the title bar's and the tab bar's" lines_meet 672-01-projects

  echo "== text in the filter"
  click "$FILTER_X" "$FILTER_Y"
  settle 1
  type_text "zz"
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 672-02-filtering
  expect "the filter's line still meets the tab bar's" lines_meet 672-02-filtering
  click "$FILTER_X" "$FILTER_Y"
  press "" Escape
  settle 1

  echo "== the Brain view"
  click "$BRAIN_X" "$HEADER_Y"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 672-03-brain
  expect "the search row's line meets the tab bar's" lines_meet 672-03-brain

  echo "== Graph from the Projects view"
  click "$PROJECTS_X" "$HEADER_Y"
  settle 1
  click "$GRAPH_X" "$HEADER_Y"
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 672-04-graph
  expect "the graph was read" bash -c "grep -q ' tools/call brain_graph ' '$E2E_WORK/rusty/calls'"

  echo "== the screens that did not fit"
  click "$MORE_X" "$HEADER_Y"
  settle 1
  # Onto the menu's last entry, so the button's tooltip does not cover the first.
  pointer_to "$MENU_X" "$MENU_Y"
  settle 1
  shot 672-05-overflow
  press "" Escape
  settle 1

  echo "== the rail wider"
  pointer_to "$EDGE_X" "$EDGE_Y"
  settle 0.5
  pointer_down
  settle 0.3
  pointer_to "$((EDGE_X + 40))" "$EDGE_Y"
  settle 0.3
  pointer_to "$WIDE_X" "$EDGE_Y"
  settle 0.3
  pointer_up
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 672-06-wide
}
