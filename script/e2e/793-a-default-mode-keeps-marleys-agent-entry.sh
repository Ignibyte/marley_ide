# shellcheck shell=bash
# #793's visual check: a mode made an agent's default no longer hides Marley's own agent entries.
# The run's profile holds the user entries Zed's writers make for them: `agent_servers.Rusty` as
# the box's settings have it (`registry`, `default_config_options.mode` `bypassPermissions`) and
# `agent_servers.Marley` as the mode selector writes it (`registry`, `default_mode` `plan`). The
# Marley and Rusty entries run a scripted ACP agent that offers a Rusty session (its `_meta` names
# Rusty's server) a `mode` select config option and any other session modes, since Zed drops a
# session's modes when it has config options; it takes `session/set_mode` and
# `session/set_config_option`, and logs every message. Rusty on through its stand-in; Marley's permission settings at their
# default, so no mode of theirs comes first.
# - New Agent, "Rusty": the Rusty row in the picker (`793-01-rusty-in-picker`, REQ-001); Enter: a
#   Rusty conversation, its session given `mode` = `bypassPermissions` (`793-02-rusty-thread`,
#   REQ-001, REQ-002).
# - New Agent, "Marley", Enter: a Marley conversation, its session set to `plan`
#   (`793-03-marley-thread`, REQ-003).
compositor sway

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo" "$E2E_WORK/rusty"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/agent.log" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY2'
import json, sys
path = sys.argv[1]
settings = json.load(open(path))
marley = settings.setdefault("marley", {})
marley["assistant"] = {"enabled": True, "agent": "claude_code"}
marley["rusty"] = {"enabled": True}
marley["list_cli_sessions"] = False
for key in ("claude_code_permissions", "codex_permissions", "agent_permissions_by_project"):
    marley.pop(key, None)
# The user entries Zed's writers make for Marley's own agents.
servers = settings.setdefault("agent_servers", {})
servers["Rusty"] = {"type": "registry", "default_config_options": {"mode": "bypassPermissions"}}
servers["Marley"] = {"type": "registry", "default_mode": "plan"}
json.dump(settings, open(path, "w"), indent=2)
PY2
  # The run's profile is a copy of the user's, conversations and all: the copy's list is emptied,
  # so New Agent starts only the run's own.
  local db
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #793's e2e test: it offers a session whose
# `_meta` names Rusty's server a `mode` select config option, and any other session Claude Code's
# modes (Zed drops a session's modes when it has config options), takes `session/set_mode` and
# `session/set_config_option`, answers each prompt with "Noted: <prompt>", and logs every message
# it reads.
import json
import os
import sys

log_path = sys.argv[1]
MODES = [("default", "Default"), ("plan", "Plan Mode"), ("bypassPermissions", "Bypass Permissions")]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def config_options(current):
    return [{
        "id": "mode",
        "name": "Mode",
        "type": "select",
        "currentValue": current,
        "options": [{"value": mode, "name": name} for mode, name in MODES],
    }]


sessions = 0
for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    with open(log_path, "a") as log_file:
        log_file.write(json.dumps(message) + "\n")
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "protocolVersion": 1,
            "agentCapabilities": {"loadSession": False, "promptCapabilities": {}},
            "authMethods": [],
            "agentInfo": {"name": "scripted", "version": "0"},
        }})
    elif method == "session/new":
        sessions += 1
        servers = (((params.get("_meta") or {}).get("claudeCode") or {}).get("options") or {}).get("mcpServers") or {}
        result = {"sessionId": "marley-%d-%d" % (os.getpid(), sessions)}
        if "rusty" in servers:
            result["configOptions"] = config_options("default")
        else:
            result["modes"] = {
                "currentModeId": "default",
                "availableModes": [{"id": mode, "name": name} for mode, name in MODES],
            }
        send({"jsonrpc": "2.0", "id": request_id, "result": result})
    elif method == "session/set_mode":
        send({"jsonrpc": "2.0", "id": request_id, "result": {}})
    elif method == "session/set_config_option":
        send({"jsonrpc": "2.0", "id": request_id,
              "result": {"configOptions": config_options(params.get("value", "default"))}})
    elif method == "session/prompt":
        text = " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        send({"jsonrpc": "2.0", "method": "session/update", "params": {
            "sessionId": params.get("sessionId"),
            "update": {"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": "Noted: " + text}},
        }})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

# How many messages of method `$1` whose line also holds `$2` the agent logged.
logged() {
  grep "\"method\": \"$1\"" "$E2E_WORK/agent.log" 2>/dev/null | grep -c -- "$2" || true
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 5

  echo "== New Agent, Rusty"
  press "CTRL ALT" n
  settle 2
  type_text "Rusty"
  settle 1
  shot 793-01-rusty-in-picker
  press "" Return
  settle 8
  shot 793-02-rusty-thread
  expect "a Rusty session started" test "$(logged session/new '"cwd"')" -ge 1
  expect "its session got mode = bypassPermissions" \
    test "$(logged session/set_config_option '"bypassPermissions"')" -ge 1

  echo "== New Agent, Marley"
  press "CTRL ALT" n
  settle 2
  type_text "Marley"
  settle 1
  press "" Return
  settle 8
  shot 793-03-marley-thread
  agent_log_summary
  expect "a second session started" test "$(logged session/new '"cwd"')" -ge 2
  expect "the Marley session was set to plan" test "$(logged session/set_mode '"plan"')" -ge 1
}

# The agent's requests in order, with the mode and option each named, for the notes.
agent_log_summary() {
  python3 - "$E2E_WORK/agent.log" <<'PY2'
import json, sys
for line in open(sys.argv[1]):
    message = json.loads(line)
    params = message.get("params") or {}
    if message.get("method"):
        named = {key: params[key] for key in ("modeId", "configId", "value") if key in params}
        print("agent got", message["method"], named or "")
PY2
}
