# shellcheck shell=bash
# #445's visual check: Marley's own remote server. Two hosts are faked on loopback: each is an sshd
# of the scenario's own whose ForceCommand runs every command, sftp's server included, in a fake
# home under the run's folder, so nothing reaches the user's ~/.zed_server or ~/.local/share. Marley
# reaches them through `ssh`, `scp` and `sftp` wrappers first on its PATH, which read nothing of
# ~/.ssh, and runs with ZED_BUILD_REMOTE_SERVER=never, so the debug build compiles no server when it
# connects. E2E_REMOTE_SERVER names the server `script/install-marley --remote-server-only` staged
# for the debug build's commit; the scenario stops when it is missing. Keys are made at run time.
# Shots: nothing staged, the connection fails naming the platform, the commit and the install step
# (REQ-004); staged, the first host opens and its ~/.zed_server holds the server named for the commit
# (REQ-001, REQ-002); the second host, whose server at that name answers another commit, opens with
# it replaced (REQ-003).
compositor sway

# The second host's stand-in answers `version` with this commit.
STALE_COMMIT=0000000000000000000000000000000000000445

# The names and times in the user's own folders the run must leave alone; a folder that is not
# there is listed as missing.
user_listing() {
  ls -la --time-style=full-iso "$HOME/.ssh" "$HOME/.zed_server" "$HOME/.local/share/marley/server_state" 2>&1 || true
}

# `fake_host <n>`: host n's home with a project of two files, its ForceCommand and its sshd. The
# homes live under $HOSTS, a short folder in the runtime directory: the remote server binds Unix
# sockets under ~/.local/share/marley/server_state, and a socket's path must stay under 108 bytes.
fake_host() {
  local n=$1 root=$HOSTS/$1 port
  local home=$root/home
  mkdir -p "$home/project" "$home/.run"
  chmod 700 "$home/.run"
  printf '# Host %s\n\nA remote project for #445.\n' "$n" >"$home/project/host-$n.md"
  printf 'fn main() {\n    println!("host %s");\n}\n' "$n" >"$home/project/host_$n.rs"
  cat >"$root/login" <<LOGIN
#!/bin/bash
# Host $n's login: every command runs in the fake home, as on a machine of its own.
home=$(printf '%q' "$home")
export HOME=\$home SHELL=/bin/bash XDG_RUNTIME_DIR=\$home/.run
export XDG_DATA_HOME=\$home/.local/share XDG_CONFIG_HOME=\$home/.config
export XDG_CACHE_HOME=\$home/.cache XDG_STATE_HOME=\$home/.local/state
cd "\$home" || exit 1
printf '%s\n' "\${SSH_ORIGINAL_COMMAND:-(a shell)}" >>$(printf '%q' "$root/commands.log")
case \${SSH_ORIGINAL_COMMAND:-} in
  */sftp-server* | internal-sftp*) exec /usr/lib/ssh/sftp-server -d "\$home" ;;
  "") exec /bin/bash -l ;;
esac
exec /bin/bash -c "\$SSH_ORIGINAL_COMMAND"
LOGIN
  chmod +x "$root/login"
  port=$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
  cat >"$root/sshd_config" <<EOF
ListenAddress 127.0.0.1
Port $port
HostKey $E2E_WORK/keys/host
AuthorizedKeysFile $E2E_WORK/keys/authorized_keys
PidFile $root/sshd.pid
UsePAM no
StrictModes no
PubkeyAuthentication yes
PasswordAuthentication no
KbdInteractiveAuthentication no
PermitRootLogin no
AllowTcpForwarding no
AllowStreamLocalForwarding no
X11Forwarding no
AllowAgentForwarding no
PermitTunnel no
Subsystem sftp /usr/lib/ssh/sftp-server
ForceCommand $root/login
EOF
  /usr/bin/sshd -D -e -f "$root/sshd_config" 2>"$root/sshd.log" &
  for _ in $(seq 50); do
    ss -ltnH "sport = :$port" | grep -q . && break
    sleep 0.1
  done
  echo "host $n: sshd pid $! on 127.0.0.1:$port, home $home"
  printf '%s\n' "$port" >"$root/port"
}

# The client's ssh, scp and sftp: the scenario's key and known-hosts file, nothing of ~/.ssh.
write_client_wrappers() {
  local program
  mkdir -p "$E2E_WORK/bin"
  for program in ssh scp sftp; do
    cat >"$E2E_WORK/bin/$program" <<WRAPPER
#!/bin/sh
printf '%s %s\n' $program "\$*" >>$(printf '%q' "$E2E_WORK/client-calls.log")
exec /usr/bin/$program -F /dev/null -i $(printf '%q' "$E2E_WORK/keys/client") -o IdentitiesOnly=yes \\
  -o IdentityAgent=none -o UserKnownHostsFile=$(printf '%q' "$E2E_WORK/keys/known_hosts") \\
  -o StrictHostKeyChecking=accept-new "\$@"
WRAPPER
    chmod +x "$E2E_WORK/bin/$program"
  done
}

