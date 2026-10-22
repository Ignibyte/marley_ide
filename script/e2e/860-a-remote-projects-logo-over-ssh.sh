# shellcheck shell=bash
# #860's visual check: a remote project's logo, read once over SSH and kept. #445's fake host (an
# sshd of the scenario's own whose ForceCommand runs in a fake home, reached through `ssh`, `scp`
# and `sftp` wrappers that read nothing of ~/.ssh) holds a project with a green `logo.png`.
# E2E_REMOTE_SERVER names the server `script/install-marley` staged for the build's commit.
# - `860-01-remote`: the remote project open: the green logo as its mark in the rail (REQ-001).
# - `kept.txt`: the copy kept in the data folder.
# Then the local window closes and `local` opens beside the remote project from Home's Recent
# Projects, so the window that holds the remote project lists it after a restart.
# - `860-02-down`: the host's sshd and server stopped, Marley started again with its last session:
#   the remote project cannot connect, and its rail header still draws the logo from the kept
#   copy (REQ-002).
compositor sway

# A first run that stops after the first shot.
STOP_AT=${STOP_AT:-}
# From the probe run's shots: `local` in the remote window's Recent list, and a point in the local
# window.
RECENT_X=${RECENT_X:-333}
RECENT_Y=${RECENT_Y:-306}
LOCAL_X=${LOCAL_X:-500}
LOCAL_Y=${LOCAL_Y:-500}

# A 32 x 32 green PNG at `$1`, written without an imaging library.
write_logo() {
  python3 - "$1" <<'PNG'
import struct, sys, zlib

def chunk(kind, data):
    body = kind + data
    return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xffffffff)

side = 32
rows = b"".join(b"\x00" + bytes([0x9b, 0xe1, 0x5d]) * side for _ in range(side))
png = (b"\x89PNG\r\n\x1a\n"
       + chunk(b"IHDR", struct.pack(">IIBBBBB", side, side, 8, 2, 0, 0, 0))
       + chunk(b"IDAT", zlib.compress(rows))
       + chunk(b"IEND", b""))
open(sys.argv[1], "wb").write(png)
PNG
}

# `fake_host <n>`: host n's home with a project holding `logo.png`, its ForceCommand and its sshd
# (#445's). The homes live under $HOSTS, a short folder in the runtime directory: the remote server
# binds Unix sockets under ~/.local/share/marley/server_state, and a socket's path must stay under
# 108 bytes.
fake_host() {
  local n=$1 root=$HOSTS/$1 port
  local home=$root/home
  mkdir -p "$home/project" "$home/.run"
  chmod 700 "$home/.run"
  printf '# Host %s\n\nA remote project for #860.\n' "$n" >"$home/project/README.md"
  write_logo "$home/project/logo.png"
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
    echo "860 needs E2E_REMOTE_SERVER: the .gz script/install-marley staged for the commit" >&2
    return 1
  fi
  HOSTS=$(mktemp -d "$XDG_RUNTIME_DIR/m860.XXXXXX")
  mkdir -p "$E2E_WORK/keys" "$E2E_WORK/local" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  chmod 700 "$E2E_WORK/keys"
  ssh-keygen -q -t ed25519 -N '' -f "$E2E_WORK/keys/host"
  ssh-keygen -q -t ed25519 -N '' -f "$E2E_WORK/keys/client"
  cp "$E2E_WORK/keys/client.pub" "$E2E_WORK/keys/authorized_keys"
  chmod 600 "$E2E_WORK/keys/authorized_keys"
  fake_host one
  write_client_wrappers
  export PATH=$E2E_WORK/bin:$PATH
  export ZED_BUILD_REMOTE_SERVER=never
  mkdir -p "$E2E_PROFILE/remote_servers/dev/linux-x86_64"
  cp "$E2E_REMOTE_SERVER" "$E2E_PROFILE/remote_servers/dev/linux-x86_64/"
  profile_setting ssh_connections "[{\"host\": \"127.0.0.1\", \"port\": $(cat "$HOSTS/one/port"), \"nickname\": \"host-one\"}]"
  printf '# Local\n' >"$E2E_WORK/local/README.md"
  open_path "$E2E_WORK/local"
}

# Stops the host's remote servers, which idle for minutes after the last client, and its sshd.
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

# Stops host `$1`'s servers and sshd, as a host that went away.
host_down() {
  local root=$HOSTS/$1 pid
  for pid in $(pgrep -f 'zed-remote-server-dev-' || true); do
    [[ $(readlink "/proc/$pid/cwd" 2>/dev/null) == "$root"/* ]] && kill "$pid" 2>/dev/null
  done
  [[ -f $root/sshd.pid ]] && kill "$(cat "$root/sshd.pid")" 2>/dev/null
  return 0
}

steps() {
  settle 12
  # Trusts the local scratch folder.
  press "" Return
  settle 2

  echo "== the remote project"
  launch_marley_with "ssh://$USER@127.0.0.1:$(cat "$HOSTS/one/port")$HOSTS/one/home/project"
  settle 30
  press "" Return
  settle 5
  shot 860-01-remote
  find "$E2E_PROFILE" -path '*project_logos*' -type f -printf '%p %s bytes\n' | tee "$(shot_file kept.txt)"
  [[ $STOP_AT == 1 ]] && return 0

  echo "== the local window closed; local opened beside the remote project, from its Home"
  click "$LOCAL_X" "$LOCAL_Y"
  settle 1
  press "CTRL SHIFT" p
  settle 1
  type_text "workspace: close window"
  settle 1
  press "" Return
  settle 4
  launch_marley_with home
  settle 3
  click "$RECENT_X" "$RECENT_Y"
  settle 8

  echo "== the host down, Marley started again with its last session"
  quit_marley
  host_down one
  settle 2
  launch_marley_with
  settle 25
  shot 860-02-down
}
