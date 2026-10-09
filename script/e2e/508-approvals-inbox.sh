# shellcheck shell=bash
# #508's e2e test: one approvals inbox at the top of the rail. A stand-in external agent (just
# enough ACP, in Python, a custom agent server in the run's settings) asks Zed's permission to
# edit README.md at each prompt and logs the answer; a stand-in `claude` in a terminal sends
# #519's hook events for a PermissionRequest (Bash `rm -rf build`, after its PreToolUse) and,
# later, the Bash's end; the stand-in agent of the browser fixture clicks "Delete account", which
# #571 holds. The inbox lists the thread's prompt with Allow and Deny (REQ-001), the terminal's
# below it (REQ-002), answers the thread in place (REQ-003), opens the terminal (REQ-005), lists
# the paused click with Allow and Refuse and refuses it (REQ-009), and leaves when nothing waits
# (REQ-006); a pick sent to the waiting terminal pastes nothing (REQ-007).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The project's + in the rail and the steps down the agents' submenu to the stand-in (501's); the
# inbox's buttons and entries, the terminal's row, the thread's message field, the pick button
# and the page's button, from the first run's shots.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-96}
STAND_IN_STEPS=${STAND_IN_STEPS:-2}
# The first entry's Allow and its body; the second entry's Deny or Refuse under a first entry
# with no buttons (a terminal's).
FIRST_ALLOW_X=${FIRST_ALLOW_X:-226}
FIRST_ALLOW_Y=${FIRST_ALLOW_Y:-158}
FIRST_X=${FIRST_X:-110}
FIRST_Y=${FIRST_Y:-125}
SECOND_DENY_X=${SECOND_DENY_X:-184}
SECOND_DENY_Y=${SECOND_DENY_Y:-204}
TERMINAL_ROW_X=${TERMINAL_ROW_X:-110}
TERMINAL_ROW_Y=${TERMINAL_ROW_Y:-240}
THREAD_FIELD_X=${THREAD_FIELD_X:-1300}
THREAD_FIELD_Y=${THREAD_FIELD_Y:-864}
PICK_X=${PICK_X:-1082}
PICK_Y=${PICK_Y:-87}
BUTTON_X=${BUTTON_X:-795}
BUTTON_Y=${BUTTON_Y:-250}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config
  offline_chromium
  mkdir -p "$home" "$bin" "$config/plugins" "$E2E_WORK/site"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  # The plugin listed at the version Marley ships, so the agent bar offers no update.
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/$SHIPPED", "version": "$SHIPPED"}]}}
JSON
  export CLAUDE_CONFIG_DIR=$config
  cat >"$E2E_WORK/site/index.html" <<'HTML'
<!doctype html><html><head><title>Account</title><style>
body { margin: 0; padding: 20px; font: 20px sans-serif }
button { font-size: 20px; margin: 8px }
</style></head><body>
<h1>Your account</h1>
<p><button id="save">Save profile</button> <button id="delete">Delete account</button></p>
<p>Log: <span id="log"></span></p>
<script>
for (const button of document.querySelectorAll('button')) {
  button.addEventListener('click', () => {
    const log = document.getElementById('log');
    log.textContent += (log.textContent ? ', ' : '') + button.textContent;
    document.title = 'Log: ' + log.textContent;
  });
}
</script>
</body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
  write_stand_in_agent
  write_stand_in_claude "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude
  write_mcp_agent
  # The stand-in agent server (501's), and #571's pause for every agent on the rules alone.
  python3 - "$E2E_PROFILE/config/settings.json" "$E2E_WORK/stand-in-agent.py" "$E2E_WORK/stand-in.log" <<'PY'
import json, re, sys
path, script, log = sys.argv[1:]
text = open(path).read()
entry = '"Stand-in": %s,' % json.dumps({
    "type": "custom", "command": "python3", "args": [script], "env": {"STAND_IN_LOG": log},
})
match = re.search(r'"agent_servers"\s*:\s*\{', text)
if match:
    text = text[:match.end()] + "\n    " + entry + text[match.end():]
else:
    at = text.index("{") + 1
    text = text[:at] + '\n  "agent_servers": {' + entry + '},' + text[at:]
open(path, "w").write(text)
PY
  marley_setting "{\"browser_click_pause_agents\": \"all_agents\", \"system_one\": {\"enabled\": true, \"provider\": \"rules\", \"projects\": [\"$E2E_WORK/repo\"], \"uses\": {\"click_consequence\": \"shadow\"}}}"
  git init -q -b inbox "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

# Merges the JSON object `$1` into `marley` in the profile copy's settings (569's).
marley_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'PY'
import json, pathlib, sys

path, changes = pathlib.Path(sys.argv[1]), json.loads(sys.argv[2])
text = path.read_text() if path.exists() else "{}"
def skip_blank(index):
    while index < len(text):
        if text[index] in " \t\r\n":
            index += 1
        elif text.startswith("//", index):
            end = text.find("\n", index)
            index = len(text) if end < 0 else end
        elif text.startswith("/*", index):
            end = text.find("*/", index + 2)
            index = len(text) if end < 0 else end + 2
        else:
            break
    return index


