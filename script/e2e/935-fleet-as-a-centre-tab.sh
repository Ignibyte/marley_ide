# shellcheck shell=bash
# #935's visual check: the Fleet in its centre tab as the Fleet board draws it, over Marley's pseudo
# provider (the contract's moving examples, nothing outside the run). No agent CLI, no Rusty.
# - `935-02-none`: `marley: toggle fleet` in a full-width pane, nothing selected: the "Fleet" title
#   and summary, the host cards on the left, "Select an agent…" on the right (REQ-002, REQ-003).
# - `935-01-beside`: an agent's row clicked: its snapshot in the right column (REQ-001, REQ-005).
# - `935-03-under`: `pane: split right` with the Fleet in front, half the width: the snapshot under
#   the list with the split's handle (REQ-004).
# - `935-04-day`: the beside view in Marley Day (REQ-001, REQ-003).
compositor sway

STOP_AT=${STOP_AT:-}
AWAY_X=${AWAY_X:-1500}
AWAY_Y=${AWAY_Y:-950}
# An agent's row in the list, from the probe run's shot.
ROW_X=${ROW_X:-340}
ROW_Y=${ROW_Y:-188}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.fleet '{"providers": [{"kind": "pseudo"}], "hosts": []}'
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
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the Fleet, nothing selected"
  palette "marley: toggle fleet" 6
  away
  take 935-02-none || return 0

  echo "== an agent selected"
  click "$ROW_X" "$ROW_Y"
  settle 3
  away
  take 935-01-beside || return 0

  echo "== half the width"
  palette "pane: split right" 3
  # The split moves the Fleet into the right half.
  away
  take 935-03-under || return 0
  palette "pane: close all items" 2

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  # Closing the split's items closed the Fleet too; it opens again full width, still selected.
  palette "marley: toggle fleet" 4
  away
  shot 935-04-day
}
