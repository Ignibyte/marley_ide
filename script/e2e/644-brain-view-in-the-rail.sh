# shellcheck shell=bash
# #644's visual check: the rail's Brain view, Rusty's vault behind a switch in the rail's header.
# `marley_rusty`'s stand-in `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, over a scratch state folder
# (`RUSTY_STAND_IN_STATE`) whose `vault/` holds made-up pages, never the user's (R-D8); it logs every
# request with its pid in `calls` and announces each change to its vault, as Rusty's watcher does.
# Rusty starts off, as the harness writes it; turning it on is an edit of the run's file from
# outside (L-607). Every change to the vault goes through Marley's menus, keys and drags, but one:
# `644-19-live`'s page, written from outside as another program would.
#
# Rusty off: PROJECTS (`644-01-header-off`) and the key's toast (`644-02-off-toast`); on: the
# switch (`644-03-switch`), the Brain view (`644-04-brain`), folders unfolded (`644-05-unfolded`), a
# page in a preview tab and kept (`644-06-preview`, `644-07-kept`), Today (`644-08-today`), search
# on Enter (`644-09-search`), the list keys (`644-10-keys`), a folder's menu (`644-11-menu`), a new
# page and a new folder (`644-12-new-page`, `644-13-new-folder`), a rename and a refused one
# (`644-14-renamed`, `644-15-refused`), a move by drag (`644-16-moved`), a delete asked and done
# (`644-17-delete-prompt`, `644-18-deleted`), a change from outside (`644-19-live`), the attention
# dot (`644-20-attention`), the key back to Projects (`644-21-flipped`) and a lost connection
# (`644-22-down`).
compositor sway

# Where the rail's parts sit, from the first runs' shots: the header's buttons, Today, the search
# field, the tree's first row and a row's height, and a point in the tree's empty space.
HEADER_Y=${HEADER_Y:-16}
PROJECTS_X=${PROJECTS_X:-16}
BRAIN_X=${BRAIN_X:-42}
TODAY_X=${TODAY_X:-20}
TODAY_Y=${TODAY_Y:-49}
SEARCH_X=${SEARCH_X:-130}
SEARCH_Y=${SEARCH_Y:-84}
ROW_X=${ROW_X:-110}
FIRST_ROW_Y=${FIRST_ROW_Y:-117}
ROW_H=${ROW_H:-23}
EMPTY_Y=${EMPTY_Y:-880}
# The first toast's close button, the terminal's tab, and a point in the terminal.
TOAST_CLOSE_X=${TOAST_CLOSE_X:-1562}
TOAST_CLOSE_Y=${TOAST_CLOSE_Y:-909}
TERMINAL_TAB_X=${TERMINAL_TAB_X:-400}
TERMINAL_TAB_Y=${TERMINAL_TAB_Y:-51}
TERMINAL_X=${TERMINAL_X:-700}
TERMINAL_Y=${TERMINAL_Y:-600}

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

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# How many times the stand-in was asked to run tool `$1`.
tool_calls() { calls | grep -c " tools/call $1 " || true; }

# Whether the stand-in was asked to run tool `$1` with exactly the arguments `$2`.
called_with() { calls | grep -qF " tools/call $1 $2"; }

# The pid of the stand-in Marley connects to: the one that read the settings last.
marleys_pid() { calls | grep ' tools/call settings_list ' | tail -1 | awk '{print $1}'; }

# A made-up page `$1` in the scratch vault, with the body `$2`.
vault_page() {
  local path=$E2E_WORK/rusty/vault/$1.md
  mkdir -p "$(dirname "$path")"
  printf -- '---\ntitle: %s\n---\n%s\n' "$(basename "$1")" "${2:-A made-up page.}" >"$path"
}

# The center of the tree's row `$1`, counted from 0, in the rail.
row_y() { echo $((FIRST_ROW_Y + $1 * ROW_H)); }

click_row() { click "$ROW_X" "$(row_y "$1")"; }

right_click_row() { click "$ROW_X" "$(row_y "$1")" right; }

