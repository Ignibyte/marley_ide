# shellcheck shell=bash
# #867's visual check: Home's prompt picks its project. A fresh profile opens `repo`, then quits;
# Marley starts again on `repo2`, so `repo` is recent and not open. Stand-in `claude` and `codex`,
# first on Marley's PATH and the terminals', print their folder and what they were asked, and wait.
# - Home from the status bar, the project chip clicked: Home, the window's `repo2`, and `repo`
#   under Recent (`867-01-menu`, REQ-001).
# - `repo` picked, "write the README" typed, Enter: `repo` opens and a terminal in its group runs
#   the stand-in in `repo` with the task (`867-02-started`, REQ-002).
# - The menu again, in Marley Day (`867-03-day`, REQ-001).
compositor sway

AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the probe run's shots: the status bar's Home, the project chip, `repo` in its menu, the
# prompt box and the trust prompt's button after the relaunch.
HOME_X=${HOME_X:-1179}
HOME_Y=${HOME_Y:-985}
CHIP_X=${CHIP_X:-1000}
CHIP_Y=${CHIP_Y:-183}
RECENT_X=${RECENT_X:-700}
RECENT_Y=${RECENT_Y:-258}
BOX_X=${BOX_X:-700}
BOX_Y=${BOX_Y:-144}
TRUST_X=${TRUST_X:-1079}
TRUST_Y=${TRUST_Y:-410}
# A first run that stops after the Home shot, to read the positions.
STOP_AT=${STOP_AT:-}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin name
  mkdir -p "$home" "$bin" "$E2E_WORK/repo" "$E2E_WORK/repo2"
  printf "PS1=%q\nexport PATH=%q:\"\$PATH\"\n" '$ ' "$bin" >"$home/.bashrc"
  terminal_env HOME "$home"
  rm -rf "$E2E_PROFILE/db"
  for name in claude codex; do
    sed -e "s|@NAME@|$name|" >"$bin/$name" <<'FAKE'
#!/bin/sh
# A stand-in agent CLI: it prints its folder and what it was asked, and waits.
printf 'stand-in @NAME@ in %s was asked:\n%s\n' "$PWD" "$*"
exec sleep 600
FAKE
    chmod +x "$bin/$name"
  done
  export PATH=$bin:$PATH
  git init -q -b main "$E2E_WORK/repo"
  git init -q -b main "$E2E_WORK/repo2"
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.rusty.enabled false
  open_path "$E2E_WORK/repo"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 12
  # Trusts repo.
  press "" Return
  settle 3

  echo "== repo once, then Marley again on repo2"
  click "$HOME_X" "$HOME_Y"
  settle 2
  quit_marley
  open_path "$E2E_WORK/repo2"
  launch_marley
  settle 15
  # After a relaunch the window takes no keys until a click (L-700).
  click "$TRUST_X" "$TRUST_Y"
  settle 3

  echo "== Home"
  click "$HOME_X" "$HOME_Y"
  settle 3
  away
  shot 867-00-home
  [[ $STOP_AT == 1 ]] && return 0

  echo "== the project chip"
  click "$CHIP_X" "$CHIP_Y"
  settle 2
  away
  shot 867-01-menu
  [[ $STOP_AT == 2 ]] && return 0

  echo "== repo with a task"
  click "$RECENT_X" "$RECENT_Y"
  settle 1
  click "$BOX_X" "$BOX_Y"
  settle 1
  type_text "write the README"
  settle 3
  press "" Return
  settle 10
  away
  shot 867-02-started

  echo "== Marley Day"
  click "$HOME_X" "$HOME_Y"
  settle 2
  profile_setting theme '"Marley Day"'
  settle 3
  click "$CHIP_X" "$CHIP_Y"
  settle 2
  away
  shot 867-03-day
}
