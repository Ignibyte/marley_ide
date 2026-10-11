# shellcheck shell=bash
# #742's visual check: a thread on a remote seat. A real runtime of the harness's built `rh` on a
# scratch root is `marley.harnesses`' `box-2`, reached through #741's fake `ssh`, which logs its
# arguments, drops everything up to the destination and runs the rest in a shell. A stand-in `rh`
# answers `seat add NAME` by writing an actor profile NAME, so the real `seat start NAME` opens a
# session the fleet labels with its profile, and answers `acp --seat NAME` with a scripted ACP agent
# that replies "Seat NAME: <prompt>" as rustal-harness's window onto a seat does (its TICKET-116;
# the real window needs the pinned Claude Code, which its own suite runs). A seat `claude-other`
# is started in setup.
# - Ctrl+Alt+N → Claude Agent → On box-2… → `/srv/web`: a thread tab on `claude-web · box-2`, its
#   agent `ssh … -- box-2 … acp --seat claude-web` (`thread`, REQ-001).
# - "hello" in the tab: the seat's reply in the thread (`reply`, REQ-002).
# - The `claude-other` row → Open as Thread: a thread tab on `claude-other · box-2`
#   (`open-as-thread`, REQ-003).
compositor sway

# The `claude-other` row in the rail, as the first run's shots found it.
OTHER_ROW_X=${OTHER_ROW_X:-90}
OTHER_ROW_Y=${OTHER_ROW_Y:-306}

RH=${RH:-/srv/stacks/rustal-harness/bin/rh}

# The harness's CLI on the scratch root, outside any tmux.
rh() { env -u TMUX -u TMUX_PANE "$RH" --state "$ROOT" "$@"; }

bytes() { python3 -c 'import json, sys; print(json.dumps(list(sys.argv[1].encode())))' "$1"; }

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin tries=50
  [[ -x /srv/stacks/rustal-harness/target/debug/rh ]] || {
    echo "no built rh: build the harness first (its scripts/setup.sh)" >&2
    return 1
  }
  # A short root: the harness's sockets live under it.
  ROOT=$XDG_RUNTIME_DIR/rh742-$$
  echo "$ROOT" >"$E2E_WORK/root"
  mkdir -m 0700 "$ROOT"
  (rh serve >"$ROOT/serve.log" 2>&1 &)
  until grep -q '"ready":true' "$ROOT/serve.log" 2>/dev/null || ((tries-- == 0)); do
    sleep 0.2
  done
  cat >"$ROOT/seat.json" <<JSON
{"version": 1, "steps": [
  {"step": "output", "bytes": $(bytes $'the seat is up\n')},
  {"step": "publish", "state": "running", "value": {}},
  {"step": "sleep", "millis": 60000}, {"step": "sleep", "millis": 60000},
  {"step": "sleep", "millis": 60000}]}
JSON
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
  write_window
  cat >"$bin/rh" <<SH
#!/bin/bash
# The stand-in: \`seat add\` writes an actor profile, \`acp --seat\` is the scripted window, and
# everything else, \`seat start\` among it, goes to the harness's own rh.
if [[ \$3 == acp && \$4 == --seat ]]; then
  exec python3 "$E2E_WORK/window.py" "$E2E_WORK/acp.log" "\$5"
fi
if [[ \$3 == seat && \$4 == add ]]; then
  printf '%s\n' "\$*" >>"$E2E_WORK/seat.log"
  mkdir -p -m 0700 "\$2/profiles"
  printf '{"version": 1, "kind": "actor", "cwd": "%s", "script": "%s/seat.json"}\n' "\$2" "\$2" \
    >"\$2/profiles/\$5.json"
  chmod 0600 "\$2/profiles/\$5.json"
  printf '{"seat": "%s", "profile": "%s/profiles/%s.json", "kind": "actor", "cwd": "%s"}\n' "\$5" "\$2" "\$5" "\$9"
  exit 0
fi
exec env -u TMUX -u TMUX_PANE "$RH" "\$@"
SH
  # The fake ssh: logs its arguments, then runs the words after the destination as the host's
  # shell would.
  cat >"$bin/ssh" <<SH
