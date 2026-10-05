# shellcheck shell=bash
# #656's visual check: a brain page's outline, and its title, name and properties edited in place.
# `marley_rusty`'s stand-in `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, over a scratch state folder
# (`RUSTY_STAND_IN_STATE`) whose `vault/` holds three made-up pages, never the user's (R-D8). The
# run's keymap binds Ctrl+Alt+Shift+Y to `rusty::OpenPage` for `notes/draft-idea` (Zed binds
# Ctrl+Alt+Shift+O itself). Each write's check reads the stand-in's call log for the tool, the key
# and the value with its JSON type.
#
# The outline (`656-01-outline`), a heading clicked (`656-02-outline-scroll`), the second of two
# Notes (`656-03-same-name`), the outline hidden (`656-04-outline-hidden`), Zed's outline panel in
# Edit (`656-05-edit-outline-panel`), the title's editor (`656-06-title-open`), the title set
# (`656-07-title-set`) and dropped (`656-08-title-escape`), a text (`656-09-text`), a number
# (`656-10-number`) and one refused (`656-11-number-refused`), a date kept on blur
# (`656-12-date-on-blur`), a checkbox (`656-13-checkbox`), a list item added (`656-14-list-add`)
# and removed (`656-15-list-remove`), a property removed (`656-16-property-removed`) and added
# (`656-17-property-added`), a key taken (`656-18-key-taken`), a page with no headings
# (`656-19-no-headings`), the page renamed (`656-20-renamed`), the history after it
# (`656-21-history-follows`), a rename refused (`656-22-rename-refused`), a rename in the Brain view
# followed (`656-23-tree-rename-follows`), and Rusty off (`656-24-not-connected`).
compositor sway

# Where things sit, from the first runs' shots: the header's Outline button and name, the title,
# each property's value and buttons, the outline's entries, the body, and the rail's tree.
OUTLINE_BUTTON_X=${OUTLINE_BUTTON_X:-331}
HEADER_Y=${HEADER_Y:-82}
NAME_X=${NAME_X:-422}
EDIT_X=${EDIT_X:-1327}
TITLE_X=${TITLE_X:-336}
TITLE_Y=${TITLE_Y:-136}
VALUE_X=${VALUE_X:-475}
# The value rows' first line and the step between rows.
FIRST_VALUE_Y=${FIRST_VALUE_Y:-180}
VALUE_H=${VALUE_H:-35}
OUTLINE_X=${OUTLINE_X:-1170}
FIRST_HEADING_Y=${FIRST_HEADING_Y:-116}
HEADING_H=${HEADING_H:-19}
BODY_X=${BODY_X:-900}
BODY_Y=${BODY_Y:-620}
SAM_LINK_X=${SAM_LINK_X:-338}
SAM_LINK_Y=${SAM_LINK_Y:-801}
REMOVE_PROPERTY_X=${REMOVE_PROPERTY_X:-1093}
CHECKBOX_X=${CHECKBOX_X:-477}
IDEA_REMOVE_X=${IDEA_REMOVE_X:-513}
TAGS_ADD_X=${TAGS_ADD_X:-609}
ADD_PROPERTY_X=${ADD_PROPERTY_X:-342}
ADD_PROPERTY_Y=${ADD_PROPERTY_Y:-424}
BRAIN_X=${BRAIN_X:-42}
RAIL_HEADER_Y=${RAIL_HEADER_Y:-16}
ROW_X=${ROW_X:-110}
FIRST_ROW_Y=${FIRST_ROW_Y:-117}
ROW_H=${ROW_H:-23}

# The rows of Draft idea's properties, in file order: title 0, type 1.
STATUS_ROW=2
STARS_ROW=3
REVIEWED_ROW=4
DUE_ROW=5
TAGS_ROW=6

value_y() { echo $((FIRST_VALUE_Y + $1 * VALUE_H)); }

click_value() { click "$VALUE_X" "$(value_y "$1")"; }

heading_y() { echo $((FIRST_HEADING_Y + $1 * HEADING_H)); }

click_heading() { click "$OUTLINE_X" "$(heading_y "$1")"; }

row_y() { echo $((FIRST_ROW_Y + $1 * ROW_H)); }

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# How many times the stand-in was asked `$1`.
asked() { calls | grep -c " tools/call $1 " || true; }

# Whether the stand-in was asked `$1` with exactly the JSON arguments `$2`, types included.
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

