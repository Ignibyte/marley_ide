# shellcheck shell=bash
# #645's visual check: a brain page in a center tab. `marley_rusty`'s stand-in `rusty-mcp`, named
# by `MARLEY_RUSTY_MCP`, over a scratch state folder (`RUSTY_STAND_IN_STATE`) whose `vault/` holds
# three made-up pages, never the user's (R-D8); it renders them as Rusty's main does (`file`
# included) and announces each change to its vault. Rusty is on with the embedded connection from
# the start, and preview tabs are on, whatever the user's own settings say.
#
# One click on a page: a preview tab with the title, properties, body and links in two colours
# (`645-01-preview`); the body's code and table (`645-02-body`); another page in the same preview
# tab (`645-03-replaced`); a double-click keeps it (`645-04-kept`); a wikilink (`645-05-wikilink`),
# Back and Forward (`645-06-back`, `645-07-forward`), a heading link (`645-08-heading`), an
# unresolved link (`645-09-unresolved`); the tab already open comes forward (`645-10-found`); Edit
# (`645-11-edit`), an edit keeps the preview tab (`645-12-edit-keeps`), Read saves
# (`645-13-read-saves`); a change from outside (`645-14-live`); no such page (`645-15-missing`);
# Rusty turned off (`645-16-not-connected`).
compositor sway

# Where things sit, from the first runs' shots: the rail's Brain button, the tree's rows, the
# page's links and the tab bar's tabs, the Edit button.
BRAIN_X=${BRAIN_X:-42}
HEADER_Y=${HEADER_Y:-16}
ROW_X=${ROW_X:-110}
FIRST_ROW_Y=${FIRST_ROW_Y:-117}
ROW_H=${ROW_H:-23}
BODY_X=${BODY_X:-800}
BODY_Y=${BODY_Y:-500}
ALIAS_LINK_X=${ALIAS_LINK_X:-420}
ALIAS_LINK_Y=${ALIAS_LINK_Y:-415}
HEADING_LINK_X=${HEADING_LINK_X:-478}
HEADING_LINK_Y=${HEADING_LINK_Y:-443}
MISSING_LINK_X=${MISSING_LINK_X:-397}
MISSING_LINK_Y=${MISSING_LINK_Y:-471}
EDIT_X=${EDIT_X:-1327}
EDIT_Y=${EDIT_Y:-82}
FIRST_TAB_X=${FIRST_TAB_X:-512}
SECOND_TAB_X=${SECOND_TAB_X:-605}
TAB_Y=${TAB_Y:-51}

TYPED="A line typed in Edit."
OUTSIDE="A line written from outside."

row_y() { echo $((FIRST_ROW_Y + $1 * ROW_H)); }

click_row() { click "$ROW_X" "$(row_y "$1")"; }

# The stand-in's scratch vault.
vault() { echo "$E2E_WORK/rusty/vault"; }

# A made-up page `$1` in the scratch vault, its text from stdin.
vault_page() {
  mkdir -p "$(dirname "$(vault)/$1.md")"
  cat >"$(vault)/$1.md"
}

