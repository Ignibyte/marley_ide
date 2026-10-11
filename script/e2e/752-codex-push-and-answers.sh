# shellcheck shell=bash
# #752's visual check: the phone push for Codex, Zed's threads and Claude Code's two kinds of
# wait, each kind behind its switch; Codex's questions and app approvals answered from Needs you.
# Fakes only: #535's fake ntfy on a loopback port that logs each request; #650's stand-in Codex on
# its App Server (`codex-fixture.sh`, `marley.codex_app_server` on), cued through its `tui.fifo`
# so it plays with Marley's window away; #535's stand-in `claude` acting out a step at each line
# of its FIFO through the plugin's real hook; #508's stand-in ACP agent, which asks a tool's
# permission three seconds into a prompt, in a thread tab a key opens. Marley's window leaves the
# active workspace for each event that should push, and comes back for its shot.
# - Codex: `command` (`codex-approval`, REQ-001), `ask` (`codex-input`, REQ-002; `question`,
#   REQ-011), its second option clicked (`answered`, REQ-012, REQ-017), `done` and `fail`
#   (`codex-ends`, REQ-003, REQ-004).
# - Claude Code: a permission, then a question (`claude-kinds`, REQ-006).
# - A Zed thread's tool permission (`thread-approval`, REQ-005).
# - The Push section's toggles, `finish` off: a Codex `done` pushes nothing (`switches`, REQ-007,
#   REQ-008).
# - A tool approval (`tool-approval`, REQ-014), Allow session clicked (`allowed-for-session`,
#   REQ-015, REQ-017); two questions, the entry clicked (`question-in-terminal`, REQ-013); a form
#   with a field (`form`, REQ-016); a question left unanswered gets nothing from Marley (REQ-018);
#   every push only the line and its headers (REQ-010).
compositor sway
# shellcheck source=script/e2e/codex-fixture.sh
. script/e2e/codex-fixture.sh

HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks

# Places in the window, as the first run's shots found them; `STOP_AT=<shot>` ends the run after
# that shot.
OPTION_DEV_X=${OPTION_DEV_X:-231}
OPTION_DEV_Y=${OPTION_DEV_Y:-151}
ALLOW_SESSION_X=${ALLOW_SESSION_X:-122}
ALLOW_SESSION_Y=${ALLOW_SESSION_Y:-267}
ENTRY_X=${ENTRY_X:-110}
ENTRY_Y=${ENTRY_Y:-230}
STOP_AT=${STOP_AT:-}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin versions=$E2E_WORK/versions
  mkdir -p "$home" "$bin" "$versions"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  write_codex_stand_in "$versions/0.155.1"
  ln -sfn "$versions/0.155.1" "$bin/codex"
  export MARLEY_CODEX=$bin/codex
  profile_setting marley.codex_app_server true
  write_fake_ntfy
  python3 -u "$E2E_WORK/ntfy.py" "$E2E_WORK/pushes.log" >"$E2E_WORK/ntfy.out" 2>&1 &
  echo "$!" >"$E2E_WORK/ntfy.pid"
  for _ in $(seq 50); do
    PORT=$(grep -oE 'port [0-9]+' "$E2E_WORK/ntfy.out" | cut -d' ' -f2 || true)
    [[ -n $PORT ]] && break
    sleep 0.1
  done
  [[ -n $PORT ]] || { echo "the fake ntfy did not start" >&2; return 1; }
  : >"$E2E_WORK/pushes.log"
  mkfifo "$E2E_WORK/steps.fifo" "$E2E_WORK/tui.fifo"
  write_steps
  write_stand_in_claude "$bin/claude"
  write_stand_in_agent
  python3 - "$E2E_PROFILE/config/settings.json" "$E2E_WORK/stand-in-agent.py" <<'PY'
import json, re, sys
path, script = sys.argv[1:]
text = open(path).read()
entry = '"Stand-in": %s,' % json.dumps({"type": "custom", "command": "python3", "args": [script]})
match = re.search(r'"agent_servers"\s*:\s*\{', text)
if match:
    text = text[:match.end()] + "\n    " + entry + text[match.end():]