setup() {
  if [[ -z ${E2E_REMOTE_SERVER:-} || ! -s $E2E_REMOTE_SERVER ]]; then
    echo "445 needs E2E_REMOTE_SERVER: the .gz script/install-marley --remote-server-only staged" >&2
    return 1
  fi
  COMMIT=$(basename "$E2E_REMOTE_SERVER" .gz)
  HOSTS=$(mktemp -d "$XDG_RUNTIME_DIR/m445.XXXXXX")
  echo "the staged server's commit: $COMMIT"
  user_listing >"$E2E_WORK/user-before.txt"
  mkdir -p "$E2E_WORK/keys" "$E2E_WORK/local"
  chmod 700 "$E2E_WORK/keys"
  ssh-keygen -q -t ed25519 -N '' -f "$E2E_WORK/keys/host"
  ssh-keygen -q -t ed25519 -N '' -f "$E2E_WORK/keys/client"
  cp "$E2E_WORK/keys/client.pub" "$E2E_WORK/keys/authorized_keys"
  chmod 600 "$E2E_WORK/keys/authorized_keys"
  fake_host one
  fake_host two
  # Host two already holds a server at Marley's name, which answers another commit.
  mkdir -p "$HOSTS/two/home/.zed_server"
  printf '#!/bin/sh\necho %s\n' "$STALE_COMMIT" >"$HOSTS/two/home/.zed_server/zed-remote-server-dev-$COMMIT"
  chmod 755 "$HOSTS/two/home/.zed_server/zed-remote-server-dev-$COMMIT"
  write_client_wrappers
  export PATH=$E2E_WORK/bin:$PATH
  export ZED_BUILD_REMOTE_SERVER=never
  profile_setting ssh_connections "[{\"host\": \"127.0.0.1\", \"port\": $(cat "$HOSTS/one/port"), \"nickname\": \"host-one\"}, {\"host\": \"127.0.0.1\", \"port\": $(cat "$HOSTS/two/port"), \"nickname\": \"host-two\"}]"
  printf '# Local\n' >"$E2E_WORK/local/README.md"
  open_path "$E2E_WORK/local"
}

# Stops each host's remote servers, which idle for minutes after the last client, and the sshds.
teardown() {
  local root pid
  # Before setup made the hosts' folder there is nothing to stop, and an empty $HOSTS would match
  # every path below.
  [[ -n ${HOSTS:-} && -d $HOSTS ]] || return 0
  # A server runs from its home with a relative path, so it is found by its working folder.
  for pid in $(pgrep -f 'zed-remote-server-dev-' || true); do
    [[ $(readlink "/proc/$pid/cwd" 2>/dev/null) == "$HOSTS"/* ]] && kill "$pid" 2>/dev/null
  done
  for root in "$HOSTS"/*; do
    [[ -f $root/sshd.pid ]] && kill "$(cat "$root/sshd.pid")" 2>/dev/null
    # The host's side of each connection goes beside the shots, since the run's folder does not
    # outlive the run.
    (cd "$root" && find commands.log sshd.log home/.local -name '*.log' -print0 2>/dev/null |
      tar --null -T - -czf "$(shot_file "host-$(basename "$root")-logs.tgz")")
  done
  rm -rf -- "$HOSTS"
  return 0
}

# `open_host <n>`: hands Marley the host's project as an ssh:// URL, as a second launch does.
open_host() {
  local root=$HOSTS/$1
  launch_marley_with "ssh://$USER@127.0.0.1:$(cat "$root/port")$root/home/project"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

server_on() { printf '%s\n' "$HOSTS/$1/home/.zed_server/zed-remote-server-dev-$COMMIT"; }
no_server_on() { [[ ! -e $(server_on "$1") ]]; }
answers_commit() { [[ $("$(server_on "$1")" version | tr -d '\r' | tail -n 1) == "$COMMIT" ]]; }
no_dev_build_on() { [[ ! -e $HOSTS/$1/home/.zed_server/zed-remote-server-dev-build ]]; }
user_unchanged() {
  user_listing >"$E2E_WORK/user-after.txt"
  cmp -s "$E2E_WORK/user-before.txt" "$E2E_WORK/user-after.txt"
}

steps() {
  settle 12
  # Trusts the local scratch folder.
  press "" Return
  settle 2

  echo "== host one, nothing staged"
  open_host one
  settle 15
  shot 445-01-not-staged
  expect "host one received no server" no_server_on one
  cat "$HOSTS/one/commands.log"

  echo "== host one, the server staged"
  press "" Escape
  settle 2
  mkdir -p "$E2E_PROFILE/remote_servers/dev/linux-x86_64"
  cp "$E2E_REMOTE_SERVER" "$E2E_PROFILE/remote_servers/dev/linux-x86_64/"
  open_host one
  settle 30
  palette "project panel: toggle focus"
  settle 3
  shot 445-02-opened
  ls -la "$HOSTS/one/home/.zed_server"
  expect "host one holds the server named for the commit" test -x "$(server_on one)"
  expect "its version answers the commit" answers_commit one
  expect "no -dev-build on host one" no_dev_build_on one

  echo "== host two, a server answering another commit"
  palette "workspace: close window"
  settle 3
  open_host two
  settle 30
  palette "project panel: toggle focus"
  settle 3
  shot 445-03-replaced
  ls -la "$HOSTS/two/home/.zed_server"
  expect "host two's server now answers Marley's commit" answers_commit two
  expect "the user's ~/.ssh, ~/.zed_server and server_state are unchanged" user_unchanged
}
