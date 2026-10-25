# shellcheck shell=bash
# #905's visual check: tabs and the Ctrl Tab switcher tinted by kind. A scratch repository with
# `rail.rs`; the Marley entry on a scripted ACP agent (`MARLEY_ASSISTANT_ADAPTER`, as #904's) for a
# thread tab; a stand-in `claude` (Python, so Zed names it by its script) first on the terminals'
# PATH through `.bashrc`, never the box's own (PR-687, F-claude-903); a tmux server of the run's
# own (`TMUX_TMPDIR`, F-claude-880) with `build` in `repo`; a Browser tab on a local page.
# - `905-01-tabs`: each tile in its kind's tint, `M` on the thread, `CC` on the agent terminal,
#   `tx` on the tmux tab, one mark on each terminal tab, dots on the thread, Browser, tmux and
#   failed-shell tabs (REQ-001 to REQ-004).
# - `905-02-switcher`: Ctrl Tab open: tinted tiles with their dots, each line "project · state"
#   (REQ-005, REQ-006).
# - `905-03-rail`: the rail's rows on the shared tints (REQ-007).
# - `905-04-day`: the tab bar and the switcher in Marley Day (REQ-001, REQ-005).
compositor sway

. script/e2e/browser-fixture.sh

AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-700}
# The rail's `build` row, as the first run's shots found it.
BUILD_X=${BUILD_X:-130}
BUILD_Y=${BUILD_Y:-232}

# tmux's socket path must stay under 108 bytes, which a run's work folder does not (#808).
SOCKETS=${XDG_RUNTIME_DIR:-/tmp}/marley-e2e-905
LOCAL_TMUX=$SOCKETS/local

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo db
  mkdir -p "$home" "$bin" "$repo/crates/rail" "$LOCAL_TMUX" "$E2E_WORK/site"
  printf "PS1=%q\nexport PATH=%q:\"\$PATH\"\n" '$ ' "$bin" >"$home/.bashrc"
  terminal_env HOME "$home"
  terminal_env TMUX_TMPDIR "$LOCAL_TMUX"
  export TMUX_TMPDIR=$LOCAL_TMUX
  cat >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code: it says so, and waits.
import sys
print("Claude Code (stand-in): " + " ".join(sys.argv[1:]), flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$bin/claude"
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place: it answers each prompt with "Noted: <prompt>".
import json
import os
import sys


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


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
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "tabs-%d" % os.getpid()}})
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
  profile_setting marley.assistant '{"enabled": true, "agent": "claude_code"}'
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  printf '<!doctype html><title>Fixture</title><body><h1>A local page</h1></body>\n' \
    >"$E2E_WORK/site/index.html"
  SITE_PORT=$(serve_site site)
  echo "$SITE_PORT" >"$E2E_WORK/site.port"
  offline_chromium
  TMUX_TMPDIR=$LOCAL_TMUX HOME=$home tmux -f /dev/null new-session -d -s build -c "$repo" \
    -x 100 -y 30
  printf 'fn walk() {}\n' >"$repo/crates/rail/rail.rs"
  git init -q -b main "$repo"
  open_path "$repo"
}

teardown() {
  browser_teardown
  TMUX_TMPDIR=$LOCAL_TMUX tmux kill-server 2>/dev/null || true
  rm -rf "$SOCKETS"
  return 0
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== a shell whose command fails"
  type_text "false"
  press "" Return
  settle 2

  echo "== tmux's build, attached"
  click "$BUILD_X" "$BUILD_Y"
  settle 4
  [ -n "${STOP_AT_TMUX:-}" ] && { away; shot 905-00-probe; return; }

  echo "== an agent terminal"
  palette "workspace: new terminal" 4
  type_text "claude"
  press "" Return
  settle 3

  echo "== a Browser tab"
  palette "marley: open browser" 6
  agent navigate "http://127.0.0.1:$(cat "$E2E_WORK/site.port")/"
  settle 3

  echo "== a Marley thread"
  palette "marley: new marley conversation" 5

  echo "== rail.rs"
  press CTRL p
  settle 2
  type_text "rail.rs"
  settle 2
  press "" Return
  settle 3
  away
  shot 905-01-tabs
  shot 905-03-rail

  echo "== the switcher"
  palette "tab switcher: toggle" 3
  pointer_to 1500 950
  settle 1
  shot 905-02-switcher
  press "" Escape
  settle 1

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  palette "tab switcher: toggle" 3
  pointer_to 1500 950
  settle 1
  shot 905-04-day
  press "" Escape
  settle 1
}
