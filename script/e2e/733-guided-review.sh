# shellcheck shell=bash
# #733's visual check: a guided review. A scratch repository with `alpha.txt` and `beta.txt`
# committed and each changed; the stand-in MCP client (`mcp_agent`) calls `review_guide_open` with
# two sections, one per file. After Allow, the Guide tab: the title, "0 of 2 reviewed", both
# sections with their summaries, chips and boxes (`733-01-guide`, REQ-001); the chip `alpha.txt:2`
# brings up the project diff at `alpha.txt` (`733-02-chip`, REQ-002); back on the Guide tab, section
# 1 ticked: "1 of 2 reviewed" and section 1 folded to its title (`733-03-ticked`, REQ-003). A second
# call with the same title replaces the sections, the tick cleared (`733-04-replaced`, REQ-005).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The question's Allow for This Session, and the tab's chip, box and tab, from the first run's
# shots; `STOP_AT=<shot>` ends the run after that shot.
ALLOW_X=${ALLOW_X:-1225}
ALLOW_Y=${ALLOW_Y:-933}
CHIP_X=${CHIP_X:-333}
CHIP_Y=${CHIP_Y:-278}
BOX_X=${BOX_X:-298}
BOX_Y=${BOX_Y:-214}
GUIDE_TAB_X=${GUIDE_TAB_X:-590}
GUIDE_TAB_Y=${GUIDE_TAB_Y:-51}
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
  take 733-00-asked || return 0
  click "$ALLOW_X" "$ALLOW_Y"
  wait "$asking"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 733-01-guide || return 0
  cat "$E2E_WORK/answer.txt"
  expect "the guide opened with two sections" holds "$E2E_WORK/answer.txt" '"sections": 2' '"replaced": false'
  if ((CHIP_X == 0)); then
    echo "the chip, the box and the tab are not placed yet: 733-01 is the last shot"
    return 0
  fi

  echo "== the chip shows alpha.txt in the project diff"
  click "$CHIP_X" "$CHIP_Y"
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 733-02-chip || return 0

  echo "== section 1 ticked: folded and counted"
  click "$GUIDE_TAB_X" "$GUIDE_TAB_Y"
  settle 2
  click "$BOX_X" "$BOX_Y"
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 733-03-ticked || return 0

  echo "== the same title again: the sections replaced, the tick cleared"
  mcp_agent tool review_guide_open "$(guide third)" >"$E2E_WORK/again.txt" 2>&1
  settle 2
  take 733-04-replaced || return 0
  cat "$E2E_WORK/again.txt"
  expect "the second call replaced the guide" holds "$E2E_WORK/again.txt" '"sections": 3' '"replaced": true'
}