# Whether Sam's file holds the typed line and, in order, every line the scenario wrote.
sam_saved_as_typed() {
  python3 - "$(vault)/people/sam.md" "$TYPED" <<'PY'
import sys

text = open(sys.argv[1], encoding="utf-8").read()
written = ["---", "title: Sam", "type: person", "---", "# Sam", "", "Sam keeps the renderer honest."]
lines = text.splitlines()
at = 0
for line in lines:
    if at < len(written) and line == written[at]:
        at += 1
sys.exit(0 if at == len(written) and sys.argv[2] in text else 1)
PY
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$(vault)/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  profile_setting preview_tabs.enabled true
  profile_setting preview_tabs.enable_preview_from_project_panel true
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '%s\n' '[{"context": "Workspace", "bindings": {"ctrl-alt-shift-y": ["rusty::OpenPage", {"slug": "ideas/nowhere"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  vault_page projects/atlas <<'PAGE'
---
title: Atlas
type: project
status: active
tags: [rust, gpui, brain]
---
# Atlas

A project page with every kind of block.

- See [[decisions/use-zeds-renderer|the renderer decision]]
- Why: [[decisions/use-zeds-renderer#Why]]
- Later: [[ideas/later]]
- The site: [Zed](https://zed.dev)

## Tasks

- [x] Read the page
- [ ] Edit the page

## Code

```rust
fn main() {
    println!("atlas");
}
```

## Table

| Part | State |
|------|-------|
| Tree | done |
| Page | doing |
PAGE
  {
    printf -- '---\ntitle: Use Zed renderer\ntype: decision\n---\n# Use Zed renderer\n\n'
    for line in $(seq 1 40); do
      printf 'Context line %s: the page is drawn by the markdown crate Zed already ships.\n\n' "$line"
    done
    printf '## Why\n\nOne renderer for every Markdown view in Marley.\n'
  } | vault_page decisions/use-zeds-renderer
  vault_page people/sam <<'PAGE'
---
title: Sam
type: person
---
# Sam

Sam keeps the renderer honest.
PAGE
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 6

  # The tree: decisions 0, people 1, projects 2; projects opened, then people: decisions 0,
  # people 1, sam 2, projects 3, atlas 4.
  click "$BRAIN_X" "$HEADER_Y"
  settle 3
  click_row 2
  settle 1
  click_row 1
  settle 1

  echo "== one click: a preview tab"
  click_row 4
  settle 4
  shot 645-01-preview

  echo "== the body's code and table"
  pointer_to "$BODY_X" "$BODY_Y"
  scroll 12
  settle 2
  shot 645-02-body
  scroll -12
  settle 1

  echo "== another page in the preview tab"
  click_row 2
  settle 3
  shot 645-03-replaced

  echo "== a double-click keeps it"
  click_row 4
  click_row 4
  settle 3
  shot 645-04-kept

  echo "== a wikilink"
  click "$ALIAS_LINK_X" "$ALIAS_LINK_Y"
  settle 3
  shot 645-05-wikilink

  echo "== Back and Forward"
  press "ALT" Left
  settle 3
  shot 645-06-back
  press "ALT" Right
  settle 3
  shot 645-07-forward

  echo "== a heading link"
  press "ALT" Left
  settle 3
  click "$HEADING_LINK_X" "$HEADING_LINK_Y"
  settle 4
  shot 645-08-heading

  echo "== an unresolved link"
  press "ALT" Left
  settle 3
  click "$MISSING_LINK_X" "$MISSING_LINK_Y"
  settle 2
  shot 645-09-unresolved

  echo "== the open tab comes forward"
  click_row 2
  settle 3
  click_row 4
  settle 3
  shot 645-10-found

  echo "== Edit"
  click "$SECOND_TAB_X" "$TAB_Y"
  settle 2
  click "$EDIT_X" "$EDIT_Y"
  settle 4
  shot 645-11-edit

  echo "== an edit keeps the tab"
  press "CTRL" End
  press "" Return
  type_text "$TYPED"
  settle 2
  shot 645-12-edit-keeps

  echo "== Read saves"
  click "$EDIT_X" "$EDIT_Y"
  settle 4
  shot 645-13-read-saves
  expect "the file holds the typed line and its lines as written" sam_saved_as_typed

  echo "== a change from outside"
  printf '\n%s\n' "$OUTSIDE" >>"$(vault)/people/sam.md"
  settle 5
  shot 645-14-live

  echo "== no such page"
  # Zed binds Ctrl-Alt-Shift-O itself (`projects::OpenRemote`), so the run binds Y.
  press "CTRL ALT SHIFT" y
  settle 4
  shot 645-15-missing

  echo "== Rusty off"
  click "$FIRST_TAB_X" "$TAB_Y"
  settle 2
  profile_setting marley.rusty.enabled false
  settle 6
  shot 645-16-not-connected
  # The pages the tab read, for the notes.
  grep -o 'tools/call brain_render {[^}]*}' "$E2E_WORK/rusty/calls" | sort | uniq -c
}
