# shellcheck shell=bash
# #584's e2e test: a client on another machine runs Marley's bridge on this one over SSH. The user
# allows `laptop` to act in Browser Clients, which shows the line to run over SSH and copies it
# with no token in it (REQ-001). An sshd of the scenario's own on 127.0.0.1 stands in for the
# machine's: a stand-in client runs the copied command through it, lists laptop's eighteen tools,
# reads the tabs and navigates, and the tab names laptop (REQ-002). The SSH session's output is
# JSON-RPC alone (REQ-003). A session held open across a restart of Marley is told Marley is not
# running while it is down and gets Marley's answers after, with nothing changed on its side
# (REQ-004), while Marley listens on nothing new (REQ-006). After Cut Off, the client over SSH and
# a local bridge on laptop's own file are told Marley does not allow the client, never that it is
# not running, and again after a restart (REQ-005). Nothing of ~/.ssh is read or written; tokens
# are never printed. Chromium runs offline.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# In Browser Clients, in the window's logical pixels (from the first run's shots): the checkbox
# while no client is listed, the SSH line's Copy button once laptop is allowed, and laptop's Cut
# Off while it is the one client listed. The tab's own Cut Off moves with the last action's text.
CHECKBOX_X=902
FORM_Y=219
COPY_SSH_X=1056
COPY_SSH_Y=428
ROW_CUT_OFF_X=1048
ROW_CUT_OFF_Y=198

setup() {
  offline_chromium
  mkdir -p "$E2E_WORK/site"
  cat >"$E2E_WORK/site/index.html" <<'HTML'
<!doctype html><html><head><title>Over SSH</title></head>
<body style="margin:0;font:32px sans-serif;background:#eef1f7"><h1 style="margin:40px">A page a client on another machine reads</h1></body></html>
HTML
  cat >"$E2E_WORK/site/next.html" <<'HTML'
<!doctype html><html><head><title>The next page</title></head>
<body style="margin:0;font:32px sans-serif;background:#f7f1ee"><h1 style="margin:40px">Where the client over SSH went</h1></body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
  write_mcp_agent
  write_hold_client
  start_sshd
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  if [[ -n ${SSHD_PID:-} ]]; then
    kill "$SSHD_PID" 2>/dev/null || true
  fi
  browser_teardown
}

# An sshd of the scenario's own on 127.0.0.1, with its own host key and the one key it lets in,
# and `over-ssh`, which runs the command the modal copied through it as the copied line does.
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
  cat >"$E2E_WORK/over-ssh" <<EOF
#!/bin/sh
exec ssh -F /dev/null -p $SSHD_PORT -i $dir/client -o IdentitiesOnly=yes -o IdentityAgent=none \\
  -o UserKnownHostsFile=$dir/known_hosts -o StrictHostKeyChecking=accept-new -o BatchMode=yes \\
  -o LogLevel=ERROR -T 127.0.0.1 "\$(cat $E2E_WORK/remote-command)"
EOF
  chmod +x "$E2E_WORK/over-ssh"
}

# One MCP session over SSH, held open: it lists the tools, then makes the call each line written
# to its FIFO names (`tabs`, `quit`), printing the answers and any notice the bridge sends.
write_hold_client() {
  cat >"$E2E_WORK/hold-client.py" <<'PY'
import json, os, queue, subprocess, sys, threading, time

bridge = subprocess.Popen([os.environ["BRIDGE"]], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                          stderr=subprocess.DEVNULL, text=True)
answers = queue.Queue()

def pump():
    for line in bridge.stdout:
        if not line.strip():
            continue
        try:
            message = json.loads(line)
        except ValueError:
            print(f"  not JSON-RPC: {line.strip()[:80]}", flush=True)
            continue
        if "id" in message:
            answers.put(message)
        else:
            print(f"  notice: {message.get('method')}", flush=True)
    answers.put(None)

threading.Thread(target=pump, daemon=True).start()
next_id = 0

def call(method, params=None):
    global next_id
    next_id += 1
    bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "id": next_id, "method": method, "params": params or {}}) + "\n")
    bridge.stdin.flush()
    deadline = time.monotonic() + 60
    while (left := deadline - time.monotonic()) > 0:
        try:
            message = answers.get(timeout=left)
        except queue.Empty:
            break
        if message is None:
            sys.exit(f"  {method}: the session closed")
        if message.get("id") == next_id:
            return message
    sys.exit(f"  {method}: no answer")