#!/bin/bash
printf '%s\n' "\$*" >>"$E2E_WORK/ssh.log"
while [[ \$# -gt 0 && \$1 != -- ]]; do shift; done
shift
shift
exec bash -c "\$*"
SH
  chmod +x "$bin/rh" "$bin/ssh"
  : >"$E2E_WORK/ssh.log"
  : >"$E2E_WORK/acp.log"
  # The seat Open as Thread opens, started before Marley.
  "$bin/rh" --state "$ROOT" seat add claude-other --agent claude --cwd /srv/other >/dev/null
  rh seat start claude-other >/dev/null
  # Marley's own search path, and its terminals', find the fakes first.
  export PATH="$bin:$PATH"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  echo "export PATH=\"$bin:\$PATH\"" >>"$home/.bashrc"
  terminal_env HOME "$home"
  profile_setting marley.harnesses "[{\"name\": \"box-2\", \"ssh\": \"box-2\", \"state\": \"$ROOT\", \"rh\": \"$bin/rh\"}]"
  profile_setting marley.harness_writes true
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

write_window() {
  cat >"$E2E_WORK/window.py" <<'PY'
# A scripted window onto a seat for #742's e2e test, answering as rustal-harness's
# `rh acp --seat NAME` does (its TICKET-116): the seat named in `initialize`, one session
# `seat-<instance>-NAME`, and each prompt answered "Seat NAME: <prompt>", then `end_turn`. It logs
# every message it reads.
import json
import sys
import uuid

log_path, seat = sys.argv[1], sys.argv[2]
session = "seat-stand-in-" + seat


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


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
            "agentCapabilities": {"loadSession": True,
                                  "promptCapabilities": {"image": False, "audio": False, "embeddedContext": False},
                                  "sessionCapabilities": {"list": {}}},
            "authMethods": [],
            "agentInfo": {"name": "rustal-harness", "title": seat, "version": "0"},
        }})
    elif method == "session/new":
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": session}})
    elif method == "session/load":
        send({"jsonrpc": "2.0", "id": request_id, "result": {}})
    elif method == "session/list":
        send({"jsonrpc": "2.0", "id": request_id,
              "result": {"sessions": [{"sessionId": session, "cwd": params.get("cwd", "/"), "title": seat}]}})
    elif method == "session/prompt":
        text = " ".join(block.get("text", "") for block in params.get("prompt", []) if block.get("type") == "text")
        send({"jsonrpc": "2.0", "method": "session/update", "params": {
            "sessionId": params.get("sessionId"),
            "update": {"sessionUpdate": "agent_message_chunk", "messageId": str(uuid.uuid4()),
                       "content": {"type": "text", "text": "Seat %s: %s" % (seat, text)}},
        }})
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "stopReason": "end_turn",
            "_meta": {"rustal-harness": {"seat": "agent/stand-in", "delivery": str(uuid.uuid4())}},
        }})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
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
  rm -rf "$ROOT"
  return 0
}

# Whether a thread's agent ran `acp --seat $1` through ssh to box-2.
window_through_ssh() {
  grep -E -- "-- box-2 .* acp --seat $1\$" "$E2E_WORK/ssh.log"
}

steps() {
  local root
  root=$(cat "$E2E_WORK/root")
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== Ctrl+Alt+N → Claude Agent → On box-2… → /srv/web"
  press "CTRL ALT" n
  settle 2
  type_text "Claude Agent"
  settle 1
  press "" Return
  settle 2
  type_text "box-2"
  settle 1
  shot 742-00-where
  press "" Return
  settle 2
  type_text "/srv/web"
  settle 1
  press "" Return
  settle 12
  shot 742-01-thread
  cat "$E2E_WORK/ssh.log"
  expect "the seat was added on box-2's root" holds "$E2E_WORK/seat.log" "--state $root seat add claude-web --agent claude --cwd /srv/web"
  expect "the thread's agent is the window onto claude-web through ssh" window_through_ssh claude-web

  echo "== hello"
  type_text "hello"
  press "" Return
  settle 4
  shot 742-02-reply
  expect "the window had the prompt" holds "$E2E_WORK/acp.log" '"text": "hello"'

  echo "== claude-other → Open as Thread"
  click "$OTHER_ROW_X" "$OTHER_ROW_Y" right
  settle 1
  shot 742-03a-menu
  press "" Home
  press "" Return
  settle 6
  shot 742-03-open-as-thread
  expect "the second thread's agent is the window onto claude-other" window_through_ssh claude-other
}
