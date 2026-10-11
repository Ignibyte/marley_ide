# shellcheck shell=bash
# #712's visual check: Zed's language servers for agents. The repository is a small Rust crate
# (`src/main.rs` with one error and `answer` used twice, `src/lib.rs` that no tab opens) and a
# made-up `.env`. A stand-in language server, named in `lsp.rust-analyzer.binary.path` (#653's
# way), publishes the error on `didOpen` of `src/main.rs` and answers definition, references and
# hover from fixed places. `lsp-ask`, on the terminal's PATH, calls the four tools through the
# plugin's bridge from a Marley terminal, so the caller is that terminal, and prints each answer.
# Shots: one file's and the project's diagnostics with the editor's underline (REQ-001, REQ-002),
# a definition (REQ-003), references (REQ-004), hover (REQ-005), a definition in a file no tab
# shows (REQ-006), the refusals (REQ-007), the editors area off (REQ-008) and Agent Activity
# (REQ-009).
compositor sway

STOP_AT=${STOP_AT:-}

# The plugin's bridge, the file Marley's marketplace ships.
BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle "${2:-2}"
}

# Types `lsp-ask …` into the terminal in front and waits for its answer.
ask() {
  type_text "clear; lsp-ask $*"
  press "" Return
  settle "${ASK_WAIT:-4}"
}

# Whether the answers saved so far hold `$1`.
answered() {
  grep -qF -- "$1" "$E2E_WORK/answers.txt"
}

write_language_server() {
  cat >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in language server: one error in src/main.rs, and fixed definitions, references and
# hover.
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

def place(uri, line, start, end):
    return {"uri": uri, "range": {"start": {"line": line, "character": start},
                                  "end": {"line": line, "character": end}}}

def sibling(uri, name):
    return uri.rsplit("/", 1)[0] + "/" + name

while True:
    message = read()
    method = message.get("method")
    params = message.get("params") or {}
    uri = (params.get("textDocument") or {}).get("uri", "")
    if method == "initialize":
        write({"jsonrpc": "2.0", "id": message["id"], "result": {
            "capabilities": {"textDocumentSync": 1, "definitionProvider": True,
                             "referencesProvider": True, "hoverProvider": True},
            "serverInfo": {"name": "stand-in"}}})
    elif method == "textDocument/didOpen" and uri.endswith("src/main.rs"):
        write({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": {
            "uri": uri, "diagnostics": [{
                "range": {"start": {"line": 6, "character": 4}, "end": {"line": 6, "character": 11}},
                "severity": 1, "source": "stand-in", "code": "E0425",
                "message": "cannot find value `missing` in this scope"}]}})
    elif method == "textDocument/definition":
        if uri.endswith("src/lib.rs"):
            result = [place(uri, 0, 7, 13)]
        else:
            result = [place(uri, 0, 3, 9)]
        write({"jsonrpc": "2.0", "id": message["id"], "result": result})
    elif method == "textDocument/references":
        main = uri if uri.endswith("src/main.rs") else sibling(uri, "main.rs")
        write({"jsonrpc": "2.0", "id": message["id"], "result": [
            place(main, 4, 16, 22), place(main, 5, 19, 25)]})
    elif method == "textDocument/hover":
        write({"jsonrpc": "2.0", "id": message["id"], "result": {
            "contents": {"kind": "markdown",
                         "value": "```rust\nfn answer() -> i32\n```\nThe answer, always 42."},
            "range": {"start": {"line": 4, "character": 16}, "end": {"line": 4, "character": 22}}}})
    elif method == "exit":
        sys.exit(0)
    elif "id" in message and method is not None:
        write({"jsonrpc": "2.0", "id": message["id"], "result": None})
FAKE
  chmod +x "$1"
}

write_client() {
  cat >"$E2E_WORK/lsp-client.py" <<'PY'
# A scripted MCP client for #712's e2e test: one language server tool through the plugin's
# bridge, its answer printed and saved.
import json
import os
import subprocess
import sys

bridge = subprocess.Popen([os.environ["BRIDGE"]], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                          stderr=subprocess.DEVNULL, text=True)
next_id = 0


def call(method, params):
    global next_id
    next_id += 1
    bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "id": next_id, "method": method,
                                   "params": params}) + "\n")
    bridge.stdin.flush()
    for line in bridge.stdout:
        message = json.loads(line)
        if message.get("id") == next_id:
            return message
    sys.exit(f"{method}: the bridge closed")


