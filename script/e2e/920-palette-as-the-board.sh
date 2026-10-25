# shellcheck shell=bash
# #920's visual check: the command palette as the Palette board draws it. A scratch repository in
# the Marley layout; the palette opens with Ctrl Shift P.
# - `920-01-empty`: an empty query: #839's groups, each row a tile and a plain name, the Commands
#   chip, the board's footer (REQ-001, REQ-003, REQ-004).
# - `920-02-typed`: "Open th" typed: plain names with the typed letters lit, Open Threads among
#   them (REQ-001, REQ-002).
# - `920-03-help`: "?" typed: Open Guide and Open Keymap (REQ-005).
# - `920-04-ran`: `marley: open threads` typed and run: the Threads page, so Zed's names still
#   match (REQ-002).
# - `920-05-day`: the empty palette in Marley Day (REQ-001, REQ-003, REQ-004).
# - `920-06-zed`: Zed's layout, the palette: Zed's rows, field and footer (REQ-006).
compositor sway

setup() {
  mkdir -p "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  git init -q -b main "$E2E_WORK/repo"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  profile_setting marley.tmux_hosts '[]'
  open_path "$E2E_WORK/repo"
}

open_palette() {
  press "CTRL SHIFT" p
  settle 3
}

steps() {
  settle 15
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the palette, empty"
  open_palette
  shot 920-01-empty

  echo "== Open th"
  type_text "Open th"
  settle 2
  shot 920-02-typed
  press "" Escape
  settle 1

  echo "== ?"
  open_palette
  type_text "?"
  settle 2
  shot 920-03-help
  press "" Escape
  settle 1

  echo "== Zed's name runs"
  open_palette
  type_text "marley: open threads"
  settle 2
  press "" Return
  settle 3
  shot 920-04-ran

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 3
  open_palette
  shot 920-05-day
  press "" Escape
  profile_setting theme '"Marley Night"'
  settle 2

  echo "== Zed's layout"
  open_palette
  type_text "marley: use zed layout"
  settle 2
  press "" Return
  settle 5
  open_palette
  shot 920-06-zed
  press "" Escape
}
