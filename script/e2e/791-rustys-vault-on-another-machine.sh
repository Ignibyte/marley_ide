# shellcheck shell=bash
# #791's visual check: a brain page's Edit and Rusty's agent folder when Rusty runs on another
# machine. The stand-in `rusty-mcp` with `RUSTY_STAND_IN_ELSEWHERE` names its vault, in
# `brain_stats`'s `vault_root` and each page's `file`, at a folder under the run that is never made,
# as a Rusty on another machine names a folder this one does not have; its pages stay in its own
# scratch vault, which Marley is never told of. The Marley and Rusty entries on #780's scripted ACP
# agent, which logs each `session/new` with its `cwd`. Rusty on, `agent_folder` unset.
#
# Keys only: the run's keymap opens Sam's page (Ctrl-Alt-Shift-Y) and toggles Edit
# (Ctrl-Alt-Shift-U).
# - Edit: Sam's whole file, frontmatter first, in the tab (`791-01-edit`, REQ-001).
# - A typed line, the unsaved dot (`791-02-typed`); Ctrl-S writes it with `brain_write_page`, the
#   dot gone (`791-03-saved`, REQ-002).
# - Another line, then Read: saved through Rusty, the page read with both lines (`791-04-read`,
#   REQ-004).
# - Edit, a third line, a change to the page from outside, Ctrl-S: the tab says the page changed in
#   Rusty, the dot stays, the file keeps the outside change, and Zed's "Failed to save" prompt says
#   the same (`791-05-changed`, REQ-003).
# - New Agent, Rusty: its `cwd` is the home folder, and the elsewhere folder was not made
#   (`791-06-rusty-agent`, REQ-005).
compositor sway

TYPED="A line typed on a machine without the vault."
READ_LINE="A line saved by Read."
THIRD="A line Rusty never gets."
OUTSIDE="A line Rusty wrote meanwhile."

# The stand-in's scratch vault, and the folder it names as its vault's.
vault() { echo "$E2E_WORK/rusty/vault"; }
elsewhere() { echo "$E2E_WORK/elsewhere/vault"; }

sam() { echo "$(vault)/people/sam.md"; }

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo" "$(vault)/people" "$(vault)/archive"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  cat >"$(sam)" <<'PAGE'
---
title: Sam
type: person
---
# Sam

Sam keeps the renderer honest.
PAGE
  write_scripted_agent
  printf '#!/bin/sh\nexec python3 %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/agent.log" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  export RUSTY_STAND_IN_ELSEWHERE
  RUSTY_STAND_IN_ELSEWHERE=$(elsewhere)
  printf '%s\n' '[{"context": "Workspace", "bindings": {"ctrl-alt-shift-y": ["rusty::OpenPage", {"slug": "people/sam"}], "ctrl-alt-shift-u": "rusty::TogglePageEdit"}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  python3 - "$E2E_PROFILE/config/settings.json" <<'PY2'
import json, sys
path = sys.argv[1]
settings = json.load(open(path))
marley = settings.setdefault("marley", {})
marley["assistant"] = {"enabled": True, "agent": "claude_code"}
# Rusty on, its own settings dropped: no `agent_folder`.
marley["rusty"] = {"enabled": True}
marley["list_cli_sessions"] = False
# A user's own `agent_servers.Rusty` (Zed writes a registry entry there when a mode is made the
# default) stands in place of the entry Marley gives, and no registry agent has that name.
(settings.get("agent_servers") or {}).pop("Rusty", None)
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
# A scripted ACP agent in the Claude adapter's place for #791's e2e test, #780's cut down: it
# answers each prompt with "Noted: <prompt>" and logs every message it reads, each `session/new`
# with its `cwd`.
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
            "agentCapabilities": {"loadSession": False, "promptCapabilities": {}},
            "authMethods": [],
            "agentInfo": {"name": "scripted", "version": "0"},
        }})
    elif method == "session/new":
        sessions += 1
        send({"jsonrpc": "2.0", "id": request_id,
              "result": {"sessionId": "marley-%d-%d" % (os.getpid(), sessions)}})
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

# Whether the agent's newest `session/new` asked for `cwd` `$1`.
started_in() {
  python3 - "$E2E_WORK/agent.log" "$1" <<'PY2'
import json, os, sys
log_path, want = sys.argv[1:]
last = None
for line in open(log_path):
    message = json.loads(line)
    if message.get("method") == "session/new":
        last = (message.get("params") or {}).get("cwd")
print("session/new cwd:", last)
sys.exit(0 if last and os.path.realpath(last) == os.path.realpath(want) else 1)
PY2
}

# How many times the stand-in was asked to write a page.
writes() { grep -c 'tools/call brain_write_page' "$E2E_WORK/rusty/calls" || true; }

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 5
  expect "Marley asked Rusty where its vault is" grep -q 'tools/call brain_stats' "$E2E_WORK/rusty/calls"

  echo "== Edit, with the vault elsewhere"
  press "CTRL ALT SHIFT" y
  settle 4
  press "CTRL ALT SHIFT" u
  settle 3
  shot 791-01-edit

  echo "== a typed line, Ctrl-S"
  press "CTRL" End
  press "" Return
  type_text "$TYPED"
  settle 2
  shot 791-02-typed
  press "CTRL" s
  settle 3
  shot 791-03-saved
  expect "the stand-in was asked to write the page once" test "$(writes)" -eq 1
  expect "Sam's file holds the typed line" grep -qF "$TYPED" "$(sam)"

  echo "== another line, Read saves"
  press "" Return
  type_text "$READ_LINE"
  settle 1
  press "CTRL ALT SHIFT" u
  settle 4
  shot 791-04-read
  expect "Read wrote the page through Rusty" test "$(writes)" -eq 2
  expect "Sam's file holds the line Read saved" grep -qF "$READ_LINE" "$(sam)"

  echo "== a change in Rusty meanwhile"
  press "CTRL ALT SHIFT" u
  settle 3
  press "CTRL" End
  press "" Return
  type_text "$THIRD"
  settle 1
  printf '%s\n' "$OUTSIDE" >>"$(sam)"
  settle 2
  press "CTRL" s
  settle 3
  shot 791-05-changed
  expect "nothing more was written" test "$(writes)" -eq 2
  expect "Sam's file keeps the outside change" grep -qF "$OUTSIDE" "$(sam)"
  expect "Sam's file lacks the refused line" bash -c "! grep -qF '$THIRD' '$(sam)'"
  # Zed's own "Failed to save" prompt after a Ctrl-S that failed: OK.
  press "" Return
  settle 2

  echo "== New Agent, Rusty"
  press "CTRL ALT" n
  settle 2
  type_text "Rusty"
  settle 1
  press "" Return
  settle 6
  shot 791-06-rusty-agent
  expect "Rusty started in the home folder" started_in "$HOME"
  expect "the elsewhere folder was not made" test ! -e "$E2E_WORK/elsewhere"
  grep -o 'tools/call brain_[a-z_]* {[^}]*}' "$E2E_WORK/rusty/calls" | cut -c1-120 | sort | uniq -c
}
