# shellcheck shell=bash
# #664's visual check: the Memory tab over Rusty's long-term memories. `marley_rusty`'s stand-in
# `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, over a scratch state folder (`RUSTY_STAND_IN_STATE`)
# whose `memories.json` holds five made-up memories in three categories and all three importances,
# never the user's Rusty (R-D8). The stand-in announces each memory write and each change to that
# file, as Rusty does; its `fail` file makes one tool refuse.
#
# The tab (`664-01-memory`), a memory added (`664-02-added`), the filter (`664-03-filter`), the
# form (`664-04-form`), a save (`664-05-saved`) and a refused one (`664-06-refused`), a delete asked
# and done (`664-07-delete-prompt`, `664-08-deleted`), a change from outside (`664-09-outside`) and
# Rusty off (`664-10-off`).
compositor sway

# Where things sit, from the first run's shots: the add row's fields and its High button, the
# Category menu, the rows, and the form's Low, Save and Delete.
ADD_X=${ADD_X:-600}
ADD_Y=${ADD_Y:-156}
CATEGORY_X=${CATEGORY_X:-1000}
HIGH_X=${HIGH_X:-1300}
FILTER_X=${FILTER_X:-1290}
FILTER_Y=${FILTER_Y:-192}
ROW_X=${ROW_X:-600}
FIRST_ROW_Y=${FIRST_ROW_Y:-240}
ROW_H=${ROW_H:-62}
FORM_LOW_X=${FORM_LOW_X:-872}
FORM_LOW_Y=${FORM_LOW_Y:-278}
FORM_SAVE_X=${FORM_SAVE_X:-1040}
FORM_DELETE_X=${FORM_DELETE_X:-565}
FORM_BUTTONS_Y=${FORM_BUTTONS_Y:-313}

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# Whether the stand-in's file holds a memory whose content is `$1` with the field `$2` set to `$3`.
memory_has() {
  python3 - "$E2E_WORK/rusty/memories.json" "$@" <<'MEMORIES'
import json, pathlib, sys

memories = json.loads(pathlib.Path(sys.argv[1]).read_text())
found = [memory for memory in memories if memory.get("content") == sys.argv[2]]
print("found:", found)
sys.exit(0 if found and found[0].get(sys.argv[3]) == sys.argv[4] else 1)
MEMORIES
}

# Whether no memory's content is `$1`.
memory_gone() {
  ! grep -qF "\"content\": \"$1\"" "$E2E_WORK/rusty/memories.json"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

row_y() { echo $((FIRST_ROW_Y + $1 * ROW_H)); }

click_row() { click "$ROW_X" "$(row_y "$1")"; }

# Chooses entry `$1` of the Category menu, counted from 0 (All). The menu opens on the choice
# shown, so Home goes to All first.
choose_category() {
  click "$FILTER_X" "$FILTER_Y"
  settle 1
  press "" Home
  local step
  for ((step = 0; step < $1; step++)); do
    press "" Down
  done
  press "" Return
  settle 1
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  # Five memories, their times a day apart in the autumn of 2026.
  cat >"$E2E_WORK/rusty/memories.json" <<'JSON'
[
  {"id": "m1", "category": "preference", "importance": "high", "content": "Prefers plain words in docs and commit messages", "type": "memory", "source": "app", "created_at": 1790000000, "updated_at": 1790000000},
  {"id": "m2", "category": "fact", "importance": "normal", "content": "The dev box builds with one cargo command at a time", "type": "memory", "source": "mcp", "created_at": 1790086400, "updated_at": 1790086400},
  {"id": "m3", "category": "context", "importance": "normal", "content": "Working on the Marley shell this month", "type": "memory", "source": "mcp", "created_at": 1790172800, "updated_at": 1790172800},
  {"id": "m4", "category": "fact", "importance": "low", "content": "The old app lived in a Qt window", "type": "memory", "source": "app", "created_at": 1790259200, "updated_at": 1790259200},
  {"id": "m5", "category": "context", "importance": "high", "content": "Release notes go out on Fridays", "type": "memory", "source": "mcp", "created_at": 1790345600, "updated_at": 1790345600}
]
JSON
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== the tab"
  palette "rusty: open memory"
  settle 2
  shot 664-01-memory
  expect "the memories were read" bash -c "grep -q ' tools/call list_memories ' '$E2E_WORK/rusty/calls'"

  # Rusty's order: Release notes (high, newest), Prefers plain words (high), Working on the shell
  # (normal), The dev box (normal), The old app (low).
  echo "== a memory added"
  click "$ADD_X" "$ADD_Y"
  settle 1
  type_text "Keeps commit subjects under sixty characters"
  click "$CATEGORY_X" "$ADD_Y"
  settle 1
  type_text "preference"
  click "$HIGH_X" "$ADD_Y"
  settle 1
  click "$CATEGORY_X" "$ADD_Y"
  press "" Return
  settle 3
  shot 664-02-added
  expect "it was stored as typed" memory_has "Keeps commit subjects under sixty characters" importance high

  # Categories: All 0, context 1, fact 2, preference 3.
  echo "== the filter"
  choose_category 3
  settle 1
  shot 664-03-filter
  choose_category 0

  # The added one heads the list now: rows 0 to 5, Working on the shell at 3.
  echo "== the form"
  click_row 3
  settle 2
  shot 664-04-form

  echo "== a save"
  press CTRL a
  type_text "Working on the Marley shell and Rusty this month"
  click "$FORM_LOW_X" "$FORM_LOW_Y"
  settle 1
  click "$FORM_SAVE_X" "$FORM_BUTTONS_Y"
  settle 3
  shot 664-05-saved
  expect "the save reached Rusty" memory_has "Working on the Marley shell and Rusty this month" importance low

  echo "== a refused save"
  printf 'update_memory\ndatabase is locked\n' >"$E2E_WORK/rusty/fail"
  click_row 0
  settle 2
  press "" Return
  settle 2
  shot 664-06-refused
  rm -f "$E2E_WORK/rusty/fail"
  press "" Escape
  settle 1

  # Rusty's order now: Keeps 0, Release notes 1, Prefers 2, The dev box 3, The old app 4 and the
  # saved one 5, both low, the old app created later.
  echo "== a delete"
  click_row 4
  settle 2
  click "$FORM_DELETE_X" "$FORM_BUTTONS_Y"
  settle 2
  shot 664-07-delete-prompt
  press "" Return
  settle 3
  shot 664-08-deleted
  expect "the memory was deleted" memory_gone "The old app lived in a Qt window"

  echo "== a change from outside"
  python3 - "$E2E_WORK/rusty/memories.json" <<'PY'
import json, pathlib, sys

path = pathlib.Path(sys.argv[1])
memories = json.loads(path.read_text())
memories.append({"id": "m9", "category": "fact", "importance": "high", "content": "Added from Rusty's CLI", "type": "memory", "source": "cli", "created_at": 1790500000, "updated_at": 1790500000})
path.write_text(json.dumps(memories, indent=2))
PY
  settle 3
  shot 664-09-outside

  echo "== Rusty off"
  profile_setting marley.rusty.enabled false
  settle 4
  shot 664-10-off
  calls | grep -E 'list_memories|store_memory|update_memory|delete_memory'
}
