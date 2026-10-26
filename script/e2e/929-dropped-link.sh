# shellcheck shell=bash
# #929's visual check of the dropped link: a remote terminal whose host stops answering, as the
# board words it. #641's sshd of the scenario's own on 127.0.0.1 stands in for the host, its
# sessions given a tmux socket folder and a HOME of the run's own; `MARLEY_SSH` names a wrapper
# that logs each call. "Silent" is SIGSTOP on the sshd and every process under it; "back" is
# SIGCONT. No agent CLI runs.
# - `929-03-link`: silent, keys typed: "<host> stopped answering · checking again in N s · N keys
#   held back", Try Now, and the tmux line under the strip (REQ-003).
# - `929-07-day-link`: the same strip in Marley Day (REQ-003).
# - `929-04-try-now`: back, Try Now clicked: the terminal attached again at once (REQ-003).
compositor sway

STOP_AT=${STOP_AT:-}
# From the probe run's shot: the strip's Try Now.
TRY_X=${TRY_X:-1564}
TRY_Y=${TRY_Y:-82}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

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
  profile_setting marley.tmux_hosts '[]'
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

# Presses Try Now each two seconds, up to `$1` times, until a reattach starts.
try_now_until_reattach() {
  local times=$1 before
  before=$(grep -c '^start .* -t ' "$E2E_WORK/ssh.log")
  for _ in $(seq "$times"); do
    click "$TRY_X" "$TRY_Y"
    sleep 2
    (($(grep -c '^start .* -t ' "$E2E_WORK/ssh.log") > before)) && return 0
  done
  return 1
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== the remote terminal"
  palette "marley: open remote terminal"
  settle 2
  press "" Return
  settle 6
  click 700 500
  settle 1

  echo "== silent: the sshd stopped, keys typed"
  silence
  settle 25
  type_text "echo held"
  settle 2
  pointer_to 700 600
  settle 1
  take 929-03-link || return 0

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 929-07-day-link
  profile_setting theme '"Marley Night"'
  settle 3

  echo "== back, Try Now"
  resume
  # A check under way when the host comes back is left to end (D3); Try Now acts once the link
  # waits again, so it is pressed each two seconds until a reattach starts.
  expect "Try Now reattaches" try_now_until_reattach 15
  grep -E ' true$' "$E2E_WORK/ssh.log" | tail -6
  settle 4
  pointer_to 700 600
  settle 1
  shot 929-04-try-now
}
