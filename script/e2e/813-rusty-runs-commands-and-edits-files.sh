# shellcheck shell=bash
# #813's visual check: Rusty's entry runs commands and edits files, the Marley agent stays
# read-only. Both entries run a scripted ACP agent (`MARLEY_ASSISTANT_ADAPTER`) that answers a
# prompt with what its session was given: the entry its instructions name, the tools its `_meta`
# disallows, whether the instructions hold #696's no-commands line and #813's Rusty-tools line,
# and the `INITIAL_AGENT_MODE` and `CODEX_CONFIG` sandbox it was started with. Rusty is on through
# `marley_rusty`'s stand-in `rusty-mcp` (#696's set-up). Conversations start from the palette.
# - On Claude Code, a Rusty conversation: `mcp__marley` alone disallowed, no no-commands line, the
#   Rusty-tools line (`rusty-claude`, REQ-001, REQ-002).
# - A Marley conversation: Claude Code's file and shell tools still disallowed, the no-commands
#   line kept (`marley-claude`, REQ-003).
# - On Codex, from a restart, Rusty's adapter with no `INITIAL_AGENT_MODE` and no sandbox
#   (`rusty-codex`), the Marley entry's read-only both (`marley-codex`, REQ-004).
# - On Zed's agent, the `rusty` profile's built-in tools: Zed's Write tools, Terminal and Edit File
#   among them, through a key bound to `agent::ManageProfiles` for it (`rusty-profile`,
#   `rusty-terminal`, `rusty-edit`, REQ-005).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo" "$E2E_WORK/rusty"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  # A `claude` and a `codex` that say they are signed in and give a version.
  cat >"$bin/claude" <<'SH'
#!/bin/sh
case "$1 $2" in
  "auth status") printf '{"loggedIn": true, "authMethod": "claude.ai"}\n' ;;
  *) printf '2.1.293 (Claude Code)\n' ;;
esac
SH
  cat >"$bin/codex" <<'SH'
#!/bin/sh
case "$1 $2" in
  "login status") printf 'Logged in using ChatGPT\n' >&2 ;;
  *) printf 'codex-cli 0.160.0\n' ;;
