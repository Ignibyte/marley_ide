# shellcheck shell=bash
# #939's visual check: Rusty's Tasks tab and a brain page as the boards draw them.
# `marley_rusty`'s stand-in `rusty-mcp` serves made-up lists and pages on its fixed day
# 2026-10-03, never the user's Rusty:
# - two decisions due: "Ship the home page", overdue since 2026-09-28, whose page carries
#   `status: decided`, `tags` and a task box; and one due that day;
# - `projects/atlas`, which links to the first;
# - one favourite, Atlas.
# - `rusty: open tasks`: the list's name as a headline, LISTS with square swatches, the tasks as
#   cards, Decisions due with "1 overdue" at its heading's right, Follow Up and Open the page, and
#   the footnote (`939-01-tasks`, REQ-001, REQ-003, REQ-004).
# - Follow Up on the overdue decision: #660's form (`939-02-form`, REQ-002).
# - Open the page: the property pills, Outline pressed, the outline with "Linked here" and Atlas
#   (`939-03-page`, REQ-006, REQ-007, REQ-009); the Brain tab's favourite with its ★
#   (`939-04-favourite`, REQ-008).
# - The body's box clicked: it shows ticked and the vault file holds `- [x]` (`939-05-ticked`,
#   REQ-005).
# - Tasks and the page in Marley Day (`939-06-day`, `939-07-day-page`).
compositor sway

STOP_AT=${STOP_AT:-0}
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-800}
# From the first run's shots: the overdue decision's Follow Up and Open the page, and the body's
# task box on the page.
FOLLOW_X=${FOLLOW_X:-1346}
FOLLOW_Y=${FOLLOW_Y:-183}
PAGE_X=${PAGE_X:-1446}
PAGE_Y=${PAGE_Y:-183}
BOX_X=${BOX_X:-311}
# The page's tab, beside Tasks.
PAGE_TAB_X=${PAGE_TAB_X:-700}
PAGE_TAB_Y=${PAGE_TAB_Y:-51}
BOX_Y=${BOX_Y:-635}

vault() { echo "$E2E_WORK/rusty/vault"; }

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$(vault)/decisions" "$(vault)/projects" \
    "$(vault)/archive" "$(vault)/.rusty"
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
  {"id": 2, "name": "Work", "tasks": [
    {"id": 21, "title": "Write the release notes"},
    {"id": 22, "title": "Review the pull request", "completed": true},
    {"id": 23, "title": "Book the train"}]},
  {"id": 1, "name": "Home", "tasks": [
    {"id": 11, "title": "Water the plants"}]},
  {"id": 3, "name": "Someday", "tasks": [
    {"id": 31, "title": "Learn the banjo"}]}
]}
JSON
  cat >"$E2E_WORK/rusty/memories.json" <<'JSON'
[
  {"id": "m1", "category": "preference", "importance": "high", "content": "Prefers plain words in docs and commit messages", "type": "memory", "source": "app", "created_at": 1790000000, "updated_at": 1790000000}
]
JSON
  cat >"$(vault)/decisions/ship-the-home-page.md" <<'PAGE'
---
title: Ship the home page
status: decided
decided: 2026-09-20
follow_up_by: 2026-09-28
tags: [home, rusty]
---
# Ship the home page

Home shows Rusty's cards beside the projects.

## Why

One place to start the day.

## Next

- [ ] Rusty cards on Home
- [x] The prompt chips
PAGE
  printf -- '---\ntitle: Ask one question at a time\nstatus: decided\ndecided: 2026-09-25\nfollow_up_by: 2026-10-03\n---\n# Ask one question at a time\n' \
    >"$(vault)/decisions/one-question-at-a-time.md"
  printf -- '---\ntitle: Atlas\ntype: project\n---\n# Atlas\n\nShipping: [[decisions/ship-the-home-page]].\n' \
    >"$(vault)/projects/atlas.md"
  printf '%s\n' '[{"kind": "file", "title": "Atlas", "path": "projects/atlas"}]' >"$(vault)/.rusty/bookmarks.json"
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

  echo "== Tasks"
  palette "rusty: open tasks" 5
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 939-01-tasks
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== Follow Up"
  click "$FOLLOW_X" "$FOLLOW_Y"
  settle 3
  shot 939-02-form
  press "" Escape
  settle 2

  echo "== the decision's page"
  click "$PAGE_X" "$PAGE_Y"
  settle 5
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 939-03-page
  if [[ $STOP_AT == 2 ]]; then
    return
  fi

  echo "== the favourite"
  palette "toggle brain view" 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 939-04-favourite
  # Back to the page's tab.
  click "$PAGE_TAB_X" "$PAGE_TAB_Y"
  settle 2

  echo "== the box ticked"
  click "$BOX_X" "$BOX_Y"
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 939-05-ticked
  expect "the vault file holds the box ticked" grep -qF -- "- [x] Rusty cards on Home" \
    "$(vault)/decisions/ship-the-home-page.md"

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 939-07-day-page
  palette "rusty: open tasks" 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 939-06-day
}
