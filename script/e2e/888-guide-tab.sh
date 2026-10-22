# shellcheck shell=bash
# #888's visual check: the guided review as the DiffReview board draws its guide. #733's stand-in
# agent asks `review_guide_open` with a guide of two sections; the ask is allowed for the session.
# - The Guide tab: "Guide · from the agent", the title, "0 of 2 reviewed" and its bar, the two
#   sections as cards with Reviewed at their right (`888-01-guide`, REQ-001).
# - The first section ticked: "1 of 2 reviewed", the bar half full, the card folded
#   (`888-02-ticked`, REQ-001).
# - Marley Day (`888-03-day`, REQ-001).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# Points from the shots: the ask's Allow for This Session, and the first section's Reviewed box.
ALLOW_X=${ALLOW_X:-1460}
ALLOW_Y=${ALLOW_Y:-929}
BOX_X=${BOX_X:-1497}
BOX_Y=${BOX_Y:-245}
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-900}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
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
  printf 'alpha one\nalpha two\nalpha three\n' >"$repo/alpha.txt"
  printf 'beta one\nbeta two\nbeta three\n' >"$repo/beta.txt"
  git -C "$repo" add -A
  git -C "$repo" -c core.hooksPath=/dev/null commit -q -m "Start"
  printf 'alpha one\nALPHA TWO\nalpha three\n' >"$repo/alpha.txt"
  printf 'beta one\nBETA TWO\nbeta three\n' >"$repo/beta.txt"
  open_path "$repo"
}

# The guide's arguments: two sections, or with `$1` set, a third section to show the replacement.
guide() {
  local repo=$E2E_WORK/repo extra=""
  [[ -n ${1:-} ]] && extra=", {\"title\": \"A third look\", \"summary\": \"Added on the second call.\", \"files\": []}"
  cat <<JSON
{"title": "Shout the second lines", "summary": "Two files, one change each: the second line in capitals.",
 "sections": [
  {"title": "Alpha first", "summary": "The **second** line of alpha is now in capitals.",
   "files": [{"path": "$repo/alpha.txt", "first_line": 2}]},
  {"title": "Then beta", "summary": "The same change in beta, to match.",
   "files": [{"path": "$repo/beta.txt", "first_line": 2}]}$extra
 ]}
JSON
}

steps() {
  local asking
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== review_guide_open asks first, then opens the Guide tab"
  mcp_agent tool review_guide_open "$(guide)" >"$E2E_WORK/answer.txt" 2>&1 &
  asking=$!
  settle 4
  click "$ALLOW_X" "$ALLOW_Y"
  wait "$asking"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 888-01-guide || return 0

  echo "== section 1 ticked"
  click "$BOX_X" "$BOX_Y"
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 888-02-ticked || return 0

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  take 888-03-day || return 0
}
