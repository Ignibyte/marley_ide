# shellcheck shell=bash
# #918's visual check: the block map, a build's own progress, and the package hint. Project `repo`
# (a git repository on `main`); a HOME whose `.bashrc` puts the stand-ins first on the terminals'
# PATH (F-claude-903): a stand-in `cargo` that prints two `Compiling` lines, then redraws cargo's
# `Building [..] N/40` bar with a carriage return, a step every quarter second, and waits. This box
# runs Arch, so `dpkg` is not found (127).
# - `918-04-hint`: `dpkg -l libfoo` failed: "This box runs Arch. Try", `pacman -Qs libfoo`, Ask an
#   agent (REQ-005).
# - `918-05-typed`: the command clicked: `pacman -Qs libfoo` in the prompt editor, not run
#   (REQ-006).
# - `918-02-bookmarked`: the newest block selected with Ctrl Up and bookmarked: its mark in the
#   accent (REQ-002).
# - `918-03-progress`, `918-01-map`: the stand-in build running: "N of 40" and the bottom line
#   filled to the share; a mark per block down the right edge in its state's colour (REQ-001 to
#   REQ-003).
# - `918-06-day`: Marley Day (REQ-002, REQ-003).
compositor sway

AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-300}
# From the probe run's shot: the hint's command.
HINT_X=${HINT_X:-458}
HINT_Y=${HINT_Y:-813}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
  printf "PS1=%q\nexport PATH=%q:\"\$PATH\"\n" '$ ' "$bin" >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$bin/cargo" <<'FAKE'
#!/bin/sh
# A stand-in cargo: two crates compile, then its progress bar redraws on one line.
printf '   Compiling alpha v0.1.0\n   Compiling beta v0.1.0\n'
sleep 1
step=1
while [ "$step" -le 40 ]; do
  bar=$(printf '%*s' $((step * 24 / 40)) '' | tr ' ' '=')
  printf '\r    Building [%-25s] %d/40: beta' "$bar>" "$step"
  step=$((step + 1))
  sleep 0.25
done
exec sleep 600
FAKE
  chmod +x "$bin/cargo"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  git init -q -b main "$E2E_WORK/repo"
  git -C "$E2E_WORK/repo" add -A
  git -C "$E2E_WORK/repo" -c user.name=t -c user.email=t@t commit -qm init
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
  run "git status"
  run false
  run "dpkg -l libfoo"
  away
  shot 918-04-hint
  [ -n "${STOP_AT_HINT:-}" ] && return

  echo "== the hint's command"
  click "$HINT_X" "$HINT_Y"
  settle 2
  away
  shot 918-05-typed
  press "CTRL" a
  press "" BackSpace
  settle 1

  echo "== a block bookmarked"
  press "CTRL" Up
  settle 1
  press "CTRL SHIFT" b
  settle 1
  press "" Escape
  settle 1
  away
  shot 918-02-bookmarked

  echo "== a build that shows its progress"
  run "cargo build" 6
  away
  shot 918-03-progress
  shot 918-01-map

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 3
  shot 918-06-day
}
