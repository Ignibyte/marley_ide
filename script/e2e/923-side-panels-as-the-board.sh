# shellcheck shell=bash
# #923's visual check: the Inspector and Files as the SidePanels board draws them. A scratch
# repository with `src/main.rs`, `src/board.rs` and `README.md` committed, then `src/main.rs` and
# `README.md` changed. The run keeps the copied profile's database, so a stored left-dock size is
# in play as in the review's tour. No agent CLI runs.
# - `923-01-button`: main.rs open, the pane focused: the tab bar's Inspector button, unlit
#   (REQ-001).
# - `923-02-inspector`: the button clicked: the Inspector at the right, the button lit, the
#   segmented switcher with Changes chosen, "Changes 2" (REQ-001 to REQ-003).
# - `923-03-count`: `src/board.rs` changed on disk: "Changes 3" (REQ-003).
# - `923-04-off`: the lit button clicked: the Inspector gone, the button unlit (REQ-004).
# - `923-05-files`: `project panel: toggle focus`: the tree about 320 px wide, "Find a file" at its top,
#   "M" at the right of main.rs and README.md (REQ-005, REQ-006).
# - `923-06-filtered`: "board" typed in the field: only `src` and `board.rs` (REQ-007).
# - `923-07-cleared`: Escape in the field: the whole tree again (REQ-007).
# - `923-08-zed`: Zed's layout: the tab bar's +, split and zoom alone, Files with no field
#   (REQ-008).
# - `923-09-day`: Marley Day, the Marley layout, the Inspector and Files open (REQ-001, REQ-002,
#   REQ-006).
compositor sway

AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-500}
STOP_AT=${STOP_AT:-}
# From the probe run's shots.
BUTTON_X=${BUTTON_X:-1477}
BUTTON_Y=${BUTTON_Y:-50}
INSPECTOR_X=${INSPECTOR_X:-1440}
INSPECTOR_Y=${INSPECTOR_Y:-500}
FIELD_X=${FIELD_X:-420}
FIELD_Y=${FIELD_Y:-58}
EDITOR_X=${EDITOR_X:-700}
EDITOR_Y=${EDITOR_Y:-300}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local repo=$E2E_WORK/repo
  mkdir -p "$E2E_WORK/home" "$repo/src"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  printf 'fn main() {}\n' >"$repo/src/main.rs"
  printf 'pub fn board() {}\n' >"$repo/src/board.rs"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  git -C "$repo" add -A
  git -C "$repo" -c user.name=t -c user.email=t@t commit -qm init
  printf 'fn main() { println!("changed"); }\n' >"$repo/src/main.rs"
  printf '# repo\n\nChanged.\n' >"$repo/README.md"
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.side_inspector 'false'
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

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 15
  # Trusts the scratch repository.
  press "" Return
  settle 3
  press CTRL p
  settle 2
  type_text "main.rs"
  settle 2
  press "" Return
  settle 3
  click "$EDITOR_X" "$EDITOR_Y"
  settle 1
  away
  take 923-01-button || return 0

  echo "== the Inspector on"
  click "$BUTTON_X" "$BUTTON_Y"
  settle 4
  away
  take 923-02-inspector || return 0

  echo "== a third change"
  printf 'pub fn board() { let _ = 1; }\n' >"$E2E_WORK/repo/src/board.rs"
  settle 5
  away
  take 923-03-count || return 0

  echo "== the Inspector off"
  click "$INSPECTOR_X" "$INSPECTOR_Y"
  settle 1
  click "$BUTTON_X" "$BUTTON_Y"
  settle 4
  away
  take 923-04-off || return 0

  echo "== Files"
  palette "project panel: toggle focus" 3
  away
  take 923-05-files || return 0

  echo "== board"
  click "$FIELD_X" "$FIELD_Y"
  settle 1
  type_text "board"
  settle 2
  away
  take 923-06-filtered || return 0

  echo "== cleared"
  click "$FIELD_X" "$FIELD_Y"
  press "" Escape
  settle 2
  away
  take 923-07-cleared || return 0

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 2
  click "$EDITOR_X" "$EDITOR_Y"
  settle 1
  click "$BUTTON_X" "$BUTTON_Y"
  settle 4
  away
  shot 923-09-day
  profile_setting theme '"Marley Night"'
  settle 2

  echo "== Zed's layout"
  palette "marley: use zed layout" 5
  click "$EDITOR_X" "$EDITOR_Y"
  settle 1
  away
  shot 923-08-zed
}