out, index, in_string = [], 0, False
while index < len(text):
    character = text[index]
    if in_string:
        out.append(character)
        if character == "\\" and index + 1 < len(text):
            out.append(text[index + 1])
            index += 2
            continue
        if character == '"':
            in_string = False
        index += 1
        continue
    if character == '"':
        in_string = True
    elif text.startswith("//", index) or text.startswith("/*", index):
        index = skip_blank(index)
        continue
    elif character == ",":
        ahead = skip_blank(index + 1)
        if ahead < len(text) and text[ahead] in "}]":
            index += 1
            continue
    out.append(character)
    index += 1
cleaned = "".join(out).strip()
settings = json.loads(cleaned) if cleaned else {}
marley = settings.setdefault("marley", {})
for key, value in changes.items():
    if key == "system_one":
        layer = marley.setdefault("system_one", {})
        for inner, setting in value.items():
            if inner == "uses":
                layer.setdefault("uses", {}).update(setting)
            else:
                layer[inner] = setting
    else:
        marley[key] = value
path.write_text(json.dumps(settings, indent=2) + "\n")
PY
}

# A stand-in external agent: at each prompt it announces a tool call, asks Zed's permission for it
# with allow-once and reject-once options, waits for the answer, logs it, and ends the turn.
write_stand_in_agent() {
  cat >"$E2E_WORK/stand-in-agent.py" <<'PY'
import json
import os
import sys

log = open(os.environ["STAND_IN_LOG"], "a", buffering=1)
state = {"calls": 0}


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def answer(request_id, result):
    send({"jsonrpc": "2.0", "id": request_id, "result": result})


def update(session, body):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session, "update": body}})


def ask_permission(session):
    state["calls"] += 1
    call = f"call-{state['calls']}"
    tool = {"toolCallId": call, "title": "Edit README.md", "kind": "edit", "status": "pending",
            "rawInput": {"path": "README.md"}}
    update(session, {"sessionUpdate": "tool_call", **tool})
    asked = f"permission-{state['calls']}"
    send({"jsonrpc": "2.0", "id": asked, "method": "session/request_permission", "params": {
        "sessionId": session, "toolCall": tool,
        "options": [{"optionId": "allow", "name": "Allow", "kind": "allow_once"},
                    {"optionId": "reject", "name": "Reject", "kind": "reject_once"}]}})
    for line in sys.stdin:
        if not line.strip():
            continue
        message = json.loads(line)
        if message.get("id") == asked and "method" not in message:
            outcome = (message.get("result") or {}).get("outcome") or {}
            chosen = outcome.get("optionId") or outcome.get("outcome") or "none"
            log.write(f"answer: {chosen}\n")
            status = "completed" if chosen == "allow" else "failed"
            update(session, {"sessionUpdate": "tool_call_update", "toolCallId": call, "status": status})
            return chosen
    return "closed"


for line in sys.stdin:
    if not line.strip():
        continue
    message = json.loads(line)
    method, request_id, params = message.get("method"), message.get("id"), message.get("params") or {}
    if method == "initialize":
        answer(request_id, {
            "protocolVersion": 1,
            "agentCapabilities": {"loadSession": False, "promptCapabilities": {}, "mcpCapabilities": {"http": False, "sse": False}},
            "authMethods": [],
            "agentInfo": {"name": "stand-in", "version": "0"},
        })
    elif method == "session/new":
        answer(request_id, {"sessionId": "stand-in-1"})
    elif method == "session/prompt":
        session = params.get("sessionId")
        chosen = ask_permission(session)
        update(session, {"sessionUpdate": "agent_message_chunk",
                         "content": {"type": "text", "text": f"The edit was {chosen}."}})
        answer(request_id, {"stopReason": "end_turn"})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "the stand-in does not do " + str(method)}})
PY
}

