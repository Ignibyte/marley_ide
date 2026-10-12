# shellcheck shell=bash
# #758's visual check: a remote manager's thread. Two scratch roots of the harness's built `rh`
# each run a manager made from #688's fixture: a terminal that reports idle, is designated with
# `rh manager`, and posts a report through `rh mcp --grant agent`'s `thread_post` whenever the
# scenario drops a cue file in its root. The first root is `marley.harness`; the second is
# `marley.harnesses`' `box-2`, reached through #741's fake `ssh`, which logs its arguments and runs
# the words after `-- box-2` here. Any other destination goes to the real `ssh`, wrapped so it reads
# nothing of ~/.ssh, for the remote project of #445's fake host: an sshd whose ForceCommand runs in
# a fake home, with the server E2E_REMOTE_SERVER names staged for the debug build's commit. Marley
# runs on a private session bus whose notification server logs each banner (#688's).
# Shots: New Agent lists both Manager entries (REQ-001); box-2's report in Needs you and a banner
# (REQ-002); its entry opens a `Manager · box-2` tab (REQ-003); a message reaches box-2's thread only
# (REQ-004); a report with the tab in front raises nothing (REQ-005); the first harness's report and
# its `Manager` tab (REQ-003); the row menu's Open Manager Thread (REQ-006); the tab opened from a
# remote project's window, a third report streaming into it (REQ-008).
compositor sway

RH=${RH:-/srv/stacks/rustal-harness/bin/rh}
REAL_RH=/srv/stacks/rustal-harness/target/debug/rh
# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
ENTRY_X=${ENTRY_X:-120}
ENTRY_Y=${ENTRY_Y:-116}
FIRST_ENTRY_Y=${FIRST_ENTRY_Y:-116}
MANAGER_ROW_X=${MANAGER_ROW_X:-150}
MANAGER_ROW_Y=${MANAGER_ROW_Y:-428}
REMOTE_ROW_X=${REMOTE_ROW_X:-900}
REMOTE_ROW_Y=${REMOTE_ROW_Y:-290}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

# The harness's CLI on a scratch root, outside any tmux.
rh_on() {
  local root=$1
  shift
  env -u TMUX -u TMUX_PANE "$RH" --state "$root" "$@"
}

write_notification_server() {
  cat >"$E2E_WORK/notifications.py" <<'PY'
# A desktop notification server on the scenario's private bus: logs each banner and answers as a
# real server does.
import sys

import dbus
import dbus.mainloop.glib
import dbus.service
from gi.repository import GLib

LOG = sys.argv[1]
INTERFACE = "org.freedesktop.Notifications"
dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)


class Notifications(dbus.service.Object):
    def __init__(self, bus):
        super().__init__(bus, "/org/freedesktop/Notifications")
        self.last = 0

    @dbus.service.method(INTERFACE, in_signature="susssasa{sv}i", out_signature="u")
    def Notify(self, app, replaces, icon, summary, body, actions, hints, timeout):
        self.last += 1
        with open(LOG, "a", encoding="utf-8") as log:
            log.write(f"{app}|{summary}|{body}\n")
        return self.last

    @dbus.service.method(INTERFACE, in_signature="", out_signature="as")
    def GetCapabilities(self):
        return ["body", "actions"]

    @dbus.service.method(INTERFACE, in_signature="u", out_signature="")
    def CloseNotification(self, id):
        pass

    @dbus.service.method(INTERFACE, in_signature="", out_signature="ssss")
    def GetServerInformation(self):
        return ("e2e", "marley", "1", "1.2")


bus = dbus.SessionBus()
name = dbus.service.BusName(INTERFACE, bus)
server = Notifications(bus)
print("ready", flush=True)
GLib.MainLoop().run()
PY
}