else:
    at = text.index("{") + 1
    text = text[:at] + '\n  "agent_servers": {' + entry + '},' + text[at:]
open(path, "w").write(text)
PY
  printf '[{"bindings": {"ctrl-alt-shift-y": ["marley::NewAgentThread", {"agent": "Stand-in", "folder": "%s"}], "ctrl-alt-shift-k": ["zed::OpenSettingsAt", {"path": "marley.push"}]}}]\n' \
    "$E2E_WORK/repo" >"$E2E_PROFILE/config/keymap.json"
  git init -q -b main "$E2E_WORK/repo"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

teardown() {
  [[ -f $E2E_WORK/ntfy.pid ]] && kill "$(cat "$E2E_WORK/ntfy.pid")" 2>/dev/null
  true
}

write_fake_ntfy() {
  cat >"$E2E_WORK/ntfy.py" <<'PY'
# #535's fake ntfy on a free loopback port: logs each request's method, path, headers and body, one
# JSON line each, and answers as ntfy does.
import http.server
import json
import sys

LOG = sys.argv[1]


class Handler(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length") or 0)
        record = {"method": "POST", "path": self.path, "headers": dict(self.headers.items()),
                  "body": self.rfile.read(length).decode("utf-8", "replace")}
        with open(LOG, "a", encoding="utf-8") as log:
            log.write(json.dumps(record) + "\n")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(b'{"id":"e2e","event":"message"}')

    def log_message(self, *args):
        pass


server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
print(f"port {server.server_address[1]}", flush=True)
server.serve_forever()
PY
}

# Claude Code's two waits: a permission, then a question.
write_steps() {
  cat >"$E2E_WORK/steps.json" <<'JSON'
[
  {"label": "a permission asked", "events": [
    {"hook_event_name": "UserPromptSubmit", "prompt": "Add a README"},
    {"hook_event_name": "PreToolUse", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# repo"}, "tool_use_id": "t1"},
    {"hook_event_name": "PermissionRequest", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# repo"}}]},
  {"label": "a question asked", "events": [
    {"hook_event_name": "PostToolUse", "tool_name": "Write", "tool_input": {"file_path": "README.md", "content": "# repo"}, "tool_use_id": "t1"},
    {"hook_event_name": "PreToolUse", "tool_name": "AskUserQuestion", "tool_use_id": "q1",
     "tool_input": {"questions": [{"question": "Which base branch?", "header": "Branch", "multiSelect": false,
                                   "options": [{"label": "main", "description": "The trunk"},
                                               {"label": "dev", "description": "The work"}]}]}}]}
]
JSON
}

write_stand_in_claude() {
  sed -e "s|@HOOKS@|$HOOKS|" -e "s|@STEPS@|$E2E_WORK/steps.json|" -e "s|@FIFO@|$E2E_WORK/steps.fifo|" \
    >"$1" <<'FAKE'
#!/usr/bin/env python3
# #535's stand-in Claude Code: each line written to the FIFO acts out the next step, the plugin's
# event.py for every event, each answer's sequence written to the terminal as Claude Code does.
import json
import os
import subprocess
import sys

STEPS = json.load(open("@STEPS@", encoding="utf-8"))
COMMON = {"session_id": "e2e-session", "transcript_path": "/tmp/e2e-transcript.jsonl",
          "cwd": os.getcwd(), "permission_mode": "default"}

print("Claude Code (stand-in): steps come through the FIFO", flush=True)
done = 0
while done < len(STEPS):
    with open("@FIFO@", encoding="utf-8") as fifo:
        for _ in fifo:
            if done >= len(STEPS):
                break
            step = STEPS[done]
            done += 1
            for event in step["events"]:
                answer = subprocess.run([sys.executable, "@HOOKS@/event.py"],
                                        input=json.dumps({**COMMON, **event}),
                                        capture_output=True, text=True, check=False).stdout
                sequence = json.loads(answer or "{}").get("terminalSequence")
                if sequence:
                    sys.stdout.write(sequence)
                    sys.stdout.flush()
            print(f"step {done}: {step['label']}", flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$1"
}

# #508's stand-in ACP agent, which asks a tool's permission three seconds into each prompt, so the
# window can leave first.
write_stand_in_agent() {
  cat >"$E2E_WORK/stand-in-agent.py" <<'PY'
import json
import sys
import time


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def update(session, body):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session, "update": body}})


for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": request_id, "result": {
            "protocolVersion": 1,
            "agentCapabilities": {"loadSession": False, "promptCapabilities": {}},
            "authMethods": [], "agentInfo": {"name": "stand-in", "version": "0"}}})
    elif method == "session/new":
        send({"jsonrpc": "2.0", "id": request_id, "result": {"sessionId": "stand-in-1"}})
    elif method == "session/prompt":
        session = params.get("sessionId")
        time.sleep(3)
        tool = {"toolCallId": "call-1", "title": "Edit README.md", "kind": "edit", "status": "pending",
                "rawInput": {"path": "README.md"}}
        update(session, {"sessionUpdate": "tool_call", **tool})
        send({"jsonrpc": "2.0", "id": "permission-1", "method": "session/request_permission", "params": {
            "sessionId": session, "toolCall": tool,
            "options": [{"optionId": "allow", "name": "Allow", "kind": "allow_once"},
                        {"optionId": "reject", "name": "Reject", "kind": "reject_once"}]}})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "not done: " + str(method)}})
