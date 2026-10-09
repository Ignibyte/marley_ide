# shellcheck shell=bash
# #723's visual check: one docs search over Zed's and Marley's docs. #704's scripted MCP client
# (`e2e-agent`) calls the run's Marley:
# - `docs_search "kill switch agent activity"` finds #703's changelog entry, a section of its own
#   (REQ-001);
# - `docs_search "practice projects"` finds the walkthrough (REQ-001);
# - `docs_search "agent activities"`: "activities" is in no doc, so the top hit holds both words
#   only through the stem of "activities" (REQ-002);
# - `docs_read marley/CHANGELOG.md` with that entry's heading reads the entry (REQ-003).
# `723-01-marley` shows the run's Marley that answered.
compositor sway

# The plugin's bridge, the file Marley's marketplace ships.
BRIDGE=$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge

setup() {
  mkdir -p "$E2E_WORK/project"
  printf '# A project\n' >"$E2E_WORK/project/README.md"
  write_client
  open_path "$E2E_WORK/project"
}

write_client() {
  cat >"$E2E_WORK/mcp-client.py" <<'PY'
# A scripted MCP client for #723's e2e test: one docs call through the plugin's bridge, its
# outcome printed.
import json
import os
import subprocess
import sys

bridge = subprocess.Popen([os.environ["BRIDGE"]], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
next_id = 0


def call(method, params):
    global next_id
    next_id += 1
    bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "id": next_id, "method": method, "params": params}) + "\n")
    bridge.stdin.flush()
    for line in bridge.stdout:
        message = json.loads(line)
        if message.get("id") == next_id:
            return message
    sys.exit(f"{method}: the bridge closed")


def tool(name, arguments):
    result = call("tools/call", {"name": name, "arguments": arguments}).get("result") or {}
    return result.get("isError", False), result.get("structuredContent") or {}


call("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "e2e-agent", "version": "0"}})
bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
bridge.stdin.flush()
command, argument = sys.argv[1], sys.argv[2]
if command == "search":
    error, answer = tool("docs_search", {"query": argument})
    if error:
        print("refused " + answer.get("code", ""))
    for result in answer.get("results", []):
        print(f"{result['page']} | {result['heading']} | words={result['words']}")
elif command == "read":
    error, answer = tool("docs_read", {"page": argument, "heading": sys.argv[3]})
    print(("refused " + answer.get("code", "")) if error else answer.get("text", "")[:600])
bridge.stdin.close()
bridge.wait(timeout=10)
PY
}

# The client, run from the harness, its reply in `$E2E_WORK/$1.txt`.
client() {
  BRIDGE=$BRIDGE MARLEY_MCP_ENDPOINT=$E2E_PROFILE/mcp-endpoint.json \
    python3 "$E2E_WORK/mcp-client.py" "${@:2}" >"$E2E_WORK/$1.txt" 2>&1
  cat "$E2E_WORK/$1.txt"
}

# Whether the reply `$1` holds `$2`.
replied() { grep -qF -- "$2" "$E2E_WORK/$1.txt"; }

# Whether the reply `$1`'s first line holds `$2`.
first_reply() { head -1 "$E2E_WORK/$1.txt" | grep -qF -- "$2"; }

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  shot 723-01-marley

  echo "== the changelog"
  client changelog search "kill switch agent activity"
  expect "#703's changelog entry is a hit" \
    replied changelog "marley/CHANGELOG.md | Agent activity and a kill switch for Marley's tools"

  echo "== the walkthrough"
  client walkthrough search "practice projects"
  expect "the walkthrough is a hit" replied walkthrough "marley/walkthrough.md | 0.3 Make the practice projects"

  echo "== a plural no doc holds"
  client plural search "agent activities"
  expect "the top hit holds both words through the stem" first_reply plural "words=2"

  echo "== the entry read by its heading"
  client entry read marley/CHANGELOG.md "Agent activity and a kill switch for Marley's tools"
  expect "docs_read gives the entry" replied entry "(#703, 2026-10-09)"
}
