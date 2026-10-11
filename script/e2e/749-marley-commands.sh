# shellcheck shell=bash
# #749's visual check: `marley <command>` typed in a terminal of the run's Marley. The repo has
# `sub/` with `page.html`; `other/` sits beside it, in no project. A scripted ACP agent is the
# profile's `Stand-in` agent server and the Marley entry's adapter, and logs each `session/new`'s
# `cwd`; a fake `claude` prints its folder; Rusty is on through its stand-in; `$E2E_WORK/bin/marley`
# is the debug binary, first on the terminals' PATH.
# - The terminal names the run's binary and profile before any verb runs (`env`, REQ-013).
# - `marley help`: the verbs (`help`, REQ-001); a wrong provider: the error and `exit 2` (`error`,
#   REQ-002).
# - From `sub/`, `marley agent-thread --provider=Stand-in`: a Stand-in thread in the repo group,
#   its session in `sub` (`thread`, REQ-003, REQ-004, REQ-005).
# - `--provider=marley`: a Marley thread (`short-name`, REQ-005); no provider: Marley again, the
#   agent started last (`default-provider`, REQ-006).
# - `--folder=<other>`: Default shown, the session in `other` (`default-group`, REQ-007).
# - `--provider=nobody`: a toast naming it (`unknown-agent`, REQ-008).
# - `marley agent-cli --provider=claude`: the fake in a new terminal in `sub` (`cli`, REQ-009).
# - `marley terminal ..`: a terminal at the repo's root (`terminal`, REQ-010).
# - `talk marley`, `threads`, `home`, `rusty brain`, `panel git`, `browser page.html`: each one's tab
#   in the repo group (REQ-011).
# - Marley quit; `marley agent-thread --provider=Stand-in --folder=<sub>` launched: Marley starts
#   with the repo group and a Stand-in thread in it (`cold`, REQ-012). A debug build reads its
#   assets from the checkout, so this launch starts there and names the folder; the installed
#   build embeds them and starts anywhere.
compositor sway

# The repo's terminal row in the rail, as the first run's shots found it; `STOP_AT=<shot>` ends the
# run after that shot.
TERMINAL_ROW_X=${TERMINAL_ROW_X:-95}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-176}
# Default's fold button, which puts the repo's rows back in place once Default has rows of its own.
DEFAULT_FOLD_X=${DEFAULT_FOLD_X:-24}
DEFAULT_FOLD_Y=${DEFAULT_FOLD_Y:-89}
STOP_AT=${STOP_AT:-}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo/sub" "$E2E_WORK/other" "$E2E_WORK/rusty/vault"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  printf '# repo\n' >"$repo/README.md"
  printf '<!doctype html><title>Sub page</title><h1>The sub page</h1>\n' >"$repo/sub/page.html"
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/agent.log" \
    >"$bin/scripted-agent"
  cat >"$bin/claude" <<'SH'
#!/bin/sh
echo "fake claude in $(pwd)"
exec sleep 600
SH
  chmod +x "$bin/scripted-agent" "$bin/claude"
  ln -s "$(marley_binary)" "$bin/marley"
  terminal_env HOME "$home"
  terminal_env PATH "$bin:$PATH"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  profile_setting marley.assistant.enabled true
  profile_setting marley.assistant.agent '"claude_code"'
  profile_setting agent_servers.Stand-in "{\"type\": \"custom\", \"command\": \"$bin/scripted-agent\", \"args\": []}"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  printf -- '---\ntitle: Start\n---\n# Start\n' >"$E2E_WORK/rusty/vault/start.md"
  # The run's profile is a copy of the user's, conversations and all: the copy's list is emptied,
  # so `marley talk marley` finds only the run's own (L-claude-738).
  local db
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  open_path "$repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent for #749's e2e test: it answers each prompt "Noted: <prompt>", loads a
# session as an empty one, and logs every message it reads, `session/new`'s `cwd` among them.
import json
import os
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
            "agentCapabilities": {"loadSession": True, "promptCapabilities": {}},
            "authMethods": [],
            "agentInfo": {"name": "scripted", "version": "0"},
        }})
    elif method == "session/new":
        sessions += 1
        send({"jsonrpc": "2.0", "id": request_id,
              "result": {"sessionId": "scripted-%d-%d" % (os.getpid(), sessions)}})
    elif method == "session/load":
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