PY
}

# Sets `marley.push` to the fake's topic, with `$1` as more of its fields, and waits for the reload.
push_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "http://127.0.0.1:$PORT" "${1:-}" <<'PY'
import json
import re
import sys

path, url, more = sys.argv[1:]
text = open(path).read()
text = re.sub(r'\n\s*"push": \{[^}]*\},', "", text)
entry = {"url": url, "topic": "marley-e2e", **(json.loads("{" + more + "}") if more else {})}
line = '"push": ' + json.dumps(entry) + ","
if '"marley": {' in text:
    text = text.replace('"marley": {', '"marley": {\n    ' + line, 1)
else:
    lines = text.split("\n")
    at = next(i for i, text_line in enumerate(lines) if text_line.startswith("{"))
    lines.insert(at + 1, '  "marley": {' + line + "},")
    text = "\n".join(lines)
open(path, "w").write(text)
PY
  settle 2
}

away() { sway_msg workspace 2 >/dev/null; settle 1; }
back() { sway_msg workspace 1 >/dev/null; settle 1; }

# Writes a cue to the stand-in Codex's TUI.
tui() {
  echo "$1" >"$E2E_WORK/tui.fifo"
  settle "${2:-2}"
}

next_step() { echo next >"$E2E_WORK/steps.fifo"; settle 2; }

cue() {
  type_text "$1"
  press "" Return
  settle "${2:-2}"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle "${2:-2}"
}

close_settings() {
  sway_msg '[title="Settings"] kill' >/dev/null
  settle 2
}

pushes() { wc -l <"$E2E_WORK/pushes.log"; }

pushed() {
  echo "pushes so far: $(pushes)"
  [[ $(pushes) -eq $1 ]]
}

# Whether the last request is a push of this line, with this priority and tag.
last_push() {
  tail -1 "$E2E_WORK/pushes.log"
  python3 - "$E2E_WORK/pushes.log" "$@" <<'PY'
import json
import sys

path, line, priority, tag = sys.argv[1:]
push = json.loads(open(path).read().splitlines()[-1])
headers = {key.lower(): value for key, value in push["headers"].items()}
ok = (push["path"] == "/marley-e2e" and push["body"] == line
      and headers.get("title") == "Marley" and headers.get("priority") == priority
      and headers.get("tags") == tag)
sys.exit(0 if ok else 1)
PY
}

# Whether every push carries only these headers: none of them is the agent's.
only_known_headers() {
  python3 - "$E2E_WORK/pushes.log" <<'PY'
import json
import sys

known = {"title", "priority", "tags", "content-type", "content-length", "authorization", "host",
         "accept", "accept-encoding", "user-agent"}
for text in open(sys.argv[1]).read().splitlines():
    extra = {key.lower() for key in json.loads(text)["headers"]} - known
    if extra:
        print("unexpected headers:", sorted(extra))
        sys.exit(1)
PY
}

