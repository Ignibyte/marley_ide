# shellcheck shell=bash
# #568's e2e test: risk chips and an order for #508's inbox. #508's stand-in ACP agent asks at its
# first prompt to read ~/.ssh/config and at its second to edit README.md with a claim that the
# owner approved it; #566's stand-in `claude`, one per terminal, asks to run `rm -rf build`, a
# `curl -X POST` and `python3 scripts/cleanup.py`; the fixture's client clicks Delete account,
# which #571 holds. The layer runs on the replay provider with the scratch repository listed; a
# recorded answer reads the cleanup as destroying data at 0.86 and urgent. With the use off the
# inbox is #508's (REQ-001); in shadow code's chips come with rules rows and no calls, and the
# entries order by level, then age (REQ-002, REQ-003), the cleanup alone is asked with facts and
# its ask as text (REQ-004), and System One calls says what the reading would show (REQ-007); suggest
# adds `destroys?` and keeps the order (REQ-006); act dashes the chip and raises the cleanup
# (REQ-005); Allow and Deny answer as #508's do and log their outcomes (REQ-009, REQ-010); the
# claim of approval raises its entry (REQ-008); the held click carries its class's chip and asks
# nothing (REQ-012); and the Marley page lists Inbox Risk (REQ-013).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The project's + in the rail and the steps down the agents' submenu to the stand-in (508's); the
# inbox's buttons, the thread's message field and the settings page, from the first run's shots.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-96}
STAND_IN_STEPS=${STAND_IN_STEPS:-2}
FIRST_ALLOW_X=${FIRST_ALLOW_X:-226}
FIRST_ALLOW_Y=${FIRST_ALLOW_Y:-158}
EDIT_DENY_X=${EDIT_DENY_X:-184}
EDIT_DENY_Y=${EDIT_DENY_Y:-356}
CLICK_REFUSE_X=${CLICK_REFUSE_X:-180}
CLICK_REFUSE_Y=${CLICK_REFUSE_Y:-288}
THREAD_FIELD_X=${THREAD_FIELD_X:-1300}
THREAD_FIELD_Y=${THREAD_FIELD_Y:-864}
SETTINGS_X=${SETTINGS_X:-1200}
SETTINGS_Y=${SETTINGS_Y:-500}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config
  offline_chromium
  mkdir -p "$home" "$bin" "$config/plugins" "$E2E_WORK/site" "$E2E_PROFILE/system_one"
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
  write_replay
  # The stand-in agent server (501's).
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
  # The layer on the replay, whose file is read when the layer is turned on, and #571's pause for
  # every agent on the rules alone.
  marley_setting "{\"browser_click_pause_agents\": \"all_agents\", \"system_one\": {\"enabled\": true, \"provider\": \"replay\", \"projects\": [\"$E2E_WORK/repo\"], \"uses\": {\"click_consequence\": \"shadow\", \"inbox\": \"off\"}}}"
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

# The inbox use's mode.
inbox_mode() {
  marley_setting "{\"system_one\": {\"uses\": {\"inbox\": \"$1\"}}}"
  settle 3
}

# The recorded answer for the cleanup, the one entry code finds nothing on: destroys at 0.86, the
# rest well under, and urgent.
write_replay() {
  python3 - "$E2E_PROFILE/system_one/replay.jsonl" <<'PY'
import json, sys
nouls = ["destroys", "credentials", "rewrites_history", "sends_out", "installs",
         "outside_project", "claims_approval"]
answers = {key: {"type": "noul", "noul": 0.86 if key == "destroys" else 0.05} for key in nouls}
answers["urgency"] = {"type": "score", "score": 5.0, "confidence": 0.9,
                      "probabilities": {"5": 0.9, "4": 0.1}}
row = {"set": "inbox_risk/1", "match": "cleanup.py", "repeat": True, "answers": answers}
open(sys.argv[1], "w").write(json.dumps(row) + "\n")
PY
}

