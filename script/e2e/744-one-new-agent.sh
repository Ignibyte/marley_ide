# shellcheck shell=bash
# #744's visual check: one New Agent. #735's set-up: a fake `claude` first on the PATH (so the
# agent CLIs list would show if it were still in the row's `+`), the Marley entry on a scripted
# ACP agent that logs `session/new`'s `cwd`, and the repo terminal moved into `sub/`.
# - The repo row's `+`: New Agent…, no New Agent Thread, no agent CLIs (`row-plus`, REQ-001).
# - New Agent… from it → Marley: Where lists `repo` "this project" first, `sub` second (`where`,
#   REQ-002); Enter starts a Marley thread in a repo tab working in `repo` (`started`, REQ-003).
# - Default shown, the status bar's sparkle: the New Agent picker, no right dock (`sparkle`,
#   REQ-004).
# - The repo shown, `agent: toggle focus` (the sparkle's and Ctrl+?'s action) and `agent: new
#   thread` from the palette: the picker again (`key`, `new-thread`, REQ-005).
# - The repo pane's `+`: New Agent… at its head (`pane-plus`, REQ-006).
compositor sway

# The repo row's `+`, the Default and repo headers, the status bar's sparkle and the pane's `+`,
# as #743's shots found the rail and the bars.
REPO_PLUS_X=${REPO_PLUS_X:-236}
REPO_PLUS_Y=${REPO_PLUS_Y:-136}
DEFAULT_X=${DEFAULT_X:-70}
DEFAULT_Y=${DEFAULT_Y:-89}
REPO_X=${REPO_X:-51}
# Once Default holds its Home page, the repo's header sits a row lower.
REPO_Y=${REPO_Y:-182}
CENTER_X=${CENTER_X:-800}
CENTER_Y=${CENTER_Y:-500}
SPARKLE_X=${SPARKLE_X:-1584}
SPARKLE_Y=${SPARKLE_Y:-985}
PANE_PLUS_X=${PANE_PLUS_X:-1291}
PANE_PLUS_Y=${PANE_PLUS_Y:-50}

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
# A scripted ACP agent in the Claude adapter's place for #744's e2e test: it answers each prompt
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

# Runs `$1` from the command palette.
palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Types `$1` into the open picker and waits for its list.
pick() {
  type_text "$1"
  settle 1
}


steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3
  type_text "cd sub"
  press "" Return
  # #700 makes Default once nothing restored can still bring it back.
  settle 5
  shot 744-00-layout

  echo "== the repo row's +"
  click "$REPO_PLUS_X" "$REPO_PLUS_Y"
  settle 1
  shot 744-01-row-plus

  echo "== New Agent… from it, Marley, Where"
  press "" Home
  press "" Down
  press "" Down
  press "" Return
  settle 2
  pick "Marley"
  press "" Return
  settle 2
  shot 744-02-where
  press "" Return
  settle 6
  expect "the thread runs the scripted agent" holds "$E2E_WORK/agent.log" '"method": "session/new"'
  expect "the thread works in the project's root" holds "$E2E_WORK/agent.log" "\"cwd\": \"$E2E_WORK/repo\""
  shot 744-03-started

  echo "== Default shown, the sparkle"
  click "$DEFAULT_X" "$DEFAULT_Y"
  settle 2
  click "$SPARKLE_X" "$SPARKLE_Y"
  settle 2
  shot 744-04-sparkle

  echo "== the repo shown, Zed's agent actions"
  press "" Escape
  settle 1
  click "$REPO_X" "$REPO_Y"
  settle 2
  # The actions reach the workspace from its center, not from the rail. wtype sends Ctrl+? as
  # Ctrl+Shift+/, which an editor takes for Ctrl+/, so the palette runs the actions the key does.
  click "$CENTER_X" "$CENTER_Y"
  settle 1
  palette "agent: toggle focus"
  settle 2
  shot 744-05-key
  press "" Escape
  settle 1
  palette "agent: new thread"
  settle 2
  shot 744-05b-new-thread

  echo "== the pane's +"
  press "" Escape
  settle 1
  click "$PANE_PLUS_X" "$PANE_PLUS_Y"
  settle 1
  shot 744-06-pane-plus
}
