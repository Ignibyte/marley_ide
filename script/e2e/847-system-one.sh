# shellcheck shell=bash
# #847's visual check: System One as its board draws it. A scratch repository; System One on with
# the `rules` provider (no key, nothing leaves the machine), never the user's account.
# - The Marley settings page searched for "system one": each use's mode as Off / Shadow / Suggest /
#   Act buttons (`847-01-uses`).
# - `marley: open system one calls`: the Provider and Today cards, the note (`847-02-calls`, REQ-002).
# - Marley Day, the uses (`847-03-day`, REQ-001).
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
  profile_setting marley.system_one '{"enabled": true, "provider": "rules", "uses": {"stop_kind": "suggest", "stall_kind": "shadow", "click_consequence": "act"}}'
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

  echo "== the uses"
  palette "marley: open settings" 5
  press "CTRL" f
  settle 1
  type_text "kind"
  settle 3
  pointer_to 1500 950
  settle 1
  shot 847-01-uses
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== System One calls"
  press "CTRL" w
  settle 2
  palette "marley: open system one calls" 4
  shot 847-02-calls

  echo "== Marley Day"
  palette "marley: open settings" 4
  press "CTRL" f
  settle 1
  type_text "kind"
  settle 2
  profile_setting theme '"Marley Day"'
  settle 4
  shot 847-03-day
}
