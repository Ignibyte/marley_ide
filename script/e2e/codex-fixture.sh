# shellcheck shell=bash
# The stand-in Codex for the App Server scenarios (#650, #651), sourced by a scenario at its top
# level. `write_codex_stand_in <path>` writes one Python program that plays Codex's three parts.
# `--version` prints its own file's name, so copies named `0.155.1` and `0.150.0` are two
# releases. `app-server --listen unix://P` serves the WebSocket and the JSON-RPC on P with the
# rules Marley rests on: the first `initialize` names the originator, a thread's subscribers are
# the connections joined when it starts and those that resume it, `thread/started` and the status
# go to every joined connection, turns, tokens and requests to subscribers, and
# `thread/settings/updated` to subscribers of the experimental API; the first answer to a request
# resolves it (`serverRequest/resolved` to the subscribers) and a later one is dropped. It logs
# each method with the client that sent it, and each answer, to `$E2E_WORK/server.log`.
# `--remote unix://P` is the TUI: it prints its arguments, starts its thread and a side thread,
# prints each request it is asked, answers the last with `y` or `n` typed into it, prints
# "closed: answered elsewhere" for one resolved by another client, and sends any other line to
# the server as a cue it plays: `work`, `done`, `fail`, `approve`, `input`, `full`, `scoped`,
# `crash`, and the requests `command <line>`, `file <paths>`, `perms write:<path> network`,
# `elicit <server> <message>` and `input <question>`. Without `--remote` it prints its arguments
# and reads lines, as #532's fake does. Never the user's Codex, and no network.

write_codex_stand_in() {
  sed -e "s|@WORK@|$E2E_WORK|g" >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Codex: --version, app-server --listen unix://P, the TUI with --remote, and #532's
# plain fake without it. It never reaches a model. The server's requests come from cues the TUI
# sends (#651), and the TUI answers the last with y or n.
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
STATE = {"originator": None, "request": 100, "pending": {}, "item": 0}
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


def ask(thread, method, params, flag="waitingOnApproval"):
    """Sends a request to the thread's subscribers, as Codex sends one, after the status that
    says the thread waits on it."""
    set_status(thread, {"type": "active", "activeFlags": [flag]})
    STATE["request"] += 1
    ident = STATE["request"]
    STATE["pending"][ident] = thread["id"]
    params = {"threadId": thread["id"], "turnId": thread["turn"] or "t0",
              "startedAtMs": now_ms(), **params}
    log("server.log", f"{os.getpid()} - asked {ident} {method}")
    for connection in list(thread["subscribers"]):
        connection.send({"id": ident, "method": method, "params": params})


def next_item(prefix):
    STATE["item"] += 1
    return f"{prefix}{STATE['item']}"


def play_request(thread, cue, rest):
    if cue == "command":
        ask(thread, "item/commandExecution/requestApproval", {
            "itemId": next_item("c"), "command": rest, "cwd": thread["cwd"],
            "availableDecisions": ["accept", "acceptForSession",
                                   {"acceptWithExecpolicyAmendment": {"execpolicy_amendment": ["rm"]}},
                                   "decline", "cancel"]})
    elif cue == "file":
        item = next_item("f")
        notify(thread["subscribers"], "item/started", {
            "threadId": thread["id"], "turnId": thread["turn"] or "t0", "startedAtMs": now_ms(),
            "item": {"type": "fileChange", "id": item, "status": "inProgress",
                     "changes": [{"path": path, "kind": {"type": "update"}, "diff": ""}
                                 for path in rest.split()]}})
        ask(thread, "item/fileChange/requestApproval",
            {"itemId": item, "reason": "Codex wants to edit these files"})
    elif cue == "perms":
        write, network = [], False
        for word in rest.split():
            if word.startswith("write:"):
                write.append(word.removeprefix("write:"))
            elif word == "network":
                network = True
        ask(thread, "item/permissions/requestApproval", {
            "itemId": next_item("p"), "cwd": thread["cwd"], "reason": "the build writes there",
            "permissions": {"fileSystem": {"write": write}, "network": {"enabled": network}}})
    elif cue == "elicit":
        server, _, message = rest.partition(" ")
        ask(thread, "mcpServer/elicitation/request", {
            "serverName": server, "mode": "form", "message": message,
            "requestedSchema": {"type": "object", "properties": {}}})
    elif cue == "input":
        ask(thread, "item/tool/requestUserInput", {
            "itemId": next_item("q"),
            "questions": [{"id": "q", "header": "Branch", "question": rest, "options": None}]},
            flag="waitingOnUserInput")


def response(connection, ident, result):
    """A client's answer to a request: the first resolves it, as Codex's server takes the first;
    a later one is dropped."""
    owner = STATE["pending"].pop(ident, None)
    if owner is None:
        log("server.log", f"{os.getpid()} - late {ident} by {connection.name}: "
            f"{json.dumps(result, sort_keys=True)}")
        return
    log("server.log", f"{os.getpid()} - answered {ident} by {connection.name}: "
        f"{json.dumps(result, sort_keys=True)}")
    thread = THREADS.get(owner)
    if thread is None:
        return
    notify(thread["subscribers"], "serverRequest/resolved",
           {"threadId": owner, "requestId": ident})
    if owner not in STATE["pending"].values():
        set_status(thread, {"type": "active", "activeFlags": []})


def play(thread, cue):
    subscribers = thread["subscribers"]
    cue, _, rest = cue.partition(" ")
    if cue in ("command", "file", "perms", "elicit", "input"):
        play_request(thread, cue, rest)
    elif cue == "work":
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
            response(connection, ident, message.get("result"))
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
    asked = []
    answered = set()

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
            elif "id" in message:
                asked.append((message["id"], message["method"]))
                print(f"asks {message['id']}: {message['method']} (y or n)", flush=True)
            elif message.get("method") == "serverRequest/resolved":
                request = message["params"]["requestId"]
                if request not in answered:
                    print(f"closed {request}: answered elsewhere", flush=True)

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
    def reply(yes):
        if not asked:
            return
        request, method = asked.pop()
        answered.add(request)
        if method == "item/permissions/requestApproval":
            result = {"permissions": {}, "scope": "turn"}
        elif method == "mcpServer/elicitation/request":
            result = {"action": "decline"}
        elif method == "item/tool/requestUserInput":
            result = {"answers": {}}
        else:
            result = {"decision": "accept" if yes else "decline"}
        sock.sendall(frame(json.dumps({"id": request, "result": result}).encode(), True))
        print(f"answered {request}", flush=True)

    for line in sys.stdin:
        word = line.strip()
        if not word:
            continue
        if word in ("y", "n"):
            reply(word == "y")
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