# Whether Marley's answer to the last request of `$1`'s method is `$2`.
answered_with() {
  local ident
  ident=$(grep -oE "asked [0-9]+ $1" "$E2E_WORK/server.log" | tail -1 | cut -d' ' -f2)
  grep -F " answered $ident by marley: " "$E2E_WORK/server.log" | grep -qF -- "$2"
}

# Whether Marley sent nothing for the last request of `$1`'s method.
unanswered_by_marley() {
  local ident
  ident=$(grep -oE "asked [0-9]+ $1" "$E2E_WORK/server.log" | tail -1 | cut -d' ' -f2)
  echo "request $ident"
  ! grep -q " $ident by marley" "$E2E_WORK/server.log"
}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

steps() {
  local count
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  push_setting
  echo "== Codex on its App Server"
  press "CTRL ALT" n
  settle 2
  type_text "Codex"
  settle 1
  press "" Return
  settle 2
  # New Agent asks where Codex starts; the project comes first.
  press "" Return
  settle 10
  tui work 3
  echo "== Codex needs approval"
  away
  tui "command rm -rf build" 3
  back
  take 752-01-codex-approval || return 0
  expect "Codex's approval pushes needs approval" last_push "repo: Codex needs approval" 4 lock
  tui y 6
  echo "== Codex asks a question"
  away
  tui "ask Branch|Which branch?|main,dev" 3
  back
  take 752-02-codex-input || return 0
  expect "Codex's question pushes needs input" last_push "repo: Codex needs input" 4 question
  take 752-07-question || return 0
  click "$OPTION_DEV_X" "$OPTION_DEV_Y"
  settle 3
  take 752-08-answered || return 0
  expect "Marley answered with the option's label" \
    answered_with item/tool/requestUserInput '{"answers": {"choice": {"answers": ["dev"]}}}'
  settle 3
  echo "== Codex finishes, then fails"
  away
  tui "done" 3
  back
  expect "Codex's turn's end pushes finished" last_push "repo: Codex finished" 3 white_check_mark
  settle 6
  tui work 2
  away
  tui fail 3
  back
  take 752-03-codex-ends || return 0
  expect "Codex's failed turn pushes failed" last_push "repo: Codex failed" 4 x
  settle 6
  echo "== Claude Code's two kinds"
  palette "workspace: new terminal" 4
  cue claude 3
  away
  next_step
  back
  expect "Claude Code's permission pushes needs approval" last_push "repo: Claude needs approval" 4 lock
  settle 6
  away
  next_step
  back
  take 752-05-claude-kinds || return 0
  expect "Claude Code's question pushes needs input" last_push "repo: Claude needs input" 4 question
  settle 6
  echo "== a Zed thread's permission"
  press "CTRL ALT SHIFT" y
  settle 8
  type_text "Edit the README"
  press "" Return
  away
  settle 6
  back
  take 752-04-thread-approval || return 0
  expect "a thread's permission pushes needs approval" last_push "repo: Stand-in needs approval" 4 lock
  settle 6
  echo "== the switches"
  push_setting '"finish": false'
  press "CTRL ALT SHIFT" k
  settle 3
  take 752-06-switches || return 0
  close_settings
  count=$(pushes)
  tui work 2
  away
  tui "done" 3
  back
  expect "a finished turn pushes nothing with finish off" pushed "$count"
  echo "== app approvals and forms"
  tui work 2
  tui "toolok docs session,always Run search?" 3
  take 752-10-tool-approval || return 0
  click "$ALLOW_SESSION_X" "$ALLOW_SESSION_Y"
  settle 3
  take 752-11-allowed-for-session || return 0
  expect "Allow session answers accept for the session" \
    answered_with mcpServer/elicitation/request '{"_meta": {"persist": "session"}, "action": "accept"}'
  tui ask2 3
  click "$ENTRY_X" "$ENTRY_Y"
  settle 2
  take 752-09-question-in-terminal || return 0
  tui n 2
  tui "form docs Your name?" 3
  take 752-12-form || return 0
  tui n 2
  echo "== a question no one answers"
  tui "askfree Anything to add?" 10
  expect "Marley sent nothing for it" unanswered_by_marley item/tool/requestUserInput
  expect "no push carries anything of the agent's" only_known_headers
}
