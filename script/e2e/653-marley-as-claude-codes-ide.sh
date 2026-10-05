# shellcheck shell=bash
# #653's visual check: Marley as Claude Code's IDE. A stand-in `claude`, first on the terminals'
# PATH and named by `MARLEY_CLAUDE`, prints the IDE port its terminal names, the lock file that
# port names in the scratch `CLAUDE_CONFIG_DIR`'s `ide` folder (its fields, its mode and its
# folder's) and that folder's listing, then links to Marley as Claude Code does: the token header,
# the `mcp` subprotocol, its version in `User-Agent` and `clientInfo`, `initialize` and
# `ide_connected` with its pid. It prints each notification Marley sends, and at each typed line
# calls a tool (`diag`, `diag <path>`, `latest`), tries a wrong token (`badtoken`), or prints
# `typed: <line>`. `--offline` links to nothing; `--as <version>` claims another version. A
# stand-in language server, named in `lsp.rust-analyzer.binary.path`, gives `src/main.rs` one error
# when it opens. The rows ship with no tested version, so the run allows the three ids (#653's D4).
# Never the user's Claude Code or its `ide` folder.
#
# `653-01-off`, `653-02-lock`, `653-03-refused`, `653-05-selection`, `653-06-open-file`,
# `653-07-latest`, `653-08-diagnostics`, `653-09-mention`, `653-10a-picker`, `653-10-fallback`,
# `653-11-old-client`, `653-12-off-again`, `653-13-untested`.
compositor sway

SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)

# Where #648's version chip sits in the agent bar, from the first run's shots.
CHIP_X=${CHIP_X:-520}
CHIP_Y=${CHIP_Y:-952}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

# Types `$1` at the program in the focused terminal and sends it.
say() {
  type_text "$1"
  press "" Return
  settle "${2:-2}"
}