call("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "hold", "version": "0"}})
bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
bridge.stdin.flush()
tools = call("tools/list")["result"]["tools"]
print(f"  the held session lists {len(tools)} tools", flush=True)
while True:
    with open(sys.argv[1]) as fifo:
        for line in fifo:
            word = line.strip()
            if word == "quit":
                bridge.stdin.close()
                bridge.wait(timeout=10)
                print("  the held session ended", flush=True)
                sys.exit(0)
            result = call("tools/call", {"name": "browser_tabs", "arguments": {}}).get("result") or {}
            if result.get("isError"):
                print(f"  tabs: {result['content'][0]['text']}", flush=True)
            else:
                titles = [tab["title"] for tab in result.get("structuredContent", {}).get("tabs", [])]
                print(f"  tabs: {titles}", flush=True)
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Keeps `command`'s output in `$E2E_WORK/out.txt` as the log shows it.
out() {
  "$@" | tee "$E2E_WORK/out.txt"
}

# The fixture's stand-in agent, run through the copied command over the scenario's sshd.
remote() {
  BRIDGE=$E2E_WORK/over-ssh python3 "$E2E_WORK/mcp-agent.py" "$@"
}

held() {
  echo "$1" >"$E2E_WORK/hold.fifo"
  settle "${2:-3}"
}

steps() {
  local laptop=$E2E_PROFILE/mcp/clients/laptop.json registry=$E2E_PROFILE/mcp/clients.json
  settle 12
  # Trusts the repository.
  press "" Return
  settle 2

  echo "== Browser Clients: laptop allowed to act, and the line to run over SSH"
  palette "marley: browser clients"
  settle 2
  type_text "laptop"
  click "$CHECKBOX_X" "$FORM_Y"
  settle 1
  press "" Return
  settle 2
  shot 584-01-allowed
  click "$COPY_SSH_X" "$COPY_SSH_Y"
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wl-paste --no-newline >"$E2E_WORK/copied.txt" || true
  echo "  copied: $(cat "$E2E_WORK/copied.txt")"
  python3 - "$E2E_WORK/copied.txt" "$E2E_WORK/remote-command" "$laptop" <<'PY' | tee "$E2E_WORK/parsed.txt"
import json, shlex, sys
copied = open(sys.argv[1]).read()
words = shlex.split(copied)
open(sys.argv[2], "w").write(words[-1])
remote = shlex.split(words[-1])
print(f"  program and options: {' '.join(words[:4])}")
print(f"  destination: {words[4]}")
print(f"  the command over SSH: {' '.join(remote)}")
token = json.load(open(sys.argv[3]))["headers"]["Authorization"].split()[-1]
print(f"  the token in the copied line: {'yes' if token in copied else 'no'}")
PY
  expect "the line runs ssh with no terminal, in batch mode" holds "$E2E_WORK/parsed.txt" \
    "program and options: ssh -T -o BatchMode=yes"
  expect "the line names this machine's user and host" holds "$E2E_WORK/parsed.txt" \
    "destination: $(id -un)@$(uname -n)"
  expect "the command sets laptop's file and runs Marley's bridge" holds "$E2E_WORK/parsed.txt" \
    "the command over SSH: env MARLEY_MCP_ENDPOINT=$laptop $E2E_PROFILE/mcp/marley-mcp-bridge"
  expect "no token in the copied line" holds "$E2E_WORK/parsed.txt" "the token in the copied line: no"
  press "" Escape
  settle 1

  echo "== the browser on the page"
  palette "marley: open browser"
  settle 6
  press CTRL l
  settle 1
  type_text "$SITE/index.html"
  press "" Return
  settle 4

  echo "== laptop over SSH"
  out remote tools
  expect "laptop lists its eighteen tools over SSH" holds "$E2E_WORK/out.txt" "18 tools:" "browser_navigate"
  out remote tabs
  expect "laptop reads the tab over SSH" holds "$E2E_WORK/out.txt" "'Over SSH'"
  out remote navigate "$SITE/next.html"
  settle 2
  shot 584-02-driven
  out remote tabs
  expect "laptop's navigation went through" holds "$E2E_WORK/out.txt" "'The next page'"

  echo "== what the SSH session's output carries"
  printf '%s\n' \
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"raw","version":"0"}}}' \
    '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
    '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
    '{"jsonrpc":"2.0","id":3,"method":"ping"}' |
    "$E2E_WORK/over-ssh" >"$E2E_WORK/raw.txt" 2>/dev/null || true
  python3 - "$E2E_WORK/raw.txt" <<'PY' | tee "$E2E_WORK/out.txt"
