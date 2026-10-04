# shellcheck shell=bash
# #641's visual check: SSH links that know they are dead. An sshd of the scenario's own on
# 127.0.0.1 (#610's) stands in for a host, its sessions given a tmux socket folder and a HOME of
# the run's own (`SetEnv`), so the remote terminal's tmux server is the run's. `MARLEY_SSH` names a
# wrapper that logs each call's start, argv, end and status, then runs the real ssh with the
# scenario's key. The settings save the sshd as `e2e` and list it as the Fleet host `lab`. "Silent"
# is SIGSTOP on the sshd and every process under it; "back" is SIGCONT.
#
# Connected, a counter ticking, `lab` with its resources (`641-01-connected`, REQ-001, REQ-011);
# silent: the terminal dimmed with its line, `lab` unreachable (`641-02-silent`, REQ-002, REQ-012),
# ssh's reason in `lab`'s tooltip (`641-03-host-reason`); typed keys counted and not sent
# (`641-04-input-off`, REQ-003); back, with a local terminal in front: the reattach leaves it in
# front (`641-05-reattached-behind`, REQ-004, REQ-006) and the remote tab is the same session with
# nothing it refused (`641-06-reattached`, REQ-007); a refused login stops the checks
# (`641-07-refused`, REQ-008); Rerun reattaches at once (`641-08-rerun-now`, REQ-009); `exit` ends
# the tab plainly (`641-09-ended`, REQ-010).
compositor sway

# Where `lab`'s chip sits on the Fleet panel, from the first shots; 0 skips its tooltip.
LAB_X=${LAB_X:-1365}
LAB_Y=${LAB_Y:-108}

SSHD_PID=""

# An sshd of the scenario's own on 127.0.0.1 (#610's), whose sessions get the run's tmux folder and
# HOME, and the wrapper Marley runs as ssh.
start_sshd() {
  local dir=$E2E_WORK/ssh tmux_dir remote=$E2E_WORK/remote-home
  mkdir -p "$dir" "$remote"
  chmod 700 "$dir"
  # tmux starts a login shell, which reads .bash_profile; sshd starts it in the user's own home,
  # so it moves to the run's.
  printf 'PS1=%q\ncd\n' 'remote$ ' >"$remote/.bash_profile"
  # A tmux socket's path must fit 108 bytes, which the run's folder does not (#543).
  tmux_dir=$(mktemp -d "${TMPDIR:-/tmp}/e2e-tmux.XXXXXX")
  echo "$tmux_dir" >"$E2E_WORK/tmux.dir"
  ssh-keygen -q -t ed25519 -N '' -f "$dir/host"
  ssh-keygen -q -t ed25519 -N '' -f "$dir/client"
  cp "$dir/client.pub" "$dir/authorized_keys"
  chmod 600 "$dir/authorized_keys"
  SSHD_PORT=$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
  echo "$SSHD_PORT" >"$E2E_WORK/sshd.port"
  cat >"$dir/sshd_config" <<EOF
ListenAddress 127.0.0.1
Port $SSHD_PORT
HostKey $dir/host
AuthorizedKeysFile $dir/authorized_keys
PidFile $dir/sshd.pid
UsePAM no
StrictModes no
PasswordAuthentication no
KbdInteractiveAuthentication no
PermitRootLogin no
AllowTcpForwarding no
AllowStreamLocalForwarding no
X11Forwarding no
AllowAgentForwarding no
PermitTunnel no
SetEnv TMUX_TMPDIR=$tmux_dir HOME=$remote SHELL=/bin/bash
EOF
  /usr/bin/sshd -D -e -f "$dir/sshd_config" 2>"$dir/sshd.log" &
  SSHD_PID=$!
  echo "$SSHD_PID" >"$E2E_WORK/sshd.pid"
  for _ in $(seq 50); do
    ss -ltnH "sport = :$SSHD_PORT" | grep -q . && break
    sleep 0.1
  done
  echo "the scenario's sshd: pid $SSHD_PID on 127.0.0.1:$SSHD_PORT"
  cat >"$E2E_WORK/bin/marley-ssh" <<EOF
#!/bin/sh
echo "start \$(date +%s.%N) \$*" >> $E2E_WORK/ssh.log
ssh -F /dev/null -i $dir/client -o IdentitiesOnly=yes -o IdentityAgent=none \\
  -o UserKnownHostsFile=$dir/known_hosts -o StrictHostKeyChecking=accept-new -o LogLevel=ERROR "\$@"
status=\$?
echo "end \$(date +%s.%N) \$status \$*" >> $E2E_WORK/ssh.log
exit \$status
EOF
  chmod +x "$E2E_WORK/bin/marley-ssh"
  export MARLEY_SSH=$E2E_WORK/bin/marley-ssh
}

