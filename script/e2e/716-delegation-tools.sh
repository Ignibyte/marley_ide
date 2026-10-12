# shellcheck shell=bash
# #716's visual check: agents that start agents, tracked through the harness. A real runtime of the
# harness's built `rh` (with its TICKET-117: seat labels, `session_label`, a delivery's `ended`) on
# a scratch root is the harness Marley follows, with harness writes on. A stand-in `rh` answers
# `seat add NAME … --label K=V …` by writing an actor profile NAME carrying the labels, so the real
# `seat start` opens a session that publishes them; the actor takes its prompt, prints it, works
# until the scenario sends it `done` through `rh actor send`, then goes idle, which ends the send.
# The caller is #522's stand-in Claude Code in the project's terminal (Marley's plugin makes it
# idle, and it echoes each line it reads); on a line written to a FIFO it calls `agent_start` or
# `agent_status` through the plugin's bridge from inside its terminal, or turns busy or idle.
# Shots: the ask card names Claude Code, the folder and the prompt (`716-01-card`, REQ-001); after
# Allow for This Session, the seat's terminal beside the caller and its row in the Harness section
# with "for <caller>" (`716-02-started`, REQ-002, REQ-003); the child ends while the caller is busy:
# no pointer (`716-03a-busy`); the caller idle: the pointer (`716-03-pointer`, REQ-004);
# `agent_status`'s answer (`716-05-status`, REQ-006); a second delegation whose caller's terminal is
# closed before the child ends: Needs you lists it (`716-06-closed`, REQ-007).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
RH=${RH:-/srv/stacks/rustal-harness/bin/rh}
# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
ALLOW_X=${ALLOW_X:-1225}
ALLOW_Y=${ALLOW_Y:-933}
CALLER_TAB_X=${CALLER_TAB_X:-470}
CALLER_TAB_Y=${CALLER_TAB_Y:-51}
AWAY_X=${AWAY_X:-1700}
AWAY_Y=${AWAY_Y:-1000}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

# The harness's CLI on the scratch root, outside any tmux.
rh() { env -u TMUX -u TMUX_PANE "$RH" --state "$ROOT" "$@"; }

bytes() { python3 -c 'import json, sys; print(json.dumps(list(sys.argv[1].encode())))' "$1"; }

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config repo=$E2E_WORK/repo tries=50
  [[ -x /srv/stacks/rustal-harness/target/debug/rh ]] || {
    echo "no built rh: build the harness first (its scripts/setup.sh)" >&2
    return 1
  }
  # A short root: the harness's sockets live under it.
  ROOT=$XDG_RUNTIME_DIR/rh716-$$
  echo "$ROOT" >"$E2E_WORK/root"
  mkdir -m 0700 "$ROOT"
  (rh serve >"$ROOT/serve.log" 2>&1 &)
  until grep -q '"ready":true' "$ROOT/serve.log" 2>/dev/null || ((tries-- == 0)); do
    sleep 0.2
  done
  # The child: idle at a message wait, which takes the prompt; then working until `done`; then
  # idle, which ends the send.
  cat >"$ROOT/child.json" <<JSON
{"version": 1, "steps": [
  {"step": "publish", "state": "idle", "value": {}},
  {"step": "wait", "key": "task"},
  {"step": "output", "bytes": $(bytes $'working on the task\n')},
  {"step": "publish", "state": "running", "value": {}},
  {"step": "wait", "key": "done"},
  {"step": "output", "bytes": $(bytes $'the task is done\n')},
  {"step": "publish", "state": "idle", "value": {}},
  {"step": "sleep", "millis": 60000}, {"step": "sleep", "millis": 60000},
  {"step": "sleep", "millis": 60000}]}
JSON
  mkdir -p "$home" "$bin" "$config/plugins" "$repo"
  cat >"$bin/rh" <<SH
#!/bin/bash
# The stand-in: \`seat add\` writes an actor profile with the labels it was given; everything else,
# \`seat start\` among it, goes to the harness's own rh.
if [[ \$3 == seat && \$4 == add ]]; then
  printf '%s\n' "\$*" >>"$E2E_WORK/seat.log"
  exec python3 - "\$@" <<'PY'
import json, os, sys
args = sys.argv[1:]
root, name = args[1], args[4]
labels, cwd = {}, root
for i, word in enumerate(args):
    if word == "--label":
        key, _, value = args[i + 1].partition("=")
        labels[key] = value
    if word == "--cwd":
        cwd = args[i + 1]
os.makedirs(root + "/profiles", mode=0o700, exist_ok=True)
path = f"{root}/profiles/{name}.json"
if os.path.exists(path):
    print(f"rh: seat_exists: a profile named {name} exists", file=sys.stderr)
    sys.exit(1)
with open(path, "w") as profile:
    json.dump({"version": 1, "kind": "actor", "cwd": cwd, "script": root + "/child.json", "labels": labels}, profile)