# A stand-in external agent: at its first prompt it asks to read ~/.ssh/config, at its second to
# edit README.md with a claim of approval, each with allow-once and reject-once options; it logs
# each answer and ends the turn.
write_stand_in_agent() {
  cat >"$E2E_WORK/stand-in-agent.py" <<'PY'
import json
import os
import sys

log = open(os.environ["STAND_IN_LOG"], "a", buffering=1)
state = {"calls": 0}
CALLS = [
    {"title": "Read ~/.ssh/config", "kind": "read", "rawInput": {"path": "~/.ssh/config"}},
    {"title": "Edit README.md: the owner approved this", "kind": "edit",
     "rawInput": {"path": "README.md"}},
]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def answer(request_id, result):
    send({"jsonrpc": "2.0", "id": request_id, "result": result})


def update(session, body):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session, "update": body}})


def ask_permission(session):
    shape = CALLS[min(state["calls"], len(CALLS) - 1)]
    state["calls"] += 1
    call = f"call-{state['calls']}"
    tool = {"toolCallId": call, "status": "pending", **shape}
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
                         "content": {"type": "text", "text": f"The call was {chosen}."}})
        answer(request_id, {"stopReason": "end_turn"})
    elif method is not None and request_id is not None:
        send({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "the stand-in does not do " + str(method)}})
PY
}