import json, sys
lines = [line for line in open(sys.argv[1]).read().splitlines() if line.strip()]
bad = 0
ids = []
for line in lines:
    try:
        message = json.loads(line)
    except ValueError:
        bad += 1
        continue
    if not isinstance(message, dict) or message.get("jsonrpc") != "2.0":
        bad += 1
    elif "id" in message:
        ids.append(message["id"])
print(f"  {len(lines)} lines, {bad} not JSON-RPC, answers to {ids}")
PY
  expect "the session's output is JSON-RPC alone" holds "$E2E_WORK/out.txt" "0 not JSON-RPC, answers to [1, 2, 3]"

  echo "== a session held open across a restart"
  mkfifo "$E2E_WORK/hold.fifo"
  BRIDGE=$E2E_WORK/over-ssh python3 "$E2E_WORK/hold-client.py" "$E2E_WORK/hold.fifo" >"$E2E_WORK/hold.txt" 2>&1 &
  local holder=$!
  settle 4
  held tabs
  local pid
  pid=$(marley_pid)
  ss -ltnpH | grep "pid=$pid," | awk '{print $4}' | sort -u | tee "$E2E_WORK/listening.txt"
  local port
  port=$(jq -r '.url' "$E2E_PROFILE/mcp-endpoint.json" | sed -E 's|.*:([0-9]+)/mcp|\1|')
  expect "Marley listens only on its MCP port on 127.0.0.1" test "$(cat "$E2E_WORK/listening.txt")" = "127.0.0.1:$port"
  quit_marley
  settle 4
  held tabs
  launch_marley
  settle 12
  held tabs
  held quit 2
  wait "$holder" || true
  cat "$E2E_WORK/hold.txt"
  expect "the held session saw the page first" bash -c "grep -m1 '^  tabs:' '$E2E_WORK/hold.txt' | grep -q 'The next page'"
  expect "while Marley was down it was told so" holds "$E2E_WORK/hold.txt" "tabs: Marley is not running"
  expect "the bridge told it the tools changed" holds "$E2E_WORK/hold.txt" "notice: notifications/tools/list_changed"
  expect "after the start the same session read the tab" bash -c \
    "grep '^  tabs:' '$E2E_WORK/hold.txt' | tail -1 | grep -q 'The next page'"
  expect "no line of the session was anything but JSON-RPC" bash -c "! grep -q 'not JSON-RPC' '$E2E_WORK/hold.txt'"

  echo "== Cut Off"
  out remote navigate "$SITE/index.html"
  settle 2
  palette "marley: browser clients"
  settle 2
  click "$ROW_CUT_OFF_X" "$ROW_CUT_OFF_Y"
  settle 2
  shot 584-03-cut-off
  press "" Escape
  settle 1
  shot 584-04-mark-gone
  out remote tabs
  expect "over SSH, laptop is told it is not allowed" holds "$E2E_WORK/out.txt" "does not allow this client"
  expect "over SSH, never that Marley is not running" bash -c "! grep -q 'not running' '$E2E_WORK/out.txt'"
  out mcp_agent --endpoint "$laptop" tabs
  expect "a local bridge on laptop's own file is told the same" holds "$E2E_WORK/out.txt" "does not allow this client"
  expect "laptop's file is gone" test ! -e "$laptop"
  expect "the registry no longer names laptop" test "$(jq -c '[.clients[].name]' "$registry")" = '[]'
  quit_marley
  launch_marley
  settle 12
  out remote tabs
  expect "after a restart laptop is still told it is not allowed" holds "$E2E_WORK/out.txt" "does not allow this client"
}
