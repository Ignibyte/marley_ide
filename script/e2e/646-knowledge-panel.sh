# shellcheck shell=bash
# #646's visual check: the Knowledge panel in the right dock. `marley_rusty`'s stand-in `rusty-mcp`,
# named by `MARLEY_RUSTY_MCP`, over a scratch state folder (`RUSTY_STAND_IN_STATE`) whose `vault/`
# holds three made-up pages, never the user's (R-D8); it logs every tool call with its arguments in
# `calls` and announces each change to its vault. Rusty starts off, as the harness writes it, and is
# turned on and off by edits of the run's settings from outside (L-607). The run's keymap binds
# Ctrl+Alt+Shift+D to `rusty::OpenPage` for `projects/demo`.
#
# Rusty off: no Knowledge button (`646-01-off-no-button`), the toggle's toast (`646-02-off-toast`);
# on: the panel with no page (`646-03-no-page`), Demo's tags, backlinks and links (`646-04-demo-page`),
# a backlink followed (`646-05-followed`), another item (`646-06-other-item`), Create
# (`646-07-created`), a tag searched (`646-08-tag-search`), a query on Enter (`646-09-search`), the
# two toggles (`646-10-toggles`), Enter on a result (`646-11-enter-opens`), Escape
# (`646-12-escape`), a change from outside (`646-13-refresh`), Rusty off while the panel shows
# (`646-14-off-live`).
compositor sway

# Where the panel's parts sit, from the first runs' shots.
SEARCH_X=${SEARCH_X:-1400}
SEARCH_Y=${SEARCH_Y:-81}
CASE_X=${CASE_X:-1561}
REGEX_X=${REGEX_X:-1583}
TOGGLE_Y=${TOGGLE_Y:-80}
TAG_X=${TAG_X:-1405}
TAG_Y=${TAG_Y:-169}
BACKLINK_X=${BACKLINK_X:-1310}
BACKLINK_Y=${BACKLINK_Y:-230}
CREATE_X=${CREATE_X:-1566}
CREATE_Y=${CREATE_Y:-401}
# The toast's close button, bottom right.
TOAST_CLOSE_X=${TOAST_CLOSE_X:-1563}
TOAST_CLOSE_Y=${TOAST_CLOSE_Y:-909}

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# How many times the stand-in was asked `$1` with arguments holding `$2`.
asked() { calls | grep " tools/call $1 " | grep -cF -- "$2" || true; }

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

demo() {
  press "CTRL ALT SHIFT" d
  settle 4
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  expect "the harness's copy turns Rusty off" setting_is marley.rusty.enabled false
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '%s\n' '[{"context": "Workspace", "bindings": {"ctrl-alt-shift-d": ["rusty::OpenPage", {"slug": "projects/demo"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  vault_page projects/demo <<'PAGE'
---
title: Demo
type: project
tags: [project, area/demo]
---
# Demo

A demo project, still a #draft.

It orbits [[concepts/orbit|Orbit]] and waits on [[Missing Page]].
PAGE
  vault_page concepts/orbit <<'PAGE'
---
title: Orbit
tags: [area/demo]
---
# Orbit

The idea [[projects/demo]] runs on.
PAGE
  vault_page decisions/demo-uses-orbit <<'PAGE'
---
title: Demo uses Orbit
---
# Demo uses Orbit

We decided that [[projects/demo|Demo]] keeps orbit.
PAGE
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== Rusty off"
  shot 646-01-off-no-button
  palette "rusty: toggle knowledge panel"
  settle 2
  shot 646-02-off-toast
  click "$TOAST_CLOSE_X" "$TOAST_CLOSE_Y"
  settle 1

  echo "== Rusty on, no page"
  profile_setting marley.rusty.enabled true
  settle 8
  palette "rusty: toggle knowledge panel"
  settle 3
  shot 646-03-no-page

  echo "== Demo's page"
  demo
  shot 646-04-demo-page
  expect "the panel read Demo's links" test "$(asked brain_get_links '"projects/demo"')" -ge 1

  echo "== a backlink followed"
  click "$BACKLINK_X" "$BACKLINK_Y"
  settle 4
  shot 646-05-followed

  echo "== another item"
  palette "workspace: new file"
  settle 3
  shot 646-06-other-item

  echo "== Create"
  demo
  click "$CREATE_X" "$CREATE_Y"
  settle 4
  shot 646-07-created
  expect "Create made the page at its path" test "$(asked brain_new_page '"path": "Missing Page"')" = 1

  echo "== a tag searched"
  demo
  click "$TAG_X" "$TAG_Y"
  settle 3
  shot 646-08-tag-search
  expect "the tag's query went to brain search" test "$(asked brain_search '"query": "tag:area/demo"')" -ge 1

  echo "== a query on Enter"
  click "$SEARCH_X" "$SEARCH_Y"
  settle 1
  press "CTRL" a
  type_text "orbit"
  settle 2
  expect "nothing is sent while typing" test "$(asked brain_search '"query": "orbit"')" = 0
  press "" Return
  settle 3
  shot 646-09-search
  expect "Enter sent the query once" test "$(asked brain_search '"query": "orbit"')" = 1

  echo "== the two toggles"
  click "$CASE_X" "$TOGGLE_Y"
  settle 2
  click "$REGEX_X" "$TOGGLE_Y"
  settle 3
  shot 646-10-toggles
  expect "the query went again with both options" \
    test "$(asked brain_search '"case_sensitive": true, "limit": 60, "query": "orbit", "regex": true')" -ge 1

  echo "== Enter on a result"
  click "$SEARCH_X" "$SEARCH_Y"
  settle 1
  press "" Down
  press "" Return
  settle 4
  shot 646-11-enter-opens

  echo "== Escape"
  demo
  click "$SEARCH_X" "$SEARCH_Y"
  settle 1
  press "CTRL" a
  type_text "orbit"
  settle 1
  press "" Escape
  settle 2
  shot 646-12-escape

  echo "== a change from outside"
  vault_page notes/later <<'PAGE'
---
title: Later
---
Written later, about [[projects/demo]].
PAGE
  settle 5
  shot 646-13-refresh

  echo "== Rusty off while the panel shows"
  profile_setting marley.rusty.enabled false
  settle 6
  shot 646-14-off-live
  calls | grep -E 'tools/call (brain_search|brain_new_page)' | sed 's/^[0-9]* //'
}
