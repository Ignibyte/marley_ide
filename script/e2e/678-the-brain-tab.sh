# shellcheck shell=bash
# #678's visual check: the vault's navigation lives in a Brain tab, not in the rail. `marley_rusty`'s
# stand-in `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, over a scratch state folder
# (`RUSTY_STAND_IN_STATE`) whose vault holds one page under `notes`, never the user's Rusty (R-D8).
#
# `678-01-header`: the rail with no Projects/Brain switch, Brain the first screen button (REQ-001).
# `678-02-tab`: Brain clicked: the Brain tab in the Rusty group, the tree on its left, "Pick a page
# on the left." on its right (REQ-002). `678-03-page`: the folder opened and the page clicked: the
# page on the tab's right (REQ-003). `678-04-new-tab`: the page's menu, Open in New Tab: a Page tab
# of its own beside the Brain tab (REQ-004).
compositor sway

# In the window's logical pixels, from the first run's shots: the header's Brain button, the
# tree's folder and page rows in the tab, the menu's Open in New Tab, and a point clear of both.
HEADER_Y=${HEADER_Y:-21}
BRAIN_X=${BRAIN_X:-19}
FOLDER_X=${FOLDER_X:-400}
FOLDER_Y=${FOLDER_Y:-152}
PAGE_X=${PAGE_X:-420}
PAGE_Y=${PAGE_Y:-186}
NEW_TAB_X=${NEW_TAB_X:-480}
NEW_TAB_Y=${NEW_TAB_Y:-236}
AWAY_X=${AWAY_X:-1000}
AWAY_Y=${AWAY_Y:-700}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty/vault/notes"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '# A note\n\nA page in the scratch vault.\n' >"$E2E_WORK/rusty/vault/notes/a-note.md"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
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
  shot 678-01-header

  echo "== Brain"
  click "$BRAIN_X" "$HEADER_Y"
  settle 4
  away
  shot 678-02-tab

  echo "== the folder, then the page"
  click "$FOLDER_X" "$FOLDER_Y"
  settle 2
  click "$PAGE_X" "$PAGE_Y"
  settle 3
  away
  shot 678-03-page

  echo "== Open in New Tab"
  click "$PAGE_X" "$PAGE_Y" right
  settle 2
  click "$NEW_TAB_X" "$NEW_TAB_Y"
  settle 3
  away
  shot 678-04-new-tab
}
