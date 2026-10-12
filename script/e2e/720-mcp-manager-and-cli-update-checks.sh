# shellcheck shell=bash
# #720's visual check: the MCP Servers tab and the notification for an agent CLI too old for a part
# Marley uses. The run points Claude Code at its own folder (`CLAUDE_CONFIG_DIR`; the runner points
# `CODEX_HOME` at one), so nothing of the user's is read:
# - `.claude.json`: a user-scope stdio server with `API_TOKEN` in its env and an `--api-key=…`
#   argument, and, for the repository, a local-scope HTTP server with a `token` in its URL's query
#   and an `Authorization` header; the repository's `.mcp.json` server is approved there.
# - Codex's `config.toml`: two servers, one `enabled = false`.
# - Zed's `context_servers`: a stand-in stdio MCP server, beside Marley's own `marley`.
# - `MARLEY_CLAUDE` a stand-in printing 2.1.280, the shared plugin on (it needs 2.1.287); prompt
#   tags need 2.1.283. `MARLEY_CODEX` a stand-in printing 0.157.0, whose App Server is off.
# Shots: the notification (REQ-001), the tab from its button (REQ-002 to REQ-006), Refresh after a
# third Codex server is written (REQ-007), the action again (REQ-002), Configure in Settings
# (REQ-008).
compositor sway

# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
NOTICE_BUTTON_X=${NOTICE_BUTTON_X:-1206}
NOTICE_BUTTON_Y=${NOTICE_BUTTON_Y:-933}
REFRESH_X=${REFRESH_X:-1383}
REFRESH_Y=${REFRESH_Y:-96}
CONFIGURE_X=${CONFIGURE_X:-1507}
CONFIGURE_Y=${CONFIGURE_Y:-96}
STOP_AT=${STOP_AT:-}

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

write_stand_ins() {
  local bin=$1
  printf '#!/bin/sh\necho "2.1.280 (Claude Code)"\n' >"$bin/claude"
  printf '#!/bin/sh\necho "codex-cli 0.157.0"\n' >"$bin/codex"
  cat >"$bin/stand-in-mcp" <<'PY'
#!/usr/bin/env python3
# A stand-in MCP server for Zed: answers `initialize` and `tools/list`, one line each.
import json, sys

for line in sys.stdin:
    message = json.loads(line)
    if "id" not in message:
        continue
    method = message.get("method")
    if method == "initialize":
        result = {"protocolVersion": message["params"].get("protocolVersion", "2025-06-18"),
                  "capabilities": {"tools": {}},
                  "serverInfo": {"name": "stand-in", "version": "0"}}
    elif method == "tools/list":
        result = {"tools": []}
    else:
        result = {}
    sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": message["id"], "result": result}) + "\n")
    sys.stdout.flush()
PY
  chmod +x "$bin/claude" "$bin/codex" "$bin/stand-in-mcp"
}

# Whether the profile's key-value store holds `$1` among the versions warned of.
warned() {
  local db
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "select key from scoped_kv_store where namespace = 'marley-agent-versions'" \
      2>/dev/null | grep -qx -- "$1" && return 0
  done
  return 1
}

setup() {
  local bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$bin" "$repo" "$E2E_WORK/claude" "$CODEX_HOME"
  write_stand_ins "$bin"
  export MARLEY_CLAUDE=$bin/claude MARLEY_CODEX=$bin/codex
  export CLAUDE_CONFIG_DIR=$E2E_WORK/claude
  python3 - "$CLAUDE_CONFIG_DIR/.claude.json" "$repo" <<'PY'
import json, sys
path, repo = sys.argv[1:]
config = {
    "numStartups": 3,
    "mcpServers": {
        "notes": {"type": "stdio", "command": "notes-mcp",
                  "args": ["--stdio", "--api-key=not-a-real-key"],
                  "env": {"API_TOKEN": "not-a-real-token", "NOTES_DIR": "/tmp/notes"}},
    },
    "projects": {
        repo: {
            "mcpServers": {
                "tracker": {"type": "http",
                            "url": "https://tracker.example.com/mcp?team=core&token=not-a-real-token",
                            "headers": {"Authorization": "Bearer not-a-real-token"}},
            },
            "enabledMcpjsonServers": ["docs"],
            "disabledMcpjsonServers": [],
        },
        "/somewhere/else": {"mcpServers": {"elsewhere": {"command": "never-listed"}}},
    },
}
with open(path, "w", encoding="utf-8") as file:
    json.dump(config, file, indent=2)
PY
  printf '{"mcpServers": {"docs": {"command": "docs-mcp", "args": ["serve"]}}}\n' >"$repo/.mcp.json"
  cat >"$CODEX_HOME/config.toml" <<'TOML'
model = "gpt-5"

[mcp_servers.search]
command = "search-mcp"
args = ["--stdio"]
env = { SEARCH_API_KEY = "not-a-real-key" }

[mcp_servers.old-wiki]
url = "https://wiki.example.com/mcp"
enabled = false
TOML
  profile_setting marley.claude_code_shared_plugin true
  profile_setting context_servers "{\"stand-in\": {\"command\": \"$bin/stand-in-mcp\", \"args\": []}}"
  git init -q -b main "$repo"
  open_path "$repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== the notification"
  take 720-01-notice || return 0
  # Remembered, so the next start with the same version says nothing.
  expect "the warning is remembered for claude 2.1.280" warned claude-2.1.280

  echo "== the tab, from the notification's button"
  click "$NOTICE_BUTTON_X" "$NOTICE_BUTTON_Y"
  settle 4
  take 720-02-tab || return 0

  echo "== a third Codex server, then Refresh"
  printf '\n[mcp_servers.third]\ncommand = "third-mcp"\n' >>"$CODEX_HOME/config.toml"
  click "$REFRESH_X" "$REFRESH_Y"
  settle 3
  take 720-03-refresh || return 0

  echo "== the action again"
  palette "marley: open mcp servers" 3
  take 720-04-again || return 0

  echo "== Configure in Settings"
  click "$CONFIGURE_X" "$CONFIGURE_Y"
  settle 4
  take 720-05-zed-page || return 0
}
