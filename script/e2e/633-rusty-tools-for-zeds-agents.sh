# shellcheck shell=bash
# #633's visual check: Rusty's tools offered to Zed's agents. A stand-in `rusty-mcp` (a stdio MCP
# server this scenario writes, listing the brain loop's four tools) is first on the PATH Marley
# starts with, never the user's Rusty, and `marley.rusty_tools` is set back on (the harness turns it
# off in every run's copy of the settings).
#
# The Settings window's MCP Servers page lists `rusty` running beside `marley` (`633-01-listed`,
# REQ-001); with the switch off, no `rusty` (`633-02-off`, REQ-002); with a `context_servers.rusty`
# of the user's own, whose command fails, that one is listed, failed (`633-03-own`, REQ-003).
compositor sway

# Sets the settings key path `$1` (dot-separated) to the JSON value `$2` in the run's copy of the
# settings.
set_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'SETTINGS'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
# Comments and trailing commas out: the settings file allows them and JSON does not.
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
*parents, last = sys.argv[2].split(".")
node = settings
for key in parents:
    node = node.setdefault(key, {})
node[last] = json.loads(sys.argv[3])
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTINGS
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo"
  cat >"$bin/rusty-mcp" <<'PY'
#!/usr/bin/env python3
# A stand-in for Rusty's MCP server: it answers initialize and lists the brain loop's tools.
import json, sys

TOOLS = [{"name": name, "description": name, "inputSchema": {"type": "object"}}
         for name in ("brain_ask", "brain_decide", "brain_no_decision", "brain_follow_up")]
for line in sys.stdin:
    message = json.loads(line)
    if "id" not in message:
        continue
    method = message.get("method")
    if method == "initialize":
        result = {"protocolVersion": "2025-06-18", "capabilities": {"tools": {}},
                  "serverInfo": {"name": "rusty-stand-in", "version": "0"}}
    elif method == "tools/list":
        result = {"tools": TOOLS}
    else:
        result = {}
    print(json.dumps({"jsonrpc": "2.0", "id": message["id"], "result": result}), flush=True)
PY
  chmod +x "$bin/rusty-mcp"
  export PATH="$bin:$PATH"
  set_setting marley.rusty_tools true
  # A key for the Settings window's MCP Servers page, which its search does not list.
  printf '%s\n' '[{"bindings": {"ctrl-alt-shift-m": ["zed::OpenSettingsAt", {"path": "context_servers"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

# Opens the Settings window on its MCP Servers page.
mcp_page() {
  press "CTRL ALT SHIFT" m
  settle 4
}

steps() {
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 5
  mcp_page
  shot 633-01-listed

  echo "== the switch off"
  set_setting marley.rusty_tools false
  settle 4
  shot 633-02-off

  echo "== a rusty of the user's own"
  set_setting marley.rusty_tools true
  set_setting context_servers.rusty '{"command": "/bin/false", "args": []}'
  settle 6
  shot 633-03-own
}
