# shellcheck shell=bash
# #921's visual check: the file finder as the Palette board's "Go to a file" panel. A scratch
# repository with `src/rail.rs`, `src/rail_order.rs`, `docs/rail.md` and `script/rail.sh`
# committed; then `src/rail.rs` changed and `src/rail_new.rs` left untracked. No agent runs.
# - `921-01-rows`: Ctrl P, "rail": each row the extension, the name over the path, "M" on
#   `rail.rs` and "U" on `rail_new.rs`; the field ends with "Ctrl-P"; the preview headed
#   "src/rail.rs · N lines" (REQ-001 to REQ-004).
# - `921-02-history`: `src/rail.rs` opened, Ctrl P with nothing typed: the history row in the same
#   layout (REQ-001).
# - `921-03-day`: `921-01-rows` in Marley Day (REQ-001 to REQ-004).
# - `921-04-zed`: Zed's layout, Ctrl P, "rail": Zed's rows, field and preview (REQ-005).
compositor sway

setup() {
  local repo=$E2E_WORK/repo line
  mkdir -p "$E2E_WORK/home" "$repo/src" "$repo/docs" "$repo/script"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  for line in $(seq 1 1200); do
    printf 'pub fn rail_%s() {}\n' "$line"
  done >"$repo/src/rail.rs"
  printf 'pub fn order() {}\n' >"$repo/src/rail_order.rs"
  printf '# The rail\n' >"$repo/docs/rail.md"
  printf '#!/bin/sh\necho rail\n' >"$repo/script/rail.sh"
  git init -q -b main "$repo"
  git -C "$repo" add -A
  git -C "$repo" -c user.name=t -c user.email=t@t commit -qm init
  printf 'pub fn changed() {}\n' >>"$repo/src/rail.rs"
  printf 'pub fn new() {}\n' >"$repo/src/rail_new.rs"
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

finder() {
  press "CTRL" p
  settle 3
  type_text "$1"
  settle 3
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
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== rail"
  finder rail
  shot 921-01-rows
  press "" Return
  settle 3

  echo "== the history"
  press "CTRL" p
  settle 3
  shot 921-02-history
  press "" Escape
  settle 1

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 3
  finder rail
  shot 921-03-day
  press "" Escape
  profile_setting theme '"Marley Night"'
  settle 2

  echo "== Zed's layout"
  palette "marley: use zed layout" 5
  finder rail
  shot 921-04-zed
  press "" Escape
}
