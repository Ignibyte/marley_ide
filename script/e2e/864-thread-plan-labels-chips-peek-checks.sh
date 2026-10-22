# shellcheck shell=bash
# #864's visual check: the thread's plan, card labels, reply chips, terminal peek and checks. #825's
# set-up (Marley's own agent on a scripted ACP agent) in a scratch crate rust-analyzer reads. For
# the prompt the agent sends a plan (one done, one under way, one waiting), writes the crate's
# src/main.rs with a call to a missing function, shows an edit of notes.md, and runs a three-line build through the
# client's terminal.
# - The answer: the edit card's EDIT label, the terminal card's RUN label, the chips under the
#   answer; the details panel's PLAN "1 of 3", ITS TERMINAL's last lines, and CHECKS with
#   src/main.rs's error (`864-01-thread`, REQ-001 to REQ-004).
# - Explain clicked: its words in the message box (`864-02-chip`, REQ-003).
# - Marley Day (`864-03-day`, all).
compositor sway

STOP_AT=${STOP_AT:-}
# The Explain chip, from the first run's shot.
EXPLAIN_X=${EXPLAIN_X:-365}
EXPLAIN_Y=${EXPLAIN_Y:-502}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo/src"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  write_scripted_agent
  printf '[package]\nname = "rail"\nversion = "0.1.0"\nedition = "2021"\n' >"$E2E_WORK/repo/Cargo.toml"
  printf 'fn main() {}\n' >"$E2E_WORK/repo/src/main.rs"
  printf '# Notes\n' >"$E2E_WORK/repo/notes.md"
  printf '#!/bin/sh\nexec python3 %s %s %s\n' "$E2E_WORK/scripted-agent.py" "$E2E_WORK/agent.log" \
    "$E2E_WORK/repo" \
    >"$bin/scripted-agent"
  chmod +x "$bin/scripted-agent"
  export MARLEY_ASSISTANT_ADAPTER=$bin/scripted-agent
  profile_setting marley.assistant.enabled true
  profile_setting marley.assistant.agent '"claude_code"'
  profile_setting use_system_path_prompts false
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.thread_details true
  open_path "$E2E_WORK/repo"
}

write_scripted_agent() {
  cat >"$E2E_WORK/scripted-agent.py" <<'PY'
# A scripted ACP agent for #864's e2e test: a plan, src/main.rs written through the client (with an
# error rust-analyzer reports), an edit tool call with a diff, and a terminal tool call run through
# the client's terminal/create.
import json
import sys

log_path = sys.argv[1]
# The scratch crate the window has open: Marley's own agent works in its config folder, which no
# language server reads.
crate_folder = sys.argv[2]
next_id = [9000]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def log(message):
    with open(log_path, "a") as log_file:
        log_file.write(json.dumps(message) + "\n")


def request(method, params):
    next_id[0] += 1
    rid = next_id[0]
    send({"jsonrpc": "2.0", "id": rid, "method": method, "params": params})
    for answer in sys.stdin:
        if not answer.strip():
            continue
        reply = json.loads(answer)
        log(reply)
        if reply.get("id") == rid and "method" not in reply:
            return reply
    return {}


def update(session, body):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session, "update": body}})


folders = {}
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
            "authMethods": [], "agentInfo": {"name": "scripted", "version": "0"}}})
    elif method == "session/new":
        session = "marley-1"
        folders[session] = params.get("cwd", ".")
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": session}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        folder = folders.get(session, ".")
        update(session, {"sessionUpdate": "plan", "entries": [
            {"content": "Default limit_content_width to false", "priority": "high", "status": "completed"},
            {"content": "Add the toggle to the bottom row", "priority": "medium", "status": "in_progress"},
            {"content": "Run the gate and the visual check", "priority": "medium", "status": "pending"}]})
        request("fs/write_text_file", {"sessionId": session, "path": crate_folder + "/src/main.rs",
                                        "content": "fn main() {\n    missing();\n}\n"})
        update(session, {"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": "I will add the toggle."}})
        update(session, {"sessionUpdate": "tool_call", "toolCallId": "edit-1", "title": "Edit notes.md", "kind": "edit",
                         "status": "completed", "locations": [{"path": folder + "/notes.md"}],
                         "content": [{"type": "diff", "path": folder + "/notes.md", "oldText": "# Notes\n", "newText": "# Notes\n\nThe toggle.\n"}]})
        created = request("terminal/create", {"sessionId": session, "command": "sh",
                                              "args": ["-c", "for i in 1 2 3; do echo Compiling crate $i; done; echo Finished"],
                                              "cwd": folder})
        terminal = (created.get("result") or {}).get("terminalId")
        if terminal:
            update(session, {"sessionUpdate": "tool_call", "toolCallId": "run-1", "title": "cargo build", "kind": "execute",
                             "status": "in_progress", "content": [{"type": "terminal", "terminalId": terminal}]})
            request("terminal/wait_for_exit", {"sessionId": session, "terminalId": terminal})
            update(session, {"sessionUpdate": "tool_call_update", "toolCallId": "run-1", "status": "completed"})
        update(session, {"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": " The build is green."}})
        send({"jsonrpc": "2.0", "id": request_id, "result": {"stopReason": "end_turn"}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 20 "$1" -s 800 -k Return
}

steps() {
  settle 12
  # Trusts the scratch crate.
  press "" Return
  settle 3

  echo "== Marley's thread, one prompt"
  palette "marley: talk to marley"
  settle 14
  type_text "Add the toggle."
  # The debug build's message box takes the typed keys well after wtype sends them.
  settle 10
  press "" Return
  # rust-analyzer reads the crate and std, then the written file.
  settle 45
  pointer_to 900 650
  settle 1
  take 864-01-thread || return 0

  echo "== Explain"
  if ((EXPLAIN_X > 0)); then
    click "$EXPLAIN_X" "$EXPLAIN_Y"
    settle 2
  fi
  pointer_to 900 650
  settle 1
  take 864-02-chip || return 0

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  take 864-03-day || return 0
}
