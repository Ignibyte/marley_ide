# shellcheck shell=bash
# #658's visual check: the Tasks tab over Rusty's to-do lists. `marley_rusty`'s stand-in
# `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, serves Rusty's twelve task tools over a scratch
# `tasks.json` this setup writes (made-up lists Home, Work and Someday), never the user's store
# (R-D8), and logs each call with its pid in `calls`. The stand-in does not watch `tasks.json`, so
# the scenario's own edit of it is a "quiet write", as another `rusty-mcp`'s write reaches no other
# process; a file written into the scratch vault, which its watcher sees, makes it send
# `list_changed`, as `rusty-cli refresh` does. A made-up project page links the window's folder to
# Work for the Knowledge panel's Open in Tasks.
#
# `658-01-from-rail` to `658-28-from-project`, one per step below.
compositor sway

# Where things sit, from the first runs' shots.
BRAIN_X=${BRAIN_X:-42}
RAIL_HEADER_Y=${RAIL_HEADER_Y:-16}
TASKS_ENTRY_X=${TASKS_ENTRY_X:-70}
FIXED_ROW_Y=${FIXED_ROW_Y:-49}
SEARCH_X=${SEARCH_X:-90}
SEARCH_Y=${SEARCH_Y:-85}
LIST_X=${LIST_X:-300}
FIRST_LIST_Y=${FIRST_LIST_Y:-113}
LIST_H=${LIST_H:-31}
NEW_LIST_X=${NEW_LIST_X:-480}
NEW_LIST_Y=${NEW_LIST_Y:-82}
ADD_X=${ADD_X:-900}
ADD_Y=${ADD_Y:-126}
TITLE_X=${TITLE_X:-600}
CHECK_X=${CHECK_X:-523}
FIRST_TASK_Y=${FIRST_TASK_Y:-166}
TASK_H=${TASK_H:-31}
SHOW_ARCHIVED_X=${SHOW_ARCHIVED_X:-1222}
REFRESH_X=${REFRESH_X:-1337}
HEADER_Y=${HEADER_Y:-88}
TASKS_TAB_X=${TASKS_TAB_X:-515}
TERMINAL_TAB_X=${TERMINAL_TAB_X:-400}
TAB_Y=${TAB_Y:-51}
MENU_ENTRY_H=${MENU_ENTRY_H:-23}
OPEN_IN_TASKS_X=${OPEN_IN_TASKS_X:-1584}
OPEN_IN_TASKS_Y=${OPEN_IN_TASKS_Y:-313}

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# How many times the stand-in was asked `$1` with arguments holding `$2`.
asked() { calls | grep " tools/call $1 " | grep -cF -- "${2:-}" || true; }

# Whether the stand-in was asked `$1` with exactly the JSON arguments `$2`.
called_with() {
  calls | python3 -c '
import json, sys
tool, wanted = sys.argv[1], json.loads(sys.argv[2])
for line in sys.stdin:
    parts = line.rstrip("\n").split(" ", 3)
    if len(parts) == 4 and parts[1] == "tools/call" and parts[2] == tool:
        if json.loads(parts[3]) == wanted:
            sys.exit(0)
sys.exit(1)' "$1" "$2"
}

tasks_json() { echo "$E2E_WORK/rusty/tasks.json"; }

# A quiet write: task `$2` titled `$3` added to list `$1` by an edit of the file, as another
# process's write; or with `$3` empty, task `$2` removed.
quiet_write() {
  python3 - "$(tasks_json)" "$1" "$2" "${3:-}" <<'WRITE'
import json, sys

path, list_id, task_id, title = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), sys.argv[4]
with open(path, encoding="utf-8") as stored:
    data = json.load(stored)
for group in data["groups"]:
    if group["id"] == list_id:
        if title:
            group["tasks"].append({"id": task_id, "title": title})
        else:
            group["tasks"] = [task for task in group["tasks"] if task["id"] != task_id]
with open(path, "w", encoding="utf-8") as stored:
    json.dump(data, stored, indent=2)
WRITE
}

# A file written into the scratch vault: the stand-in's watcher sends `list_changed`.
announce() {
  date +%s%N >"$E2E_WORK/rusty/vault/refresh.txt"
  settle 2
}

