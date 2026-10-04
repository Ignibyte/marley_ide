# shellcheck shell=bash
# #647's visual check: the Graph tab. `marley_rusty`'s stand-in `rusty-mcp`, named by
# `MARLEY_RUSTY_MCP`, over a scratch state folder (`RUSTY_STAND_IN_STATE`) whose `vault/` holds
# twelve made-up pages of six types, never the user's (R-D8): links, a decision's three typed edges,
# the tags `storage` and `storage/cold`, a link to a missing page and two orphans. It logs every
# tool call with its arguments in `calls` and announces each change to its vault. Rusty is turned
# on in the run's settings before Marley starts and off by an edit from outside (L-607). The run's
# keymap binds Ctrl+Alt+Shift+Y to `rusty::OpenPage` for `projects/orbit`, and Ctrl+Alt+Shift+K
# for `decisions/old-orbit-plan`, a neighbour of the decision graph's centre. The layout has no
# randomness, so the places clicked, measured from the first run's shots, hold from run to run; a
# step that changes the graph waits for Marley.log's next "graph layout:" line, the run settling.
#
# `647-01-vault` the vault; `647-02-click-opens-page` a node clicked after a filter and Fit;
# `647-03-local` orbit's neighbourhood; `647-04-depth-2`; `647-05-hover`; `647-06-decision-edges`;
# `647-07-follows-page`; `647-08-type-hidden`; `647-09-filter-tag`; `647-10-filter-path`;
# `647-11-tags`; `647-12-unresolved`; `647-13-no-decision-edges`; `647-14-no-orphans`;
# `647-15-pan`; `647-16-zoom`; `647-17-drag-node`; `647-18-before-change` and
# `647-19-after-change`; `647-20-capped`; `647-21-rail-graph`; `647-22-off`.
compositor sway

# Where things sit, from the first runs' shots: the canvas's middle, the panel's controls, the
# rail's Graph entry, and the nodes clicked.
CENTRE_X=${CENTRE_X:-810}
CENTRE_Y=${CENTRE_Y:-518}
FILTER_X=${FILTER_X:-1230}
FILTER_Y=${FILTER_Y:-185}
FIT_X=${FIT_X:-1310}
FIT_Y=${FIT_Y:-95}
VAULT_X=${VAULT_X:-1180}
SCOPE_Y=${SCOPE_Y:-125}
DEPTH_2_X=${DEPTH_2_X:-1185}
DEPTH_Y=${DEPTH_Y:-151}
SWITCH_X=${SWITCH_X:-1131}
TAGS_Y=${TAGS_Y:-217}
UNRESOLVED_Y=${UNRESOLVED_Y:-239}
DECISIONS_Y=${DECISIONS_Y:-261}
ORPHANS_Y=${ORPHANS_Y:-283}
DECISION_TYPE_X=${DECISION_TYPE_X:-1158}
DECISION_TYPE_Y=${DECISION_TYPE_Y:-374}
EMPTY_X=${EMPTY_X:-400}
EMPTY_Y=${EMPTY_Y:-850}
ZOOM_NODE_X=${ZOOM_NODE_X:-805}
ZOOM_NODE_Y=${ZOOM_NODE_Y:-457}
RAIL_GRAPH_X=${RAIL_GRAPH_X:-44}
RAIL_GRAPH_Y=${RAIL_GRAPH_Y:-49}

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# How many times the stand-in was asked `$1` with arguments holding `$2`.
asked() { calls | grep " tools/call $1 " | grep -cF -- "$2" || true; }

layouts() { grep -c "graph layout:" "$E2E_PROFILE/logs/Marley.log" 2>/dev/null || true; }

# Waits up to 30 s for a layout to settle after `$1` had, then until no later run settles for
# half a second: typing a filter starts a run a key.
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

# Whether the run's copy of the settings holds the JSON value `$2` at the dotted key path `$1`.
setting_is() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'SETTINGS'
import json, pathlib, sys

node = json.loads(pathlib.Path(sys.argv[1]).read_text())
for key in sys.argv[2].split("."):
    node = node.get(key) if isinstance(node, dict) else None
sys.exit(0 if node == json.loads(sys.argv[3]) else 1)
SETTINGS
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

# The filter field holding `$1`, or empty with no argument.
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

fit() {
  click "$FIT_X" "$FIT_Y"
  settle 1
}

orbit_page() {
  press "CTRL ALT SHIFT" y
  settle 3
}

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