write_post() {
  cat >"$E2E_WORK/post.py" <<'PY'
# Posts one record to the thread as the root's manager, through `rh mcp --grant agent` from inside
# the manager's harness terminal.
import json
import subprocess
import sys

rh, root, kind, text = sys.argv[1:5]
server = subprocess.Popen([rh, "--state", root, "mcp", "--grant", "agent"], stdin=subprocess.PIPE,
                          stdout=subprocess.PIPE, text=True)


def call(message):
    server.stdin.write(json.dumps(message) + "\n")
    server.stdin.flush()
    return json.loads(server.stdout.readline()) if "id" in message else None


call({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
    "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "fixture", "version": "0"}}})
call({"jsonrpc": "2.0", "method": "notifications/initialized"})
print(json.dumps(call({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
    "name": "thread_post", "arguments": {"kind": kind, "text": text}}})))
PY
}

# `start_root <name>`: a runtime on a short root (its sockets live under it) with a manager that
# posts "<name>'s report <cue>" for each cue file, `post1` to `post3`; prints the root.
start_root() {
  local name=$1 root tries=50 id
  root=$XDG_RUNTIME_DIR/rh758-$name-$$
  mkdir -m 0700 "$root"
  (rh_on "$root" serve >"$root/serve.log" 2>&1 &)
  until grep -q '"ready":true' "$root/serve.log" 2>/dev/null || ((tries-- == 0)); do
    sleep 0.2
  done
  cat >"$E2E_WORK/mgr-$name.sh" <<SH
#!/bin/bash
env -u TMUX "$REAL_RH" --state "$root" report --source fixture --seq 1 idle
for cue in post1 post2 post3; do
  while [[ ! -f "$root/\$cue" ]]; do sleep 0.3; done
  python3 "$E2E_WORK/post.py" "$REAL_RH" "$root" report "$name's report \$cue
the rest of the report" >>"$E2E_WORK/posted-$name.log" 2>&1
done
sleep 900
SH
  chmod +x "$E2E_WORK/mgr-$name.sh"
  rh_on "$root" new mgr --cwd "$root" -- "$E2E_WORK/mgr-$name.sh" >/dev/null
  tries=20
  until [[ -n ${id:-} ]] || ((tries-- == 0)); do
    id=$(rh_on "$root" fleet | python3 -c 'import json, sys
for seat in json.load(sys.stdin).get("seats", []):
    if seat["id"].startswith("agent/"):
        print(seat["id"])')
    [[ -n $id ]] || sleep 0.5
  done
  rh_on "$root" manager "$id" >/dev/null
  printf '%s\n' "$root"
}

# #445's fake host: an sshd on loopback whose ForceCommand runs every command, sftp's server
# included, in a fake home under a short folder (the remote server's sockets live there).
fake_host() {
  local home=$HOSTS/home port
  mkdir -p "$home/project" "$home/.run"
  chmod 700 "$home/.run"
  printf '# On the fake host\n' >"$home/project/remote-notes.md"
  cat >"$HOSTS/login" <<LOGIN
#!/bin/bash
home=$(printf '%q' "$home")
export HOME=\$home SHELL=/bin/bash XDG_RUNTIME_DIR=\$home/.run
export XDG_DATA_HOME=\$home/.local/share XDG_CONFIG_HOME=\$home/.config
export XDG_CACHE_HOME=\$home/.cache XDG_STATE_HOME=\$home/.local/state
cd "\$home" || exit 1
case \${SSH_ORIGINAL_COMMAND:-} in
  */sftp-server* | internal-sftp*) exec /usr/lib/ssh/sftp-server -d "\$home" ;;
  "") exec /bin/bash -l ;;
esac
exec /bin/bash -c "\$SSH_ORIGINAL_COMMAND"
LOGIN
  chmod +x "$HOSTS/login"
  port=$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
  cat >"$HOSTS/sshd_config" <<EOF
ListenAddress 127.0.0.1
Port $port
HostKey $E2E_WORK/keys/host
AuthorizedKeysFile $E2E_WORK/keys/authorized_keys
PidFile $HOSTS/sshd.pid
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
ForceCommand $HOSTS/login
EOF
  /usr/bin/sshd -D -e -f "$HOSTS/sshd_config" 2>"$HOSTS/sshd.log" &
  for _ in $(seq 50); do
    ss -ltnH "sport = :$port" | grep -q . && break
    sleep 0.1
  done
  printf '%s\n' "$port" >"$HOSTS/port"
}

# The client's programs: #741's fake `ssh` for box-2, which hands any other destination to the real
# one; the real `ssh`, `scp` and `sftp` with the scenario's key, reading nothing of ~/.ssh.
write_client_wrappers() {
  local bin=$E2E_WORK/bin program real
  mkdir -p "$bin"
  for program in ssh scp sftp; do
    real=$bin/$program
    [[ $program == ssh ]] && real=$bin/ssh-real
    cat >"$real" <<WRAPPER
#!/bin/sh
exec /usr/bin/$program -F /dev/null -i $(printf '%q' "$E2E_WORK/keys/client") -o IdentitiesOnly=yes \\
  -o IdentityAgent=none -o UserKnownHostsFile=$(printf '%q' "$E2E_WORK/keys/known_hosts") \\
  -o StrictHostKeyChecking=accept-new "\$@"
WRAPPER
    chmod +x "$real"
  done
  cat >"$bin/ssh" <<SH
#!/bin/bash
# box-2 is here: everything up to \`-- box-2\` goes, and the rest runs in a shell. Any other
# destination is a real host.
case " \$* " in
  *" -- box-2 "*) ;;
  *) exec $(printf '%q' "$bin/ssh-real") "\$@" ;;
