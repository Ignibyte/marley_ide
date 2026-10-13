# shellcheck shell=bash
# #782's visual check: one Home page, with Rusty's cards while Rusty is on, and Home's row first
# under Default in the rail. A fresh profile opens `repo` with Rusty on through `marley_rusty`'s
# stand-in `rusty-mcp` (`MARLEY_RUSTY_MCP`), over a scratch state folder seeded as #679's scenario
# seeds it: a page under `notes`, a decision whose follow-up was due yesterday, and `tasks.json`
# with two made-up lists, never the user's Rusty.
#
# `782-01-home-rusty`: Default's header clicked: Home with Marley's cards, then the RUSTY card, the
# recent pages and the follow-up due; the status bar's right end with Home alone (REQ-001, REQ-004,
# REQ-005).
# `782-02-home-bottom`: Home scrolled down: TASKS, AGENT ACTIVITY, CONFIGURE last (REQ-001,
# REQ-004).
# `782-03-rail-home-first`: Home's New Terminal: the rail lists Home, then the terminal, under
# Default (REQ-008).
# `782-04-open-home`: from the terminal, `rusty: open home` in the palette: Home in front (REQ-006).
# `782-05-verb-home`: `marley rusty` typed in that terminal, the debug binary first on its PATH:
# Home in front (REQ-007).
# `782-06-not-connected`: the stand-in swapped for one that fails, and stopped: one RUSTY card with
# the reason (REQ-002).
# `782-07-rusty-off`: `marley.rusty.enabled` off: Home with no Rusty card (REQ-003).
compositor sway

# In the window's logical pixels, from the first run's shots: Default's header in the rail, Home's
# New Terminal, the terminal's row in the rail, a point over Home's cards, a point clear of all.
DEFAULT_X=${DEFAULT_X:-100}
DEFAULT_Y=${DEFAULT_Y:-89}
TERMINAL_X=${TERMINAL_X:-354}
TERMINAL_Y=${TERMINAL_Y:-134}
TERMINAL_ROW_X=${TERMINAL_ROW_X:-100}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-173}
PAGE_X=${PAGE_X:-800}
PAGE_Y=${PAGE_Y:-500}
AWAY_X=${AWAY_X:-1000}
AWAY_Y=${AWAY_Y:-900}
# A first run that stops after the first shot, to read the positions.
STOP_AT=${STOP_AT:-}

setup() {
  local bin=$E2E_WORK/bin vault=$E2E_WORK/rusty/vault
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$vault/notes" "$vault/decisions"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  ln -s "$(marley_binary)" "$bin/marley"
  terminal_env PATH "$bin:$PATH"
  rm -rf "$E2E_PROFILE/db"
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
  # Trusts repo.
  press "" Return
  settle 3

  echo "== Home in Default, Rusty on"
  click "$DEFAULT_X" "$DEFAULT_Y"
  settle 4
  away
  shot 782-01-home-rusty
  [[ $STOP_AT == 1 ]] && return 0

  echo "== Home's end"
  pointer_to "$PAGE_X" "$PAGE_Y"
  scroll 20
  settle 2
  shot 782-02-home-bottom
  scroll -20
  settle 1

  echo "== a terminal in Default"
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 4
  away
  shot 782-03-rail-home-first

  echo "== rusty: open home"
  palette "rusty: open home"
  away
  shot 782-04-open-home

  echo "== marley rusty in the terminal"
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y"
  settle 2
  type_text "marley rusty"
  press "" Return
  settle 4
  away
  shot 782-05-verb-home

  echo "== Rusty not connected"
  rm "$E2E_WORK/bin/rusty-mcp"
  printf '#!/bin/sh\nexit 1\n' >"$E2E_WORK/bin/rusty-mcp"
  chmod +x "$E2E_WORK/bin/rusty-mcp"
  pkill -f "$E2E_WORK/bin/rusty-mcp" || true
  # Marley pings Rusty every 5 s and gives the ping 5 s, so a stopped server is down within 10 s.
  settle 15
  away
  shot 782-06-not-connected

  echo "== Rusty off"
  profile_setting marley.rusty.enabled false
  settle 4
  away
  shot 782-07-rusty-off
}
