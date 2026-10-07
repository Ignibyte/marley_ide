# shellcheck shell=bash
# #679's visual check: one Rusty button in the rail, and the Rusty home page first in the Rusty
# group. `marley_rusty`'s stand-in `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, over a scratch state
# folder (`RUSTY_STAND_IN_STATE`): a page under `notes`, a decision whose follow-up was due
# yesterday, and `tasks.json` with two made-up lists, never the user's Rusty (R-D8). The page is
# opened once first, so it heads the recent pages.
#
# `679-01-header`: the rail's header with one Rusty button and PROJECTS (REQ-001). `679-02-home`:
# the Rusty button clicked: the home page first among the Rusty group's tabs, the screens' card, the
# recent pages, the follow-up due and the open tasks (REQ-002, REQ-003). `679-03-recent`: the recent
# page clicked: the Brain tab on it (REQ-004). `679-04-screen`: back on the home page, Graph clicked:
# the Graph tab, the home page still first (REQ-005).
compositor sway

# In the window's logical pixels, from the first run's shots: the Rusty button, the home page's first
# recent page and its Graph button, and a point clear of both.
HEADER_Y=${HEADER_Y:-21}
RUSTY_X=${RUSTY_X:-19}
RECENT_X=${RECENT_X:-374}
RECENT_Y=${RECENT_Y:-331}
GRAPH_X=${GRAPH_X:-627}
GRAPH_Y=${GRAPH_Y:-192}
AWAY_X=${AWAY_X:-1000}
AWAY_Y=${AWAY_Y:-900}

setup() {
  local bin=$E2E_WORK/bin vault=$E2E_WORK/rusty/vault
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$vault/notes" "$vault/decisions"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '# A note\n\nA page in the scratch vault.\n' >"$vault/notes/a-note.md"
  printf -- '---\ntitle: Ship the home page\nstatus: decided\ndecided: %s\nfollow_up_by: %s\n---\n# Ship the home page\n\nA made-up decision.\n' \
    "$(date -d '-10 days' +%F)" "$(date -d '-1 day' +%F)" >"$vault/decisions/ship-the-home-page.md"
  cat >"$E2E_WORK/rusty/tasks.json" <<'JSON'
{"groups": [
  {"id": 1, "name": "Home", "tasks": [
    {"id": 11, "title": "Water the plants"},
    {"id": 12, "title": "Call the plumber", "completed": true},
    {"id": 13, "title": "Return the library books"}]},
  {"id": 2, "name": "Work", "tasks": [
    {"id": 21, "title": "Draft the release notes"},
    {"id": 22, "title": "Review the rail change"}]}
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
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  away
  shot 679-01-header

  echo "== the page opened once"
  palette "rusty: open page"
  type_text "a note"
  settle 2
  press "" Return
  settle 3

  echo "== the Rusty button"
  click "$RUSTY_X" "$HEADER_Y"
  settle 4
  away
  shot 679-02-home

  echo "== the recent page"
  click "$RECENT_X" "$RECENT_Y"
  settle 4
  away
  shot 679-03-recent

  echo "== home, then Graph"
  click "$RUSTY_X" "$HEADER_Y"
  settle 2
  click "$GRAPH_X" "$GRAPH_Y"
  settle 4
  away
  shot 679-04-screen
}
