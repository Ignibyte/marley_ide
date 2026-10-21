# shellcheck shell=bash
# #826's visual check: terminal blocks framed, timed, and asking an agent. Project `repo` (a git
# repository on `main`), the prompt editor at every prompt (the default), a stand-in `claude` first
# on the terminals' PATH that prints what it was asked and waits.
# - `true`, `sleep 2` and `false` run: three framed blocks (green, green, red) with their
#   durations, the failed one with Ask an agent and Explain, and the prompt editor with the folder
#   and branch chips (`826-01-blocks`, REQ-001 to REQ-004).
# - Explain clicked with no agent running: a new terminal runs the stand-in, which printed the
#   explain request and the block (`826-02-explain`, REQ-003).
# - Marley Day (`826-03-day`, REQ-001, REQ-002, REQ-004).
compositor sway

AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the probe run's shots: the failed block's Explain.
EXPLAIN_X=${EXPLAIN_X:-1501}
EXPLAIN_Y=${EXPLAIN_Y:-846}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
  printf "PS1=%q\nexport PATH=%q:\"\$PATH\"\n" '$ ' "$bin" >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$bin/claude" <<'FAKE'
#!/bin/sh
# A stand-in Claude Code: it prints what it was asked and waits.
printf 'stand-in claude was asked:\n%s\n' "$*"
exec sleep 600
FAKE
  chmod +x "$bin/claude"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  git init -q -b main "$E2E_WORK/repo"
  git -C "$E2E_WORK/repo" add -A
  git -C "$E2E_WORK/repo" -c user.name=t -c user.email=t@t commit -qm init
  profile_setting marley.tmux_hosts '[]'
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts repo.
  press "" Return
  settle 4
  for command in true "sleep 2" false; do
    type_text "$command"
    press "" Return
    settle 3
  done
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 2
  shot 826-01-blocks

  echo "== Explain"
  pointer_to "$EXPLAIN_X" "$EXPLAIN_Y"
  settle 1
  click "$EXPLAIN_X" "$EXPLAIN_Y"
  settle 8
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 826-02-explain

  echo "== Marley Day"
  press "CTRL" Page_Up
  settle 2
  profile_setting theme '"Marley Day"'
  settle 4
  shot 826-03-day
}
