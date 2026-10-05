# shellcheck shell=bash
# #655's visual check: the Knowledge panel's project view. `marley_rusty`'s stand-in `rusty-mcp`,
# named by `MARLEY_RUSTY_MCP`, over a scratch state folder (`RUSTY_STAND_IN_STATE`) whose `vault/`
# holds made-up pages and whose `tasks.json` holds two task groups, never the user's (R-D8). The
# window opens a scratch folder named `demo`, which the project page Demo lists in its `path:`.
# Every change is an edit of the vault from outside, which the stand-in announces; each write's
# check reads the stand-in's call log.
#
# By path (`655-01-by-path`), Open Page (`655-02-open-page`), back (`655-03-back`), a task_group
# written (`655-04-group-property`), the Graph tab on the project (`655-05-graph-project`), a
# follow-up opened (`655-06-follow-up-opens`), a page in front wins (`655-07-graph-page-wins`), the
# local graph on the project (`655-08-graph-local`), by name (`655-09-by-name`), a name tie
# (`655-10-name-tie`), linked from the tie (`655-11-linked-from-tie`), none (`655-12-none`), the
# page picker (`655-13-page-picker`), linked (`655-14-page-linked`), the group picker
# (`655-15-group-picker`), the group linked (`655-16-group-linked`).
compositor sway

# Where the panel's parts sit, from the first runs' shots.
OPEN_PAGE_X=${OPEN_PAGE_X:-1545}
OPEN_PAGE_Y=${OPEN_PAGE_Y:-174}
FOLLOW_UP_X=${FOLLOW_UP_X:-1380}
FOLLOW_UP_Y=${FOLLOW_UP_Y:-235}
LINK_X=${LINK_X:-1565}
LINK_Y=${LINK_Y:-182}

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# How many times the stand-in was asked `$1` with arguments holding `$2`.
asked() { calls | grep " tools/call $1 " | grep -cF -- "$2" || true; }

# Whether the stand-in was asked to set `$1` on `$2` to the JSON value `$3`.
wrote() {
  calls | python3 -c '
import json, sys
key, slug, value = sys.argv[1], sys.argv[2], json.loads(sys.argv[3])
for line in sys.stdin:
    parts = line.split(" ", 3)
    if len(parts) == 4 and parts[2] == "brain_set_property":
        arguments = json.loads(parts[3])
        if arguments == {"slug": slug, "key": key, "value": value}:
            sys.exit(0)
sys.exit(1)' "$1" "$2" "$3"
}

vault() { echo "$E2E_WORK/rusty/vault"; }

# A made-up page `$1` in the stand-in's scratch vault, its text from stdin.
vault_page() {
  local path
  path=$(vault)/$1.md
  mkdir -p "$(dirname "$path")"
  cat >"$path"
}