# Drags row `$1` onto row `$2`, in small steps, as a hand would.
drag_row() {
  local from to step
  from=$(row_y "$1")
  to=$(row_y "$2")
  pointer_to "$ROW_X" "$from"
  settle 0.3
  pointer_down
  settle 0.3
  for step in 1 2 3 4 5 6; do
    pointer_to "$ROW_X" "$((from + (to - from) * step / 6))"
    sleep 0.1
  done
  settle 0.5
  pointer_up
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
  vault_page daily/2026-01-05
  vault_page decisions/use-a-rail
  vault_page decisions/keep-zed
  vault_page notes/alpha "The first note."
  vault_page notes/beta "The second note, with beta in it."
  vault_page projects/marley/plan
  vault_page projects/marley/shell
  vault_page home
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== Rusty off"
  shot 644-01-header-off
  press "CTRL ALT" v
  settle 2
  shot 644-02-off-toast
  expect "no rusty-mcp ran" test ! -e "$E2E_WORK/rusty/calls"
  click "$TOAST_CLOSE_X" "$TOAST_CLOSE_Y"
  settle 1

  echo "== Rusty on"
  profile_setting marley.rusty.enabled true
  settle 8
  shot 644-03-switch
  click "$BRAIN_X" "$HEADER_Y"
  settle 3
  shot 644-04-brain
  expect "the Brain view read the vault" bash -c "grep -q ' tools/call brain_tree ' '$E2E_WORK/rusty/calls'"

  # The top level, `archive/` left out as Rusty leaves it: daily 0, decisions 1, notes 2,
  # projects 3, home 4.
  echo "== folders unfolded"
  click_row 3
  settle 1
  # projects/marley at 4.
  click_row 4
  settle 1
  shot 644-05-unfolded

  echo "== a page in a preview tab, then kept"
  click_row 2
  settle 1
  # notes open: alpha 3, beta 4.
  click_row 3
  settle 3
  shot 644-06-preview
  click_row 3
  click_row 3
  settle 3
  shot 644-07-kept

  echo "== Today"
  click "$TODAY_X" "$TODAY_Y"
  settle 3
  shot 644-08-today
  expect "Today asked for the daily note" bash -c "grep -q ' tools/call brain_daily_note ' '$E2E_WORK/rusty/calls'"

  echo "== search on Enter"
  click "$SEARCH_X" "$SEARCH_Y"
  settle 1
  type_text "beta"
  settle 2
  expect "nothing is searched while typing" test "$(tool_calls brain_search)" = 0
  press "" Return
  settle 2
  shot 644-09-search
  expect "Enter searched once" test "$(tool_calls brain_search)" = 1

  # daily is open since Today: daily 0, its two notes 1 and 2, decisions 3, notes 4, alpha 5,
  # beta 6, projects 7, marley 8, plan 9, shell 10, home 11.
  echo "== the list keys"
  press "" Escape
  settle 1
  press "" Escape
  settle 1
  press "" Home
  for _ in 1 2 3 4; do
    press "" Down
  done
  settle 1
  press "" Left
  settle 1
  press "" Right
  settle 1
  press "" Down
  settle 1
  press "" Return
  settle 3
  shot 644-10-keys

  echo "== a folder's menu"
  right_click_row 4
  settle 1
  shot 644-11-menu

  echo "== a new page"
  press "" Return
  settle 1
  type_text "gamma"
  press "" Return
  settle 3
  shot 644-12-new-page
  expect "the page was made with brain_new_page" \
    called_with brain_new_page '{"folder": "notes", "name": "gamma"}'

  # notes: alpha 5, beta 6, gamma 7.
  echo "== a new folder"
  right_click_row 4
  settle 1
  press "" Down
  press "" Return
  settle 1
  type_text "drafts"
  press "" Return
  settle 3
  shot 644-13-new-folder
  expect "the folder was made with brain_new_folder" \
    called_with brain_new_folder '{"path": "notes/drafts"}'

  # notes: drafts 5, alpha 6, beta 7, gamma 8.
  echo "== a rename"
  right_click_row 8
  settle 1
  press "" Down
  press "" Return
  settle 1
  type_text "delta"
  press "" Return
  settle 3
  shot 644-14-renamed
  expect "the page was renamed with brain_rename" \
    called_with brain_rename '{"from": "notes/gamma", "to": "notes/delta"}'

  echo "== a refused rename"
  right_click_row 8
  settle 1
  press "" Down
  press "" Return
  settle 1
  type_text "alpha"
  press "" Return
  settle 2
  shot 644-15-refused

  echo "== a move by drag"
  drag_row 8 3
  settle 3
  shot 644-16-moved
  expect "the page was moved with brain_rename into decisions" \
    called_with brain_rename '{"from": "notes/delta", "to": "decisions/"}'

  # decisions open: delta 4, keep-zed 5, use-a-rail 6; notes 7, drafts 8, alpha 9, beta 10;
  # projects 11, marley 12, plan 13, shell 14; home 15.
  echo "== a delete, asked first"
  right_click_row 12
  settle 1
  press "" Down
  press "" Down
  press "" Down
  press "" Return
  settle 2
  shot 644-17-delete-prompt
  press "" Return
  settle 3
  click "$ROW_X" "$EMPTY_Y"
  settle 1
  shot 644-18-deleted
  expect "the folder was deleted with brain_delete_folder" \
    called_with brain_delete_folder '{"path": "projects/marley"}'

  echo "== a change from outside"
  vault_page notes/epsilon "Written by another program."
  settle 4
  shot 644-19-live

  echo "== the attention dot"
  # The terminal's tab first: the pages opened since stand in front of it.
  click "$TERMINAL_TAB_X" "$TERMINAL_TAB_Y"
  settle 1
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  type_text "sleep 3; printf '\\a'"
  press "" Return
  settle 0.5
  click "$ROW_X" "$EMPTY_Y"
  settle 6
  shot 644-20-attention

  echo "== the key back to Projects"
  press "CTRL ALT" v
  settle 2
  shot 644-21-flipped
  press "CTRL ALT" v
  settle 2

  echo "== the connection lost"
  mv "$E2E_WORK/bin/rusty-mcp" "$E2E_WORK/bin/rusty-mcp.away"
  kill "$(marleys_pid)"
  settle 14
  shot 644-22-down
  # Every change the Brain view made, for REQ-024's review: each one a tool call.
  calls | grep 'tools/call brain_' | sed 's/^[0-9]* //'
}
