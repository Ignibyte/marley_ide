# shellcheck shell=bash
# #848's visual check: the Fleet panel as its board draws it, over Marley's pseudo provider (the
# contract's moving examples, nothing outside the run).
# - `marley: toggle fleet`: the summary line, the hosts as cards, each agent a row with its tile,
#   work and state (`848-01-fleet`, REQ-001).
# - Marley Day (`848-02-day`, REQ-001).
compositor sway

STOP_AT=${STOP_AT:-0}

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

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the Fleet panel"
  palette "marley: toggle fleet" 6
  pointer_to 1500 950
  settle 1
  shot 848-01-fleet
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 848-02-day
}
