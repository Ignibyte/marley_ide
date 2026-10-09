# shellcheck shell=bash
# #698's visual check: the Marley entry's Claude Code sessions keep out every Marley tool but its
# eight. The Marley entry runs a scripted ACP agent (`MARLEY_ASSISTANT_ADAPTER`) that logs what it
# reads, so the `_meta` of its `session/new` can be checked; nothing is typed into the thread
# (PR-687). The profile starts with no database, so the rail lists Home above the project (#700).
#
# `698-01-marley-thread`: a Marley thread on the scripted agent, and the check of its `_meta`:
# `terminal_run`, `terminal_type` and the browser's write tools among the disallowed (REQ-001), none
# of its eight (REQ-002).
compositor sway

# The project's +, under Home's header, and how many steps down New Agent Thread's submenu Marley
# sits (Zed Agent, Claude Agent, Marley), from the first run's shot.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-136}
MARLEY_STEPS=${MARLEY_STEPS:-2}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  rm -rf "$E2E_PROFILE/db"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/agent.log" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  profile_setting marley.assistant.enabled true
  profile_setting marley.assistant.agent '"claude_code"'
  profile_setting marley.rusty.enabled false
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #698's e2e test: it logs every message it
# reads, the `_meta` of `session/new` among them, and answers what a thread's start asks.
import json
import sys

log_path = sys.argv[1]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


sessions = 0
for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    with open(log_path, "a") as log_file:
        log_file.write(json.dumps(message) + "\n")
    method, request_id = message.get("method"), message.get("id")
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "protocolVersion": 1,
            "agentCapabilities": {"loadSession": False, "promptCapabilities": {}},
            "authMethods": [],
            "agentInfo": {"name": "scripted", "version": "0"},
        }})
    elif method == "session/new":
        sessions += 1
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "marley-%d" % sessions}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

# Whether the Marley thread's `session/new` kept out Marley's other tools and none of its eight.
marley_meta_keeps_its_tools() {
  python3 - "$E2E_WORK/agent.log" <<'PY'
import json, sys
EIGHT = {"docs_search", "docs_read", "settings_schema", "settings_read", "settings_change",
         "keymap_change", "seat_add", "actions_list"}
for line in open(sys.argv[1]):
    message = json.loads(line)
    if message.get("method") != "session/new":
        continue
    options = ((message.get("params", {}).get("_meta") or {}).get("claudeCode") or {}).get("options") or {}
    disallowed = options.get("disallowedTools") or []
    marley = sorted(tool for tool in disallowed if tool.startswith("mcp__marley"))
    print("  Claude Code's own:", [tool for tool in disallowed if not tool.startswith("mcp__")])
    print("  Marley's kept out (%d):" % len(marley), ", ".join(marley))
    kept_eight = sorted(tool for tool in EIGHT if "mcp__marley__" + tool in disallowed)
    print("  of the eight kept out:", kept_eight or "none")
    wanted = {"mcp__marley__terminal_run", "mcp__marley__terminal_type", "mcp__marley__browser_click"}
    if (set(disallowed) >= wanted | {"Bash", "Edit", "Write", "NotebookEdit", "MultiEdit"}
            and not kept_eight and "mcp__marley" not in disallowed):
        sys.exit(0)
    sys.exit(1)
print("  no session/new in the log")
sys.exit(1)
PY
}

steps() {
  local step
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== a Marley thread on the scripted agent"
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  # A menu opens with nothing chosen (Zed #64365); Home chooses New Terminal.
  press "" Home
  press "" Down
  press "" Down
  press "" Right
  settle 1
  for ((step = 0; step < MARLEY_STEPS; step++)); do
    press "" Down
  done
  settle 1
  press "" Return
  settle 6
  shot 698-01-marley-thread
  expect "the Marley thread's _meta keeps out Marley's other tools, not its eight" marley_meta_keeps_its_tools
}
