# shellcheck shell=bash
# #904's visual check: Home's prompt asks Marley or Rusty. Both assistant entries run a scripted
# ACP agent (`MARLEY_ASSISTANT_ADAPTER`, as #738's: "Noted: <prompt>"); Rusty is on through
# `marley_rusty`'s stand-in `rusty-mcp`; stand-in `claude` and `codex` sit first on the terminals'
# PATH, so no real agent CLI runs (PR-687, F-claude-903). Home opens with `marley home`.
# - `904-01-chips`: Marley first and Rusty last among the prompt's chips (REQ-001).
# - `904-02-marley-picked`: Marley picked: its chip lit, the project chip greyed in its place (REQ-002).
# - `904-03-marley-thread`: a task and Enter: a Marley tab with the task sent and answered
#   (REQ-003).
# - `904-04-rusty-thread`: back on Home, Rusty picked, a task and Start: a Rusty tab (REQ-003).
# - `904-05-day`: Marley Day, Home with Rusty picked; the project chip still reads Home, though
#   both conversations' folders joined Home's project hidden (REQ-001, REQ-002, REQ-004).
# - `904-06-cli-home`: Claude Code picked, a task, Enter: the stand-in says it runs in the home
#   folder, not in Marley's settings folder (REQ-004).
compositor sway

AWAY_X=${AWAY_X:-1580}
AWAY_Y=${AWAY_Y:-120}
# Home's prompt box, its Marley and Rusty chips, its Start, and Home's tab, from the first run's
# shots.
BOX_X=${BOX_X:-900}
BOX_Y=${BOX_Y:-218}
MARLEY_X=${MARLEY_X:-642}
MARLEY_Y=${MARLEY_Y:-278}
RUSTY_X=${RUSTY_X:-1158}
CLAUDE_X=${CLAUDE_X:-758}
RUSTY_Y=${RUSTY_Y:-278}
START_X=${START_X:-1327}
START_Y=${START_Y:-278}
HOME_TAB_X=${HOME_TAB_X:-380}
HOME_TAB_Y=${HOME_TAB_Y:-50}

setup() {
  local bin=$E2E_WORK/bin name db
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$E2E_WORK/rusty"
  printf "PS1=%q\nexport PATH=%q:\"\$PATH\"\n" '$ ' "$bin" >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  for name in claude codex; do
    cat >"$bin/$name" <<'SH'
#!/bin/sh
# A stand-in agent CLI: it says what it was asked, and waits.
printf 'stand-in %s in %s was asked: %s\n' "$(basename "$0")" "$PWD" "$*"
exec sleep 600
SH
    chmod +x "$bin/$name"
  done
  export PATH=$bin:$PATH
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #904's check: it answers each prompt with
# "Noted: <prompt>".
import json
import os
import sys


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


sessions = 0
for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
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
        send({"jsonrpc": "2.0", "id": request_id,
              "result": {"sessionId": "home-%d-%d" % (os.getpid(), sessions)}})
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
  printf '#!/bin/sh\nexec python3 %s\n' "$E2E_WORK/scripted-agent.py" >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  profile_setting marley.assistant '{"enabled": true, "agent": "claude_code"}'
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  profile_setting marley.tmux_hosts '[]'
  # The profile is a copy of the user's: its conversations are emptied, so none of theirs shows.
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  git init -q -b main "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 15
  # Trusts repo.
  press "" Return
  settle 4

  echo "== Home's chips"
  launch_marley_with home
  settle 8
  away
  shot 904-01-chips
  [ -n "${STOP_AT_CHIPS:-}" ] && return

  echo "== Marley picked"
  click "$MARLEY_X" "$MARLEY_Y"
  away
  shot 904-02-marley-picked

  echo "== a task for Marley, Enter"
  click "$BOX_X" "$BOX_Y"
  settle 1
  type_text "which theme am I on"
  settle 1
  press "" Return
  settle 8
  away
  shot 904-03-marley-thread

  echo "== Rusty picked, a task, Start"
  click "$HOME_TAB_X" "$HOME_TAB_Y"
  settle 2
  click "$RUSTY_X" "$RUSTY_Y"
  click "$BOX_X" "$BOX_Y"
  settle 1
  type_text "what is on my list today"
  settle 1
  click "$START_X" "$START_Y"
  settle 8
  away
  shot 904-04-rusty-thread

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  click "$HOME_TAB_X" "$HOME_TAB_Y"
  settle 2
  click "$RUSTY_X" "$RUSTY_Y"
  away
  shot 904-05-day

  echo "== Claude Code from Home"
  click "$CLAUDE_X" "$RUSTY_Y"
  click "$BOX_X" "$BOX_Y"
  settle 1
  type_text "tidy the notes"
  settle 1
  press "" Return
  settle 6
  away
  shot 904-06-cli-home
}