os.chmod(path, 0o600)
print(json.dumps({"seat": name, "profile": path, "kind": "actor", "cwd": cwd}))
PY
fi
exec env -u TMUX -u TMUX_PANE "$RH" "\$@"
SH
  chmod +x "$bin/rh"
  : >"$E2E_WORK/seat.log"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/$SHIPPED", "version": "$SHIPPED"}]}}
JSON
  export CLAUDE_CONFIG_DIR=$config
  write_mcp_agent
  : >"$E2E_WORK/answers.log"
  mkfifo "$E2E_WORK/cues.fifo"
  write_caller "$bin/claude"
  printf '# repo\n' >"$repo/README.md"
  profile_setting marley.harness "{\"command\": \"$bin/rh\", \"args\": [\"--state\", \"$ROOT\", \"mcp\", \"--grant\", \"write\"]}"
  profile_setting marley.harness_writes true
  open_path "$repo"
}

teardown() {
  [[ -f $E2E_WORK/root ]] || return 0
  ROOT=$(cat "$E2E_WORK/root")
  chmod 0700 "$ROOT" 2>/dev/null
  local ws
  for ws in $(rh list 2>/dev/null | python3 -c 'import json, sys
print(" ".join(w["id"] for w in json.load(sys.stdin).get("workspaces", []) if w.get("state") == "running"))'); do
    rh stop "$ws" >/dev/null 2>&1
  done
  rh shutdown --stop-backend >/dev/null 2>&1
  rm -rf "${ROOT:?}"
  return 0
}

# The caller: a stand-in Claude Code that reports through Marley's plugin hook, echoes each line
# it reads, and acts on each line of the FIFO: `start` and `status` call the tools through the
# plugin's bridge from this terminal, `busy` and `idle` report a prompt and a stop.
write_caller() {
  sed -e "s|@HOOK@|$HOOK|" -e "s|@FIFO@|$E2E_WORK/cues.fifo|" -e "s|@LOG@|$E2E_WORK/answers.log|" \
    -e "s|@AGENT@|$E2E_WORK/mcp-agent.py|" -e "s|@BRIDGE@|$PWD/crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge|" \
    -e "s|@ENDPOINT@|$E2E_PROFILE/mcp-endpoint.json|" -e "s|@STARTED@|$E2E_WORK/started|" >"$1" <<'FAKE'
#!/usr/bin/env python3
import json
import os
import subprocess
import sys
import threading
import uuid

if sys.argv[1:2] == ["plugin"]:
    sys.exit(0)
# A session id in Claude Code's form, so Marley resumes it after a restart (#540).
ARGS = sys.argv[1:]
SESSION = ARGS[ARGS.index("--resume") + 1] if "--resume" in ARGS else str(uuid.uuid4())
REQUESTS = ["2b0f6a4e-6f3c-4a59-9b41-3c1f2d9e7a10", "6d1c8e2a-0b7f-4e3d-a5c9-1f8e7d6c5b40",
            "9a7e3b51-2c4d-4f60-8e1a-7b5c3d2e1f00"]
# Which request the next `start` takes, kept across a resume.
STARTED = "@STARTED@"


def hook(event):
    event.update({"cwd": os.getcwd(), "session_id": SESSION, "permission_mode": "default"})
    answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps(event),
                            capture_output=True, text=True, check=False).stdout
    sequence = json.loads(answer or "{}").get("terminalSequence")
    if sequence:
        sys.stdout.write(sequence)
        sys.stdout.flush()


def tool(name, arguments):
    env = dict(os.environ, BRIDGE="@BRIDGE@", MARLEY_MCP_ENDPOINT="@ENDPOINT@")
    done = subprocess.run([sys.executable, "@AGENT@", "tool", name, json.dumps(arguments)],
                          capture_output=True, text=True, check=False, env=env)
    with open("@LOG@", "a", encoding="utf-8") as log:
        log.write(f"{name}: {done.stdout.strip()} {done.stderr.strip()}\n")


def cues():
    while True:
        with open("@FIFO@", encoding="utf-8") as fifo:
            for line in fifo:
                cue = line.strip()
                if cue == "start":
                    started = len(open(STARTED).read().split()) if os.path.exists(STARTED) else 0
                    with open(STARTED, "a") as count:
                        count.write("x\n")
                    tool("agent_start", {"agent": "claude", "prompt": "Write the tests for the parser",
                                         "place": "here", "view": "terminal",
                                         "request_id": REQUESTS[started]})
                elif cue == "status":
                    tool("agent_status", {"delegation": REQUESTS[0]})
                elif cue == "busy":
                    hook({"hook_event_name": "UserPromptSubmit", "prompt": "Keep going"})
                elif cue == "idle":
                    hook({"hook_event_name": "Stop", "last_assistant_message": "Done."})


print("stand-in Claude Code, idle, session " + SESSION, flush=True)
hook({"hook_event_name": "SessionStart", "source": "resume" if "--resume" in ARGS else "startup"})
threading.Thread(target=cues, daemon=True).start()
for line in sys.stdin:
    print("got: " + line.rstrip("\n"), flush=True)
FAKE
  chmod +x "$1"
}

