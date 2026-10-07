# shellcheck shell=bash
# #683's visual check: Marley's own agent in the Agent Panel. A fake `claude` says it is signed in,
# so Marley offers the agent a few seconds after start (`offer`, REQ-001); Turn On writes the
# switch (`turned-on`, REQ-002) and the project's New Agent Thread submenu lists Marley (`menu`,
# REQ-003). `MARLEY_ASSISTANT_ADAPTER` puts a scripted ACP agent in the adapter's place, so a
# Marley thread runs it, and its log shows the `_meta` Zed sent with `session/new`: the
# instructions as a system-prompt append, the five tools disallowed, no bypass (`thread`,
# REQ-005, REQ-006). The switch off, the submenu no longer lists Marley (`off`, REQ-004).
compositor sway

# The offer's Turn On, the project's +, how many steps down New Agent Thread's submenu Marley
# sits, as the first run's shots found them.
TURN_ON_X=${TURN_ON_X:-958}
TURN_ON_Y=${TURN_ON_Y:-901}
PLUS_X=${PLUS_X:-224}
PLUS_Y=${PLUS_Y:-123}
MARLEY_STEPS=${MARLEY_STEPS:-2}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
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
  # The run starts undecided, so the offer comes.
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY'
import json, sys
path = sys.argv[1]
settings = json.load(open(path))
settings.get("marley", {}).pop("assistant", None)
json.dump(settings, open(path, "w"), indent=2)
PY
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #683's e2e test: it answers each prompt
# and logs every message it reads, the `_meta` of `session/new` among them.
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
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "marley-%d" % sessions}})
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

# Whether the settings copy holds `marley.assistant.enabled` as `$1`.
assistant_is() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'PY'
import json, re, sys
text = open(sys.argv[1]).read()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
value = json.loads(text).get("marley", {}).get("assistant", {}).get("enabled")
sys.exit(0 if json.dumps(value) == sys.argv[2] else 1)
PY
}

# Whether the scripted agent's `session/new` carried the Marley agent's `_meta`.
meta_reached_the_agent() {
  python3 - "$E2E_WORK/agent.log" <<'PY'
import json, sys
for line in open(sys.argv[1]):
    message = json.loads(line)
    if message.get("method") == "session/new":
        meta = message.get("params", {}).get("_meta") or {}
        append = (meta.get("systemPrompt") or {}).get("append", "")
        options = (meta.get("claudeCode") or {}).get("options") or {}
        print("  append starts:", append[:80].replace("\n", " "))
        print("  disallowedTools:", options.get("disallowedTools"))
        print("  allowDangerouslySkipPermissions:", options.get("allowDangerouslySkipPermissions"))
        ok = ("Marley's own agent" in append
              and set(options.get("disallowedTools") or []) >= {"Bash", "Edit", "Write", "NotebookEdit", "MultiEdit"}
              and options.get("allowDangerouslySkipPermissions") is False)
        sys.exit(0 if ok else 1)
print("  no session/new in the log")
sys.exit(1)
PY
}

# The project's + menu, then New Agent Thread's submenu.
open_agent_submenu() {
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  press "" Down
  press "" Down
  press "" Right
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== the offer"
  shot 683-01-offer

  echo "== Turn On"
  click "$TURN_ON_X" "$TURN_ON_Y"
  settle 3
  expect "the switch is written on" assistant_is true
  shot 683-02-turned-on

  echo "== the submenu lists Marley"
  open_agent_submenu
  shot 683-03-menu

  echo "== a Marley thread"
  for ((step = 0; step < MARLEY_STEPS; step++)); do
    press "" Down
  done
  settle 1
  press "" Return
  settle 6
  type_text "hello"
  press "" Return
  settle 4
  shot 683-04-thread
  expect "its session/new carried the Marley agent's _meta" meta_reached_the_agent

  echo "== off"
  press "" Escape
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY'
import json, re, sys
path = sys.argv[1]
text = open(path).read()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text)
settings.setdefault("marley", {}).setdefault("assistant", {})["enabled"] = False
json.dump(settings, open(path, "w"), indent=2)
PY
  settle 3
  open_agent_submenu
  shot 683-05-off
  press "" Escape
}