# Demo's page, with the frontmatter lines `$1` (path and task_group as wanted).
demo_page() {
  vault_page projects/demo <<PAGE
---
title: Demo
type: project
summary: A made-up project the scenario links to its folder.
$1
---
# Demo

A demo project. It orbits [[concepts/orbit|Orbit]].
PAGE
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repos/demo" "$E2E_WORK/home" "$(vault)/archive"
  FOLDER=$(realpath "$E2E_WORK/repos/demo")
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  demo_page "path: $FOLDER"
  vault_page concepts/orbit <<'PAGE'
---
title: Orbit
---
# Orbit

The idea [[projects/demo]] runs on.
PAGE
  vault_page decisions/demo-uses-orbit <<'PAGE'
---
title: Demo uses Orbit
status: decided
follow_up_by: 2026-01-05
---
# Demo uses Orbit

We decided that [[projects/demo|Demo]] keeps orbit.
PAGE
  vault_page decisions/demo-keeps-its-name <<'PAGE'
---
title: Demo keeps its name
status: decided
follow_up_by: 2099-01-01
---
# Demo keeps its name

[[projects/demo]] stays Demo.
PAGE
  vault_page decisions/orbit-gets-a-ring <<'PAGE'
---
title: Orbit gets a ring
status: decided
follow_up_by: 2026-01-06
---
# Orbit gets a ring

[[concepts/orbit]] gets a ring.
PAGE
  vault_page projects/orbit-site <<'PAGE'
---
title: Orbit Site
type: project
path: /elsewhere/orbit-site
---
# Orbit Site

The site of Orbit.
PAGE
  cat >"$E2E_WORK/rusty/tasks.json" <<'JSON'
{"groups": [
  {"id": 1, "name": "Demo", "tasks": [
    {"id": 1, "title": "Write the README"},
    {"id": 2, "title": "Sketch the ring"},
    {"id": 3, "title": "Pick a name", "completed": true}]},
  {"id": 2, "name": "Chores", "tasks": [
    {"id": 4, "title": "Water the plants"}]}
]}
JSON
  printf '# demo\n' >"$FOLDER/README.md"
  open_path "$FOLDER"
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== by path"
  palette "rusty: toggle knowledge panel"
  settle 5
  shot 655-01-by-path
  expect "no write before a pick" test "$(asked brain_set_property '')" = 0

  echo "== Open Page"
  click "$OPEN_PAGE_X" "$OPEN_PAGE_Y"
  settle 4
  shot 655-02-open-page

  echo "== back to the project view"
  press CTRL w
  settle 3
  shot 655-03-back

  echo "== a task_group written"
  demo_page "path: $FOLDER
task_group: Chores"
  settle 8
  shot 655-04-group-property

  echo "== the Graph tab on the project"
  palette "rusty: open graph"
  settle 5
  shot 655-05-graph-project
  expect "the graph was read around Demo" test "$(asked brain_graph '"around": "projects/demo"')" -ge 1

  echo "== a follow-up opens its decision"
  click "$FOLLOW_UP_X" "$FOLLOW_UP_Y"
  settle 4
  shot 655-06-follow-up-opens

  echo "== a page in front wins"
  palette "rusty: open graph"
  settle 5
  shot 655-07-graph-page-wins

  echo "== the local graph on the project"
  press CTRL w
  settle 2
  press CTRL w
  settle 2
  palette "rusty: open local graph"
  settle 5
  shot 655-08-graph-local
  press CTRL w
  settle 2

  echo "== by name"
  demo_page "path: old laptop ~/code/demo
task_group: Chores"
  settle 8
  shot 655-09-by-name
  expect "still no write" test "$(asked brain_set_property '')" = 0

  echo "== a name tie"
  vault_page projects/demo-2 <<'PAGE'
---
title: Demo
type: project
---
# Demo

A copy of Demo.
PAGE
  settle 8
  shot 655-10-name-tie

  echo "== linked from the tie"
  click "$LINK_X" "$LINK_Y"
  settle 8
  shot 655-11-linked-from-tie
  expect "the folder was added to Demo's path" wrote path projects/demo "\"old laptop ~/code/demo, $FOLDER\""

  echo "== no page"
  rm -f "$(vault)/projects/demo.md" "$(vault)/projects/demo-2.md"
  vault_page projects/demo-site <<'PAGE'
---
title: Demo Site
type: project
---
# Demo Site

Where Demo will live.
PAGE
  settle 8
  shot 655-12-none

  echo "== the page picker"
  palette "rusty: link project page"
  settle 2
  shot 655-13-page-picker
  type_text "demo"
  settle 2
  press "" Return
  settle 8
  shot 655-14-page-linked
  expect "the folder was written to Demo Site's path" wrote path projects/demo-site "\"$FOLDER\""

  echo "== the group picker"
  palette "rusty: link task group"
  settle 2
  type_text "chores"
  settle 2
  shot 655-15-group-picker
  press "" Return
  settle 8
  shot 655-16-group-linked
  expect "Chores was written to Demo Site's task_group" wrote task_group projects/demo-site '"Chores"'
  expect "three writes in all" test "$(asked brain_set_property '')" = 3
  calls | grep -E 'brain_set_property|brain_list_pages' | tail -8
}
