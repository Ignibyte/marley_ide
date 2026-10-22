# shellcheck shell=bash
# #875's visual check: Rusty's Brain tab, a page and the Knowledge panel as the boards draw them. `marley_rusty`'s stand-in `rusty-mcp` over #645's three made-up pages (Atlas, which
# links to the renderer decision; Use Zed renderer; Sam), never the user's Rusty.
# - `marley: toggle brain view`: the Brain tab, the tree under Vault (`875-01-brain`, REQ-001).
# - Ctrl+Alt+Shift+Y opens Atlas: its outline under "On this page" (`875-02-page`, REQ-001).
# - `rusty: toggle knowledge panel`: Following, Backlinks and Links out as cards, ideas/later "not
#   written yet" (`875-03-knowledge`, REQ-002).
# - Marley Day, the page (`875-04-day`, REQ-001).
compositor sway

STOP_AT=${STOP_AT:-0}

# The stand-in's scratch vault.
vault() { echo "$E2E_WORK/rusty/vault"; }

# A made-up page `$1` in the scratch vault, its text from stdin.
vault_page() {
  mkdir -p "$(dirname "$(vault)/$1.md")"
  cat >"$(vault)/$1.md"
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$(vault)/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  profile_setting preview_tabs.enabled true
  profile_setting marley.tmux_hosts '[]'
  profile_setting preview_tabs.enable_preview_from_project_panel true
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf '%s\n' '[{"context": "Workspace", "bindings": {"ctrl-alt-shift-y": ["rusty::OpenPage", {"slug": "projects/atlas"}]}}]' \
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

  echo "== the Brain tab"
  palette "toggle brain view" 4
  pointer_to 1500 900
  settle 1
  shot 875-01-brain

  echo "== Atlas"
  press "CTRL ALT SHIFT" y
  settle 4
  shot 875-02-page

  echo "== Knowledge"
  palette "rusty: toggle knowledge panel" 4
  shot 875-03-knowledge

  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== Marley Day"
  press "CTRL ALT SHIFT" y
  settle 3
  profile_setting theme '"Marley Day"'
  settle 4
  shot 875-04-day
}
