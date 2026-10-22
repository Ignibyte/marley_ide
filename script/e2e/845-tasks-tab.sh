# shellcheck shell=bash
# #845's visual check: Rusty's Tasks tab as the Tasks board draws it. `marley_rusty`'s stand-in
# `rusty-mcp` serves #658's three made-up lists (Home with a done and an archived task, Work,
# Someday), two made-up decision pages due on its fixed day 2026-10-03 (one overdue), and three
# made-up memories; never the user's Rusty.
# - `rusty: open tasks`, Home chosen: each list's dot and open count, "4 open · 1 done", the open
#   tasks, Done with Call the plumber, and the Rusty column with the decisions due and the
#   memories (`845-01-tasks`, REQ-001, REQ-002).
# - Work chosen (`845-02-work`, REQ-001).
# - Marley Day (`845-03-day`, REQ-001, REQ-002).
compositor sway

AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-700}
# From #658's shots: the lists' column and its first row.
LIST_X=${LIST_X:-300}
FIRST_LIST_Y=${FIRST_LIST_Y:-113}
LIST_H=${LIST_H:-31}

vault() { echo "$E2E_WORK/rusty/vault"; }

# A made-up decision page: slug `$1`, title `$2`, decided `$3`, follow-up day `$4`.
decision_page() {
  mkdir -p "$(vault)/decisions"
  printf -- '---\ntitle: %s\nstatus: decided\ndecided: %s\nfollow_up_by: %s\n---\n# %s\n\nA made-up decision.\n' \
    "$2" "$3" "$4" "$2" >"$(vault)/decisions/$1.md"
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$(vault)/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  profile_setting marley.tmux_hosts '[]'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '2026-10-03\n' >"$E2E_WORK/rusty/today"
  cat >"$E2E_WORK/rusty/tasks.json" <<'JSON'
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
  decision_page ship-the-home-page "Ship the home page" 2026-09-20 2026-09-28
  decision_page one-question-at-a-time "Ask one question at a time" 2026-09-25 2026-10-03
  cat >"$E2E_WORK/rusty/memories.json" <<'JSON'
[
  {"id": "m1", "category": "preference", "importance": "high", "content": "Prefers plain words in docs and commit messages", "type": "memory", "source": "app", "created_at": 1790000000, "updated_at": 1790000000},
  {"id": "m2", "category": "fact", "importance": "normal", "content": "The dev box builds with one cargo command at a time", "type": "memory", "source": "mcp", "created_at": 1790086400, "updated_at": 1790086400},
  {"id": "m3", "category": "context", "importance": "normal", "content": "Release notes go out on Fridays", "type": "memory", "source": "mcp", "created_at": 1790172800, "updated_at": 1790172800}
]
JSON
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the Tasks tab"
  palette "rusty: open tasks" 5
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 845-01-tasks

  echo "== Work"
  click "$LIST_X" $((FIRST_LIST_Y + LIST_H))
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 845-02-work

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 845-03-day
}
