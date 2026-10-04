# shellcheck shell=bash
# #650's visual check: Codex's state from its own App Server. A stand-in `codex`, one Python
# program the scenario writes, plays Codex's three parts. `--version` prints its own file's name,
# so copies under `versions/` named `0.155.1` and `0.150.0` are two releases, and `bin/codex`
# links to one, as Codex's installer moves its link (#648's way); `MARLEY_CODEX` names the link.
# `app-server --listen unix://P` serves the WebSocket and the JSON-RPC on P with the rules Marley
# rests on: the first `initialize` names the originator, a thread's subscribers are the
# connections joined when it starts and those that resume it, `thread/started` and the status go
# to every joined connection, turns, tokens and requests to subscribers, and
# `thread/settings/updated` to subscribers of the experimental API. It logs each method with the
# client that sent it to `server.log`. `--remote unix://P` is the TUI: it prints its arguments,
# starts its thread and a side thread, and sends each line typed into it to the server as a cue
# the server plays (`work`, `approve`, `input`, `done`, `fail`, `full`, `scoped`, `crash`).
# Without `--remote` it prints its arguments and reads lines, as #532's fake does. Never the
# user's Codex, and no network.
#
# `650-01-off`, `650-02-launched`, `650-03-working`, `650-04-approval`, `650-05-input`,
# `650-06-opened`, `650-07-done`, `650-08-failed`, `650-09-close-asks`, `650-10-full`,
# `650-11-scoped`, `650-12-server-gone`, `650-13-outside`, `650-14-page`, `650-15-versions`.
compositor sway

# Places in the window, from the first run's shots. The rail sorts its rows by what each agent
# needs, so a row moves; the steps that would click one use the palette instead.
INBOX_FIRST_X=${INBOX_FIRST_X:-110}
INBOX_FIRST_Y=${INBOX_FIRST_Y:-123}
CHIP_B_X=${CHIP_B_X:-183}
CHIP_B_Y=${CHIP_B_Y:-277}
BAR_CHIP_X=${BAR_CHIP_X:-640}
BAR_CHIP_Y=${BAR_CHIP_Y:-954}

server_log() { cat "$E2E_WORK/server.log" 2>/dev/null; }
launches() { cat "$E2E_WORK/launches.log" 2>/dev/null; }

# How many lines of the server's log hold `$1`.
logged() { server_log | grep -cF -- "$1" || true; }

marley_log() { cat "$E2E_PROFILE/logs/Marley.log" 2>/dev/null; }

