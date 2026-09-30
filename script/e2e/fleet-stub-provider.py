#!/usr/bin/env python3
"""A stub workflow store for #611's scenario: the fleet contract's fixtures, served as
marley.work/v1 over plain HTTP (`http <port>`) or as MCP tools on stdio (`mcp`).

The fixtures come from crates/marley_sdk/fixtures, with every time moved to now. A control file
(`--control <file>`) is read at each request: `ok` answers, `v9` names another contract in the
handshake, and `silent` never answers. Over HTTP, STUB_TOKEN, when set, is the bearer every
request must carry. Over MCP the handshake offers `changes`.
"""

import json
import os
import sys
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import unquote, urlparse

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURES = os.path.join(HERE, "..", "..", "crates", "marley_sdk", "fixtures")
FIXTURE_NOW_MS = 1_759_243_100_000

control_file = None
changes_cursor = 0


def control():
    if not control_file:
        return "ok"
    try:
        with open(control_file) as handle:
            return handle.read().strip() or "ok"
    except OSError:
        return "ok"


def load(name):
    with open(os.path.join(FIXTURES, name)) as handle:
        return json.load(handle)


def rebase(value, shift):
    if isinstance(value, dict):
        return {
            key: (item + shift if key.endswith("_ms") and isinstance(item, int) else rebase(item, shift))
            for key, item in value.items()
        }
    if isinstance(value, list):
        return [rebase(item, shift) for item in value]
    return value


def details():
    now = int(time.time() * 1000)
    shift = now - FIXTURE_NOW_MS
    hosts = {host["host"]["id"]: host for host in rebase(load("hosts.json"), shift)}
    found = rebase(load("details.json"), shift)
    for detail in found:
        detail["agent"]["last_seen_ms"] = now
        host = hosts.get(detail["agent"].get("host_id"))
        if host is not None:
            detail["host"] = host
    return found


def summary(detail):
    agent = detail["agent"]
    run = detail.get("run") or {}
    phases = run.get("phases", [])
    phase = None
    for index, step in enumerate(phases):
        if step.get("state") in ("active", "failed"):
            phase = {"name": step["name"], "index": index, "count": len(phases)}
            break
    failed = any(step.get("state") == "failed" for step in phases) or any(
        gate.get("state") == "fail" for step in phases for gate in step.get("gates", [])
    )
    attention = "question" if detail.get("question") else ("failed" if failed else "none")
    item = detail.get("work_item")
    return {
        "id": agent["id"],
        "name": agent["name"],
        "runtime": agent["runtime"],
        "state": agent["state"],
        "state_since_ms": agent.get("state_since_ms"),
        "last_seen_ms": agent.get("last_seen_ms"),
        "host_id": agent.get("host_id"),
        "work_item": {"id": item["id"], "key": item["key"], "title": item["title"]} if item else None,
        "phase": phase,
        "attention": attention,
        "question": detail.get("question"),
    }


def answer(call, arguments, over):
    """The object a call answers with, or None for a call this stub does not know."""
    global changes_cursor
    if call == "work_handshake":
        handshake = load("handshake.json")
        handshake["provider"] = {"name": f"stub over {over}", "version": "1"}
        handshake["poll_s"] = 2
        if over == "MCP":
            handshake["capabilities"] = handshake["capabilities"] + ["changes"]
        if control() == "v9":
            handshake["contract"] = "marley.work/v9"
        return handshake
    if call == "work_agents":
        return {"agents": [summary(detail) for detail in details()], "cursor": f"c-{changes_cursor}"}
    if call == "work_agent":
        for detail in details():
            if detail["agent"]["id"] == arguments.get("id"):
                return detail
        return None
    if call == "work_changes":
        changes_cursor += 1
        return {
            "cursor": f"c-{changes_cursor}",
            "reset": False,
            "changed": [{"kind": "agent", "id": detail["agent"]["id"]} for detail in details()],
        }
    return None


class Http(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def do_GET(self):
        if control() == "silent":
            time.sleep(600)
            return
        token = os.environ.get("STUB_TOKEN")
        if token and self.headers.get("Authorization") != f"Bearer {token}":
            self.send_response(401)
            self.end_headers()
            return
        url = urlparse(self.path)
        parts = [unquote(part) for part in url.path.split("/") if part]
        calls = {
            ("marley", "v1", "handshake"): ("work_handshake", {}),
            ("marley", "v1", "agents"): ("work_agents", {}),
            ("marley", "v1", "changes"): ("work_changes", {}),
        }
        call, arguments = calls.get(tuple(parts), (None, {}))
        if call is None and len(parts) == 4 and parts[:3] == ["marley", "v1", "agents"]:
            call, arguments = "work_agent", {"id": parts[3]}
        body = answer(call, arguments, "HTTP") if call else None
        if body is None:
            self.send_response(404)
            self.end_headers()
            return
        data = json.dumps(body).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


TOOLS = ["work_handshake", "work_agents", "work_agent", "work_changes"]


def serve_mcp():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if "id" not in message:
            continue
        method = message.get("method")
        params = message.get("params") or {}
        if control() == "silent" and method == "tools/call":
            continue
        if method == "initialize":
            result = {
                "protocolVersion": params.get("protocolVersion", "2025-06-18"),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "fleet-stub", "version": "1"},
            }
        elif method == "tools/list":
            result = {
                "tools": [
                    {"name": name, "description": name, "inputSchema": {"type": "object"}}
                    for name in TOOLS
                ]
            }
        elif method == "tools/call":
            body = answer(params.get("name"), params.get("arguments") or {}, "MCP")
            if body is None:
                result = {"content": [{"type": "text", "text": "unknown"}], "isError": True}
            else:
                result = {
                    "content": [{"type": "text", "text": json.dumps(body)}],
                    "structuredContent": body,
                }
        else:
            result = {}
        sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": message["id"], "result": result}) + "\n")
        sys.stdout.flush()


def main():
    global control_file
    args = sys.argv[1:]
    if "--control" in args:
        at = args.index("--control")
        control_file = args[at + 1]
        del args[at : at + 2]
    if args[:1] == ["http"] and len(args) == 2:
        ThreadingHTTPServer(("127.0.0.1", int(args[1])), Http).serve_forever()
    elif args[:1] == ["mcp"]:
        serve_mcp()
    else:
        sys.exit("usage: fleet-stub-provider.py (http <port> | mcp) [--control <file>]")


if __name__ == "__main__":
    main()
