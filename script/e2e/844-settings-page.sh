# shellcheck shell=bash
# #844's visual check: the Settings window's Marley page as the boards draw it. The profile sets
# marley.rail_order to "attention", away from its default.
# - `marley: open settings`: the Layout card: Layout and Rail order as segmented choices, Rail
#   order "changed · reset", each row's key under it, switches (`844-01-layout`, REQ-001,
#   REQ-002).
# - "changed · reset" clicked: Rail order back to Window, the marker gone (`844-02-reset`,
#   REQ-002).
# - The settings search for "harness": the new rows Run the Harness and Harness Writes
#   (`844-03-new`, REQ-003).
# - Marley Day, the page as the search left it (`844-04-day`, REQ-001).
compositor sway

AWAY_X=${AWAY_X:-1500}
AWAY_Y=${AWAY_Y:-900}
# From the probe run's shot: Rail order's "changed · reset".
RESET_X=${RESET_X:-1189}
RESET_Y=${RESET_Y:-332}
STOP_AT=${STOP_AT:-0}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.rail_order '"attention"'
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

  echo "== the Marley page"
  palette "marley: open settings" 5
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 844-01-layout
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== changed · reset"
  click "$RESET_X" "$RESET_Y"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 844-02-reset

  echo "== the new rows"
  press "CTRL" f
  settle 1
  type_text "harness"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 844-03-new

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 844-04-day
}
