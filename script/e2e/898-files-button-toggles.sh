# shellcheck shell=bash
# #898's visual check: the rail's Files button puts the tree away on a second click, wherever the
# focus is. A scratch repository with a source file open.
# - The Files button: the tree beside the rail (`898-01-open`, REQ-001).
# - A click in the editor, which takes the focus from the tree, then the Files button again: the
#   tree gone (`898-02-closed`, REQ-001). Before #898 this click focused the tree again.
compositor sway

AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-600}
# The rail header's Files button, as #868 measured it.
FILES_X=${FILES_X:-196}
FILES_Y=${FILES_Y:-17}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo/src"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf 'pub fn total() -> u32 {\n    0\n}\n' >"$repo/src/lib.rs"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
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
  palette "file finder: toggle" 3
  type_text "lib.rs"
  settle 3
  press "" Return
  settle 3

  echo "== the Files button"
  click "$FILES_X" "$FILES_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 898-01-open

  echo "== a click in the editor, then the Files button again"
  click "$AWAY_X" "$AWAY_Y"
  settle 1
  click "$FILES_X" "$FILES_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 898-02-closed
}
