# shellcheck shell=bash
# #874's visual check: Rusty's Decisions tab and Follow Up as the boards draw them. `marley_rusty`'s
# stand-in `rusty-mcp` serves six made-up decision pages on its fixed day 2026-10-03: two due (one
# overdue), one decided with no follow-up, one decided with a later follow-up, one kept, one
# superseded; never the user's Rusty.
# - `rusty: open decisions`: the loop line, Due with its rows and Follow Up, the count, each
#   decision with its status chip in its colour and its dates (`874-01-decisions`, REQ-001).
# - Follow Up on the overdue decision: the form, its outcome as Kept / Revised / Superseded
#   (`874-02-follow-up`, REQ-002).
# - Marley Day (`874-03-day`, REQ-001).
compositor sway

AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-800}
# From the probe run's shot: the overdue decision's Follow Up.
FOLLOW_X=${FOLLOW_X:-1548}
FOLLOW_Y=${FOLLOW_Y:-189}
STOP_AT=${STOP_AT:-0}

vault() { echo "$E2E_WORK/rusty/vault"; }

# A made-up decision page: slug `$1`, title `$2`, decided `$3`, follow-up day `$4` (none when
# empty).
decision_page() {
  mkdir -p "$(vault)/decisions"
  {
    printf -- '---\ntitle: %s\nstatus: decided\ndecided: %s\n' "$2" "$3"
    if [[ -n $4 ]]; then
      printf 'follow_up_by: %s\n' "$4"
    fi
    printf -- '---\n# %s\n\nA made-up decision.\n' "$2"
  } >"$(vault)/decisions/$1.md"
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
  decision_page rail-order "Rail order: the window's, not attention" 2026-09-15 ""
  decision_page one-switch "Rusty in Marley is one switch" 2026-09-06 2026-11-01
  printf -- '---\ntitle: Home group named Default\nstatus: superseded\ndecided: 2026-09-01\nsuperseded_by: ship-the-home-page\n---\n# Home group named Default\n' >"$(vault)/decisions/home-group-default.md"
  printf -- '---\ntitle: Feature Map on localhost\nstatus: kept\ndecided: 2026-09-20\nfollowed_up: 2026-09-30\n---\n# Feature Map on localhost\n' >"$(vault)/decisions/feature-map-localhost.md"
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

  echo "== Decisions"
  palette "rusty: open decisions" 5
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 874-01-decisions
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== Follow Up"
  click "$FOLLOW_X" "$FOLLOW_Y"
  settle 3
  shot 874-02-follow-up
  press "" Escape
  settle 1

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 874-03-day
}
