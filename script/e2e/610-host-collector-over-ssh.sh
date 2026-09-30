# shellcheck shell=bash
# #610's e2e test: the host collector. An sshd of the scenario's own on 127.0.0.1 stands in for a
# host (L-claude-584), and MARLEY_SSH points Marley at an ssh that logs in there with the
# scenario's key. Two stand-in agents run on this machine, `sleep` under the names `claude` and
# `codex`; the `claude` one carries MARLEY_FLEET_SESSION=agent-7f3a, the pseudo provider's
# build-1. The settings list four hosts: the sshd as `lab` with the store's id `host-build-1`,
# this machine, `gone` (a closed port) with the id `host-vps-2`, and `evil`, whose destination is
# an ssh option.
#
# `hosts` and `hosts-more` show the pseudo provider's hosts, lab's resources on build-1's host,
# vps-2 unreachable with docs-1 offline, and the Hosts section: lab and this machine with their
# resources and the `codex` stand-in running, `gone` unreachable and `evil` refused (REQ-001,
# REQ-003, REQ-004). `refused.txt` holds Marley's log line refusing `evil`, and says whether the
# file its ProxyCommand would make exists (REQ-005). A click on the `codex` stand-in under lab shows
# its snapshot with lab's bars (`snapshot`, REQ-002), one on build-1 its bars from lab and the
# `claude` stand-in as its process (`joined`, REQ-002), and the pointer on `gone`'s chip the SSH
# error (`tooltip`, REQ-003).
compositor sway

# Where the panel draws what the clicks need, on the 1600 x 1000 output: build-1's row, `gone`'s
# chip, and lab's first process row in the Hosts section, each row after it 47 px lower.
ROW_X=1400
BUILD_Y=153
GONE_X=1378
GONE_Y=366
LAB_FIRST_ROW_Y=472
ROW_STEP=47

SSHD_PID=""
STANDIN_PIDS=()

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home" "$E2E_WORK/repo" "$E2E_WORK/bin" "$E2E_WORK/standin-claude" \
    "$E2E_WORK/standin-codex"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  start_sshd
  start_standins
  python3 - "$E2E_PROFILE/config/settings.json" "$SSHD_PORT" "$E2E_WORK/pwned" <<'PY'
import re, sys
path, port, pwned = sys.argv[1], sys.argv[2], sys.argv[3]
entry = (
    '"fleet": { "providers": [ { "kind": "pseudo" } ], "hosts": ['
    f'{{ "ssh": "127.0.0.1:{port}", "name": "lab", "id": "host-build-1" }}, '
    '{ "local": true, "name": "this machine" }, '
    '{ "ssh": "127.0.0.1:1", "name": "gone", "id": "host-vps-2" }, '
    f'{{ "ssh": "-oProxyCommand=touch {pwned}", "name": "evil" }}'
    '] },'
)
text = open(path).read()
match = re.search(r'"marley"\s*:\s*\{', text)
if match:
    text = text[:match.end()] + "\n    " + entry + text[match.end():]
else:
    at = text.index("{") + 1
    text = text[:at] + '\n  "marley": {' + entry + '},' + text[at:]
open(path, "w").write(text)
PY
  open_path "$E2E_WORK/repo"
}

teardown() {
  local pid
  for pid in "${STANDIN_PIDS[@]}"; do
    kill "$pid" 2>/dev/null || true
  done
  if [[ -n $SSHD_PID ]]; then
    kill "$SSHD_PID" 2>/dev/null || true
  fi
}

