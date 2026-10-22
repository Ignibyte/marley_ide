# shellcheck shell=bash
# #897's visual check: the Marley Guide's Try it links. A scratch repository with three commits;
# Marley's Chromium reaches no host but this machine (`offline_chromium`).
# - `marley: open guide`, at the rail's section: its Try it link (`897-01-guide`, REQ-001).
# - The link clicked: the rail folded to its strip, which only the command does (`897-02-tried`,
#   REQ-001). Clicked again, the same link unfolds it: the page cleared its fragment (D2).
# - The "This guide" section, its Changes link clicked: Changes, then its History tab with the
#   commits (`897-03-changes`, REQ-002).
# - Marley Day, the guide's links (`897-04-day`, REQ-001).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

STOP_AT=${STOP_AT:-}
# The rail section's Try it, the "This guide" section's Changes link, the git panel's History tab
# and the guide's tab, from the first run's shots.
TRY_X=${TRY_X:-912}
TRY_Y=${TRY_Y:-195}
# The same link with the rail folded, 200 px to the left.
FOLDED_X=${FOLDED_X:-712}
CHANGES_X=${CHANGES_X:-944}
CHANGES_Y=${CHANGES_Y:-641}
HISTORY_X=${HISTORY_X:-1264}
HISTORY_Y=${HISTORY_Y:-83}
GUIDE_X=${GUIDE_X:-545}
GUIDE_Y=${GUIDE_Y:-50}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo subject
  offline_chromium
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$E2E_WORK/gitconfig" <<'CONFIG'
[user]
	name = Scenario
	email = scenario@example.invalid
CONFIG
  export GIT_CONFIG_GLOBAL=$E2E_WORK/gitconfig GIT_CONFIG_NOSYSTEM=1
  git init -q -b main "$repo"
  for subject in "Start the notes" "Show tmux sessions under their project" \
    "File the three design boards as tickets"; do
    printf '%s\n' "$subject" >>"$repo/notes.txt"
    git -C "$repo" add -A
    git -C "$repo" commit -q -m "$subject"
  done
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

teardown() {
  browser_teardown
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

# Moves the guide's tab to the article `$1`.
guide_at() {
  press CTRL l
  settle 1
  type_text "file://$E2E_PROFILE/guide/index.html#$1"
  press "" Return
  settle 3
}

# Clicks `$1`,`$2` when the first run placed it.
click_placed() {
  if (($1 > 0)); then
    click "$1" "$2"
    settle 3
  fi
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the guide, at the rail"
  palette "marley: open guide" 8
  guide_at rail-overview
  pointer_to 900 950
  settle 1
  take 897-01-guide || return 0

  echo "== Try it: the rail folds"
  click_placed "$TRY_X" "$TRY_Y"
  pointer_to 900 950
  settle 1
  take 897-02-tried || return 0
  echo "   the same link again: the rail unfolds"
  click_placed "$FOLDED_X" "$TRY_Y"

  echo "== This guide: Changes, then its History"
  guide_at this-guide
  click_placed "$CHANGES_X" "$CHANGES_Y"
  click_placed "$HISTORY_X" "$HISTORY_Y"
  pointer_to 900 950
  settle 1
  take 897-03-changes || return 0

  echo "== Marley Day, the guide"
  profile_setting theme '"Marley Day"'
  settle 3
  click_placed "$GUIDE_X" "$GUIDE_Y"
  pointer_to 900 950
  settle 1
  take 897-04-day || return 0
}
