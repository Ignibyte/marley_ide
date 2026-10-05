# shellcheck shell=bash
# #657's visual check: the Graph tab's Groups, Display and Forces, and the tab and its settings
# kept across a restart. `marley_rusty`'s stand-in `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, serves
# #647's twelve made-up pages over a scratch state folder (`RUSTY_STAND_IN_STATE`), never the
# user's brain (R-D8), and logs each call with its pid in `calls`. The harness's copy of the
# user's database holds no graph settings (D13), which setup checks. Rusty is on from the start;
# the third launch turns it off by an edit from outside first. The layout has no randomness, so
# the places clicked, measured from the first runs' shots, hold; a step that moves the layout
# waits for Marley.log's next "graph layout:" line.
#
# `657-01-group`, `657-02-first-group-wins`, `657-03-group-removed`, `657-04-next-colour`,
# `657-05-arrows`, `657-06-node-size`, `657-07-slider-keys`, `657-08-link-thickness`,
# `657-09-thumb-drag`, `657-10-text-fade-low`, `657-11-text-fade-high`, `657-12-repel`,
# `657-13-link-distance`, `657-14-before-restart`, `657-15-restored`,
# `657-16-restored-settings`, `657-17-settings-kept`, `657-18-off-not-restored`.
compositor sway

# Where things sit, from the first runs' shots: the canvas's middle, #647's panel controls, the
# three sections' headers (folded), the controls inside them, and the legend's `note` row.
CENTRE_X=${CENTRE_X:-810}
CENTRE_Y=${CENTRE_Y:-518}
FILTER_X=${FILTER_X:-1230}
FILTER_Y=${FILTER_Y:-185}
FIT_X=${FIT_X:-1310}
FIT_Y=${FIT_Y:-95}
DEPTH_2_X=${DEPTH_2_X:-1185}
DEPTH_Y=${DEPTH_Y:-151}
SWITCH_X=${SWITCH_X:-1131}
TAGS_Y=${TAGS_Y:-217}
HEADER_X=${HEADER_X:-1150}
GROUPS_Y=${GROUPS_Y:-312}
DISPLAY_Y=${DISPLAY_Y:-342}
FORCES_Y=${FORCES_Y:-372}
# Inside Groups, with Groups open alone: the first row and the step between rows, its swatch and
# remove button, and New group under the rows.
GROUP_ROW_Y=${GROUP_ROW_Y:-344}
GROUP_ROW_H=${GROUP_ROW_H:-32}
SWATCH_X=${SWATCH_X:-1127}
REMOVE_X=${REMOVE_X:-1336}
NEW_GROUP_X=${NEW_GROUP_X:-1165}
# Inside Display, with Display open alone: Arrows, then the three sliders' tracks.
ARROWS_Y=${ARROWS_Y:-371}
TRACK_LEFT=${TRACK_LEFT:-1130}
TRACK_RIGHT=${TRACK_RIGHT:-1333}
TEXT_FADE_Y=${TEXT_FADE_Y:-411}
NODE_SIZE_Y=${NODE_SIZE_Y:-450}
LINK_THICKNESS_Y=${LINK_THICKNESS_Y:-489}
# Inside Forces, with Forces open alone: Repel force's and Link distance's tracks.
REPEL_Y=${REPEL_Y:-456}
LINK_DISTANCE_Y=${LINK_DISTANCE_Y:-534}
NOTE_TYPE_X=${NOTE_TYPE_X:-1150}
NOTE_TYPE_Y=${NOTE_TYPE_Y:-464}

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# How many times the stand-in was asked `$1` with arguments holding `$2`.
asked() { calls | grep " tools/call $1 " | grep -cF -- "$2" || true; }

# The stand-in's pids, one a line, in the order they first called.
pids() { calls | awk '!seen[$1]++ {print $1}'; }

log_file() { echo "$E2E_PROFILE/logs/Marley.log"; }

layouts() { grep -c "graph layout:" "$(log_file)" 2>/dev/null || true; }

# Waits up to 30 s for a layout to settle after `$1` had, then until no later run settles for
# half a second (#647's).
await_layout() {
  local before=$1 waited last
  for ((waited = 0; waited < 150; waited++)); do
    if (($(layouts) > before)); then
      break
    fi
    sleep 0.2
  done
  if ((waited == 150)); then
    echo "no layout settled after $before"
    return 0
  fi
  last=$(layouts)
  for ((waited = 0; waited < 20; waited++)); do
    sleep 0.5
    if (($(layouts) == last)); then
      return 0
    fi
    last=$(layouts)
  done
}