# Opens `$1` through the file finder in the focused pane.
open_file() {
  palette "file finder: toggle"
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

# Selects lines 2 to 4 of the focused editor.
select_lines() {
  press CTRL Home
  press "" Down
  press SHIFT Down
  press SHIFT Down
  press SHIFT End
  settle 1
}

# Whether the stand-ins printed `$1`.
printed() { grep -qF -- "$1" "$E2E_WORK/stand-in.log"; }

# How many of the stand-ins' lines hold `$1`.
printed_count() { grep -cF -- "$1" "$E2E_WORK/stand-in.log" || true; }

allow() { profile_setting "marley.allow_untested_versions.$1" "$2"; }

ide_listing() { find "$E2E_WORK/claude-config/ide" -maxdepth 1 -name '*.lock' -printf '%f\n' | sort | tr '\n' ' '; }

write_stand_in() {
  cat >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code linking to Marley as Claude Code links to an IDE.
import base64, json, os, queue, socket, stat, struct, sys, threading, time

work = os.environ["E2E_WORK"]
version = open(f"{work}/claude-version", encoding="utf-8").read().strip()
args = sys.argv[1:]
if args == ["--version"]:
    print(f"{version} (Claude Code)")
    sys.exit(0)
if args and not args[0].startswith("--"):
    sys.exit(0)
offline = "--offline" in args
if "--as" in args:
    version = args[args.index("--as") + 1]
log = open(f"{work}/stand-in.log", "a", encoding="utf-8")
lock_out = threading.Lock()

def say(line):
    with lock_out:
        print(line, flush=True)
        log.write(line + "\n")
        log.flush()

def shown(path):
    return os.path.relpath(path) if path.startswith(os.getcwd() + "/") else path

port = os.environ.get("CLAUDE_CODE_SSE_PORT", "")
say(f"port: {port or 'unset'} · auto-connect: {os.environ.get('CLAUDE_CODE_AUTO_CONNECT_IDE') or 'unset'}")
ide = os.path.join(os.environ["CLAUDE_CONFIG_DIR"], "ide")
listing = lambda: " ".join(sorted(n for n in os.listdir(ide) if n.endswith(".lock"))) if os.path.isdir(ide) else "none"
lock_path = os.path.join(ide, f"{port}.lock")
if port and os.path.exists(lock_path):
    lock = json.load(open(lock_path, encoding="utf-8"))
    mode = oct(stat.S_IMODE(os.stat(lock_path).st_mode))
    folder_mode = oct(stat.S_IMODE(os.stat(ide).st_mode))
    say(f"lock: ideName {lock['ideName']} · transport {lock['transport']} · pid {'Marley' if lock['pid'] != os.getpid() else '?'} · token {len(lock['authToken'])} chars · windows {lock['runningInWindows']}")
    say(f"lock folders: {', '.join(shown(f) or '.' for f in lock['workspaceFolders'])}")
    say(f"lock mode {mode} · ide folder mode {folder_mode}")
    token = lock["authToken"]
else:
    say("lock: none")
    token = None
say(f"ide folder: {listing()}")
if offline or not token:
    if not offline:
        sys.exit(0)
    for line in sys.stdin:
        if line.strip():
            say(f"typed: {line.rstrip()}")
    sys.exit(0)

def upgrade(token_given):
    conn = socket.create_connection(("127.0.0.1", int(port)))
    key = base64.b64encode(os.urandom(16)).decode()
    conn.sendall((f"GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nUpgrade: websocket\r\n"
                  f"Connection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n"
                  f"Sec-WebSocket-Protocol: mcp\r\nUser-Agent: claude-code/{version}\r\n"
                  f"X-Claude-Code-Ide-Authorization: {token_given}\r\n\r\n").encode())
    head = b""
    while b"\r\n\r\n" not in head:
        chunk = conn.recv(1)
        if not chunk:
            break
        head += chunk
    lines = head.decode(errors="replace").split("\r\n")
    headers = {l.split(":", 1)[0].lower(): l.split(":", 1)[1].strip() for l in lines[1:] if ":" in l}
    return conn, lines[0], headers

conn, status, headers = upgrade(token)
say(f"upgrade: {status} · subprotocol {headers.get('sec-websocket-protocol', 'none')}")
send_lock = threading.Lock()

def send_frame(opcode, payload):
    mask = os.urandom(4)
    head = bytes([0x80 | opcode])
    n = len(payload)
    head += bytes([0x80 | n]) if n < 126 else bytes([0x80 | 126]) + struct.pack("!H", n) if n < 65536 else bytes([0x80 | 127]) + struct.pack("!Q", n)
    with send_lock:
        conn.sendall(head + mask + bytes(b ^ mask[i % 4] for i, b in enumerate(payload)))

def recv_exact(n):
    data = b""
    while len(data) < n:
        chunk = conn.recv(n - len(data))
        if not chunk:
            raise EOFError
        data += chunk
    return data

answers = {}
next_id = [0]

def request(method, params=None):
    next_id[0] += 1
    answers[next_id[0]] = queue.Queue()
    send_frame(1, json.dumps({"jsonrpc": "2.0", "id": next_id[0], "method": method, "params": params or {}}).encode())
    return answers[next_id[0]].get(timeout=20)

def notify(method, params):
    send_frame(1, json.dumps({"jsonrpc": "2.0", "method": method, "params": params}).encode())

def place(p):
    return f"{p['line']}:{p['character']}"

def reader():
    try:
        while True:
            b0, b1 = recv_exact(2)
            n = b1 & 0x7F
            if n == 126:
                n = struct.unpack("!H", recv_exact(2))[0]
            elif n == 127:
                n = struct.unpack("!Q", recv_exact(8))[0]
            payload = recv_exact(n)
            opcode = b0 & 0x0F
            if opcode == 9:
                send_frame(10, payload)
            elif opcode == 8:
                break
            elif opcode == 1:
                message = json.loads(payload)
                if "id" in message and message["id"] in answers:
                    answers[message["id"]].put(message)
                elif message.get("method") == "selection_changed":
                    p = message["params"]
                    s = p["selection"]
                    text = p["text"].replace("\n", "⏎")
                    say(f"selection_changed {shown(p['filePath'])} {place(s['start'])}-{place(s['end'])}{' empty' if s['isEmpty'] else ''} {text}")
                elif message.get("method") == "at_mentioned":
                    p = message["params"]
                    say(f"at_mentioned {shown(p['filePath'])} {p.get('lineStart')}-{p.get('lineEnd')}")
    except (EOFError, OSError):
        pass
    say("closed")
    time.sleep(1.5)
    say(f"ide folder: {listing()}")
    os._exit(0)

threading.Thread(target=reader, daemon=True).start()
init = request("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                              "clientInfo": {"name": "claude-code", "version": version}})
info = init["result"]["serverInfo"]
say(f"server: {info['name']} · protocol {init['result']['protocolVersion']}")
notify("notifications/initialized", {})
notify("ide_connected", {"pid": os.getpid()})
tools = request("tools/list")["result"]["tools"]
say(f"tools: {', '.join(t['name'] for t in tools)}")

def call(name, arguments=None):
    result = request("tools/call", {"name": name, "arguments": arguments or {}})["result"]
    text = result["content"][0]["text"]
    return result.get("isError", False), text

for line in sys.stdin:
    line = line.rstrip()
    if line == "diag" or line.startswith("diag "):
        arguments = {"uri": "file://" + os.path.abspath(line[5:])} if line != "diag" else {}
        error, text = call("getDiagnostics", arguments)
        if error:
            say(f"diag error: {text}")
            continue
        files = json.loads(text)
        say(f"diag {line[5:] or 'all'}: {len(files)} file(s)")
        for file in files:
            for d in file["diagnostics"]:
                say(f"  {shown(file['uri'][7:])} {place(d['range']['start'])} {d['severity']} {d['source']} {d['code']}: {d['message']}")
            if not file["diagnostics"]:
                say(f"  {shown(file['uri'][7:])}: none")
    elif line == "latest":
        error, text = call("getLatestSelection")
        answer = json.loads(text) if not error else {}
        if answer.get("success"):
            s = answer["selection"]
            say(f"latest: {shown(answer['filePath'])} {place(s['start'])}-{place(s['end'])}{' empty' if s['isEmpty'] else ''}")
        else:
            say(f"latest: {text}")
    elif line == "badtoken":
        bad, status, _ = upgrade("0" * 32)
        say(f"badtoken: {status}")
        bad.close()
    elif line:
        say(f"typed: {line}")
FAKE
  chmod +x "$1"
}

write_language_server() {
  cat >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in language server: one error in src/main.rs when it opens.
import json, sys

def read():
    length = 0
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            sys.exit(0)
        if line in (b"\r\n", b"\n"):
            break
        if line.lower().startswith(b"content-length:"):
            length = int(line.split(b":")[1])
    return json.loads(sys.stdin.buffer.read(length))

def write(message):
    body = json.dumps(message).encode()
    sys.stdout.buffer.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
    sys.stdout.buffer.flush()

while True:
    message = read()
    method = message.get("method")
    if method == "initialize":
        write({"jsonrpc": "2.0", "id": message["id"], "result": {
            "capabilities": {"textDocumentSync": 1}, "serverInfo": {"name": "stand-in"}}})
    elif method == "textDocument/didOpen" and message["params"]["textDocument"]["uri"].endswith("src/main.rs"):
        write({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": {
            "uri": message["params"]["textDocument"]["uri"], "diagnostics": [{
                "range": {"start": {"line": 2, "character": 4}, "end": {"line": 2, "character": 11}},
                "severity": 1, "source": "stand-in", "code": "E0425",
                "message": "cannot find value `missing` in this scope"}]}})
    elif method == "exit":
        sys.exit(0)
    elif "id" in message and method is not None:
        write({"jsonrpc": "2.0", "id": message["id"], "result": None})
FAKE
  chmod +x "$1"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo config=$E2E_WORK/claude-config
  mkdir -p "$home" "$bin" "$repo/src" "$config/plugins" "$config/ide"
  chmod 700 "$config/ide"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  echo 2.1.288 >"$E2E_WORK/claude-version"
  : >"$E2E_WORK/stand-in.log"
  write_stand_in "$bin/claude"
  write_language_server "$bin/fake-lsp"
  for line in $(seq 10); do echo "line $line of auth.txt"; done >"$repo/src/auth.txt"
  printf '[package]\nname = "fixture"\nversion = "0.1.0"\nedition = "2021"\n' >"$repo/Cargo.toml"
  printf 'fn main() {\n    let total = 1;\n    missing + total;\n}\n' >"$repo/src/main.rs"
  # A lock file a Marley left, its process gone, and another IDE's.
  sh -c 'exit 0' &
  local dead=$!
  wait "$dead"
  printf '{"pid": %s, "workspaceFolders": [], "ideName": "Marley", "transport": "ws", "authToken": "x", "runningInWindows": false}' "$dead" >"$config/ide/1.lock"
  printf '{"pid": %s, "workspaceFolders": [], "ideName": "Neovim", "transport": "ws", "authToken": "x", "runningInWindows": false}' "$dead" >"$config/ide/2.lock"
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/$SHIPPED", "version": "$SHIPPED"}]}}
JSON
  profile_setting session.trust_all_worktrees true
  profile_setting lsp.rust-analyzer.binary.path "\"$bin/fake-lsp\""
  profile_setting marley.ask_before_ending_a_working_agent false
  unset CLAUDE_CODE_SSE_PORT CLAUDE_CODE_AUTO_CONNECT_IDE
  export MARLEY_CLAUDE=$bin/claude CLAUDE_CONFIG_DIR=$config
  git init -q -b main "$repo"
  open_path "$repo"
}

steps() {
  settle 12

  echo "== the switch off"
  expect "the harness's copy has the link off" grep -q '"claude_code_ide": false' "$E2E_PROFILE/config/settings.json"
  say "claude" 3
  shot 653-01-off
  expect "no port with the switch off" printed "port: unset"

  echo "== the switch on"
  allow claude_ide_connection true
  allow claude_ide_selection true
  allow claude_ide_mention true
  profile_setting marley.claude_code_ide true
  settle 4
  palette "workspace: new terminal"
  say "claude" 4
  shot 653-02-lock
  expect "the lock names Marley" printed "lock: ideName Marley"
  expect "the lock is the user's alone" printed "lock mode 0o600 · ide folder mode 0o700"
  expect "the stale lock went, the other IDE's stayed" test "$(ide_listing | tr ' ' '\n' | grep -c '^[12]\.lock$')" = 1
  expect "the link took mcp" printed "subprotocol mcp"
  expect "five tools" printed "tools: getDiagnostics, getWorkspaceFolders, getCurrentSelection, getLatestSelection, getOpenEditors"

  echo "== a wrong token"
  say "badtoken"
  shot 653-03-refused
  expect "a wrong token is refused" printed "badtoken: HTTP/1.1 401 Unauthorized"

  echo "== a selection"
  open_file "auth.txt"
  palette "pane: split and move right"
  select_lines
  settle 1
  shot 653-05-selection
  expect "the selection was sent" printed "selection_changed src/auth.txt 1:0-3:18"

  echo "== the open file"
  open_file "main.rs"
  settle 4
  shot 653-06-open-file
  expect "the open file was sent with an empty selection" printed "selection_changed src/main.rs 0:0-0:0 empty"
  expect "each selection was sent once" test "$(printed_count 'selection_changed src/auth.txt 0:0-0:0 empty')" -le 1

  echo "== the latest selection from the terminal"
  palette "workspace: activate pane left"
  say "latest"
  shot 653-07-latest
  expect "the latest is main.rs's caret" printed "latest: src/main.rs 0:0-0:0 empty"

  echo "== diagnostics"
  say "diag" 3
  say "diag src/auth.txt" 3
  shot 653-08-diagnostics
  expect "main.rs's error" printed "src/main.rs 2:4 Error stand-in E0425"
  expect "auth.txt has none" printed "src/auth.txt: none"

  echo "== send selection as a mention"
  palette "workspace: activate pane right"
  open_file "auth.txt"
  select_lines
  press CTRL greater
  settle 2
  press "" Return
  settle 1
  shot 653-09-mention
  expect "the mention was sent" printed "at_mentioned src/auth.txt 1-3"

  echo "== send selection to a Claude Code with no link"
  palette "workspace: new terminal"
  say "claude --offline" 3
  palette "workspace: activate pane right"
  press CTRL greater
  settle 2
  shot 653-10a-picker
  press "" Return
  settle 2
  press "" Return
  settle 1
  shot 653-10-fallback
  expect "the reference was typed" printed "typed: @src/auth.txt#L2-4"

  echo "== a client whose selection is off"
  allow claude_ide_selection false
  settle 3
  palette "workspace: activate pane left"
  palette "workspace: new terminal"
  say "claude --as 9.9.9" 4
  palette "workspace: activate pane right"
  press CTRL Home
  press SHIFT Down
  settle 2
  palette "workspace: activate pane left"
  say "diag" 3
  shot 653-11-old-client
  expect "two tools for it" printed "tools: getDiagnostics, getWorkspaceFolders"
  expect "no selection after the change" test "$(printed_count 'selection_changed src/auth.txt 0:0-1:0')" = 0

  echo "== the switch off again"
  profile_setting marley.claude_code_ide false
  settle 4
  shot 653-12-off-again
  expect "the clients were closed" test "$(printed_count 'closed')" -ge 2
  expect "only the other IDE's lock is left" test "$(ide_listing)" = "2.lock "

  echo "== the link untested"
  allow claude_ide_connection false
  profile_setting marley.claude_code_ide true
  settle 4
  palette "workspace: new terminal"
  say "claude --offline" 3
  pointer_to "$CHIP_X" "$CHIP_Y"
  settle 2
  shot 653-13-untested
  expect "no port while the link is untested" test "$(grep -F 'port:' "$E2E_WORK/stand-in.log" | tail -1 | cut -d' ' -f2)" = unset

  echo "== the quit removes the lock"
  allow claude_ide_connection true
  settle 4
  expect "a lock while the link runs" test "$(ide_listing | wc -w)" = 2
  quit_marley
  expect "the quit removed Marley's lock" test "$(ide_listing)" = "2.lock "
  expect "one mention" test "$(printed_count 'at_mentioned')" = 1
  expect "one typed reference" test "$(printed_count 'typed: @src/auth.txt#L2-4')" = 1
  cat "$E2E_WORK/stand-in.log"
}