list_y() { echo $((FIRST_LIST_Y + $1 * LIST_H)); }
task_y() { echo $((FIRST_TASK_Y + $1 * TASK_H)); }

click_list() { click "$LIST_X" "$(list_y "$1")"; }
click_task() { click "$TITLE_X" "$(task_y "$1")"; }

# Right-clicks row `$1` of the column at x `$2` (a list's or a task's) and picks entry `$3`.
menu_pick() {
  local x=$1 y=$2 entry=$3
  click "$x" "$y" right
  settle 1
  click $((x + 40)) $((y + 17 + entry * MENU_ENTRY_H))
  settle 1
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# The pane's other tab, the project's terminal, in front: the Tasks tab hidden in its pane.
show_terminal() {
  click "$TERMINAL_TAB_X" "$TAB_Y"
  settle 2
}

setup() {
  local bin=$E2E_WORK/bin folder
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault/archive" \
    "$E2E_WORK/rusty/vault/projects"
  folder=$(realpath "$E2E_WORK/repo")
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  cat >"$(tasks_json)" <<'JSON'
{"groups": [
  {"id": 1, "name": "Home", "tasks": [
    {"id": 11, "title": "Water the plants"},
    {"id": 12, "title": "Call the plumber", "completed": true},
    {"id": 13, "title": "Return the library books"},
    {"id": 14, "title": "Fix the gate", "archived": true},
    {"id": 15, "title": "Buy stamps"},
    {"id": 16, "title": "Clean the gutters"}]},
  {"id": 2, "name": "Work", "tasks": [
    {"id": 21, "title": "Write the release notes"},
    {"id": 22, "title": "Review the pull request"},
    {"id": 23, "title": "Book the train"}]},
  {"id": 3, "name": "Someday", "tasks": [
    {"id": 31, "title": "Learn the banjo"}]}
]}
JSON
  cat >"$E2E_WORK/rusty/vault/projects/repo.md" <<PAGE
---
title: Repo
type: project
path: $folder
task_group: Work
---
# Repo

A made-up project page for the scratch folder.
PAGE
  printf '# repo\n' >"$folder/README.md"
  open_path "$folder"
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== from the rail"
  click "$BRAIN_X" "$RAIL_HEADER_Y"
  settle 3
  click "$TASKS_ENTRY_X" "$FIXED_ROW_Y"
  settle 3
  shot 658-01-from-rail

  echo "== from the palette"
  show_terminal
  palette "rusty: open tasks"
  settle 2
  shot 658-02-from-palette

  echo "== a list chosen"
  click_list 1
  settle 2
  shot 658-03-list-chosen

  echo "== a task added"
  click "$ADD_X" "$ADD_Y"
  type_text "Pack the charger"
  press "" Return
  settle 2
  type_text "   "
  press "" Return
  settle 2
  shot 658-04-added
  expect "one task was added" test "$(asked create_task)" = 1
  expect "it went to Work" called_with create_task '{"group_id": 2, "title": "Pack the charger"}'

  echo "== a task done"
  click "$CHECK_X" "$(task_y 0)"
  settle 2
  shot 658-05-done
  expect "the release notes were toggled" called_with toggle_task '{"id": 21}'

  echo "== the selection moved"
  click_list 0
  settle 2
  click_task 0
  press "" Down
  settle 1
  shot 658-06-moved-selection

  echo "== Space"
  press "" space
  settle 2
  shot 658-07-space
  expect "the plumber was toggled" called_with toggle_task '{"id": 12}'

  echo "== renamed"
  press "" F2
  settle 1
  press CTRL a
  type_text "Call the electrician"
  press "" Return
  settle 2
  shot 658-08-renamed
  expect "the title was changed" called_with update_task_title '{"id": 12, "title": "Call the electrician"}'

  echo "== archived"
  press "" Delete
  settle 2
  shot 658-09-archived
  expect "it was archived" called_with archive_task '{"id": 12}'

  echo "== archived shown"
  click "$SHOW_ARCHIVED_X" "$HEADER_Y"
  settle 2
  shot 658-10-archived-shown

  echo "== restored"
  # Home with archived shown: plants 0, electrician 1, books 2, gate 3, stamps 4, gutters 5.
  menu_pick "$TITLE_X" "$(task_y 3)" 1
  settle 1
  shot 658-11-restored
  expect "the gate was restored" called_with unarchive_task '{"id": 14}'

  echo "== the delete prompt"
  click "$SHOW_ARCHIVED_X" "$HEADER_Y"
  settle 2
  # Home: plants 0, books 1, gate 2, stamps 3, gutters 4.
  menu_pick "$TITLE_X" "$(task_y 1)" 2
  settle 1
  shot 658-12-delete-prompt

  echo "== deleted"
  press "" Return
  settle 2
  shot 658-13-deleted
  expect "the books were deleted" called_with delete_task '{"id": 13}'

  echo "== dragging"
  # Home: plants 0, gate 1, stamps 2, gutters 3.
  pointer_to "$TITLE_X" "$(task_y 3)"
  pointer_down
  pointer_to "$TITLE_X" $(($(task_y 3) - 10))
  pointer_to "$TITLE_X" $(($(task_y 0) + 2))
  settle 1
  shot 658-14-dragging

  echo "== dropped"
  pointer_up
  settle 2
  shot 658-15-dropped
  expect "the new order went to Rusty" called_with reorder_tasks '{"group_id": 1, "task_ids": [16, 11, 14, 15]}'

  echo "== moved by keys"
  click_task 2
  press ALT Up
  settle 1
  press ALT Up
  settle 1
  press ALT Up
  settle 2
  shot 658-16-moved-by-keys
  expect "two more orders, the third press at the top calling nothing" test "$(asked reorder_tasks)" = 3

  # Pack the charger took id 32, so the new list is 33.
  echo "== a new list"
  click "$NEW_LIST_X" "$NEW_LIST_Y"
  settle 1
  type_text "Reading"
  press "" Return
  settle 3
  shot 658-17-new-list
  expect "the list was made" called_with create_task_group '{"name": "Reading"}'

  echo "== a list renamed"
  menu_pick "$LIST_X" "$(list_y 3)" 0
  press CTRL a
  type_text "Books"
  press "" Return
  settle 2
  shot 658-18-list-renamed
  expect "the list was renamed" called_with rename_task_group '{"group_id": 33, "name": "Books"}'

  echo "== the list delete prompt"
  menu_pick "$LIST_X" "$(list_y 3)" 1
  shot 658-19-list-delete-prompt

  echo "== the list deleted"
  press "" Return
  settle 3
  shot 658-20-list-deleted
  expect "the list was deleted" called_with delete_task_group '{"group_id": 33}'

  echo "== live"
  quiet_write 1 41 "Sweep the porch"
  announce
  settle 2
  shot 658-21-live

  echo "== read on show"
  show_terminal
  quiet_write 1 42 "Oil the hinges"
  local reads
  reads=$(asked list_tasks)
  announce
  settle 2
  expect "no read while hidden" test "$(asked list_tasks)" = "$reads"
  click "$TASKS_TAB_X" "$TAB_Y"
  settle 3
  shot 658-22-read-on-show

  echo "== a quiet write"
  click "$SEARCH_X" "$SEARCH_Y"
  settle 1
  quiet_write 1 43 "Wash the car"
  settle 3
  shot 658-23-quiet-write

  echo "== read on focus"
  click_task 0
  settle 3
  shot 658-24-read-on-focus

  echo "== Refresh"
  quiet_write 1 44 "Feed the cat"
  settle 1
  click "$REFRESH_X" "$HEADER_Y"
  settle 3
  shot 658-25-refresh

  echo "== refused"
  # Home: gate 0, gutters 1, plants 2, stamps 3, then the quiet ones.
  quiet_write 1 15
  click "$CHECK_X" "$(task_y 3)"
  settle 3
  shot 658-26-refused

  echo "== off"
  local before
  profile_setting marley.rusty.enabled false
  settle 5
  before=$(calls | wc -l)
  settle 3
  shot 658-27-off
  expect "no call once Rusty was off" test "$(calls | wc -l)" = "$before"

  echo "== from the project view"
  profile_setting marley.rusty.enabled true
  settle 8
  palette "rusty: toggle knowledge panel"
  settle 5
  click "$OPEN_IN_TASKS_X" "$OPEN_IN_TASKS_Y"
  settle 3
  shot 658-28-from-project
  calls | grep -E 'tools/call (create|rename|delete|toggle|update|archive|unarchive|reorder)' || true
}