esac
printf '%s\n' "\$*" >>$(printf '%q' "$E2E_WORK/ssh.log")
while [[ \$# -gt 0 && \$1 != -- ]]; do shift; done
shift
shift
exec bash -c "\$*"
SH
  chmod +x "$bin/ssh"
  : >"$E2E_WORK/ssh.log"
}

setup() {
  [[ -x $REAL_RH ]] || {
    echo "no built rh: build the harness first (its scripts/setup.sh)" >&2
    return 1
  }
  if [[ -z ${E2E_REMOTE_SERVER:-} || ! -s $E2E_REMOTE_SERVER ]]; then
    echo "758 needs E2E_REMOTE_SERVER: the .gz script/install-marley --remote-server-only staged" >&2
    return 1
  fi
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  write_notification_server
  python3 -u "$E2E_WORK/notifications.py" "$E2E_WORK/banners.log" >"$E2E_WORK/notifications.out" 2>&1 &
  echo "$!" >"$E2E_WORK/notifications.pid"
  : >"$E2E_WORK/banners.log"
  write_post
  FIRST=$(start_root first)
  BOX2=$(start_root box2)
  printf '%s\n%s\n' "$FIRST" "$BOX2" >"$E2E_WORK/roots"
  mkdir -p "$E2E_WORK/keys"
  chmod 700 "$E2E_WORK/keys"
  ssh-keygen -q -t ed25519 -N '' -f "$E2E_WORK/keys/host"
  ssh-keygen -q -t ed25519 -N '' -f "$E2E_WORK/keys/client"
  cp "$E2E_WORK/keys/client.pub" "$E2E_WORK/keys/authorized_keys"
  chmod 600 "$E2E_WORK/keys/authorized_keys"
  HOSTS=$(mktemp -d "$XDG_RUNTIME_DIR/m758.XXXXXX")
  fake_host
  write_client_wrappers
  export PATH=$E2E_WORK/bin:$PATH
  export ZED_BUILD_REMOTE_SERVER=never
  mkdir -p "$E2E_PROFILE/remote_servers/dev/linux-x86_64"
  cp "$E2E_REMOTE_SERVER" "$E2E_PROFILE/remote_servers/dev/linux-x86_64/"
  profile_setting marley.harness "{\"command\": \"$RH\", \"args\": [\"--state\", \"$FIRST\", \"mcp\", \"--grant\", \"write\"]}"
  profile_setting marley.harnesses "[{\"name\": \"box-2\", \"ssh\": \"box-2\", \"state\": \"$BOX2\", \"rh\": \"$REAL_RH\"}]"
  profile_setting marley.harness_writes true
  mkdir -p "$E2E_WORK/repo" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

teardown() {
  local root ws pid
  if [[ -f $E2E_WORK/roots ]]; then
    while read -r root; do
      chmod 0700 "$root" 2>/dev/null
      for ws in $(rh_on "$root" list 2>/dev/null | python3 -c 'import json, sys
print(" ".join(w["id"] for w in json.load(sys.stdin).get("workspaces", []) if w.get("state") == "running"))'); do
        rh_on "$root" stop "$ws" >/dev/null 2>&1
      done
      rh_on "$root" shutdown --stop-backend >/dev/null 2>&1
      rm -rf -- "$root"
    done <"$E2E_WORK/roots"
  fi
  if [[ -n ${HOSTS:-} && -d $HOSTS ]]; then
    for pid in $(pgrep -f 'zed-remote-server-dev-' || true); do
      [[ $(readlink "/proc/$pid/cwd" 2>/dev/null) == "$HOSTS"/* ]] && kill "$pid" 2>/dev/null
    done
    [[ -f $HOSTS/sshd.pid ]] && kill "$(cat "$HOSTS/sshd.pid")" 2>/dev/null
    rm -rf -- "$HOSTS"
  fi
  [[ -f $E2E_WORK/notifications.pid ]] && kill "$(cat "$E2E_WORK/notifications.pid")" 2>/dev/null
  [[ -f $E2E_WORK/bus.pid ]] && kill "$(head -1 "$E2E_WORK/bus.pid")" 2>/dev/null
  return 0
}

# `banners <text>`: how many banners hold the text.
banners() { grep -c -- "$1" "$E2E_WORK/banners.log" || true; }

# `person_said <root> <text>`: whether the root's thread holds the person's message.
person_said() {
  rh_on "$1" thread show | python3 -c 'import json, sys
records = json.load(sys.stdin).get("records", [])
sys.exit(0 if any(r.get("author") == "person" and r.get("text") == sys.argv[1] for r in records) else 1)' "$2"
}
person_never_said() { ! person_said "$@"; }

# `manager_posted <root> <text>`: whether the root's manager posted a record holding the text.
manager_posted() {
  rh_on "$1" thread show | python3 -c 'import json, sys
records = json.load(sys.stdin).get("records", [])
sys.exit(0 if any(r.get("author") == "manager" and sys.argv[1] in r.get("text", "") for r in records) else 1)' "$2"
}

steps() {
  settle 16
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== New Agent lists both managers"
  press "CTRL ALT" n
  settle 2
  type_text "Manager"
  settle 2
  take 758-01-entries || return 0
  press "" Escape
  settle 1

  echo "== box-2's manager reports"
  touch "$BOX2/post1"
  settle 6
  cat "$E2E_WORK/posted-box2.log"
  pointer_to 900 400
  settle 1
  take 758-02-needs-you || return 0
  cat "$E2E_WORK/banners.log"
  expect "the banner names box-2's manager" holds "$E2E_WORK/banners.log" \
    "Manager · box-2: Report" "box2's report post1"

  echo "== its entry opens box-2's Manager tab"
  click "$ENTRY_X" "$ENTRY_Y"
  settle 8
  take 758-03-tab || return 0
  cat "$E2E_WORK/ssh.log"
  expect "box-2's window runs over ssh" grep -q -- "-- box-2 .*acp" "$E2E_WORK/ssh.log"

  echo "== a message reaches box-2's thread only"
  type_text "hello manager"
  press "" Return
  settle 6
  take 758-04-sent || return 0
  expect "box-2's thread holds the message" person_said "$BOX2" "hello manager"
  expect "the first root's thread does not" person_never_said "$FIRST" "hello manager"

  echo "== a report with the tab in front"
  touch "$BOX2/post2"
  settle 6
  take 758-05-in-front || return 0
  expect "no second banner for box-2" [ "$(banners 'Manager · box-2')" = 1 ]

  echo "== the first harness's manager reports"
  click 600 500
  settle 1
  touch "$FIRST/post1"
  settle 6
  cat "$E2E_WORK/banners.log"
  expect "its banner reads Manager: Report" holds "$E2E_WORK/banners.log" \
    "Marley|Manager: Report|first's report post1"
  click "$ENTRY_X" "$FIRST_ENTRY_Y"
  settle 8
  take 758-06-first || return 0

  echo "== Open Manager Thread on box-2's row"
  if ((MANAGER_ROW_Y == 0)); then
    echo "box-2's manager row is not placed yet: 758-06 is the last shot"
    return 0
  fi
  click "$MANAGER_ROW_X" "$MANAGER_ROW_Y" right
  # Before the row's tooltip, which would cover the menu.
  sleep 0.3
  take 758-07a-menu || return 0
  press "" Home
  press "" Return
  settle 3
  take 758-07-row-menu || return 0
  # Its window's tab closes, so the remote window opens the thread afresh.
  press "CTRL" w
  settle 2

  echo "== from a remote project's window"
  launch_marley_with "ssh://$USER@127.0.0.1:$(cat "$HOSTS/port")$HOSTS/home/project"
  settle 30
  if ((REMOTE_ROW_Y == 0)); then
    shot 758-08a-remote
    echo "the remote window's manager row is not placed yet"
    return 0
  fi
  click "$REMOTE_ROW_X" "$REMOTE_ROW_Y" right
  settle 1
  press "" Home
  press "" Return
  settle 8
  # A message would wait behind step 4's turn, which the fixture manager never ends (rh acp ends a
  # manager's turn once the fleet shows it busy, then idle), so a third report shows the tab live.
  touch "$BOX2/post3"
  settle 6
  take 758-08-remote-window || return 0
  expect "box-2's third report reached its thread" manager_posted "$BOX2" "box2's report post3"
  expect "with the tab in front there, still one banner for box-2" \
    [ "$(banners 'Manager · box-2')" = 1 ]
}
