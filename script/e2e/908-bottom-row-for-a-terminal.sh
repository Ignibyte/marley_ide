# shellcheck shell=bash
# #908's visual check: the bottom row for a terminal. A scratch repository on `main` with one
# committed file, then changed; a stand-in `claude` (Python, so Zed names it by its script) first
# on the terminals' PATH through `.bashrc`, never the box's own (PR-687, F-claude-903); a tmux
# server of the run's own (`TMUX_TMPDIR`, F-claude-880) with `one` and `two` detached and `here`
# attached from a second terminal tab.
# - `908-01-root`: the project's terminal at the root after `false`: the `>_` chip, the folder,
#   "main · 1 changed" in violet, the red "false · exit 1 · Ask an agent" chip, the blocks, the
#   size and "tmux: 3 · 2 detached" (REQ-001, REQ-002, REQ-003, REQ-005, REQ-006, REQ-007).
# - `908-02-bookmarked`: the block bookmarked: "· 1 bookmarked" (REQ-005).
# - `908-03-asked`: the chip clicked: with no agent running, the stand-in Claude Code starts with
#   "This command failed. Fix it:" and the block (REQ-004).
# - `908-04-sessions`: the rail shrunk, the tmux chip clicked: the rail opens on its Sessions view
#   (REQ-008).
# - `908-05-resized`: the pane split right: the size has fewer columns (REQ-006).
# - `908-06-day`: Marley Day (REQ-001, REQ-002, REQ-003, REQ-007).
compositor sway

AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-500}
# The bottom row's failed chip and tmux chip, from the first run's shots.
FAILED_X=${FAILED_X:-710}
TMUX_STRIP_X=${TMUX_STRIP_X:-780}
ROW_Y=${ROW_Y:-985}

# tmux's socket path must stay under 108 bytes, which a run's work folder does not (#808).
SOCKETS=${XDG_RUNTIME_DIR:-/tmp}/marley-e2e-908
LOCAL_TMUX=$SOCKETS/local

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo name
  mkdir -p "$home" "$bin" "$repo" "$LOCAL_TMUX"
  printf "PS1=%q\nexport PATH=%q:\"\$PATH\"\n" '$ ' "$bin" >"$home/.bashrc"
  terminal_env HOME "$home"
  terminal_env TMUX_TMPDIR "$LOCAL_TMUX"
  export TMUX_TMPDIR=$LOCAL_TMUX
  cat >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code: it says what it was asked, and waits.
import sys
print("Claude Code (stand-in) was asked:", flush=True)
print(" ".join(sys.argv[1:]), flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$bin/claude"
  for name in one two here; do
    TMUX_TMPDIR=$LOCAL_TMUX HOME=$home tmux -f /dev/null new-session -d -s "$name" -c "$home" \
      -x 100 -y 30
  done
  git init -q -b main "$repo"
  printf '# repo\n' >"$repo/README.md"
  git -C "$repo" add README.md
  git -C "$repo" -c user.name=e2e -c user.email=e2e@example.invalid commit -q -m first
  printf '# repo\n\nChanged.\n' >"$repo/README.md"
  open_path "$repo"
}

teardown() {
  TMUX_TMPDIR=$LOCAL_TMUX tmux kill-server 2>/dev/null || true
  rm -rf "$SOCKETS"
  return 0
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 15
  # Trusts the scratch repository.
  press "" Return
  settle 4

  echo "== here attached from a second terminal"
  palette "workspace: new terminal" 4
  type_text "tmux attach -t here"
  press "" Return
  settle 3
  palette "pane: activate previous item" 3

  echo "== false at the root"
  type_text "false"
  press "" Return
  settle 4
  away
  shot 908-01-root
  [ -n "${STOP_AT_ROOT:-}" ] && return

  echo "== the block bookmarked"
  palette "marley: toggle bookmark" 2
  away
  shot 908-02-bookmarked

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 908-06-day
  profile_setting theme '"Marley Night"'
  settle 3

  echo "== Ask an agent"
  click "$FAILED_X" "$ROW_Y"
  settle 6
  away
  shot 908-03-asked

  echo "== the tmux chip with the rail shrunk"
  palette "marley: toggle rail strip" 3
  click "$TMUX_STRIP_X" "$ROW_Y"
  settle 3
  away
  shot 908-04-sessions

  echo "== split right"
  # The terminal takes the focus back from the rail first.
  click 900 400
  settle 1
  palette "pane: split right" 4
  # The new pane takes the focus; the narrowed terminal takes it back.
  click 500 400
  settle 2
  away
  shot 908-05-resized
}
