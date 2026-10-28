# shellcheck shell=bash
# #944's visual check: remote folders, the layouts and About as the boards draw them. #445's two
# hosts faked on loopback with their wrappers, the sftp wrapper waiting five seconds so the upload
# line stays up; both hosts saved; the server `script/install-marley` staged for this commit. A
# scratch folder opens first.
# - `projects: open remote`: "ssh://user@host/path", each host with its dot, "Add a host…"
#   (`944-01-remote`, REQ-001, REQ-003).
# - Host two's address typed: "Open <path> on 127.0.0.1" first (`944-02-address`, REQ-002).
# - Enter: "Uploading Marley's server for this build · N MB" (`944-03-upload`, REQ-004).
# - The dialog again once host two is connected: its dot in the success colour (`944-04-connected`,
#   REQ-003).
# - Settings, the Marley page: the two layout cards, Marley outlined (`944-05-layouts`, REQ-005).
# - `zed: about`: the mark, "Marley", the build line, the cat line, the update line and the two
#   buttons (`944-06-about`, REQ-006); Enter on Copy the command puts `git pull && just install` on
#   the clipboard (REQ-007); Open the log opens Marley's log in a tab (`944-07-log`, REQ-008).
# - Marley Day: About and the layout cards (`944-08-day`, `944-08-day-layouts`).
compositor sway

AWAY_X=${AWAY_X:-1500}
AWAY_Y=${AWAY_Y:-950}
STOP_AT=${STOP_AT:-0}
STAGED=${E2E_REMOTE_SERVER:-$HOME/.local/share/marley/remote_servers/dev/linux-x86_64/$(git rev-parse HEAD).gz}

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
# The upload waits, so its status line stays up for a shot (#944).
[ $program = sftp ] && sleep 5
exec /usr/bin/$program -F /dev/null -i $(printf '%q' "$E2E_WORK/keys/client") -o IdentitiesOnly=yes \\
  -o IdentityAgent=none -o UserKnownHostsFile=$(printf '%q' "$E2E_WORK/keys/known_hosts") \\
  -o StrictHostKeyChecking=accept-new "\$@"
WRAPPER
    chmod +x "$E2E_WORK/bin/$program"
  done
}

setup() {
  if [[ ! -s $STAGED ]]; then
    echo "944 needs the server script/install-marley staged for $(git rev-parse HEAD)" >&2
    return 1
  fi
  HOSTS=$(mktemp -d "$XDG_RUNTIME_DIR/m944.XXXXXX")
  mkdir -p "$E2E_WORK/keys" "$E2E_WORK/local" "$E2E_PROFILE/remote_servers/dev/linux-x86_64"
  chmod 700 "$E2E_WORK/keys"
  ssh-keygen -q -t ed25519 -N '' -f "$E2E_WORK/keys/host"
  ssh-keygen -q -t ed25519 -N '' -f "$E2E_WORK/keys/client"
  cp "$E2E_WORK/keys/client.pub" "$E2E_WORK/keys/authorized_keys"
  chmod 600 "$E2E_WORK/keys/authorized_keys"
  fake_host one
  fake_host two
  cp "$STAGED" "$E2E_PROFILE/remote_servers/dev/linux-x86_64/"
  write_client_wrappers
  export PATH=$E2E_WORK/bin:$PATH
  export ZED_BUILD_REMOTE_SERVER=never
  profile_setting ssh_connections "[{\"host\": \"127.0.0.1\", \"username\": \"$USER\", \"port\": $(cat "$HOSTS/one/port"), \"nickname\": \"host-one\"}, {\"host\": \"127.0.0.1\", \"username\": \"$USER\", \"port\": $(cat "$HOSTS/two/port"), \"nickname\": \"host-two\"}]"
  profile_setting marley.tmux_hosts '[]'
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

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 12
  # Trusts the local scratch folder.
  press "" Return
  settle 3

  echo "== the remote dialog"
  palette "projects: open remote" 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 944-01-remote
  type_text "ssh://$USER@127.0.0.1:$(cat "$HOSTS/two/port")$HOSTS/two/home/project"
  settle 2
  shot 944-02-address
  press "" Return
  settle 4
  shot 944-03-upload
  settle 30
  if [[ $STOP_AT == 1 ]]; then
    return
  fi
  palette "projects: open remote" 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 944-04-connected
  press "" Escape
  settle 1

  echo "== the layout cards"
  palette "marley: open settings" 5
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 944-05-layouts
  # Marley Day at the same window width; later the windows tile narrower.
  profile_setting theme '"Marley Day"'
  settle 4
  shot 944-08-day-layouts
  profile_setting theme '"Marley Night"'
  settle 3
  press "CTRL" w
  settle 2

  echo "== About"
  palette "zed: about" 4
  shot 944-06-about
  press "" Return
  settle 2
  echo "the clipboard after Copy the command: $(WAYLAND_DISPLAY=$SWAY_DISPLAY wl-paste -n 2>/dev/null)"
  expect "Copy the command copied it" bash -c "[[ \$(WAYLAND_DISPLAY=$SWAY_DISPLAY wl-paste -n) == 'git pull && just install' ]]"
  palette "zed: about" 4
  press "" Tab
  settle 1
  press "" Return
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 944-07-log

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  palette "zed: about" 4
  shot 944-08-day
  press "" Escape
  settle 2
}