# A Codex from the New Agent picker.
picker_codex() {
  press "CTRL ALT" n
  settle 2
  type_text "Codex"
  settle 1
  press "" Return
  settle 6
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

# The pid of the `$1`th server the stand-in started, and its socket.
server_pid() { sed -n "${1}p" "$E2E_WORK/servers.txt" | cut -d' ' -f1; }
server_socket() { sed -n "${1}p" "$E2E_WORK/servers.txt" | cut -d' ' -f2; }

# Types `$1` and Enter into the terminal in front.
cue() {
  type_text "$1"
  press "" Return
  settle "${2:-2}"
}

close_settings() {
  sway_msg '[title="Settings"] kill' >/dev/null
  settle 2
}

# Whether every `initialize` from Marley in the server's log comes after the TUI's on the same
# server.
marley_joined_second() {
  python3 - "$E2E_WORK/server.log" <<'ORDER'
import collections, sys

seen = collections.defaultdict(list)
for line in open(sys.argv[1], encoding="utf-8"):
    parts = line.split()
    if len(parts) >= 3 and parts[2] == "initialize":
        seen[parts[0]].append(parts[1])
ok = all(names.index("marley") > names.index("codex-tui")
         for names in seen.values() if "marley" in names and "codex-tui" in names)
joined = any("marley" in names for names in seen.values())
sys.exit(0 if ok and joined else 1)
ORDER
}

# Whether Marley's requests in the server's log are only the five it may send.
marley_methods_allowed() {
  ! server_log | awk '$2 == "marley" && $3 !~ /^(initialize|initialized|thread\/loaded\/list|thread\/read|thread\/resume|thread\/unsubscribe)$/' | grep -q .
}

# Whether the server process `$1` has ended.
gone() { ! kill -0 "$1" 2>/dev/null; }

write_stand_in() {
  sed -e "s|@WORK@|$E2E_WORK|g" >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Codex: --version, app-server --listen unix://P, the TUI with --remote, and #532's
# plain fake without it. It never reaches a model.
import base64
import hashlib
import json
import os
import signal
import socket
import struct
import sys
import threading
import time
import uuid

WORK = "@WORK@"
VERSION = os.path.basename(os.path.realpath(__file__))
ARGS = sys.argv[1:]
GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"


def log(name, line):
    with open(f"{WORK}/{name}", "a", encoding="utf-8") as file:
        file.write(line + "\n")


def frame(payload, masked):
    head = bytearray([0x81])
    size = len(payload)
    bit = 0x80 if masked else 0
    if size < 126:
        head.append(bit | size)
    elif size < 65536:
        head.append(bit | 126)
        head += struct.pack(">H", size)
    else:
        head.append(bit | 127)
        head += struct.pack(">Q", size)
    if not masked:
        return bytes(head) + payload
    mask = os.urandom(4)
    return bytes(head) + mask + bytes(b ^ mask[i % 4] for i, b in enumerate(payload))


class Frames:
    """Reads WebSocket frames from a socket: (opcode, payload), or None at the end."""

    def __init__(self, sock, rest=b""):
        self.sock, self.buf = sock, rest

    def need(self, size):
        while len(self.buf) < size:
            chunk = self.sock.recv(65536)
            if not chunk:
                raise EOFError
            self.buf += chunk

    def next(self):
        try:
            self.need(2)
            opcode, size, masked, at = self.buf[0] & 0x0F, self.buf[1] & 0x7F, self.buf[1] & 0x80, 2
            if size == 126:
                self.need(4)
                size, at = struct.unpack(">H", self.buf[2:4])[0], 4
            elif size == 127:
                self.need(10)
                size, at = struct.unpack(">Q", self.buf[2:10])[0], 10
            mask = b""
            if masked:
                self.need(at + 4)
                mask, at = self.buf[at:at + 4], at + 4
            self.need(at + size)
            payload = self.buf[at:at + size]
            self.buf = self.buf[at + size:]
            if masked:
                payload = bytes(b ^ mask[i % 4] for i, b in enumerate(payload))
            return opcode, payload
        except (EOFError, OSError):
            return None


# --- The App Server --------------------------------------------------------------------------

LOCK = threading.RLock()
CONNECTIONS = []
THREADS = {}
STATE = {"originator": None, "request": 100, "pending": {}}
SANDBOXES = {"danger-full-access": "dangerFullAccess", "workspace-write": "workspaceWrite",
             "read-only": "readOnly"}


class Connection:
    def __init__(self, sock):
        self.sock, self.name, self.joined, self.experimental = sock, "?", False, False
        self.opt_out = set()
        self.write = threading.Lock()

    def send(self, message):
        data = frame(json.dumps(message).encode(), masked=False)
        with self.write:
            try:
                self.sock.sendall(data)
            except OSError:
                pass


def now_ms():
    return int(time.time() * 1000)


def notify(connections, method, params, experimental=False):
    for connection in list(connections):
        if method in connection.opt_out or (experimental and not connection.experimental):
            continue
        connection.send({"method": method, "params": params, "emittedAtMs": now_ms()})


def joined():
    return [connection for connection in CONNECTIONS if connection.joined]


def thread_json(thread):
    return {"id": thread["id"], "ephemeral": thread["ephemeral"], "threadSource": thread["source"],
            "createdAt": thread["created"], "updatedAt": thread["created"], "cwd": thread["cwd"],
            "status": thread["status"], "source": "vscode", "preview": "", "turns": [],
            "cliVersion": VERSION, "modelProvider": "openai", "projectId": None,
            "sessionId": thread["id"], "parentThreadId": None}


def set_status(thread, status):
    thread["status"] = status
    notify(joined(), "thread/status/changed", {"threadId": thread["id"], "status": status})


def tokens(thread, total):
    usage = {"totalTokens": total, "inputTokens": total, "cachedInputTokens": 0,
             "outputTokens": 0, "reasoningOutputTokens": 0}
    notify(thread["subscribers"], "thread/tokenUsage/updated",
           {"threadId": thread["id"], "turnId": thread["turn"],
            "tokenUsage": {"total": usage, "last": usage, "modelContextWindow": 258400}})


def play(thread, cue):
    subscribers = thread["subscribers"]
    if cue == "work":
        thread["turn"] = str(uuid.uuid4())
        if thread.pop("settings_changed", False):
            notify(subscribers, "thread/settings/updated", {
                "threadId": thread["id"],
                "threadSettings": {"approvalPolicy": thread["approval"],
                                   "sandboxPolicy": {"type": thread["sandbox"]},
                                   "approvalsReviewer": "user", "cwd": thread["cwd"],
                                   "model": "stand-in", "modelProvider": "openai",
                                   "collaborationMode": {"mode": "default"}}}, experimental=True)
        notify(subscribers, "turn/started",
               {"threadId": thread["id"], "turn": {"id": thread["turn"], "status": "inProgress"}})
        set_status(thread, {"type": "active", "activeFlags": []})
        notify(subscribers, "item/started", {"threadId": thread["id"], "turnId": thread["turn"],
                                             "item": {"type": "reasoning", "id": "r1"}})
        tokens(thread, 12400)
    elif cue == "approve":
        set_status(thread, {"type": "active", "activeFlags": ["waitingOnApproval"]})
        STATE["request"] += 1
        STATE["pending"][STATE["request"]] = thread["id"]
        for connection in list(subscribers):
            connection.send({"id": STATE["request"], "method": "item/commandExecution/requestApproval",
                             "params": {"threadId": thread["id"], "turnId": thread["turn"],
                                        "itemId": "c1", "command": "rm -rf build"}})
    elif cue == "input":
        set_status(thread, {"type": "active", "activeFlags": ["waitingOnUserInput"]})
    elif cue in ("done", "fail"):
        for request, owner in list(STATE["pending"].items()):
            if owner == thread["id"]:
                del STATE["pending"][request]
                notify(subscribers, "serverRequest/resolved",
                       {"threadId": thread["id"], "requestId": request})
        turn = {"id": thread["turn"], "status": "completed"}
        if cue == "fail":
            turn = {"id": thread["turn"], "status": "failed",
                    "error": {"message": "stream disconnected before completion"}}
        notify(subscribers, "turn/completed", {"threadId": thread["id"], "turn": turn})
        if cue == "done":
            tokens(thread, 15800)
        set_status(thread, {"type": "idle"})
    elif cue in ("full", "scoped"):
        thread["sandbox"] = "dangerFullAccess" if cue == "full" else "workspaceWrite"
        thread["settings_changed"] = True
    elif cue == "crash":
        sys.stderr.write("stand-in: the App Server crashed on purpose\n")
        sys.stderr.flush()
        log("server.log", f"{os.getpid()} - crashed")
        os._exit(3)


def answer(connection, message):
    method, params, ident = message.get("method"), message.get("params") or {}, message.get("id")
    log("server.log", f"{os.getpid()} {connection.name if method != 'initialize' else (params.get('clientInfo') or {}).get('name', '?')} {method}"
        + ("" if ident is not None else " (notification)"))
    reply = lambda result: connection.send({"id": ident, "result": result})
    fail = lambda code, text: connection.send({"id": ident, "error": {"code": code, "message": text}})
    with LOCK:
        if method == "initialize":
            info = params.get("clientInfo") or {}
            capabilities = params.get("capabilities") or {}
            connection.name = info.get("name", "?")
            connection.experimental = bool(capabilities.get("experimentalApi"))
            connection.opt_out = set(capabilities.get("optOutNotificationMethods") or [])
            STATE["originator"] = STATE["originator"] or connection.name
            reply({"userAgent": f"{STATE['originator']}/{VERSION} (stand-in) ({connection.name}; "
                                f"{info.get('version', '?')})",
                   "codexHome": f"{WORK}/codex-home", "platformFamily": "unix",
                   "platformOs": "linux"})
        elif method == "initialized":
            connection.joined = True
        elif method == "thread/start":
            ident_thread = str(uuid.uuid4())
            thread = {"id": ident_thread, "ephemeral": bool(params.get("ephemeral")),
                      "source": params.get("threadSource", "user"), "created": int(time.time()),
                      "cwd": params.get("cwd") or os.getcwd(), "status": {"type": "idle"},
                      "approval": params.get("approvalPolicy") or "on-request",
                      "sandbox": SANDBOXES.get(params.get("sandbox"), "workspaceWrite"),
                      "turn": None, "subscribers": set(joined()) | {connection}}
            THREADS[ident_thread] = thread
            log("server.log", f"{os.getpid()} - thread {ident_thread} ephemeral={thread['ephemeral']} "
                f"subscribers={sorted(c.name for c in thread['subscribers'])}")
            reply({"thread": thread_json(thread), "approvalPolicy": thread["approval"],
                   "sandbox": {"type": thread["sandbox"]}, "cwd": thread["cwd"],
                   "model": "stand-in", "modelProvider": "openai"})
            notify(joined(), "thread/started", {"thread": thread_json(thread)})
        elif method == "thread/loaded/list":
            reply({"data": list(THREADS), "nextCursor": None})
        elif method == "thread/read":
            thread = THREADS.get(params.get("threadId"))
            if thread is None:
                fail(-32600, f"thread not loaded: {params.get('threadId')}")
            else:
                reply({"thread": thread_json(thread)})
        elif method == "thread/resume":
            thread = THREADS.get(params.get("threadId"))
            if thread is None:
                fail(-32600, "no such thread")
                return
            thread["subscribers"].add(connection)
            log("server.log", f"{os.getpid()} - resumed by {connection.name} while "
                f"{thread['status']['type']}")
            reply({"thread": thread_json(thread), "approvalPolicy": thread["approval"],
                   "approvalsReviewer": "user", "sandbox": {"type": thread["sandbox"]},
                   "cwd": thread["cwd"], "model": "stand-in", "modelProvider": "openai"})
        elif method == "thread/unsubscribe":
            thread = THREADS.get(params.get("threadId"))
            if thread is not None:
                thread["subscribers"].discard(connection)
            reply({"status": "unsubscribed"})
        elif method == "standIn/cue":
            thread = THREADS.get(params.get("threadId"))
            reply({})
            if thread is not None:
                play(thread, params.get("cue"))
        elif method is None and ident is not None:
            log("server.log", f"{os.getpid()} - {connection.name} answered request {ident}")
        elif ident is not None:
            fail(-32601, f"method not found: {method}")


def serve_connection(sock):
    buf = b""
    while b"\r\n\r\n" not in buf:
        chunk = sock.recv(4096)
        if not chunk:
            return
        buf += chunk
    head, rest = buf.split(b"\r\n\r\n", 1)
    key = ""
    for line in head.decode("latin-1").split("\r\n"):
        if line.lower().startswith("sec-websocket-key:"):
            key = line.split(":", 1)[1].strip()
    accept = base64.b64encode(hashlib.sha1((key + GUID).encode()).digest()).decode()
    sock.sendall(("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n"
                  f"Connection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\n\r\n").encode())
    connection = Connection(sock)
    with LOCK:
        CONNECTIONS.append(connection)
    frames = Frames(sock, rest)
    while True:
        got = frames.next()
        if got is None or got[0] == 8:
            break
        if got[0] == 9:
            with connection.write:
                sock.sendall(bytes([0x8A, 0]))
            continue
        if got[0] == 1:
            answer(connection, json.loads(got[1]))
    with LOCK:
        CONNECTIONS.remove(connection)
        for thread in THREADS.values():
            thread["subscribers"].discard(connection)


def app_server(path):
    listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    if os.path.exists(path):
        os.unlink(path)
    listener.bind(path)
    os.chmod(path, 0o600)
    listener.listen(16)

    def stop(*_):
        log("server.log", f"{os.getpid()} - stopped by SIGTERM")
        if os.path.exists(path):
            os.unlink(path)
        os._exit(0)

    signal.signal(signal.SIGTERM, stop)
    log("server.log", f"{os.getpid()} - listening on {path} in {os.getcwd()}")
    log("servers.txt", f"{os.getpid()} {path}")
    while True:
        sock, _ = listener.accept()
        threading.Thread(target=serve_connection, args=(sock,), daemon=True).start()


# --- The TUI ---------------------------------------------------------------------------------

def tui(path, folder):
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.connect(path)
    key = base64.b64encode(os.urandom(16)).decode()
    sock.sendall((f"GET /rpc HTTP/1.1\r\nHost: localhost\r\nUpgrade: websocket\r\n"
                  f"Connection: Upgrade\r\nSec-WebSocket-Key: {key}\r\n"
                  "Sec-WebSocket-Version: 13\r\n\r\n").encode())
    buf = b""
    while b"\r\n\r\n" not in buf:
        buf += sock.recv(4096)
    rest = buf.split(b"\r\n\r\n", 1)[1]
    answers, arrived = {}, threading.Condition()

    def read():
        frames = Frames(sock, rest)
        while True:
            got = frames.next()
            if got is None:
                return
            message = json.loads(got[1])
            if "id" in message and "method" not in message:
                with arrived:
                    answers[message["id"]] = message
                    arrived.notify_all()

    threading.Thread(target=read, daemon=True).start()
    counter = [0]

    def call(method, params):
        counter[0] += 1
        ident = counter[0]
        sock.sendall(frame(json.dumps({"id": ident, "method": method, "params": params}).encode(), True))
        with arrived:
            arrived.wait_for(lambda: ident in answers, timeout=5)
            return answers.get(ident, {}).get("result", {})

    call("initialize", {"clientInfo": {"name": "codex-tui", "version": VERSION},
                        "capabilities": {"experimentalApi": True}})
    sock.sendall(frame(json.dumps({"method": "initialized"}).encode(), True))
    option = lambda name, default: ARGS[ARGS.index(name) + 1] if name in ARGS else default
    started = call("thread/start", {"cwd": folder, "threadSource": "user",
                                     "approvalPolicy": option("--ask-for-approval", "on-request"),
                                     "sandbox": option("--sandbox", "workspace-write")})
    thread = started.get("thread", {}).get("id", "?")
    call("thread/start", {"cwd": folder, "ephemeral": True, "threadSource": "system",
                          "approvalPolicy": "never", "sandbox": "read-only"})
    print(f"thread {thread[:8]}: type a cue", flush=True)
    titled = False
    for line in sys.stdin:
        word = line.strip()
        if not word:
            continue
        call("standIn/cue", {"threadId": thread, "cue": word})
        print(f"cue: {word}", flush=True)
        if word == "work" and not titled:
            titled = True
            call("thread/start", {"cwd": folder, "ephemeral": True, "threadSource": "system",
                                  "approvalPolicy": "never", "sandbox": "read-only"})


if ARGS[:1] == ["--version"]:
    print(f"codex-cli {VERSION}")
    sys.exit(0)
log("launches.log", "codex: " + " ".join(ARGS))
if ARGS[:1] == ["app-server"]:
    app_server(ARGS[ARGS.index("--listen") + 1].removeprefix("unix://"))
elif "--remote" in ARGS:
    print("codex started with: " + " ".join(ARGS), flush=True)
    tui(ARGS[ARGS.index("--remote") + 1].removeprefix("unix://"),
        ARGS[ARGS.index("--cd") + 1] if "--cd" in ARGS else os.getcwd())
else:
    print("codex started with: " + " ".join(ARGS), flush=True)
    for _ in sys.stdin:
        pass
FAKE
  chmod +x "$1"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin versions=$E2E_WORK/versions
  mkdir -p "$home" "$bin" "$versions"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  write_stand_in "$versions/0.155.1"
  cp "$versions/0.155.1" "$versions/0.150.0"
  ln -sfn "$versions/0.155.1" "$bin/codex"
  export MARLEY_CODEX=$bin/codex
  printf '%s\n' '[{"context": "Workspace", "bindings": {"ctrl-alt-shift-k": ["zed::OpenSettingsAt", {"path": "marley.codex_app_server"}], "ctrl-alt-shift-j": ["zed::OpenSettingsAt", {"path": "marley.allow_untested_versions.codex_app_server"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  marley_log | grep "codex app server" || true
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== the switch off"
  picker_codex
  settle 3
  shot 650-01-off
  expect "Codex started with no --remote" test "$(launches | grep -c -- '--remote' || true)" = 0

  echo "== the switch on: Codex A"
  profile_setting marley.codex_app_server true
  settle 3
  picker_codex
  settle 4
  shot 650-02-launched
  expect "A's TUI was typed with --remote and --cd" test "$(launches | grep -c -- '--remote unix://.*/codex/.*\.sock --cd ' || true)" = 1
  expect "the server came up" test "$(logged 'listening on')" = 1
  expect "Marley joined after the TUI" marley_joined_second

  echo "== work"
  cue work 6
  shot 650-03-working

  echo "== Codex B, and an approval in it"
  picker_codex
  settle 4
  cue approve 3
  shot 650-04-approval

  echo "== input in A"
  palette "pane: activate previous item"
  cue input 3
  shot 650-05-input

  echo "== the inbox opens B"
  click "$INBOX_FIRST_X" "$INBOX_FIRST_Y"
  settle 2
  shot 650-06-opened

  echo "== done in B"
  cue "done" 3
  shot 650-07-done

  echo "== work then fail in B"
  cue work 2
  cue fail 3
  shot 650-08-failed

  echo "== a working Codex asks before it closes"
  cue work 6
  palette "pane: close active item"
  shot 650-09-close-asks
  press "" Escape
  settle 2

  echo "== full access from the thread"
  cue full 1
  cue work 3
  pointer_to 800 400
  settle 1
  pointer_to "$CHIP_B_X" "$CHIP_B_Y"
  settle 2
  shot 650-10-full

  echo "== Codex C started with full access, then scoped"
  profile_setting marley.agent_permissions_by_project "{\"$E2E_WORK/repo\": {\"codex\": \"full_access\"}}"
  settle 3
  picker_codex
  settle 4
  shot 650-11a-full-from-start
  cue scoped 1
  cue work 3
  shot 650-11-scoped
  expect "C's arguments still ask for full access" test "$(launches | grep -c -- '--sandbox danger-full-access' || true)" -ge 1

  echo "== the server crashes"
  cue crash 4
  shot 650-12-server-gone

  echo "== B closed when idle"
  palette "pane: activate previous item"
  cue "done" 3
  palette "pane: close active item"
  settle 4
  expect "B's server ended with its terminal" gone "$(server_pid 2)"
  expect "B's socket is gone" test ! -e "$(server_socket 2)"

  echo "== an untested Codex"
  ln -sfn "$E2E_WORK/versions/0.150.0" "$E2E_WORK/bin/codex"
  settle 11
  picker_codex
  settle 3
  pointer_to 800 400
  settle 1
  pointer_to "$BAR_CHIP_X" "$BAR_CHIP_Y"
  settle 2
  shot 650-13-outside
  expect "the untested Codex started with no --remote" test "$(launches | tail -1 | grep -c -- '--remote' || true)" = 0

  echo "== the settings"
  press "CTRL ALT SHIFT" k
  settle 4
  shot 650-14-page
  close_settings
  press "CTRL ALT SHIFT" j
  settle 4
  shot 650-15-versions
  close_settings

  echo "== the logs"
  expect "Marley sent only its five methods" marley_methods_allowed
  expect "Marley answered no request" test "$(logged 'marley answered')" = 0
  expect "Marley resumed only idle threads" test "$(logged 'resumed by marley while active')" = 0

  echo "== quit"
  local folder
  folder=$(dirname "$(dirname "$(server_socket 1)")")
  # A still waits on its input, which the close guard would ask about at the quit.
  profile_setting marley.ask_before_ending_a_working_agent false
  settle 3
  quit_marley
  settle 2
  expect "A's server ended with Marley" gone "$(server_pid 1)"
  expect "Marley's socket folder is gone" test ! -e "$folder"
  server_log
}