# Whether the scripted agent's log holds at least `$1` sessions started in the folder `$2`.
sessions_in() {
  local count
  count=$(python3 - "$E2E_WORK/agent.log" "$2" <<'COUNT'
import json, sys
count = 0
for line in open(sys.argv[1]):
    message = json.loads(line)
    if message.get("method") == "session/new" and (message.get("params") or {}).get("cwd") == sys.argv[2]:
        count += 1
print(count)
COUNT
)
  echo "sessions in $2: $count"
  [[ $count -ge $1 ]]
}

# Whether the terminal named the run's binary and the run's profile.
names_the_run() {
  local seen
  seen=$(cat "$E2E_WORK/which.txt" 2>/dev/null)
  echo "the terminal named: ${seen//$'\n'/ | }"
  [[ $seen == "$E2E_WORK/bin/marley"$'\n'"$(realpath "$E2E_PROFILE")" || $seen == "$E2E_WORK/bin/marley"$'\n'"$E2E_PROFILE" ]]
}

# Types `$1` into the repo's terminal, which the rail's row brings forward, and runs it.
run_in_terminal() {
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y"
  settle 1
  type_text "$1"
  press "" Return
}

# Shoots `$1`, and ends the run there when `STOP_AT` names it.
take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the terminal's marley and data directory"
  run_in_terminal "{ command -v marley; echo \"\$MARLEY_USER_DATA_DIR\"; } > $E2E_WORK/which.txt"
  settle 2
  take 749-00-env || return 0
  expect "the terminal names the run's binary and profile" names_the_run || return 1

  echo "== marley help, then a wrong provider"
  run_in_terminal "clear; marley help"
  settle 2
  take 749-01-help || return 0
  run_in_terminal 'clear; marley agent-cli --provider=nope; echo "exit $?"'
  settle 2
  take 749-02-error || return 0

  echo "== a Stand-in thread from sub/"
  run_in_terminal "cd $E2E_WORK/repo/sub && marley agent-thread --provider=Stand-in"
  settle 8
  take 749-03-thread || return 0
  expect "a Stand-in session in sub" sessions_in 1 "$E2E_WORK/repo/sub"

  echo "== --provider=marley, then no provider"
  run_in_terminal "cd $E2E_WORK/repo/sub && marley agent-thread --provider=marley"
  settle 8
  take 749-04-short-name || return 0
  expect "a Marley session in sub" sessions_in 2 "$E2E_WORK/repo/sub"
  run_in_terminal "cd $E2E_WORK/repo/sub && marley agent-thread"
  settle 8
  take 749-05-default-provider || return 0
  expect "the agent started last, in sub again" sessions_in 3 "$E2E_WORK/repo/sub"

  echo "== a folder no project holds"
  run_in_terminal "marley agent-thread --provider=Stand-in --folder=$E2E_WORK/other"
  settle 8
  take 749-06-default-group || return 0
  expect "a Stand-in session in other" sessions_in 1 "$E2E_WORK/other"
  click "$DEFAULT_FOLD_X" "$DEFAULT_FOLD_Y"
  settle 1

  echo "== an agent no one has"
  run_in_terminal "marley agent-thread --provider=nobody"
  settle 3
  take 749-07-unknown-agent || return 0

  echo "== the claude CLI"
  run_in_terminal "cd $E2E_WORK/repo/sub && marley agent-cli --provider=claude"
  settle 5
  take 749-08-cli || return 0

  echo "== a terminal at the repo's root"
  run_in_terminal "cd $E2E_WORK/repo/sub && marley terminal .."
  settle 4
  take 749-09-terminal || return 0

  echo "== talk, threads, home, rusty, panel, browser"
  run_in_terminal "marley talk marley"
  settle 6
  take 749-10-talk || return 0
  run_in_terminal "marley threads"
  settle 3
  take 749-11-threads || return 0
  run_in_terminal "marley home"
  settle 3
  take 749-12-home || return 0
  run_in_terminal "marley rusty brain"
  settle 5
  take 749-13-rusty || return 0
  run_in_terminal "marley panel git"
  settle 3
  take 749-14-panel || return 0
  run_in_terminal "cd $E2E_WORK/repo/sub && marley browser page.html"
  settle 6
  take 749-15-browser || return 0

  echo "== a cold start for sub/"
  quit_marley
  # From the checkout, whose assets a debug build reads, with the folder named.
  launch_marley_with agent-thread --provider=Stand-in "--folder=$E2E_WORK/repo/sub"
  settle 20
  take 749-16-cold || return 0
  expect "a Stand-in session in sub after the cold start" sessions_in 4 "$E2E_WORK/repo/sub"
}
