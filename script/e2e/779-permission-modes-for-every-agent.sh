# shellcheck shell=bash
# #779's visual check: the permission settings reach every agent's threads. A stand-in ACP agent
# runs as `claude-acp` and `codex-acp` (custom agent servers in the run's settings) and as the
# Marley entry, offering the real adapters' modes (claude-agent-acp's, Bypass Permissions only
# while the session's `_meta` allows it; codex-acp 2.2.2's), answering "Noted: <prompt>", and
# logging every message. `claude_code_permissions` is `bypass` and `codex_permissions`
# `full_access`.
# - A Claude Agent thread starts in Bypass Permissions (`claude-bypass`, REQ-001).
# - Shift+Tab in its box moves it to the next mode (`cycled`, REQ-004), which Zed saves as the
#   agent's default; a second Claude Agent thread still starts in Bypass Permissions
#   (`second-claude`, REQ-006).
# - A Codex thread starts in Full access (`codex-full`, REQ-002).
# - A Marley conversation starts in Bypass Permissions, its `_meta` allowing it
#   (`marley-bypass`, REQ-003).
# - Both settings back to ask: a new Claude Agent thread stays in Default (`ask`, REQ-005).
compositor sway

# The close button of the note Marley shows when its key opens the picker from a terminal, as the
# first run's shots found it.
NOTE_CLOSE_X=${NOTE_CLOSE_X:-1563}
NOTE_CLOSE_Y=${NOTE_CLOSE_Y:-879}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  write_stand_in
  local kind
  for kind in claude codex marley; do
    printf '#!/bin/sh\nexec python3 %s %s %s\n' "$E2E_WORK/stand-in.py" "$kind" "$E2E_WORK/$kind.log" \
      >"$bin/stand-in-$kind"
    chmod +x "$bin/stand-in-$kind"
  done
  export MARLEY_ASSISTANT_ADAPTER=$bin/stand-in-marley
  python3 - "$E2E_PROFILE/config/settings.json" "$bin" <<'PY'
import json, sys
path, bin_dir = sys.argv[1:]
settings = json.load(open(path))
servers = settings.setdefault("agent_servers", {})
for agent, kind in (("claude-acp", "claude"), ("codex-acp", "codex")):
    servers[agent] = {"type": "custom", "command": f"{bin_dir}/stand-in-{kind}", "args": []}
marley = settings.setdefault("marley", {})
marley["assistant"] = {"enabled": True, "agent": "claude_code"}
marley["claude_code_permissions"] = "bypass"
marley["codex_permissions"] = "full_access"
marley["list_cli_sessions"] = False
json.dump(settings, open(path, "w"), indent=2)
PY
  # The run's profile is a copy of the user's, conversations and all: the copy's list is emptied.
  local db
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  open_path "$E2E_WORK/repo"
}

write_stand_in() {
  cat >"$E2E_WORK/stand-in.py" <<'PY'
# A stand-in ACP agent for #779's e2e test, run as `claude` (and as the Marley entry, `marley`) or
# `codex`: it offers that adapter's session modes, takes `session/set_mode`, answers each prompt
# with "Noted: <prompt>", and logs every message it reads.
import json
import os
import sys

kind, log_path = sys.argv[1], sys.argv[2]

CLAUDE = [("default", "Default"), ("acceptEdits", "Accept Edits"), ("plan", "Plan Mode"),
          ("dontAsk", "Don't Ask"), ("bypassPermissions", "Bypass Permissions")]
CODEX = [("read-only", "Read-only"), ("workspace-write", "Workspace access"),
         ("agent", "Auto review"), ("agent-full-access", "Full access")]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def log(message):
    with open(log_path, "a") as log_file:
        log_file.write(json.dumps(message) + "\n")


def modes(params):
    if kind == "codex":
        return CODEX, "agent"
    options = ((params.get("_meta") or {}).get("claudeCode") or {}).get("options") or {}
    allowed = options.get("allowDangerouslySkipPermissions") is not False
    return [mode for mode in CLAUDE if allowed or mode[0] != "bypassPermissions"], "default"


sessions = 0
for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    log(message)
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "protocolVersion": 1,
            "agentCapabilities": {"loadSession": False, "promptCapabilities": {}},
            "authMethods": [],
            "agentInfo": {"name": "stand-in-" + kind, "version": "0"},
        }})
    elif method == "session/new":
        sessions += 1
        available, current = modes(params)
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "sessionId": "%s-%d-%d" % (kind, os.getpid(), sessions),
            "modes": {
                "currentModeId": current,
                "availableModes": [{"id": mode, "name": name} for mode, name in available],
            },
        }})
    elif method == "session/set_mode":
        send({"jsonrpc": "2.0", "id": request_id, "result": {}})
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

