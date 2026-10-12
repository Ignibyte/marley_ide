# shellcheck shell=bash
# #731's visual check: Viewed marks in the project diff. A scratch repository with two committed
# files, `alpha.txt` and `beta.txt`, each with one line changed; `git: diff` from the palette. Viewed
# on `alpha.txt`'s header ticks it and folds it, `beta.txt` open below (`731-01-ticked`, REQ-001,
# REQ-002); after a quit and a start on the same profile, `alpha.txt` is still ticked and folded
# (`731-02-restart`, REQ-003); a line of `alpha.txt` rewritten from outside, as an agent would,
# unticks and unfolds it (`731-03-changed`, REQ-004).
compositor sway

# Viewed on alpha.txt's header, from the first run's shot; `STOP_AT=<shot>` ends the run after
# that shot.
VIEWED_X=${VIEWED_X:-1513}
VIEWED_Y=${VIEWED_Y:-136}
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-700}
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

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  palette "git: diff"
  settle 4
  take 731-00-diff || return 0
  if ((VIEWED_X == 0)); then
    echo "Viewed is not placed yet: 731-00 is the last shot"
    return 0
  fi

  echo "== Viewed on alpha.txt: ticked and folded"
  click "$VIEWED_X" "$VIEWED_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 731-01-ticked || return 0

  echo "== after a restart: still ticked and folded"
  quit_marley
  settle 2
  launch_marley_with "$E2E_WORK/repo"
  settle 14
  palette "git: diff"
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 731-02-restart || return 0

  echo "== alpha.txt rewritten from outside: unticked and unfolded"
  printf 'alpha one\nALPHA TWO\nalpha THREE\n' >"$E2E_WORK/repo/alpha.txt"
  settle 4
  take 731-03-changed || return 0
}
