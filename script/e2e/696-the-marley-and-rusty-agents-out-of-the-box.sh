# shellcheck shell=bash
# #696's visual check: the Marley agent out of the box, and Rusty's entry beside it. The run's
# settings leave `marley.assistant` out, so the defaults apply: on, with `auto`. A fake `claude`
# says it is signed in, so `auto` chooses Claude Code (the log, REQ-002) and no offer comes
# (`no-offer`, REQ-001). Rusty is on with `marley_rusty`'s stand-in `rusty-mcp`, so New Agent
# Thread lists Marley and Rusty (`agents`, REQ-001, REQ-003). `MARLEY_ASSISTANT_ADAPTER` puts a
# scripted ACP agent in the adapter's place. A Rusty thread runs it, and its log shows the `_meta`
# Zed sent with `session/new`: Rusty's instructions, Rusty's server and Marley's tools disallowed
# (`rusty-thread`, REQ-004). Nothing is typed into the thread. Rusty off, the submenu lists Marley
# alone (`rusty-off`, REQ-005). `STOP_AT_MENU=1` ends the run at the submenu's shot, for a first run
# that finds where Rusty sits.
compositor sway

# The project's +, and how many steps down New Agent Thread's submenu Rusty sits, as the first
# run's shots found them.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-92}
RUSTY_STEPS=${RUSTY_STEPS:-3}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo" "$E2E_WORK/rusty"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  # A `claude` that says it is signed in, and gives a version for Marley's check.
  cat >"$bin/claude" <<'SH'
#!/bin/sh
case "$1 $2" in
  "auth status") printf '{"loggedIn": true, "authMethod": "claude.ai"}\n' ;;
  *) printf '2.1.293 (Claude Code)\n' ;;
esac
SH
  chmod +x "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/agent.log" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  # The defaults decide the Marley agent; Rusty is on.
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY'
import json, sys
path = sys.argv[1]
settings = json.load(open(path))
marley = settings.setdefault("marley", {})
marley.pop("assistant", None)
marley["rusty"] = {"enabled": True}
json.dump(settings, open(path, "w"), indent=2)
PY
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #696's e2e test: it logs every message it
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
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "rusty-%d" % sessions}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

# Whether a `session/new` the scripted agent read carried Rusty's `_meta`: its instructions, the
# stand-in `rusty-mcp` as the server `rusty`, and Claude Code's file and shell tools and Marley's
# tools disallowed.
rusty_meta_reached_the_agent() {
  python3 - "$E2E_WORK/agent.log" <<'PY'
import json, sys
found = False
for line in open(sys.argv[1]):
    message = json.loads(line)
    if message.get("method") != "session/new":
        continue
    found = True
    meta = message.get("params", {}).get("_meta") or {}
    append = (meta.get("systemPrompt") or {}).get("append", "")
    options = (meta.get("claudeCode") or {}).get("options") or {}
    rusty = (options.get("mcpServers") or {}).get("rusty") or {}
    print("  append starts:", append[:80].replace("\n", " "))
    print("  mcpServers.rusty:", rusty)
    print("  disallowedTools:", options.get("disallowedTools"))
    print("  allowDangerouslySkipPermissions:", options.get("allowDangerouslySkipPermissions"))
    if ("You are Rusty" in append
            and rusty.get("type") == "stdio"
            and str(rusty.get("command", "")).endswith("rusty-mcp")
            and set(options.get("disallowedTools") or []) >= {"Bash", "Edit", "Write", "NotebookEdit", "MultiEdit", "mcp__marley"}
            and options.get("allowDangerouslySkipPermissions") is False):
        sys.exit(0)
if not found:
    print("  no session/new in the log")
sys.exit(1)
PY
}

# The project's + menu, then New Agent Thread's submenu.
open_agent_submenu() {
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  # A menu opens with nothing chosen (Zed #64365); Home chooses New Terminal.
  press "" Home
  press "" Down
  press "" Down
  press "" Right
  settle 1
}

steps() {
  local step
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== no offer; auto chose Claude Code"
  shot 696-01-no-offer
  expect "auto chose Claude Code" holds "$E2E_PROFILE/logs/Marley.log" "assistant: auto chose Claude Code"

  echo "== the submenu lists Marley and Rusty"
  open_agent_submenu
  shot 696-02-agents
  if [[ ${STOP_AT_MENU:-} == 1 ]]; then
    press "" Escape
    return 0
  fi

  echo "== a Rusty thread, on the scripted agent"
  for ((step = 0; step < RUSTY_STEPS; step++)); do
    press "" Down
  done
  settle 1
  press "" Return
  settle 6
  shot 696-03-rusty-thread
  expect "its session/new carried Rusty's _meta" rusty_meta_reached_the_agent

  echo "== Rusty off"
  press "" Escape
  profile_setting marley.rusty.enabled false
  settle 3
  open_agent_submenu
  shot 696-04-rusty-off
  press "" Escape
}