# 3,000 made-up pages under `gen/`, each linking up to three others, a few of them hubs; seeded,
# so every run writes the same vault.
generated_vault() {
  python3 - "$E2E_WORK/rusty/vault/gen" <<'GENERATE'
import os, random, sys

folder, count = sys.argv[1], 3000
rng = random.Random(647)
os.makedirs(folder, exist_ok=True)
weights = [1.0 / (at + 1) ** 0.8 for at in range(count)]
for at in range(count):
    targets = {rng.choices(range(count), weights)[0] for _ in range(3)} - {at}
    body = " ".join(f"[[gen/page-{target:04d}]]" for target in sorted(targets))
    with open(os.path.join(folder, f"page-{at:04d}.md"), "w", encoding="utf-8") as page:
        page.write(f"---\ntitle: Page {at:04d}\n---\nLinks {body}.\n")
GENERATE
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  expect "the harness's copy turns Rusty off" setting_is marley.rusty.enabled false
  profile_setting marley.rusty.connection '"embedded"'
  profile_setting marley.rusty.enabled true
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '%s\n' '[{"context": "Workspace", "bindings": {"ctrl-alt-shift-y": ["rusty::OpenPage", {"slug": "projects/orbit"}], "ctrl-alt-shift-k": ["rusty::OpenPage", {"slug": "decisions/old-orbit-plan"}]}}]' \
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

  echo "== the vault"
  laid_out palette "rusty: open graph"
  settle 1
  shot 647-01-vault

  echo "== a filter, Fit and a click"
  laid_out filter_to "path:projects/orbit"
  fit
  click "$CENTRE_X" "$CENTRE_Y"
  settle 3
  shot 647-02-click-opens-page

  echo "== the local graph"
  palette "rusty: open local graph"
  settle 3
  laid_out filter_to
  settle 1
  shot 647-03-local
  expect "the local graph asked around orbit" test "$(asked brain_graph '"around": "projects/orbit"')" -ge 1

  echo "== depth 2"
  laid_out click "$DEPTH_2_X" "$DEPTH_Y"
  shot 647-04-depth-2
  expect "depth 2 was asked" test "$(asked brain_graph '"depth": 2')" -ge 1

  echo "== hover"
  pointer_to "$CENTRE_X" "$CENTRE_Y"
  settle 1
  shot 647-05-hover

  echo "== decision edges"
  laid_out click "$VAULT_X" "$SCOPE_Y"
  laid_out filter_to "path:decisions/use-orbit"
  fit
  click "$CENTRE_X" "$CENTRE_Y"
  settle 3
  palette "rusty: open local graph"
  settle 3
  laid_out filter_to
  shot 647-06-decision-edges

  echo "== a neighbour followed"
  press "CTRL ALT SHIFT" k
  settle 3
  laid_out palette "rusty: open graph"
  shot 647-07-follows-page

  echo "== a type hidden"
  laid_out click "$VAULT_X" "$SCOPE_Y"
  fit
  laid_out click "$DECISION_TYPE_X" "$DECISION_TYPE_Y"
  shot 647-08-type-hidden
  laid_out click "$DECISION_TYPE_X" "$DECISION_TYPE_Y"

  echo "== filters"
  laid_out filter_to "tag:storage"
  shot 647-09-filter-tag
  laid_out filter_to "path:research/"
  shot 647-10-filter-path
  laid_out filter_to

  echo "== switches"
  laid_out click "$SWITCH_X" "$TAGS_Y"
  shot 647-11-tags
  laid_out click "$SWITCH_X" "$TAGS_Y"
  laid_out click "$SWITCH_X" "$UNRESOLVED_Y"
  shot 647-12-unresolved
  expect "unresolved targets were asked for" test "$(asked brain_graph '"unresolved": true')" -ge 1
  laid_out click "$SWITCH_X" "$DECISIONS_Y"
  shot 647-13-no-decision-edges
  laid_out click "$SWITCH_X" "$ORPHANS_Y"
  shot 647-14-no-orphans
  laid_out click "$SWITCH_X" "$ORPHANS_Y"
  laid_out click "$SWITCH_X" "$DECISIONS_Y"
  laid_out click "$SWITCH_X" "$UNRESOLVED_Y"
  fit

  echo "== pan"
  pointer_to "$EMPTY_X" "$EMPTY_Y"
  pointer_down
  pointer_to $((EMPTY_X + 100)) $((EMPTY_Y - 50))
  pointer_to $((EMPTY_X + 200)) $((EMPTY_Y - 100))
  pointer_up
  settle 1
  shot 647-15-pan

  echo "== zoom"
  fit
  pointer_to "$ZOOM_NODE_X" "$ZOOM_NODE_Y"
  # The wheel up, which zooms in.
  scroll -5
  settle 1
  shot 647-16-zoom

  echo "== a node dragged"
  orbit_page
  laid_out palette "rusty: open local graph"
  settle 1
  pointer_to "$CENTRE_X" "$CENTRE_Y"
  pointer_down
  pointer_to $((CENTRE_X + 75)) "$CENTRE_Y"
  pointer_to $((CENTRE_X + 150)) "$CENTRE_Y"
  settle 1
  shot 647-17-drag-node
  pointer_up
  settle 2

  echo "== a change from outside"
  shot 647-18-before-change
  laid_out vault_page notes/new-idea <<'PAGE'
---
title: New idea
---
Builds on [[projects/orbit]].
PAGE
  settle 1
  shot 647-19-after-change

  echo "== the cap"
  local before
  generated_vault
  settle 4
  before=$(layouts)
  click "$VAULT_X" "$SCOPE_Y"
  settle 2
  shot 647-20-capped
  await_layout "$before"
  grep "graph layout:" "$E2E_PROFILE/logs/Marley.log" | tail -1 || true
  expect "the vault laid out at the cap within 300 steps" \
    grep -qE "graph layout: 2000 nodes, [0-9]+ edges, ([0-9]|[0-9][0-9]|[12][0-9][0-9]|300) steps" "$E2E_PROFILE/logs/Marley.log"

  echo "== the rail's Graph entry"
  orbit_page
  press "CTRL ALT" v
  settle 3
  click "$RAIL_GRAPH_X" "$RAIL_GRAPH_Y"
  settle 3
  shot 647-21-rail-graph

  echo "== Rusty off"
  local graphs
  profile_setting marley.rusty.enabled false
  settle 3
  graphs=$(asked brain_graph '')
  settle 5
  shot 647-22-off
  expect "no graph was asked once Rusty was off" test "$(asked brain_graph '')" = "$graphs"
  grep "graph layout:" "$E2E_PROFILE/logs/Marley.log" || true
}