# Runs `$@`, then waits for the layout it starts to settle.
laid_out() {
  local before
  before=$(layouts)
  "$@"
  await_layout "$before"
  settle 1
}

# How many graph settings rows the run's copy of the database holds.
kept_settings() {
  local db count=0 rows
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    [[ -e $db ]] || continue
    rows=$(sqlite3 -readonly "$db" \
      "select count(*) from scoped_kv_store where namespace = 'marley-rusty-graph'" 2>/dev/null ||
      echo 0)
    count=$((count + rows))
  done
  echo "$count"
}

# A made-up page `$1` in the stand-in's scratch vault, its text from stdin.
vault_page() {
  local path=$E2E_WORK/rusty/vault/$1.md
  mkdir -p "$(dirname "$path")"
  cat >"$path"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

filter_to() {
  click "$FILTER_X" "$FILTER_Y"
  settle 0.5
  press "CTRL" a
  press "" BackSpace
  if [[ -n ${1:-} ]]; then
    type_text "$1"
  fi
  settle 1
}

open_graph() {
  press "CTRL ALT SHIFT" y
  settle 2
}

fit() {
  click "$FIT_X" "$FIT_Y"
  settle 1
}

# Opens or folds a section by its header's place.
section() {
  click "$HEADER_X" "$1"
  settle 1
}

# A slider set to an end: a press on its track at `$1`, then Home or End (`$2`).
slider_to() {
  click "$TRACK_LEFT" "$1"
  settle 0.5
  press "" "$2"
  settle 1
}

group_y() { echo $((GROUP_ROW_Y + $1 * GROUP_ROW_H)); }

graph_vault() {
  vault_page projects/orbit <<'PAGE'
---
title: Orbit
tags: [storage]
---
The project everything circles.
PAGE
  vault_page projects/lantern <<'PAGE'
---
title: Lantern
---
Lantern runs on [[projects/orbit]].
PAGE
  vault_page concepts/cold-storage <<'PAGE'
---
title: Cold storage
tags: [storage/cold]
---
Where [[projects/orbit]] keeps old things.
PAGE
  vault_page concepts/warm-cache <<'PAGE'
---
title: Warm cache
tags: [storage]
---
Sits in front of [[concepts/cold-storage]].
PAGE
  vault_page research/orbit-survey <<'PAGE'
---
title: Orbit survey
type: research
---
Read about [[projects/orbit]] and [[projects/lantern]].
PAGE
  vault_page research/latency-notes <<'PAGE'
---
title: Latency notes
type: research
---
Follows [[research/orbit-survey]].
PAGE
  vault_page notes/standup <<'PAGE'
---
title: Standup
---
Talked about [[projects/orbit]] and [[ghost-page]].
PAGE
  vault_page decisions/use-orbit <<'PAGE'
---
title: Use Orbit
consulted: [research/orbit-survey, concepts/cold-storage]
supersedes: decisions/old-orbit-plan
---
We build on [[projects/orbit]].
PAGE
  vault_page decisions/old-orbit-plan <<'PAGE'
---
title: Old Orbit plan
---
The first plan.
PAGE
  vault_page decisions/keep-lantern <<'PAGE'
---
title: Keep Lantern
superseded_by: decisions/use-orbit
---
Keep [[projects/lantern]] going.
PAGE
  vault_page ideas/stray-thought <<'PAGE'
---
title: Stray thought
---
Nothing links here.
PAGE
  vault_page notes/loose-end <<'PAGE'
---
title: Loose end
---
Nor here.
PAGE
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  expect "the run's database copy holds no graph settings" test "$(kept_settings)" = 0
  profile_setting marley.rusty.connection '"embedded"'
  profile_setting marley.rusty.enabled true
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  # The palette ranks a command used last first, so after "open local graph" it would take
  # "rusty: open graph" for it; a key names the action.
  printf '%s\n' '[{"context": "Workspace", "bindings": {"ctrl-alt-shift-y": "rusty::OpenGraph"}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  graph_vault
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== a group"
  laid_out open_graph
  section "$GROUPS_Y"
  click "$NEW_GROUP_X" "$(group_y 0)"
  settle 1
  type_text "path:research/"
  settle 1
  shot 657-01-group

  echo "== the first group wins"
  click "$NEW_GROUP_X" "$(group_y 1)"
  settle 1
  type_text "type:research"
  settle 1
  shot 657-02-first-group-wins

  echo "== a group removed"
  click "$REMOVE_X" "$(group_y 0)"
  settle 1
  shot 657-03-group-removed

  echo "== the next colour"
  click "$SWATCH_X" "$(group_y 0)"
  settle 1
  shot 657-04-next-colour
  section "$GROUPS_Y"

  echo "== arrows"
  section "$DISPLAY_Y"
  click "$SWITCH_X" "$ARROWS_Y"
  settle 1
  laid_out click "$SWITCH_X" "$TAGS_Y"
  pointer_to "$CENTRE_X" "$CENTRE_Y"
  scroll -3
  settle 1
  shot 657-05-arrows
  # Tags off again, so the filter below leaves Orbit alone to click.
  laid_out click "$SWITCH_X" "$TAGS_Y"

  echo "== node size"
  slider_to "$NODE_SIZE_Y" End
  shot 657-06-node-size

  echo "== slider keys"
  press "" Left
  press "" Left
  settle 1
  shot 657-07-slider-keys

  echo "== link thickness"
  slider_to "$LINK_THICKNESS_Y" End
  shot 657-08-link-thickness

  echo "== the thumb dragged"
  pointer_to "$TRACK_RIGHT" "$LINK_THICKNESS_Y"
  pointer_down
  pointer_to $(((TRACK_LEFT + TRACK_RIGHT) / 2 + 10)) "$LINK_THICKNESS_Y"
  pointer_to $(((TRACK_LEFT + TRACK_RIGHT) / 2)) "$LINK_THICKNESS_Y"
  pointer_up
  settle 1
  shot 657-09-thumb-drag

  echo "== the text fade"
  fit
  slider_to "$TEXT_FADE_Y" Home
  shot 657-10-text-fade-low
  slider_to "$TEXT_FADE_Y" End
  shot 657-11-text-fade-high
  section "$DISPLAY_Y"

  echo "== repel force"
  section "$FORCES_Y"
  laid_out slider_to "$REPEL_Y" End
  shot 657-12-repel
  expect "the run used repel 20" grep -q "repel 20.0" "$(log_file)"

  echo "== link distance"
  laid_out slider_to "$LINK_DISTANCE_Y" Home
  shot 657-13-link-distance
  expect "the run used distance 30" grep -q "distance 30" "$(log_file)"
  # The far end, so the restored picture is spread out to read.
  laid_out slider_to "$LINK_DISTANCE_Y" End
  section "$FORCES_Y"

  echo "== before the restart"
  laid_out filter_to "path:projects/orbit"
  fit
  click "$CENTRE_X" "$CENTRE_Y"
  settle 3
  palette "rusty: open local graph"
  settle 3
  laid_out click "$DEPTH_2_X" "$DEPTH_Y"
  laid_out filter_to "path:research/"
  laid_out click "$NOTE_TYPE_X" "$NOTE_TYPE_Y"
  shot 657-14-before-restart

  echo "== restored"
  local first_pids graphs_before
  first_pids=$(pids | wc -l)
  quit_marley
  # With no path, as a launch from the menu: a path is an open request, not a restore (L-601).
  open_path ""
  launch_marley
  settle 15
  await_layout 0
  shot 657-15-restored
  calls | grep -E 'initialize|brain_graph' | tail -4
  expect "a new stand-in started" test "$(pids | wc -l)" -gt "$first_pids"
  expect "the new stand-in read orbit's depth 2" \
    sh -c "grep '^$(pids | tail -1) tools/call brain_graph ' '$E2E_WORK/rusty/calls' | grep -q '\"around\": \"projects/orbit\"'"

  echo "== the settings after the restart"
  # Opened from the bottom up and folded from the top down, so each header is where it was
  # measured, folded, when it is clicked.
  section "$FORCES_Y"
  section "$DISPLAY_Y"
  section "$GROUPS_Y"
  shot 657-16-restored-settings
  section "$GROUPS_Y"
  section "$DISPLAY_Y"
  section "$FORCES_Y"

  echo "== kept for a new tab"
  press "CTRL" w
  settle 2
  laid_out open_graph
  shot 657-17-settings-kept

  echo "== Rusty off: no tab restored"
  profile_setting marley.rusty.enabled false
  settle 4
  graphs_before=$(pids | wc -l)
  quit_marley
  open_path ""
  launch_marley
  settle 15
  shot 657-18-off-not-restored
  expect "Marley said why the tab was not restored" \
    grep -q "Rusty is off; the Graph tab is not restored" "$(log_file)"
  expect "no stand-in started" test "$(pids | wc -l)" = "$graphs_before"
  grep -E "graph layout:|Graph tab|graph settings" "$(log_file)" | tail -12 || true
}