# The sshd and every process under it, by parent, never by `pgrep -f` (L-448).
sshd_tree() {
  local pids=("$SSHD_PID") index=0 child
  while ((index < ${#pids[@]})); do
    for child in $(pgrep -P "${pids[$index]}"); do
      pids+=("$child")
    done
    index=$((index + 1))
  done
  echo "${pids[@]}"
}

silence() {
  # shellcheck disable=SC2046 # one pid a word
  kill -STOP $(sshd_tree)
}

resume() {
  # shellcheck disable=SC2046 # one pid a word
  kill -CONT $(sshd_tree)
}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo" "$E2E_WORK/bin"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  : >"$E2E_WORK/ssh.log"
  start_sshd
  profile_setting ssh_connections "[{\"host\": \"127.0.0.1\", \"port\": $SSHD_PORT, \"nickname\": \"e2e\"}]"
  profile_setting marley.fleet "{\"providers\": [], \"hosts\": [{\"ssh\": \"127.0.0.1:$SSHD_PORT\", \"name\": \"lab\"}]}"
  open_path "$E2E_WORK/repo"
}

teardown() {
  local tmux_dir
  if [[ -n $SSHD_PID ]]; then
    resume 2>/dev/null
    kill "$SSHD_PID" 2>/dev/null
  fi
  if [[ -f $E2E_WORK/tmux.dir ]]; then
    tmux_dir=$(cat "$E2E_WORK/tmux.dir")
    TMUX_TMPDIR=$tmux_dir tmux -L marley kill-server 2>/dev/null
    [[ $tmux_dir == */e2e-tmux.* ]] && rm -rf "$tmux_dir"
  fi
  return 0
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# The remote session's screen, from its tmux server.
remote_screen() {
  TMUX_TMPDIR=$(cat "$E2E_WORK/tmux.dir") tmux -L marley capture-pane -p
}

# Whether the terminal's first argv carries the keepalive and the connect timeout before `--`.
# `grep -m1`, not `head`: under the runner's pipefail, a reader that quits early fails the pipe.
terminal_argv_has_options() {
  grep -m1 '^start .* -t ' "$E2E_WORK/ssh.log" | grep -E -- \
    '-t -o ServerAliveInterval=5 -o ServerAliveCountMax=3 -o ConnectTimeout=10 -p [0-9]+ -- ' >/dev/null
}

# Whether the collector's argv carries the keepalive.
collector_argv_has_options() {
  grep -m1 '^start .*BatchMode=yes.*ConnectTimeout=5' "$E2E_WORK/ssh.log" |
    grep -F -- '-o ServerAliveInterval=5 -o ServerAliveCountMax=3' >/dev/null
}

# The link checks' lines: `start` and `end` lines of the ssh that runs `true`.
checks() { grep -E '^(start|end) .* true$' "$E2E_WORK/ssh.log"; }

# Waits up to `$1` seconds for a reattach: a terminal argv started after the drop.
wait_for_reattach() {
  local seconds=$1 before
  before=$(grep -c '^start .* -t ' "$E2E_WORK/ssh.log")
  for _ in $(seq "$seconds"); do
    (($(grep -c '^start .* -t ' "$E2E_WORK/ssh.log") > before)) && return 0
    sleep 1
  done
  return 1
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== connected: the remote terminal, a counter, the Fleet panel"
  palette "marley: open remote terminal"
  settle 2
  press "" Return
  settle 5
  type_text "n=0; while :; do n=\$((n+1)); echo \"tick \$n\"; sleep 1; done &"
  press "" Return
  palette "marley: toggle fleet"
  settle 12
  # The panel took the focus; the keys typed later go to the remote terminal, and a click on a
  # dimmed one lands on its line.
  click 700 500
  settle 1
  shot 641-01-connected
  expect "the terminal's ssh has the keepalive and the connect timeout before --" terminal_argv_has_options
  expect "the collector's ssh has the keepalive" collector_argv_has_options

  echo "== silent: the sshd stopped"
  silence
  settle 25
  shot 641-02-silent
  if ((LAB_X > 0)); then
    pointer_to "$LAB_X" "$LAB_Y"
    settle 2
    shot 641-03-host-reason
    pointer_to 700 500
    settle 1
  fi

  echo "== input off"
  type_text "echo typed-while-down"
  press "" Return
  settle 2
  shot 641-04-input-off

  echo "== back, with a local terminal in front"
  palette "workspace: new terminal"
  settle 30
  resume
  expect "the checks reattach once the host answers" wait_for_reattach 90
  settle 4
  shot 641-05-reattached-behind
  checks

  echo "== the remote tab"
  palette "pane: activate previous item"
  settle 3
  shot 641-06-reattached
  remote_screen | grep -v '^$' | tail -4
  expect "the host's session never got the refused line" bash -c "! TMUX_TMPDIR=$(cat "$E2E_WORK/tmux.dir") tmux -L marley capture-pane -p -S - | grep -q typed-while-down"

  echo "== a refused login: the key taken away, the session's connection killed"
  # A link up a minute starts its checks over from 1 s.
  settle 62
  : >"$E2E_WORK/ssh/authorized_keys"
  # shellcheck disable=SC2046 # one pid a word
  kill $(pgrep -P "$SSHD_PID")
  settle 8
  shot 641-07-refused
  local before after
  before=$(checks | grep -c '^start')
  settle 20
  after=$(checks | grep -c '^start')
  expect "no check after the refusal" test "$before" -eq "$after"
  checks | tail -2

  echo "== Rerun now"
  cp "$E2E_WORK/ssh/client.pub" "$E2E_WORK/ssh/authorized_keys"
  palette "terminal: rerun task"
  settle 5
  shot 641-08-rerun-now

  echo "== ended by the user"
  type_text "kill %1; exit"
  press "" Return
  settle 4
  shot 641-09-ended
  before=$(checks | grep -c '^start')
  settle 6
  after=$(checks | grep -c '^start')
  expect "no check after the user's exit" test "$before" -eq "$after"
  tail -6 "$E2E_WORK/ssh.log"
}