# Whether Draft idea's property `$1` was set to the JSON value `$2`.
set_to() {
  called_with brain_set_property "{\"slug\": \"notes/draft-idea\", \"key\": \"$1\", \"value\": $2}"
}

vault() { echo "$E2E_WORK/rusty/vault"; }

# A made-up page `$1` in the stand-in's scratch vault, its text from stdin.
vault_page() {
  mkdir -p "$(dirname "$(vault)/$1.md")"
  cat >"$(vault)/$1.md"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Selects all of an open editor's text and types `$1` over it.
retype() {
  press CTRL a
  type_text "$1"
}

draft_page() {
  local line
  {
    cat <<'PAGE'
---
title: Draft idea
type: note
status: open
stars: 3
reviewed: false
due: 2026-10-10
tags:
  - idea
  - marley
---
A draft with a heading for each part.

## Context

Where the idea came from.

### Background

What came before it.

```text
# not a heading
```

## Links to [[people/sam|Sam]]

Ask [[people/sam|Sam]] first.

## Notes

First notes.

## Plan

### Steps

PAGE
    for line in $(seq 1 40); do printf 'Step filler line %s.\n\n' "$line"; done
    printf '## Notes\n\nSecond notes.\n\n'
    for line in $(seq 1 30); do printf 'Notes filler line %s.\n\n' "$line"; done
    printf '## Why\n\nThe reason is here.\n\n## Timeline\n\nSoon.\n\n'
    for line in $(seq 1 40); do printf 'Timeline filler line %s.\n\n' "$line"; done
  } | vault_page notes/draft-idea
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$(vault)/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '%s\n' '[{"context": "Workspace", "bindings": {"ctrl-alt-shift-y": ["rusty::OpenPage", {"slug": "notes/draft-idea"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  draft_page
  vault_page notes/linker <<'PAGE'
---
title: Linker
---
This page links to [[notes/draft-idea|the draft]].
PAGE
  vault_page people/sam <<'PAGE'
---
title: Sam
type: person
---
Sam keeps the renderer honest.
PAGE
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the outline"
  press "CTRL ALT SHIFT" y
  settle 5
  shot 656-01-outline

  echo "== a heading clicked"
  click_heading 7
  settle 2
  shot 656-02-outline-scroll

  echo "== the second Notes"
  click_heading 6
  settle 2
  shot 656-03-same-name

  echo "== the outline hidden"
  click "$OUTLINE_BUTTON_X" "$HEADER_Y"
  settle 2
  shot 656-04-outline-hidden

  echo "== Zed's outline panel in Edit"
  click "$OUTLINE_BUTTON_X" "$HEADER_Y"
  settle 1
  click "$EDIT_X" "$HEADER_Y"
  settle 3
  palette "outline panel: toggle focus"
  settle 3
  shot 656-05-edit-outline-panel
  # The project panel back in the right dock, at its own width, and the focus back in the tab.
  palette "outline panel: toggle focus"
  settle 1
  palette "project panel: toggle focus"
  settle 1
  palette "project panel: toggle focus"
  settle 1
  click "$EDIT_X" "$HEADER_Y"
  settle 4
  pointer_to "$BODY_X" "$BODY_Y"
  scroll -100
  settle 1

  echo "== the title's editor"
  click "$TITLE_X" "$TITLE_Y"
  settle 2
  shot 656-06-title-open

  echo "== the title set"
  type_text "Draft plan"
  press "" Return
  settle 3
  shot 656-07-title-set
  expect "the title was written" set_to title '"Draft plan"'

  echo "== the title dropped"
  click "$TITLE_X" "$TITLE_Y"
  settle 1
  type_text "zzz"
  press "" Escape
  settle 2
  shot 656-08-title-escape
  expect "one write so far" test "$(asked brain_set_property)" = 1

  echo "== a text value"
  click_value "$STATUS_ROW"
  settle 1
  retype "doing"
  press "" Return
  settle 3
  shot 656-09-text
  expect "the status was written as text" set_to status '"doing"'

  echo "== a number"
  click_value "$STARS_ROW"
  settle 1
  retype "4"
  press "" Return
  settle 3
  shot 656-10-number
  expect "the stars were written as a number" set_to stars 4

  echo "== a number refused"
  click_value "$STARS_ROW"
  settle 1
  retype "four"
  press "" Return
  settle 2
  shot 656-11-number-refused
  expect "nothing written for four" test "$(asked brain_set_property)" = 3
  press "" Escape
  settle 1

  echo "== a date kept on blur"
  click_value "$DUE_ROW"
  settle 1
  retype "2026-11-01"
  click "$BODY_X" "$BODY_Y"
  settle 3
  shot 656-12-date-on-blur
  expect "the date was written as text" set_to due '"2026-11-01"'

  echo "== a checkbox"
  click "$CHECKBOX_X" "$(value_y "$REVIEWED_ROW")"
  settle 3
  shot 656-13-checkbox
  expect "reviewed was written as true" set_to reviewed true

  echo "== a list item added"
  click "$TAGS_ADD_X" "$(value_y "$TAGS_ROW")"
  settle 1
  type_text "rusty"
  press "" Return
  settle 3
  shot 656-14-list-add
  expect "the tags were written with rusty last" set_to tags '["idea", "marley", "rusty"]'
  press "" Escape
  settle 1

  echo "== a list item removed"
  click "$IDEA_REMOVE_X" "$(value_y "$TAGS_ROW")"
  settle 3
  shot 656-15-list-remove
  expect "the tags were written without idea" set_to tags '["marley", "rusty"]'

  echo "== a property removed"
  click "$REMOVE_PROPERTY_X" "$(value_y "$STATUS_ROW")"
  settle 3
  shot 656-16-property-removed
  expect "status was removed" called_with brain_remove_property '{"slug": "notes/draft-idea", "key": "status"}'

  # Without status, Add property moves up a row. The menu opens on Text, then List, Number,
  # Checkbox and Date.
  echo "== a property added"
  click "$ADD_PROPERTY_X" "$((ADD_PROPERTY_Y - VALUE_H))"
  settle 1
  press "" Down
  press "" Down
  press "" Return
  settle 1
  type_text "priority"
  press "" Return
  settle 3
  shot 656-17-property-added
  expect "priority was written as the number 0" set_to priority 0

  echo "== a key taken"
  click "$ADD_PROPERTY_X" "$ADD_PROPERTY_Y"
  settle 1
  press "" Return
  settle 1
  type_text "stars"
  press "" Return
  settle 2
  shot 656-18-key-taken
  expect "nothing written for a taken key" test "$(asked brain_set_property)" = 8
  press "" Escape
  settle 1

  echo "== a page with no headings"
  pointer_to "$BODY_X" "$BODY_Y"
  scroll -100
  settle 1
  click "$SAM_LINK_X" "$SAM_LINK_Y"
  settle 3
  shot 656-19-no-headings
  press ALT Left
  settle 3

  echo "== the page renamed"
  click "$NAME_X" "$HEADER_Y"
  settle 1
  retype "first-idea"
  press "" Return
  settle 4
  shot 656-20-renamed
  expect "the page was renamed in its folder" called_with brain_rename '{"from": "notes/draft-idea", "to": "notes/first-idea"}'

  echo "== the history follows"
  press ALT Right
  settle 3
  press ALT Left
  settle 3
  shot 656-21-history-follows

  echo "== a rename refused"
  click "$NAME_X" "$HEADER_Y"
  settle 1
  retype "linker"
  press "" Return
  settle 3
  shot 656-22-rename-refused

  # The tree: notes 0, people 1; people opened: notes 0, people 1, sam 2.
  echo "== a rename in the Brain view"
  click "$BRAIN_X" "$RAIL_HEADER_Y"
  settle 3
  click "$ROW_X" "$(row_y 1)"
  settle 1
  click "$ROW_X" "$(row_y 2)" right
  settle 1
  press "" Down
  press "" Return
  settle 1
  type_text "samuel"
  press "" Return
  settle 4
  click "$BODY_X" "$BODY_Y"
  settle 1
  press ALT Right
  settle 3
  shot 656-23-tree-rename-follows
  expect "Sam was renamed from the tree" called_with brain_rename '{"from": "people/sam", "to": "people/samuel"}'

  echo "== Rusty off"
  press ALT Left
  settle 3
  profile_setting marley.rusty.enabled false
  settle 6
  click "$TITLE_X" "$TITLE_Y"
  settle 2
  shot 656-24-not-connected
  calls | grep -E 'brain_set_property|brain_remove_property|brain_rename'
}
