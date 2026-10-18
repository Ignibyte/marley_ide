# shellcheck shell=bash
# #808's visual check: the rail lists the tmux sessions on the hosts `marley.tmux_hosts` names.
# Two tmux servers of the run's own (`TMUX_TMPDIR`) stand in for this machine and for the saved
# host `box-2`, which a fake ssh (`MARLEY_SSH`) reaches; the saved host `gone` refuses the fake's
# connection, and `nowhere` is no saved host.
# - The setting unset: no TMUX heading (`808-01-off`, REQ-006).
# - The setting written: four headings; this machine's `build` (a terminal) and `agent` (Claude
#   Code), `box-2`'s `deploy` (Codex), `gone`'s ssh error and `nowhere`'s (`808-02-rail`, REQ-001,
#   REQ-003, REQ-005).
# - A click on `build`: a tab `this machine · build` attached to it (`808-03-attached`); a second
#   click shows the same tab (`808-04-again`); a click on `deploy`: a tab `box-2 · deploy`
#   (`808-05-remote`, REQ-002).
# - `box-2`'s +: a tab in a new session there, and the session's row under `box-2` (`808-06-new`,
#   REQ-004).
compositor sway

# The rows and the + to click, as the first run's shots found them.
BUILD_X=${BUILD_X:-130}
BUILD_Y=${BUILD_Y:-320}
DEPLOY_X=${DEPLOY_X:-130}
DEPLOY_Y=${DEPLOY_Y:-510}
NEW_X=${NEW_X:-236}
NEW_Y=${NEW_Y:-527}

# tmux's socket path must stay under 108 bytes, which a run's work folder does not.
SOCKETS=${XDG_RUNTIME_DIR:-/tmp}/marley-e2e-808
LOCAL_TMUX=$SOCKETS/local
REMOTE_TMUX=$SOCKETS/remote

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo" "$LOCAL_TMUX" "$REMOTE_TMUX"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  terminal_env TMUX_TMPDIR "$LOCAL_TMUX"
  export TMUX_TMPDIR=$LOCAL_TMUX
  # The fake ssh: `gone` refuses, and any other host runs the words after the destination, as the
  # host's shell would, against the remote server.
  cat >"$bin/ssh" <<SSH
#!/bin/bash
while [[ \$# -gt 0 && \$1 != -- ]]; do shift; done
shift
host=\$1
shift
if [[ \$host == gone ]]; then
  echo "ssh: connect to host gone port 22: Connection refused" >&2
  exit 255
fi
export TMUX_TMPDIR=$REMOTE_TMUX
exec bash -c "\$*"
SSH
  chmod +x "$bin/ssh"
  export MARLEY_SSH=$bin/ssh
  TMUX_TMPDIR=$LOCAL_TMUX HOME=$home tmux -f /dev/null new-session -d -s build -x 100 -y 30
  TMUX_TMPDIR=$LOCAL_TMUX tmux send-keys -t build 'echo hello from build' Enter
  TMUX_TMPDIR=$LOCAL_TMUX tmux new-session -d -s agent \
    "bash -c 'echo Claude Code is thinking; exec -a claude sleep 900'"
  TMUX_TMPDIR=$REMOTE_TMUX HOME=$home tmux -f /dev/null new-session -d -s deploy -x 100 -y 30 \
    "bash -c 'echo Codex is working on box-2; exec -a codex sleep 900'"
  profile_setting ssh_connections '[{"host": "box-2"}, {"host": "gone"}]'
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  TMUX_TMPDIR=$LOCAL_TMUX tmux kill-server 2>/dev/null || true
  TMUX_TMPDIR=$REMOTE_TMUX tmux kill-server 2>/dev/null || true
  rm -rf "$SOCKETS"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  shot 808-01-off

  echo "== the setting names four hosts"
  profile_setting marley.tmux_hosts '["local", "box-2", "gone", "nowhere"]'
  settle 6
  shot 808-02-rail

  echo "== a click on build attaches it"
  click "$BUILD_X" "$BUILD_Y"
  settle 4
  shot 808-03-attached
  TMUX_TMPDIR=$LOCAL_TMUX tmux list-clients -F '#{session_name}' | tee "$E2E_WORK/clients1.txt"
  expect "a client attached build" holds "$E2E_WORK/clients1.txt" "build"

  echo "== a second click shows the same tab"
  click "$BUILD_X" "$BUILD_Y"
  settle 3
  shot 808-04-again
  TMUX_TMPDIR=$LOCAL_TMUX tmux list-clients -F '#{session_name}' | wc -l | tee "$E2E_WORK/clients2.txt"
  expect "still one client" holds "$E2E_WORK/clients2.txt" "1"

  echo "== a click on deploy attaches it on box-2"
  click "$DEPLOY_X" "$DEPLOY_Y"
  settle 4
  shot 808-05-remote

  echo "== box-2's + starts a session there"
  click "$NEW_X" "$NEW_Y"
  settle 5
  TMUX_TMPDIR=$REMOTE_TMUX tmux list-sessions -F '#{session_name}' | tee "$E2E_WORK/remote.txt"
  expect "a marley- session on box-2" holds "$E2E_WORK/remote.txt" "marley-"
  shot 808-06-new
}
