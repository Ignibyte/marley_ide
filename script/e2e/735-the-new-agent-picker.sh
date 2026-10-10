# shellcheck shell=bash
# #735's visual check: the New Agent picker asks where. A fake `claude` first on the PATH prints
# and logs the folder it starts in; the Marley entry runs #734's scripted agent, which logs
# `session/new`'s `cwd`. Zed's system path prompts are off, so Browse… is Zed's own picker in the
# run's sway, never the desktop portal on the user's session.
# - With the repo terminal in `sub/`, Ctrl+Alt+N → Marley: Where lists the guess `sub/` first, the
#   open project, recent projects and Browse… (`where`, REQ-001, REQ-002); Enter on the guess opens
#   a thread tab working in `sub/` (`thread`, REQ-004).
# - Ctrl+Alt+N → Claude Code → Browse… → `other/`: the CLI in a new terminal in `other/`
#   (`browse`, REQ-003).
# - The repo terminal's row → Open Agent Here… → Claude Code: the CLI in `sub/`, no Where
#   (`here-terminal`, REQ-005).
# - The project panel's `docs` → Open Agent Here → Claude Code: the CLI in `docs/`
#   (`here-panel`, REQ-006).
# - The project's `+` menu heads its agent part with New Agent… (`plus`, REQ-007).
compositor sway

# The repo terminal's row, the project's `+`, and `docs` in the project panel, as the first run's
# shots found them; how many steps down `docs`'s menu Open Agent Here sits.
TERMINAL_ROW_X=${TERMINAL_ROW_X:-130}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-175}
PROJECT_PLUS_X=${PROJECT_PLUS_X:-236}
PROJECT_PLUS_Y=${PROJECT_PLUS_Y:-136}
DOCS_X=${DOCS_X:-1430}
DOCS_Y=${DOCS_Y:-74}
HERE_STEPS=${HERE_STEPS:-5}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo/sub" "$E2E_WORK/repo/docs" "$E2E_WORK/other"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  echo "export PATH=\"$bin:\$PATH\"" >>"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  printf '# docs\n' >"$E2E_WORK/repo/docs/index.md"
  printf '# sub\n' >"$E2E_WORK/repo/sub/notes.md"
  printf '# notes from other\n' >"$E2E_WORK/other/notes.md"
  # The fake Claude Code: says and logs where it started, then waits as an agent would.
  cat >"$bin/claude" <<FAKE
#!/bin/sh
printf 'fake claude in %s\n' "\$PWD"
printf '%s\n' "\$PWD" >>"$E2E_WORK/claude.log"
exec sleep 600
FAKE
  chmod +x "$bin/claude"
  : >"$E2E_WORK/claude.log"
  # Marley's own search path finds the fake first, as its terminals do.
  export PATH="$bin:$PATH"
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/agent.log" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  profile_setting marley.assistant.enabled true
  profile_setting marley.assistant.agent '"claude_code"'
  profile_setting use_system_path_prompts false
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the Claude adapter's place for #735's e2e test: it answers each prompt
# with "Noted: <prompt>" and logs every message it reads.
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

# Types `$1` into the open picker and waits for its list.
pick() {
  type_text "$1"
  settle 1
}

# Whether the fake `claude` has started `$1` times.
claude_started() {
  [[ $(grep -c . "$E2E_WORK/claude.log") -ge $1 ]]
}

steps() {
  local step
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  type_text "cd sub"
  press "" Return
  settle 2
  shot 735-00-layout

  echo "== Ctrl+Alt+N → Marley: Where, then the guess"
  press "CTRL ALT" n
  settle 2
  pick "Marley"
  press "" Return
  settle 2
  shot 735-01-where
  press "" Return
  settle 6
  expect "the thread runs the scripted agent" holds "$E2E_WORK/agent.log" '"method": "session/new"'
  expect "the thread works in the guessed folder" holds "$E2E_WORK/agent.log" "\"cwd\": \"$E2E_WORK/repo/sub\""
  shot 735-03-thread

  echo "== Ctrl+Alt+N → Claude Code → Browse… → other/"
  press "CTRL ALT" n
  settle 2
  pick "Claude Code"
  press "" Return
  settle 2
  pick "Browse"
  press "" Return
  settle 2
  # Zed's folder prompt starts on a folder of its own; the typed path replaces it.
  press CTRL a
  type_text "$E2E_WORK/other/"
  settle 2
  shot 735-02a-prompt
  press "" Return
  settle 4
  expect "the CLI started in other/" holds "$E2E_WORK/claude.log" "$E2E_WORK/other"
  shot 735-02-browse

  echo "== the repo terminal's row → Open Agent Here…"
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y" right
  settle 1
  press "" Home
  press "" Down
  settle 1
  shot 735-04a-row-menu
  press "" Return
  settle 2
  pick "Claude Code"
  press "" Return
  settle 4
  expect "the CLI started a second time" claude_started 2
  expect "the CLI started in the terminal's folder" holds "$E2E_WORK/claude.log" "$E2E_WORK/repo/sub"
  shot 735-04-here-terminal

  echo "== the project panel's docs → Open Agent Here"
  click "$DOCS_X" "$DOCS_Y" right
  settle 1
  press "" Home
  for ((step = 0; step < HERE_STEPS; step++)); do
    press "" Down
  done
  settle 1
  shot 735-05a-panel-menu
  press "" Return
  settle 2
  pick "Claude Code"
  press "" Return
  settle 4
  expect "the CLI started in docs/" holds "$E2E_WORK/claude.log" "$E2E_WORK/repo/docs"
  shot 735-05-here-panel

  echo "== the project's + menu"
  click "$PROJECT_PLUS_X" "$PROJECT_PLUS_Y"
  settle 1
  shot 735-06-plus
  press "" Escape
}
