# shellcheck shell=bash
# #819's visual check: with tmux installed, a project's + offers New tmux Terminal and New Agent in
# tmux. tmux runs on a server of the run's own (`TMUX_TMPDIR`), `marley.tmux_hosts` is unset, and a
# stand-in `claude` comes first on the PATH of the terminals' shells, the tmux panes' among them; it
# prints the folder it runs in and waits as a program named `claude`. Marley's own PATH comes from the login shell, so the menu
# lists the real CLIs, but the pane's shell runs the stand-in.
# - `repo`'s +: New tmux Terminal after New Terminal, New Agent in tmux › after New Agent in
#   Worktree (`819-01-plus`, REQ-001).
# - New tmux Terminal: a tab attached to a session in `repo`, `pwd` there, and TMUX · this machine
#   in the rail with no setting (`819-02-tmux-terminal`, REQ-002, REQ-004).
# - New Agent in tmux › Claude Code: a tab attached to a session running the stand-in in `repo`,
#   its rail row marked Claude Code (`819-03-tmux-agent`, REQ-003).
compositor sway

# The rail's points on the 1600 x 1000 output, from the first run's shots: `repo`'s +, and in its
# menu New tmux Terminal, New Agent in tmux and, in that submenu, Claude Code.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-136}
TMUX_TERMINAL_X=${TMUX_TERMINAL_X:-150}
TMUX_TERMINAL_Y=${TMUX_TERMINAL_Y:-187}
TMUX_AGENT_X=${TMUX_AGENT_X:-150}
TMUX_AGENT_Y=${TMUX_AGENT_Y:-302}
SUBMENU_CLAUDE_X=${SUBMENU_CLAUDE_X:-320}
SUBMENU_CLAUDE_Y=${SUBMENU_CLAUDE_Y:-302}

# tmux's socket path must stay under 108 bytes, which a run's work folder does not (#808).
SOCKETS=${XDG_RUNTIME_DIR:-/tmp}/marley-e2e-819
LOCAL_TMUX=$SOCKETS/local

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$LOCAL_TMUX"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  terminal_env TMUX_TMPDIR "$LOCAL_TMUX"
  export TMUX_TMPDIR=$LOCAL_TMUX
  # The stand-in answers Marley's version and sign-in checks (#648, #696) and otherwise waits.
  # tmux names a pane by its program's argv[0], so the wait is a `sleep` named `claude` (#808).
  cat >"$bin/claude" <<'FAKE'
#!/bin/bash
case "$1 $2" in
  "--version ") printf '2.1.293 (Claude Code)\n'; exit 0 ;;
  "auth status") printf '{"loggedIn": false}\n'; exit 0 ;;
esac
printf 'claude (stand-in) in %s\n' "$PWD"
exec -a claude sleep 900
FAKE
  chmod +x "$bin/claude"
  git init -q -b tmux "$E2E_WORK/repo"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

teardown() {
  TMUX_TMPDIR=$LOCAL_TMUX tmux kill-server 2>/dev/null || true
  rm -rf "$SOCKETS"
}

# Whether the run's tmux holds a session whose folder is `repo` and whose pane runs `$1`.
pane_runs() {
  TMUX_TMPDIR=$LOCAL_TMUX tmux list-panes -a -F '#{session_path} #{pane_current_command}' |
    tee -a "$E2E_WORK/panes.txt" | grep -q "/repo $1\$"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== repo's +"
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  shot 819-01-plus

  echo "== New tmux Terminal"
  click "$TMUX_TERMINAL_X" "$TMUX_TERMINAL_Y"
  settle 4
  type_text "pwd"
  press "" Return
  settle 3
  shot 819-02-tmux-terminal
  expect "a tmux session runs a shell in repo" pane_runs bash

  echo "== New Agent in tmux, Claude Code"
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  pointer_to "$TMUX_AGENT_X" "$TMUX_AGENT_Y"
  settle 1
  click "$SUBMENU_CLAUDE_X" "$SUBMENU_CLAUDE_Y"
  settle 4
  shot 819-03-tmux-agent
  cat "$E2E_WORK/panes.txt"
  expect "a tmux session runs the claude stand-in in repo" pane_runs claude
}
