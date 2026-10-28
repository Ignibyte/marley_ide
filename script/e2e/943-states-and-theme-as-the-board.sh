# shellcheck shell=bash
# #943's visual check: empty states and theme details as the boards draw them. A scratch repository
# with a README; `marley.harness` names a command that is not there (#857's).
# - The repository's window: the rail's harness card, "The harness is stopped" (`943-03-harness`,
#   REQ-003).
# - The README edited and closed, Right and Left back to Save: the lighter ring round Save
#   (`943-04-ring`, REQ-004), and in Marley Day (`943-04-ring-day`).
# - `zed: about`: both sentences under the headline (`943-05-about`, REQ-005).
# - `workspace: new window`, which holds Home's group only: "No project open" over Home's grid
#   (`943-01-home`, REQ-001); Files shown: the card in the panel (`943-02-files`, REQ-002); Home in
#   Marley Day (`943-06-day`, REQ-001).
compositor sway

STOP_AT=${STOP_AT:-0}
AWAY_X=${AWAY_X:-1500}
AWAY_Y=${AWAY_Y:-950}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.harness "{\"command\": \"$E2E_WORK/no-rh\", \"args\": [\"mcp\"]}"
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

# Edits the README, closes it, and chooses Save with the arrow keys; the shot is `$1`.
save_prompt() {
  palette "file finder: toggle" 3
  type_text "README.md"
  settle 3
  press "" Return
  settle 3
  type_text "edited "
  settle 1
  palette "pane: close active item" 2
  # Away from Save and back to it with the arrow keys.
  press "" Right
  settle 1
  press "" Left
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot "$1"
  press "" Escape
  settle 2
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 6

  echo "== the harness card"
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 943-03-harness

  echo "== the chosen Save"
  save_prompt 943-04-ring
  if [[ $STOP_AT == 1 ]]; then
    return
  fi
  profile_setting theme '"Marley Day"'
  settle 4
  save_prompt 943-04-ring-day
  profile_setting theme '"Marley Night"'
  settle 4

  echo "== About"
  palette "zed: about" 4
  shot 943-05-about
  press "" Escape
  settle 2

  echo "== a window with no project"
  palette "workspace: new window" 8
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 943-01-home
  palette "project panel: toggle focus" 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 943-02-files
  profile_setting theme '"Marley Day"'
  settle 4
  shot 943-06-day
}