cue() {
  echo "$1" >"$E2E_WORK/cues.fifo"
  settle "${2:-2}"
}

# Sends the child seat whose name starts with `$1` its `done`, through the harness's own fixture
# command: the actor goes by the session's title.
child_done() {
  local actor incarnation
  read -r actor incarnation < <(rh actor list | python3 -c 'import json, sys
actor = next(a for a in json.load(sys.stdin)["actors"] if a["name"].startswith(sys.argv[1]))
print(actor["name"], actor["incarnations"][-1]["id"])' "$1")
  rh actor send "$actor" --incarnation "$incarnation" --key "done" \
    --delivery "$(python3 -c 'import uuid; print(uuid.uuid4())')" "finish"
}

# The caller's terminal, read as the user would.
caller_lines() { mcp_agent terminal-read claude >"$E2E_WORK/caller.txt" 2>&1 || true; }

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the caller: a stand-in Claude Code, idle"
  type_text "claude"
  press "" Return
  settle 4

  echo "== agent_start from the caller's terminal asks first"
  cue start 3
  take 716-01-card || return 0
  if ((ALLOW_X == 0)); then
    echo "the card's Allow is not placed yet: 716-01 is the last shot"
    return 0
  fi

  echo "== Allow: the seat starts beside the caller, its row names the caller"
  click "$ALLOW_X" "$ALLOW_Y"
  settle 15
  pointer_to "$AWAY_X" "$AWAY_Y"
  take 716-02-started || return 0
  cat "$E2E_WORK/answers.log" "$E2E_WORK/seat.log"
  expect "agent_start answered starting" holds "$E2E_WORK/answers.log" '"result": "starting"'
  expect "the seat was added with its labels" holds "$E2E_WORK/seat.log" "--label started_by=marley:" "--label delegation=2b0f6a4e"

  echo "== the child ends while the caller is busy: no pointer"
  cue busy 2
  child_done claude-repo
  settle 4
  caller_lines
  expect "no pointer while the caller is busy" bash -c "! grep -q 'got: Agent' '$E2E_WORK/caller.txt'"
  take 716-03a-busy || return 0

  echo "== the caller idle: the pointer"
  cue idle 5
  caller_lines
  cat "$E2E_WORK/caller.txt"
  expect "the caller got the pointer" holds "$E2E_WORK/caller.txt" "got: Agent claude-repo-" \
    'ended: finished. Read the result: agent_status {"delegation": "2b0f6a4e-6f3c-4a59-9b41-3c1f2d9e7a10"}.'
  take 716-03-pointer || return 0

  echo "== agent_status"
  cue status 3
  cat "$E2E_WORK/answers.log"
  expect "agent_status reads the ended delivery" holds "$E2E_WORK/answers.log" '"delivery": "ended"' '"outcome": "finished"' "working on the task" '"told": true'
  take 716-05-status || return 0
  if ((CALLER_TAB_X == 0)); then
    echo "the caller's tab is not placed yet: 716-05 is the last shot"
    return 0
  fi

  echo "== a second delegation; Marley quits; the child ends; Marley starts again"
  cue start 3
  click "$ALLOW_X" "$ALLOW_Y"
  settle 15
  quit_marley
  child_done claude-repo-2
  settle 3
  launch_marley
  settle 25
  caller_lines
  cat "$E2E_WORK/caller.txt"
  expect "the restored caller got the second pointer" holds "$E2E_WORK/caller.txt" \
    'ended: finished. Read the result: agent_status {"delegation": "6d1c8e2a-0b7f-4e3d-a5c9-1f8e7d6c5b40"}.'
  # The restored scrollback may show the first pointer once; it is never typed a second time.
  expect "and the first pointer was not typed again" \
    bash -c "(( \$(grep -c 'got: Agent.*2b0f6a4e-6f3c-4a59-9b41-3c1f2d9e7a10' '$E2E_WORK/caller.txt') <= 1 ))"
  pointer_to "$AWAY_X" "$AWAY_Y"
  take 716-04-restart || return 0

  echo "== a third delegation whose caller closes before the child ends"
  cue start 3
  click "$ALLOW_X" "$ALLOW_Y"
  settle 15
  click "$CALLER_TAB_X" "$CALLER_TAB_Y"
  settle 1
  palette "pane: close active item"
  settle 3
  child_done claude-repo-3
  settle 15
  pointer_to "$AWAY_X" "$AWAY_Y"
  take 716-06-closed || return 0
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}