esac
SH
  chmod +x "$bin/claude" "$bin/codex"
  export MARLEY_CLAUDE=$bin/claude MARLEY_CODEX=$bin/codex
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/agent.log" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY'
import json, sys
path = sys.argv[1]
settings = json.load(open(path))
marley = settings.setdefault("marley", {})
marley["assistant"] = {"enabled": True, "agent": "claude_code"}
marley["rusty"] = {"enabled": True}
json.dump(settings, open(path, "w"), indent=2)
PY
  # A key for the `rusty` profile's built-in tools, which the palette's Manage Profiles reaches
  # only through the modal's own list.
  printf '%s\n' '[{"bindings": {"ctrl-alt-shift-r": ["agent::ManageProfiles", {"customize_tools": "rusty"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  # The run's profile is a copy of the user's, conversations and all: the copy's list is emptied,
  # so nothing the scenario opens is one of the user's.
  local db
  for db in "$E2E_PROFILE"/db/*/db.sqlite; do
    sqlite3 "$db" "delete from sidebar_threads_v2" 2>/dev/null || true
    sqlite3 "$db" "delete from sidebar_threads" 2>/dev/null || true
  done
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent in the adapter's place for #813's e2e test: it answers each prompt with
# what its session was given, from the `_meta` of `session/new` and its own environment, and logs
# every message it reads and every answer it gives.
import json
import os
import sys

log_path = sys.argv[1]
NO_COMMANDS = "You do not edit files, run commands or change code here"
RUSTY_TOOLS = "through the rusty server's tools"


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def log(message):
    with open(log_path, "a") as log_file:
        log_file.write(json.dumps(message) + "\n")


def summary(meta):
    try:
        codex = json.loads(os.environ.get("CODEX_CONFIG") or "{}")
    except ValueError:
        codex = {}
    options = ((meta or {}).get("claudeCode") or {}).get("options") or {}
    instructions = ((meta or {}).get("systemPrompt") or {}).get("append") \
        or codex.get("developer_instructions", "")
    if instructions.startswith("You are Rusty"):
        entry = "Rusty"
    elif instructions.startswith("You are Marley"):
        entry = "Marley"
    else:
        entry = "unknown"
    disallowed = options.get("disallowedTools")
    if disallowed is None:
        shown = "none sent"
    else:
        own = [tool for tool in disallowed if not tool.startswith("mcp__marley__")]
        each = len(disallowed) - len(own)
        shown = ", ".join(own) or "nothing"
        if each:
            shown += " and %d of Marley's tools by name" % each
    if "CODEX_CONFIG" in os.environ:
        sandbox = str(codex.get("sandbox_mode", "none"))
    else:
        sandbox = "no CODEX_CONFIG"
    lines = [
        "entry: " + entry,
        "disallowed: " + shown,
        "no-commands line: " + ("yes" if NO_COMMANDS in instructions else "no"),
        "rusty-tools line: " + ("yes" if RUSTY_TOOLS in instructions else "no"),
        "INITIAL_AGENT_MODE: " + os.environ.get("INITIAL_AGENT_MODE", "unset"),
        "sandbox: " + sandbox,
    ]
    return "\n".join("- " + line for line in lines)


metas = {}
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
            "agentInfo": {"name": "scripted", "version": "0"},
        }})
    elif method == "session/new":
        sessions += 1
        session = "scripted-%d-%d" % (os.getpid(), sessions)
        metas[session] = params.get("_meta")
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": session}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        text = summary(metas.get(session))
        log({"answer": text})
        send({"jsonrpc": "2.0", "method": "session/update", "params": {
            "sessionId": session,
            "update": {"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": text}},
        }})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

# Whether the scripted agent has started exactly `$1` sessions.
sessions() {
  [[ $(grep -c '"method": "session/new"' "$E2E_WORK/agent.log" 2>/dev/null) -eq $1 ]]
}

# Whether the scripted agent started a session after line `$1` of its log.
session_since() {
  tail -n "+$(($1 + 1))" "$E2E_WORK/agent.log" | grep -q '"method": "session/new"'
}

# Whether the scripted agent's answer number `$1` holds each of the lines after it.
answer_holds() {
  python3 - "$E2E_WORK/agent.log" "$@" <<'PY'
import json, sys
log_path, number, *lines = sys.argv[1:]
answers = [entry["answer"] for entry in map(json.loads, open(log_path)) if "answer" in entry]
if len(answers) < int(number):
    print("  only %d answers" % len(answers))
    sys.exit(1)
answer = answers[int(number) - 1]
print("  " + answer.replace("\n", "\n  "))
sys.exit(0 if all(line in answer for line in lines) else 1)
PY
}

# Whether `settings_read`'s answer in `$1` is a profile whose built-in tools turn on each of the
# tools after it, or, with `none`, turn on none.
profile_tools() {
  python3 - "$@" <<'PY'
import json, sys
path, *wanted = sys.argv[1:]
text = open(path).read()
start = text.find("{")
answer = json.JSONDecoder().raw_decode(text[start:])[0] if start >= 0 else {}
profile = answer.get("global") or answer.get("effective") or {}
on = sorted(name for name, value in (profile.get("tools") or {}).items() if value)
print("  name:", profile.get("name"), "tools on:", on)
if wanted == ["none"]:
    sys.exit(0 if profile.get("name") and not on else 1)
sys.exit(0 if profile.get("name") and set(wanted) <= set(on) else 1)
PY
}

# Runs the palette command `$1`.
palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 4
}

# Sends `$1` to the focused conversation and waits for the answer.
ask() {
  type_text "$1"
  press "" Return
  settle 4
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== Claude Code: a Rusty conversation"
  palette "marley: new rusty conversation"
  settle 2
  expect "Rusty's conversation runs the scripted agent" sessions 1
  ask "what were you given"
  shot 813-01-rusty-claude
  expect "Rusty disallows Marley's tools alone and keeps no no-commands line" answer_holds 1 \
    "- entry: Rusty" "- disallowed: mcp__marley" "- no-commands line: no" "- rusty-tools line: yes"

  echo "== Claude Code: a Marley conversation"
  palette "marley: new marley conversation"
  settle 2
  expect "Marley's conversation runs the scripted agent" sessions 2
  ask "what were you given"
  shot 813-02-marley-claude
  expect "Marley still disallows Claude Code's file and shell tools" answer_holds 2 \
    "- entry: Marley" "- disallowed: Bash, Edit, Write, NotebookEdit, MultiEdit and " \
    "- no-commands line: yes"

  echo "== Codex, after a restart"
  # A fresh start on Codex; #814's scenario checks the switch with Marley running.
  quit_marley
  profile_setting marley.assistant.agent '"codex"'
  launch_marley
  settle 15
  local mark
  mark=$(wc -l <"$E2E_WORK/agent.log")
  palette "marley: new rusty conversation"
  settle 2
  expect "Rusty's Codex conversation runs the scripted agent" session_since "$mark"
  ask "what were you given"
  shot 813-03-rusty-codex
  expect "Rusty on Codex starts with no mode and no sandbox" answer_holds 3 \
    "- entry: Rusty" "- INITIAL_AGENT_MODE: unset" "- sandbox: none" "- no-commands line: no"
  mark=$(wc -l <"$E2E_WORK/agent.log")
  palette "marley: new marley conversation"
  settle 2
  expect "Marley's Codex conversation runs the scripted agent" session_since "$mark"
  ask "what were you given"
  shot 813-04-marley-codex
  expect "Marley on Codex keeps read-only" answer_holds 4 \
    "- entry: Marley" "- INITIAL_AGENT_MODE: read-only" "- sandbox: read-only"

  echo "== Zed's agent: the profiles"
  profile_setting marley.assistant.agent '"zed"'
  settle 4
  mcp_agent tool settings_read '{"key": "agent.profiles.rusty"}' >"$E2E_WORK/rusty-profile.txt" 2>&1
  cat "$E2E_WORK/rusty-profile.txt"
  expect "the rusty profile turns on terminal and edit_file" \
    profile_tools "$E2E_WORK/rusty-profile.txt" terminal edit_file write_file
  mcp_agent tool settings_read '{"key": "agent.profiles.marley"}' >"$E2E_WORK/marley-profile.txt" 2>&1
  cat "$E2E_WORK/marley-profile.txt"
  expect "the marley profile turns on no built-in tool" profile_tools "$E2E_WORK/marley-profile.txt" none
  press "CTRL ALT SHIFT" r
  settle 2
  shot 813-05-rusty-profile
  type_text "terminal"
  settle 1
  shot 813-06-rusty-terminal
  press "CTRL" a
  type_text "edit"
  settle 1
  shot 813-07-rusty-edit
  press "" Escape
}