# #566's stand-in `claude`: at each line it reads, the next step's hook events through the
# plugin's hook, in one session, and every line it reads logged.
write_stand_in_claude() {
  python3 - "$E2E_WORK/steps.json" <<'PY'
import json, sys

def call(event, number, **rest):
    return {"hook_event_name": event, "tool_name": "Bash", "tool_input": {"command": "rm -rf build"},
            "tool_use_id": f"t{number}", **rest}

steps = [
    {"label": "a permission", "events": [
        {"hook_event_name": "UserPromptSubmit", "prompt": "Clean the build"},
        call("PreToolUse", 1),
        {"hook_event_name": "PermissionRequest", "tool_name": "Bash", "tool_input": {"command": "rm -rf build"}}]},
    {"label": "the Bash ran", "events": [
        call("PostToolUse", 1),
        {"hook_event_name": "Stop", "last_assistant_message": "The build is clean."}]},
]
json.dump(steps, open(sys.argv[1], "w"), indent=1)
PY
  sed -e "s|@HOOK@|$HOOK|" -e "s|@STEPS@|$E2E_WORK/steps.json|" -e "s|@READ@|$E2E_WORK/stdin.log|" \
    >"$1" <<'FAKE'
#!/usr/bin/env python3
import json
import os
import subprocess
import sys

if sys.argv[1:2] == ["plugin"]:
    sys.exit(0)

STEPS = json.load(open("@STEPS@", encoding="utf-8"))
COMMON = {"transcript_path": "/tmp/e2e-transcript.jsonl", "cwd": os.getcwd(),
          "permission_mode": "default", "session_id": "s1", "prompt_id": "p1"}


def read_line():
    line = sys.stdin.readline()
    with open("@READ@", "a", encoding="utf-8") as log:
        log.write(json.dumps(line) + "\n")
    return line


print("Claude Code (stand-in): press Enter for each step", flush=True)
for number, step in enumerate(STEPS, 1):
    if not read_line():
        break
    for event in step["events"]:
        answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps({**COMMON, **event}),
                                capture_output=True, text=True, check=False).stdout
        sequence = json.loads(answer or "{}").get("terminalSequence")
        if sequence:
            sys.stdout.write(sequence)
            sys.stdout.flush()
    print(f"step {number}: {step['label']}", flush=True)
while read_line():
    pass
FAKE
  chmod +x "$1"
}

# The stand-in's log, and whether it holds a text.
logged() {
  holds "$E2E_WORK/stand-in.log" "$@"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  mcp_agent navigate "$SITE/index.html"
  settle 3

  echo "== a thread of the stand-in agent asks to edit README.md"
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  # A menu opens with nothing chosen (Zed #64365); Home chooses New Terminal.
  press "" Home
  press "" Down
  press "" Down
  press "" Right
  settle 1
  for ((step = 0; step < STAND_IN_STEPS; step++)); do
    press "" Down
  done
  settle 1
  press "" Return
  settle 6
  type_text "edit the readme"
  press "" Return
  settle 4
  shot 508-01-thread-entry

  echo "== Claude Code in a terminal asks to run rm -rf build: two entries, oldest first"
  click "$TERMINAL_ROW_X" "$TERMINAL_ROW_Y"
  settle 1
  type_text "claude"
  press "" Return
  settle 3
  press "" Return
  settle 4
  shot 508-02-two-entries

  echo "== Allow on the thread's entry, the first"
  click "$FIRST_ALLOW_X" "$FIRST_ALLOW_Y"
  settle 3
  shot 508-03-allowed
  expect "the stand-in read Allow" logged 'answer: allow'

  echo "== a second prompt, and Deny on its entry, now the second"
  click "$THREAD_FIELD_X" "$THREAD_FIELD_Y"
  settle 1
  type_text "edit it again"
  press "" Return
  settle 4
  shot 508-04a-second-prompt
  click "$SECOND_DENY_X" "$SECOND_DENY_Y"
  settle 3
  shot 508-04-denied
  expect "the stand-in read Deny" logged 'answer: reject'

  echo "== the terminal's entry shows its terminal"
  click "$FIRST_X" "$FIRST_Y"
  settle 2
  shot 508-05-terminal-opened

  # The page is the one the run opened: an agent's navigate here would put its chip in the
  # toolbar, which moves Pick.
  echo "== a pick sent to the waiting terminal pastes nothing"
  click "$PICK_X" "$PICK_Y"
  settle 1
  pointer_to "$BUTTON_X" "$BUTTON_Y"
  settle 2
  click "$BUTTON_X" "$BUTTON_Y"
  settle 3
  shot 508-06a-pick-staged
  type_text "this button"
  press "" Return
  settle 2
  shot 508-06-pick-refused
  mcp_agent picks | tee "$E2E_WORK/picks.txt"
  expect "the pick waits in the tray, unsent" holds "$E2E_WORK/picks.txt" 'pick 1: ' 'not sent'
  cat "$E2E_WORK/stdin.log"
  expect "the terminal read nothing but the scenario's Enters" python3 -c '
import json, sys
lines = [json.loads(line) for line in open(sys.argv[1])]
sys.exit(0 if lines and all(line == "\n" for line in lines) else 1)' "$E2E_WORK/stdin.log"

  echo "== the paused click waits in the inbox, second, and its Refuse answers it"
  mcp_agent click-on button "Delete account" >"$E2E_WORK/delete.txt" 2>&1 &
  local clicking=$!
  settle 3
  shot 508-08-paused-click
  click "$SECOND_DENY_X" "$SECOND_DENY_Y"
  settle 2
  wait "$clicking" || true
  cat "$E2E_WORK/delete.txt"
  expect "the click was refused from the inbox" holds "$E2E_WORK/delete.txt" 'The user refused it'

  echo "== the terminal's Bash ran: nothing waits"
  click "$FIRST_X" "$FIRST_Y"
  settle 1
  press "" Return
  settle 3
  shot 508-07-cleared
}
