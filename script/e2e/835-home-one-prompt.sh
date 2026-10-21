# shellcheck shell=bash
# #835's visual check: Home opens on one prompt. Project `repo`; stand-in `claude` and `codex`
# first on Marley's PATH and the terminals', each printing what it was asked and waiting.
# - Home from the status bar: the PROMPT card first, with the box, the Claude Code and Codex chips
#   (Claude Code lit), where it starts, Start, and the links; the cards after; CONFIGURE's entries
#   as links (`835-01-night`, REQ-001, REQ-002, REQ-004).
# - Codex picked, "write the README" typed, Enter: a terminal runs the stand-in codex with the task
#   (`835-02-started`, REQ-003).
# - Home in Marley Day (`835-03-day`, REQ-001, REQ-004).
compositor sway

AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the probe run's shots: the status bar's Home, the Codex chip and the prompt box.
HOME_X=${HOME_X:-1422}
HOME_Y=${HOME_Y:-985}
CODEX_X=${CODEX_X:-437}
CODEX_Y=${CODEX_Y:-183}
BOX_X=${BOX_X:-700}
BOX_Y=${BOX_Y:-144}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin name
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
  printf "PS1=%q\nexport PATH=%q:\"\$PATH\"\n" '$ ' "$bin" >"$home/.bashrc"
  terminal_env HOME "$home"
  for name in claude codex; do
    sed -e "s|@NAME@|$name|" >"$bin/$name" <<'FAKE'
#!/bin/sh
# A stand-in agent CLI: it prints what it was asked and waits.
printf 'stand-in @NAME@ was asked:\n%s\n' "$*"
exec sleep 600
FAKE
    chmod +x "$bin/$name"
  done
  export PATH=$bin:$PATH
  git init -q -b main "$E2E_WORK/repo"
  profile_setting marley.tmux_hosts '[]'
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts repo.
  press "" Return
  settle 3

  echo "== Home"
  click "$HOME_X" "$HOME_Y"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 835-01-night

  echo "== Codex with a task"
  click "$CODEX_X" "$CODEX_Y"
  settle 1
  click "$BOX_X" "$BOX_Y"
  settle 1
  type_text "write the README"
  settle 3
  press "" Return
  settle 8
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 835-02-started

  echo "== Marley Day"
  click "$HOME_X" "$HOME_Y"
  settle 2
  profile_setting theme '"Marley Day"'
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 4
  shot 835-03-day
}
