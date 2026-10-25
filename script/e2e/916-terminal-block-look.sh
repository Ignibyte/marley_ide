# shellcheck shell=bash
# #916's visual check: terminal blocks as the Terminal board draws them. Project `repo` (a git
# repository on `main`), the prompt editor at every prompt (the default), a stand-in `claude` first
# on the terminals' PATH that prints what it was asked and waits.
# - `916-01-blocks`: `git status --short` then `false`: two rounded frames with a gap, the passed
#   one neutral with a green dot, the failed one red with its header band and the action row under
#   its output (REQ-001 to REQ-003).
# - `916-02-running`: `sleep 8` running: a honey frame and dot, "running · N s", the progress line
#   along its bottom (REQ-004, REQ-005).
# - `916-03-ticked`: two seconds on: the time moved (REQ-004).
# - `916-04-ask`: `false` again and Ask an agent clicked: a new terminal runs the stand-in, which
#   printed the fix request and the block (REQ-003).
# - `916-05-day`: Marley Day: the first terminal's blocks and a running one (REQ-001 to REQ-005).
compositor sway

AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-300}
# From the probe run's shot: the failed block's Ask an agent.
ASK_X=${ASK_X:-322}
ASK_Y=${ASK_Y:-859}

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
  printf 'notes\n' >"$E2E_WORK/repo/notes.md"
  profile_setting marley.tmux_hosts '[]'
  open_path "$E2E_WORK/repo"
}

run() {
  type_text "$1"
  press "" Return
  settle "${2:-3}"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 12
  # Trusts repo.
  press "" Return
  settle 4
  run "git status --short"
  run false
  away
  shot 916-01-blocks
  [ -n "${STOP_AT_BLOCKS:-}" ] && return

  echo "== a running block"
  run "sleep 8" 3
  away
  shot 916-02-running
  settle 2
  shot 916-03-ticked
  settle 6

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 3
  run "sleep 5" 3
  away
  shot 916-05-day
  settle 4
  profile_setting theme '"Marley Night"'
  settle 3

  echo "== Ask an agent"
  run false
  away
  pointer_to "$ASK_X" "$ASK_Y"
  settle 1
  click "$ASK_X" "$ASK_Y"
  settle 8
  away
  shot 916-04-ask
}