# An sshd of the scenario's own on 127.0.0.1, with its own host key and the one key it lets in,
# and the `ssh` Marley runs through MARLEY_SSH, which logs in with that key and trusts that host.
start_sshd() {
  local dir=$E2E_WORK/ssh
  mkdir -p "$dir"
  chmod 700 "$dir"
  ssh-keygen -q -t ed25519 -N '' -f "$dir/host"
  ssh-keygen -q -t ed25519 -N '' -f "$dir/client"
  cp "$dir/client.pub" "$dir/authorized_keys"
  chmod 600 "$dir/authorized_keys"
  SSHD_PORT=$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
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
EOF
  /usr/bin/sshd -D -e -f "$dir/sshd_config" 2>"$dir/sshd.log" &
  SSHD_PID=$!
  for _ in $(seq 50); do
    ss -ltnH "sport = :$SSHD_PORT" | grep -q . && break
    sleep 0.1
  done
  echo "the scenario's sshd: pid $SSHD_PID on 127.0.0.1:$SSHD_PORT"
  cat >"$E2E_WORK/bin/marley-ssh" <<EOF
#!/bin/sh
exec ssh -F /dev/null -i $dir/client -o IdentitiesOnly=yes -o IdentityAgent=none \\
  -o UserKnownHostsFile=$dir/known_hosts -o StrictHostKeyChecking=accept-new -o LogLevel=ERROR "\$@"
EOF
  chmod +x "$E2E_WORK/bin/marley-ssh"
  export MARLEY_SSH=$E2E_WORK/bin/marley-ssh
}

# `sleep` under the names the collector knows, each in a folder of its own.
start_standins() {
  local sleep
  sleep=$(command -v sleep)
  ln -s "$sleep" "$E2E_WORK/bin/claude"
  ln -s "$sleep" "$E2E_WORK/bin/codex"
  (cd "$E2E_WORK/standin-claude" && MARLEY_FLEET_SESSION=agent-7f3a exec "$E2E_WORK/bin/claude" 900) &
  STANDIN_PIDS+=($!)
  (cd "$E2E_WORK/standin-codex" && exec "$E2E_WORK/bin/codex" 900) &
  STANDIN_PIDS+=($!)
  echo "stand-ins: ${STANDIN_PIDS[*]}"
}

# The `codex` stand-in's place among lab's process rows: the collector lists `claude` and `codex`
# processes in /proc's order, and the `claude` stand-in is build-1's, not a row of its own.
standin_row() {
  local index=0 path comm pid
  for path in /proc/[0-9]*/comm; do
    comm=$(cat "$path" 2>/dev/null) || continue
    case $comm in
      claude | codex) ;;
      *) continue ;;
    esac
    pid=${path#/proc/}
    pid=${pid%/comm}
    [[ $pid == "${STANDIN_PIDS[0]}" ]] && continue
    if [[ $pid == "${STANDIN_PIDS[1]}" ]]; then
      echo "$index"
      return
    fi
    index=$((index + 1))
  done
  echo 0
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  press "CTRL SHIFT" p
  settle 1
  type_text "marley: toggle fleet"
  settle 1
  press "" Return

  echo "== two collections in"
  settle 14
  shot hosts
  pointer_to 1440 600
  scroll 30
  settle 2
  shot hosts-more
  scroll -30
  settle 2

  echo "== the codex stand-in, unclaimed, under lab"
  local row
  row=$(LC_ALL=C standin_row)
  echo "the stand-in is row $row of lab's"
  click "$ROW_X" "$((LAB_FIRST_ROW_Y + row * ROW_STEP))"
  settle 3
  shot snapshot

  echo "== build-1, joined to the claude stand-in on lab"
  click "$ROW_X" "$BUILD_Y"
  settle 3
  shot joined

  echo "== gone's reason"
  pointer_to "$GONE_X" "$GONE_Y"
  settle 2
  shot tooltip

  echo "== the refused host"
  {
    grep -h 'refused' "$E2E_PROFILE/logs/Marley.log" || echo "no refusal logged"
    if [[ -e $E2E_WORK/pwned ]]; then
      echo "the ProxyCommand's file exists: it ran"
    else
      echo "the ProxyCommand's file does not exist: nothing ran"
    fi
  } | tee "$(shot_file refused.txt)"
}