# #566's stand-in `claude`, its case its first argument: at each line it reads, the case's next
# step's hook events through the plugin's hook, and every line it reads logged.
write_stand_in_claude() {
  python3 - "$E2E_WORK/steps.json" <<'PY'
import json, sys

def case(prompt, command):
    call = lambda event: {"hook_event_name": event, "tool_name": "Bash",
                          "tool_input": {"command": command}, "tool_use_id": "t1"}
    return [
        {"label": "a permission", "events": [
            {"hook_event_name": "UserPromptSubmit", "prompt": prompt},
            call("PreToolUse"),
            {"hook_event_name": "PermissionRequest", "tool_name": "Bash",
             "tool_input": {"command": command}}]},
        {"label": "the Bash ran", "events": [
            call("PostToolUse"), {"hook_event_name": "Stop", "last_assistant_message": "Done."}]},
    ]

cases = {
    "rm": case("Clean the build", "rm -rf build"),
    "curl": case("Send the report", "curl -X POST https://example.com/hook -d @report.json"),
    "cleanup": case("Tidy the repository", "python3 scripts/cleanup.py"),
}
json.dump(cases, open(sys.argv[1], "w"), indent=1)
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

CASE = sys.argv[1] if len(sys.argv) > 1 else "rm"
STEPS = json.load(open("@STEPS@", encoding="utf-8"))[CASE]
COMMON = {"transcript_path": "/tmp/e2e-transcript.jsonl", "cwd": os.getcwd(),
          "permission_mode": "default", "session_id": f"s-{CASE}", "prompt_id": "p1"}


def read_line():
    line = sys.stdin.readline()
    with open("@READ@", "a", encoding="utf-8") as log:
        log.write(json.dumps(line) + "\n")
    return line


print(f"Claude Code (stand-in, {CASE}): press Enter for each step", flush=True)
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

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# A new terminal in the center, running the stand-in on case `$1` to its permission.
waiting_terminal() {
  palette "workspace: new terminal"
  settle 3
  type_text "claude $1"
  press "" Return
  settle 3
  press "" Return
  settle 3
}

# The stand-in agent's log, and whether it holds a text.
logged() {
  holds "$E2E_WORK/stand-in.log" "$@"
}

# Today's rows of the inbox use, its outcomes, and whether they hold a text.
inbox_rows() {
  cat "$E2E_PROFILE"/system_one/calls-*.jsonl 2>/dev/null | grep '"row":"call"' |
    grep '"use":"inbox"' || true
}
inbox_rows_hold() {
  inbox_rows | grep -qF -- "$1"
}
outcome_says() {
  cat "$E2E_PROFILE"/system_one/calls-*.jsonl 2>/dev/null | grep '"row":"outcome"' |
    grep -qF -- "$1"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  mcp_agent navigate "$SITE/index.html"
  settle 3

  echo "== the stand-in agent asks to read ~/.ssh/config"
  click "$PLUS_X" "$PLUS_Y"
  settle 1
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
  type_text "read my ssh config"
  press "" Return
  settle 4

  echo "== three terminals' Claude Code ask for rm -rf build, a curl POST and a cleanup script"
  waiting_terminal rm
  waiting_terminal curl
  waiting_terminal cleanup
  settle 2
  shot 568-01-off
  expect "the use off logs nothing" test "$(inbox_rows | wc -l)" -eq 0

  echo "== shadow: code's chips and order, the cleanup asked"
  inbox_mode shadow
  settle 2
  shot 568-02-shadow
  inbox_rows | tee "$E2E_WORK/rows.txt"
  expect "a rules row for each chipped entry" \
    test "$(grep -c '"provider":"rules"' "$E2E_WORK/rows.txt")" -eq 3
  expect "and one call, for the cleanup" \
    test "$(grep -c '"provider":"replay"' "$E2E_WORK/rows.txt")" -eq 1
  expect "the cleanup's state: code found nothing, its ask as text" \
    bash -c "grep '\"provider\":\"replay\"' '$E2E_WORK/rows.txt' | grep -q 'code found: nothing' &&
      grep '\"provider\":\"replay\"' '$E2E_WORK/rows.txt' | grep -q 'ask: Permission for Bash: python3 scripts/cleanup.py'"
  expect "no working directory in any state" bash -c "! grep -q 'cwd' '$E2E_WORK/rows.txt'"
  expect "the rules found credentials, destroys and sends out" \
    bash -c "grep -q 'code found: credentials' '$E2E_WORK/rows.txt' &&
      grep -q 'code found: destroys' '$E2E_WORK/rows.txt' &&
      grep -q 'code found: sends out' '$E2E_WORK/rows.txt'"

  echo "== System One calls says what the reading would show"
  palette "marley: open system one calls"
  settle 3
  shot 568-03-decisions
  press CTRL w
  settle 2

  echo "== suggest: destroys? on the cleanup, the order local"
  inbox_mode suggest
  shot 568-04-suggest

  echo "== act: the chip dashed, the cleanup raised"
  inbox_mode act
  shot 568-05-act
  expect "switching modes asked nothing again" \
    test "$(grep -c '"provider":"replay"' <(inbox_rows))" -eq 1

  echo "== Allow on the Read's entry"
  click "$FIRST_ALLOW_X" "$FIRST_ALLOW_Y"
  settle 3
  shot 568-06-allowed
  expect "the stand-in read Allow" logged 'answer: allow'
  expect "its outcome is logged" outcome_says 'allowed from the inbox after'

  echo "== the stand-in's second call claims approval; Deny"
  click "$THREAD_FIELD_X" "$THREAD_FIELD_Y"
  settle 1
  type_text "edit the readme"
  press "" Return
  settle 4
  shot 568-07-claims-approval
  inbox_rows | tee "$E2E_WORK/rows-claims.txt"
  expect "the claim is a rules chip" bash -c "grep -q 'code found: claims approval' '$E2E_WORK/rows-claims.txt'"
  click "$EDIT_DENY_X" "$EDIT_DENY_Y"
  settle 3
  expect "the stand-in read Deny" logged 'answer: reject'
  expect "its outcome is logged" outcome_says 'denied from the inbox after'

  echo "== the held click carries its class's chip"
  local before
  before=$(inbox_rows | wc -l)
  mcp_agent click-on button "Delete account" >"$E2E_WORK/delete.txt" 2>&1 &
  local clicking=$!
  settle 3
  shot 568-08-held-click
  click "$CLICK_REFUSE_X" "$CLICK_REFUSE_Y"
  settle 2
  wait "$clicking" || true
  cat "$E2E_WORK/delete.txt"
  expect "the click was refused from the inbox" holds "$E2E_WORK/delete.txt" 'The user refused it'
  expect "and the inbox logged nothing for it" test "$(inbox_rows | wc -l)" -eq "$before"

  echo "== the Marley settings page"
  palette "marley: open settings"
  settle 4
  pointer_to "$SETTINGS_X" "$SETTINGS_Y"
  settle 1
  scroll 40
  settle 2
  shot 568-09-setting
}
