# shellcheck shell=bash
# #687's visual check, Codex: Marley's own agent for someone signed in to Codex and not to Claude
# Code. A fake `claude` says it is signed out and a fake `codex` (MARLEY_CODEX) that it is signed
# in, so the offer names Codex (`codex-offer`, REQ-001); Turn On writes the switch with the agent
# (REQ-002). `MARLEY_ASSISTANT_ADAPTER` puts a scripted ACP agent in the adapter's place; a Marley
# thread runs it, and it writes the environment it was started with: Codex's read-only mode and
# the config holding the instructions and the read-only sandbox (`codex-thread`, REQ-003). The
# Settings window shows Agent: Codex (`setting`, REQ-004), and the terminal command starts the
# fake `codex` in its read-only sandbox with the instructions (`codex-terminal`, REQ-005).
compositor sway

# The offer's Turn On, the project's +, how many steps down New Agent Thread's submenu Marley
# sits, as the first runs found them.
TURN_ON_X=${TURN_ON_X:-958}
TURN_ON_Y=${TURN_ON_Y:-901}
PLUS_X=${PLUS_X:-224}
PLUS_Y=${PLUS_Y:-123}
MARLEY_STEPS=${MARLEY_STEPS:-3}
# The Settings window's search field, beside the main window under sway.
SEARCH_X=${SEARCH_X:-910}
SEARCH_Y=${SEARCH_Y:-79}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  # A `claude` signed out, and a `codex` signed in that otherwise records its arguments.
  cat >"$bin/claude" <<'SH'
#!/bin/sh
case "$1 $2" in
  "auth status") printf '{"loggedIn": false}\n' ;;
  *) printf '2.1.293 (Claude Code)\n' ;;
esac
SH
  cat >"$bin/codex" <<SH
#!/bin/sh
case "\$1 \$2" in
  "--version ") printf 'codex-cli 0.160.0\n' ; exit 0 ;;
  "login status") printf 'Logged in using ChatGPT\n' >&2 ; exit 0 ;;
esac
printf '%s\n' "\$@" >"$E2E_WORK/codex-args.txt"
echo "fake codex got: \$*"
sleep 600
SH
  chmod +x "$bin/claude" "$bin/codex"
  export MARLEY_CLAUDE=$bin/claude MARLEY_CODEX=$bin/codex
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/agent.log" \
    "$E2E_WORK/agent-env.json" >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  # The run starts undecided, so the offer comes.
  profile_setting marley.assistant '{}'
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Codex adapter's place for #687's e2e test: it writes the variables
# Marley started it with, then answers each prompt.
import json
import os
import sys

log_path, env_path = sys.argv[1], sys.argv[2]
with open(env_path, "w") as env_file:
    json.dump({name: os.environ.get(name) for name in
               ("INITIAL_AGENT_MODE", "CODEX_CONFIG", "CLAUDE_CODE_EXECUTABLE")}, env_file)


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

# Whether the copy's `marley.assistant` holds `enabled: $1` and `agent: $2`.
assistant_is() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'PY'
import json, re, sys
text = open(sys.argv[1]).read()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
assistant = json.loads(text).get("marley", {}).get("assistant", {})
print("  marley.assistant:", assistant)
sys.exit(0 if json.dumps(assistant.get("enabled")) == sys.argv[2]
         and json.dumps(assistant.get("agent")) == sys.argv[3] else 1)
PY
}

# Whether the scripted agent was started in Codex's read-only mode with the Marley agent's config.
started_as_codex() {
  python3 - "$E2E_WORK/agent-env.json" <<'PY'
import json, sys
env = json.load(open(sys.argv[1]))
config = json.loads(env.get("CODEX_CONFIG") or "{}")
instructions = config.get("developer_instructions") or ""
print("  INITIAL_AGENT_MODE:", env.get("INITIAL_AGENT_MODE"))
print("  CODEX_CONFIG keys:", sorted(config), "sandbox_mode:", config.get("sandbox_mode"))
print("  developer_instructions start:", instructions[:60].replace("\n", " "))
print("  CLAUDE_CODE_EXECUTABLE:", env.get("CLAUDE_CODE_EXECUTABLE"))
sys.exit(0 if env.get("INITIAL_AGENT_MODE") == "read-only"
         and config.get("sandbox_mode") == "read-only"
         and instructions.startswith("You are Marley's own agent")
         and "keymap_change" in instructions
         and env.get("CLAUDE_CODE_EXECUTABLE") is None else 1)
PY
}

# Whether the fake `codex` got the read-only sandbox and the instructions as a config value.
codex_arguments_are_right() {
  python3 - "$E2E_WORK/codex-args.txt" <<'PY'
import json, sys
args = open(sys.argv[1]).read().rstrip("\n").split("\n")
print("  arguments:", [arg[:70] for arg in args])
sandbox = "--sandbox" in args and args[args.index("--sandbox") + 1] == "read-only"
values = [args[i + 1] for i, arg in enumerate(args[:-1]) if arg == "-c"]
instructions = [json.loads(value.split("=", 1)[1]) for value in values
                if value.startswith("developer_instructions=")]
print("  sandbox read-only:", sandbox, "instructions given:", len(instructions))
sys.exit(0 if sandbox and instructions and instructions[0].startswith("You are Marley's own agent")
         else 1)
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 2
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== the offer names Codex"
  shot 687-01-codex-offer

  echo "== Turn On"
  click "$TURN_ON_X" "$TURN_ON_Y"
  settle 3
  expect "the switch is written on with Codex" assistant_is true '"codex"'

  echo "== a Marley thread runs the adapter as Codex"
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  press "" Down
  press "" Down
  press "" Right
  settle 1
  shot 687-02a-codex-menu
  for ((step = 0; step < MARLEY_STEPS; step++)); do
    press "" Down
  done
  settle 1
  press "" Return
  settle 6
  # Nothing is typed into a thread unless it runs the stand-in: an entry a step off is a real
  # agent on the user's login.
  expect "the thread runs the stand-in" test -f "$E2E_WORK/agent-env.json"
  type_text "hello"
  press "" Return
  settle 4
  shot 687-02-codex-thread
  expect "the adapter started in Codex's read-only mode with the instructions" started_as_codex
  press "" Escape

  echo "== the Settings window names the agent"
  palette "marley: open settings"
  press "" Return
  settle 4
  click "$SEARCH_X" "$SEARCH_Y"
  settle 1
  type_text "Marley Agent"
  settle 2
  shot 687-03-setting
  press "CTRL" w
  settle 2

  echo "== the terminal command starts Codex read-only"
  palette "open marley agent"
  press "" Return
  settle 6
  shot 687-04-codex-terminal
  expect "the fake codex ran read-only with the instructions" codex_arguments_are_right
}
