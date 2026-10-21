# shellcheck shell=bash
# #868's visual check: Files pops out from the left. A scratch repository with a source file.
# - At start: the rail's header with its Files button, no tree (`868-01-closed`, REQ-003).
# - The Files button: the tree in the left dock beside the rail, the editor in view, no Files tab
#   (`868-02-open`, REQ-001, REQ-003).
# - The button again, then Ctrl+Shift+E: the tree back in the left dock, no tab (`868-03-key`,
#   REQ-002).
# - `marley.side_inspector` on, `git panel: toggle focus`: the Inspector with Changes and Outline
#   only (`868-04-inspector`, REQ-004).
# - Marley Day, the tree out (`868-05-day`, REQ-001).
compositor sway

AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-600}
# The rail header's Files button, from the probe run's shots.
FILES_X=${FILES_X:-196}
FILES_Y=${FILES_Y:-17}
STOP_AT_START=${STOP_AT_START:-0}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo/src"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$repo/src/lib.rs" <<'RS'
pub fn total(entries: &[u32]) -> u32 {
    entries.iter().sum()
}
RS
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

# Runs `$1` from the command palette.
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
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 868-01-closed
  if [[ $STOP_AT_START == 1 ]]; then
    return
  fi

  echo "== the Files button"
  click "$FILES_X" "$FILES_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 868-02-open

  echo "== closed, then Ctrl+Shift+E"
  click "$FILES_X" "$FILES_Y"
  settle 2
  click "$AWAY_X" "$AWAY_Y"
  settle 1
  press "CTRL SHIFT" e
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 868-03-key

  echo "== the Inspector without Files"
  profile_setting marley.side_inspector 'true'
  settle 3
  palette "git panel: toggle focus" 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 868-04-inspector

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 868-05-day
}