# How many `session/set_mode` requests the `$1` stand-in read for mode `$2`.
mode_requests() {
  grep -c "\"method\": \"session/set_mode\".*\"modeId\": \"$2\"" "$E2E_WORK/$1.log" 2>/dev/null || true
}

# Whether the `$1` stand-in read `$3` requests for mode `$2`, or more.
asked_mode() {
  [[ $(mode_requests "$1" "$2") -ge $3 ]]
}

# Whether the Marley entry's last new session allowed bypass in its `_meta`.
marley_allows_bypass() {
  grep '"method": "session/new"' "$E2E_WORK/marley.log" | tail -1 \
    | grep -q '"allowDangerouslySkipPermissions": true'
}

# Runs `$1` from the command palette.
palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Starts a thread of the New Agent picker's entry matching `$1` in this project; with `$2`, a shot
# of the picker by that name first; with `$3`, that many rows down first (the Thread entry under
# a CLI of the same name).
new_thread() {
  local step
  press "CTRL ALT" n
  settle 2
  type_text "$1"
  settle 1
  if [[ -n ${2:-} ]]; then
    shot "$2"
  fi
  for ((step = 0; step < ${3:-0}; step++)); do
    press "" Down
  done
  press "" Return
  settle 2
  press "" Return
  settle 6
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 5

  echo "== a Claude Agent thread"
  new_thread "claude agent" 779-00-picker
  # The note that the key opened Marley's picker from a terminal sits over the thread's mode
  # switch: its close button.
  click "$NOTE_CLOSE_X" "$NOTE_CLOSE_Y"
  settle 1
  shot 779-01-claude-bypass
  expect "the thread asked for Bypass Permissions" asked_mode claude bypassPermissions 1

  echo "== Shift+Tab in its box"
  press "SHIFT" Tab
  settle 2
  shot 779-02-cycled
  expect "Shift+Tab asked for the next mode" asked_mode claude default 1

  echo "== a second Claude Agent thread"
  new_thread "claude agent"
  shot 779-03-second-claude
  expect "the second thread asked for Bypass Permissions" asked_mode claude bypassPermissions 2

  echo "== a Codex thread"
  new_thread "codex" 779-00-codex-picker 1
  shot 779-03-codex-full
  expect "the thread asked for Full access" asked_mode codex agent-full-access 1

  echo "== a Marley conversation"
  palette "marley: new marley conversation"
  settle 6
  shot 779-04-marley-bypass
  expect "Marley's session allowed bypass" marley_allows_bypass
  expect "the conversation asked for Bypass Permissions" asked_mode marley bypassPermissions 1

  echo "== both settings back to ask, a new Claude Agent thread"
  local before
  before=$(mode_requests claude bypassPermissions)
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY'
import json, sys
path = sys.argv[1]
settings = json.load(open(path))
settings["marley"]["claude_code_permissions"] = "ask"
settings["marley"]["codex_permissions"] = "ask"
json.dump(settings, open(path, "w"), indent=2)
PY
  settle 4
  new_thread "claude agent"
  shot 779-05-ask
  expect "no new request for Bypass Permissions" test "$(mode_requests claude bypassPermissions)" -eq "$before"
}
