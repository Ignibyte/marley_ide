# shellcheck shell=bash
# #869's visual check: toasts as the Inbox board draws them. A scratch repository; Voice off.
# - `marley: toggle dictation`, then `workspace: use classic layout`: two toasts, each an accent dot,
#   a title and a muted line, the layout one with its Use Zed's Layout button (`869-01-night`,
#   REQ-001).
# - Marley Day, the same toasts (`869-02-day`, REQ-001).
compositor sway

AWAY_X=${AWAY_X:-700}
AWAY_Y=${AWAY_Y:-300}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.voice '{"enabled": false}'
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

  echo "== two toasts"
  palette "marley: toggle dictation" 3
  palette "workspace: use classic layout" 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 869-01-night

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 869-02-day
}