call("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                    "clientInfo": {"name": "e2e-agent", "version": "0"}})
bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
bridge.stdin.flush()
verb, *rest = sys.argv[1:]
tools = {"diag": "editor_diagnostics", "def": "editor_definition", "refs": "editor_references",
         "hover": "editor_hover"}
arguments = {}
if rest:
    path = rest[0]
    arguments["path"] = path if path.startswith("/") else os.path.join(os.getcwd(), path)
if len(rest) > 1:
    arguments["line"] = int(rest[1])
if len(rest) > 2:
    arguments["symbol"] = rest[2]
result = call("tools/call", {"name": tools[verb], "arguments": arguments}).get("result") or {}
answer = result.get("structuredContent") or {}
if result.get("isError"):
    out = f"{tools[verb]}: refused {answer.get('code', '')}: {answer.get('reason', '')}"
else:
    texts = [part.get("text", "") for part in result.get("content", []) if part.get("type") == "text"]
    out = f"{tools[verb]}:\n" + "\n".join(texts)
    for key in ("total", "next", "cut", "redacted"):
        if key in answer:
            out += f"\n{key}={answer[key]}"
print(out)
with open(os.environ["ANSWERS"], "a", encoding="utf-8") as saved:
    saved.write(out + "\n")
bridge.stdin.close()
bridge.wait(timeout=10)
PY
  cat >"$E2E_WORK/bin/lsp-ask" <<SH
#!/bin/sh
BRIDGE='$BRIDGE' MARLEY_MCP_ENDPOINT='$E2E_PROFILE/mcp-endpoint.json' ANSWERS='$E2E_WORK/answers.txt' \\
  exec python3 '$E2E_WORK/lsp-client.py' "\$@"
SH
  chmod +x "$E2E_WORK/bin/lsp-ask"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo/src"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  write_language_server "$bin/fake-lsp"
  write_client
  : >"$E2E_WORK/answers.txt"
  printf '[package]\nname = "fixture"\nversion = "0.1.0"\nedition = "2021"\n' >"$repo/Cargo.toml"
  cat >"$repo/src/main.rs" <<'RS'
fn answer() -> i32 {
    42
}
fn main() {
    let total = answer();
    println!("{}", answer() + total);
    missing + total;
}
RS
  printf 'pub fn helper() -> u8 {\n    7\n}\n' >"$repo/src/lib.rs"
  printf 'API_TOKEN=not-a-real-token\n' >"$repo/.env"
  profile_setting session.trust_all_worktrees true
  profile_setting lsp.rust-analyzer.binary.path "\"$bin/fake-lsp\""
  git init -q -b main "$repo"
  open_path "$repo"
}

steps() {
  local repo=$E2E_WORK/repo
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== main.rs, and a terminal beside it"
  palette "file finder: toggle" 1
  type_text "main.rs"
  settle 1
  press "" Return
  settle 4
  palette "workspace: new terminal" 3
  palette "pane: split right" 3
  # main.rs in front on the left; the answers typed on the right.
  palette "workspace: activate previous pane" 1
  palette "pane: activate previous item" 1
  palette "workspace: activate next pane" 1
  settle 2
  echo "== diagnostics"
  ASK_WAIT=6 ask diag src/main.rs
  take 712-01-diagnostics || return 0
  ask diag
  take 712-01b-project-diagnostics || return 0
  expect "the file's error, at its line and column" answered "src/main.rs:7:5: error: cannot find value"
  echo "== definition, references, hover"
  ask def src/main.rs 5 answer
  take 712-02-definition || return 0
  expect "the definition on line 1" answered "src/main.rs:1:4: fn answer() -> i32 {"
  ask refs src/main.rs 1 answer
  take 712-03-references || return 0
  expect "both uses" answered "total=2"
  ask hover src/main.rs 5 answer
  take 712-04-hover || return 0
  expect "the hover text" answered "The answer, always 42."
  echo "== a file no tab shows"
  ask def src/lib.rs 1 helper
  take 712-05-no-tab || return 0
  expect "lib.rs answered" answered "src/lib.rs:1:8: pub fn helper() -> u8 {"
  echo "== refusals"
  ask hover .env 1 API_TOKEN
  ask hover /etc/hostname 1 x
  ask def src/main.rs 5 nowhere
  type_text "clear; tail -3 $E2E_WORK/answers.txt"
  press "" Return
  settle 2
  take 712-06-refused || return 0
  expect "a secret file refused" answered "refused secret_file"
  expect "a file outside refused" answered "refused outside_projects"
  expect "a symbol not on its line refused" answered "refused no_symbol"
  echo "== the editors area off"
  profile_setting marley.agent_control.editors '"off"'
  settle 3
  ask hover src/main.rs 5 answer
  take 712-07-off || return 0
  expect "refused while off" answered "refused agent_control_off"
  echo "== Agent Activity"
  palette "marley: open agent activity" 3
  take 712-08-activity || return 0
}
