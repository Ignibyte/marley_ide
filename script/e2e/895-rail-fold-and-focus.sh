# shellcheck shell=bash
# #895's visual check: the rail's labels fade as it folds, and one focus ring. A scratch
# repository; `MARLEY_MOTION_SCALE=40` (read by debug builds only) stretches the 180 ms fold to
# 7.2 s and the labels' 60 ms fade to 2.4 s, so a shot can land inside them.
# - `895-01-fold-early`: just after `marley: toggle rail strip`: the rows still drawn, fading
#   (REQ-001).
# - `895-02-fold-late`: later in the fold: the strip, the width still easing (REQ-001).
# - `895-05-field`: the Settings window open, its search field focused: the ring (REQ-002).
# - `895-03-ring`: Tab onto the next controls: the ring outside the focused one (REQ-002).
# - `895-04-click`: a control clicked with the mouse: no ring (REQ-002).
# - `895-06-day`: Marley Day, Tab again: the ring (REQ-002).
compositor sway

AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-600}
# From the probe run's shots: a control in the Settings window to click.
CONTROL_X=${CONTROL_X:-1445}
CONTROL_Y=${CONTROL_Y:-397}
FIELD_X=${FIELD_X:-912}
FIELD_Y=${FIELD_Y:-59}
TABS=${TABS:-3}
# A first run that stops after the field's shot, to read the positions.
STOP_AT=${STOP_AT:-}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  export MARLEY_MOTION_SCALE=40
  open_path "$repo"
}

tabs() {
  local count
  for count in $(seq 1 "$1"); do
    echo "tab $count"
    press "" Tab
    settle 0.5
  done
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the rail folds"
  pointer_to "$AWAY_X" "$AWAY_Y"
  press "CTRL SHIFT" p
  settle 2
  type_text "marley: toggle rail strip"
  settle 3
  press "" Return
  settle 0.6
  shot 895-01-fold-early
  settle 2.4
  shot 895-02-fold-late
  settle 6

  echo "== the Settings window's search field"
  press CTRL comma
  settle 5
  click "$FIELD_X" "$FIELD_Y"
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 895-05-field
  [[ $STOP_AT == 1 ]] && return 0

  echo "== Tab onto the controls"
  tabs "$TABS"
  shot 895-03-ring

  echo "== a control clicked"
  click "$CONTROL_X" "$CONTROL_Y"
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 895-04-click

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 3
  tabs 1
  shot 895-06-day
}
