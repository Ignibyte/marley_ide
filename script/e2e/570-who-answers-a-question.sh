# shellcheck shell=bash
# #570's e2e test: who should answer what an agent waits on. #566's stand-in `claude`, one per
# terminal, waits on A, `Bash: git push --force origin main` (the owner's by #568's rewrites
# history), B, `Read: src/lib.rs` (could proceed by rule; its request comes once the focus has moved
# on, and its stand-in ends the wait by itself later), C, `Bash: npm test` (open; the recorded
# answer reads the manager's at 0.88) and D, an AskUserQuestion with two options (open; read as
# the agent's to go on with at 0.90), whose answer asks E, `Bash: cargo test` (open; read as
# cannot tell). The layer runs on the replay provider with the scratch repository listed, #568's
# use in shadow so its rules' chips and levels show. The rules mark A and B with rows and no call
# (REQ-001, REQ-002); C, D and E are asked (REQ-003); suggest shows the readings with a question
# mark (REQ-004) and the tooltips name the rule or the reading (REQ-005); the fleet listing
# carries D's options and the hook keeps any payload under the bound (REQ-008); act orders the
# level owner and unclear, manager, could proceed, with E unclear (REQ-006, REQ-007); off shows
# no marks (REQ-009); A answered in its terminal logs `owner`, B cleared by its own stand-in
# `agent` (REQ-010); nothing but the scenario's Enters reaches a stand-in (REQ-011); and the
# Marley page lists Question Route (REQ-012).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The marks and entries, and the settings page, from the first run's shots.
C_MARK_X=${C_MARK_X:-55}
C_MARK_Y=${C_MARK_Y:-221}
A_MARK_X=${A_MARK_X:-177}
A_MARK_Y=${A_MARK_Y:-156}
D_ENTRY_X=${D_ENTRY_X:-110}
D_ENTRY_Y=${D_ENTRY_Y:-247}
A_ENTRY_X=${A_ENTRY_X:-110}
A_ENTRY_Y=${A_ENTRY_Y:-115}
SETTINGS_X=${SETTINGS_X:-1200}
SETTINGS_Y=${SETTINGS_Y:-500}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config
  mkdir -p "$home" "$bin" "$config/plugins" "$E2E_PROFILE/system_one"
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
  write_stand_in_claude "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude
  write_mcp_agent
  write_replay
  bound_check
  marley_setting "{\"system_one\": {\"enabled\": true, \"provider\": \"replay\", \"projects\": [\"$E2E_WORK/repo\"], \"uses\": {\"inbox\": \"shadow\", \"question_route\": \"off\"}}}"
  git init -q -b route "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
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

# The question route's mode.
route_mode() {
  marley_setting "{\"system_one\": {\"uses\": {\"question_route\": \"$1\"}}}"
  settle 3
}

# The recorded answers for the three open entries.
write_replay() {
  python3 - "$E2E_PROFILE/system_one/replay.jsonl" <<'PY'
import json, sys
def row(match, choice, confidence):
    probabilities = {option: 0.02 for option in ("owner", "manager", "agent_proceeds", "cannot_tell")}
    probabilities[choice] = confidence
    return {"set": "question_route/1", "match": match, "repeat": True, "answers": {
        "route": {"type": "choice", "choice": choice, "confidence": confidence,
                  "probabilities": probabilities},
        "answerable_from_prompt": {"type": "noul", "noul": 0.2}}}
rows = [row("npm test", "manager", 0.88), row("Which base branch", "agent_proceeds", 0.9),
        row("cargo test", "cannot_tell", 0.9)]
open(sys.argv[1], "w").write("".join(json.dumps(item) + "\n" for item in rows))
PY
}

# The plugin's hook, fed 100 KB of options, still answers under Claude Code's 4,096 bytes with the
# tool and its preview kept.
bound_check() {
  python3 - "$HOOK" "$E2E_WORK/bound.txt" <<'PY'
import base64, json, subprocess, sys
hook, out = sys.argv[1:]
options = [{"label": "option " + "x" * 1000 + str(number), "description": "d"} for number in range(100)]
event = {"hook_event_name": "PreToolUse", "tool_name": "AskUserQuestion", "tool_use_id": "q1",
         "session_id": "s", "cwd": "/" + "c" * 3000,
         "tool_input": {"questions": [{"question": "Which one?", "header": "h", "options": options,
                                       "multiSelect": False}]}}
answer = subprocess.run([sys.executable, hook], input=json.dumps(event), capture_output=True,
                        text=True, env={"TERM_PROGRAM": "zed"}, check=False).stdout
sequence = json.loads(answer).get("terminalSequence", "")
body = json.loads(base64.b64decode(sequence.split(";")[-1].rstrip("\x07")))
open(out, "w").write(f"answer {len(answer.encode())} bytes; fields {sorted(body)}; "
                     f"options {len(body.get('options', []))}\n")
PY
  cat "$E2E_WORK/bound.txt"
}

# #566's stand-in `claude`, its case its first argument. A step's events run at a line read, or,
# for a step marked `auto`, after its `after` seconds with nothing read; an event `sleep` waits.
write_stand_in_claude() {
  python3 - "$E2E_WORK/steps.json" <<'PY'
import json, sys

def bash(event, command, number=1):
    return {"hook_event_name": event, "tool_name": "Bash", "tool_input": {"command": command},
            "tool_use_id": f"t{number}"}

def permission(prompt, command):
    return [
        {"label": "a permission", "events": [
            {"hook_event_name": "UserPromptSubmit", "prompt": prompt},
            bash("PreToolUse", command),
            {"hook_event_name": "PermissionRequest", "tool_name": "Bash",
             "tool_input": {"command": command}}]},
        {"label": "the Bash ran", "events": [
            bash("PostToolUse", command),
            {"hook_event_name": "Stop", "last_assistant_message": "Done."}]},
    ]

read = {"hook_event_name": "PreToolUse", "tool_name": "Read",
        "tool_input": {"file_path": "src/lib.rs"}, "tool_use_id": "r1"}
question = {"hook_event_name": "PreToolUse", "tool_name": "AskUserQuestion", "tool_use_id": "q1",
            "tool_input": {"questions": [{"question": "Which base branch?", "header": "Branch",
                                          "multiSelect": False,
                                          "options": [{"label": "main", "description": "The trunk"},
                                                      {"label": "release", "description": "The release line"}]}]}}
cases = {
    "push": permission("Publish the branch", "git push --force origin main"),
    "test": permission("Run the tests", "npm test"),
    "read": [
        {"label": "a read, asked once the focus has moved", "events": [
            {"hook_event_name": "UserPromptSubmit", "prompt": "Look at the library"},
            dict(read), {"sleep": 10},
            {"hook_event_name": "PermissionRequest", "tool_name": "Read",
             "tool_input": {"file_path": "src/lib.rs"}}]},
        {"label": "the read went on by itself", "auto": True, "after": 150, "events": [
            {**read, "hook_event_name": "PostToolUse"},
            {"hook_event_name": "Stop", "last_assistant_message": "Read it."}]},
    ],
    "ask": [
        {"label": "a question", "events": [
            {"hook_event_name": "UserPromptSubmit", "prompt": "Open the pull request"}, question]},
        {"label": "answered, and a cargo test", "events": [
            {**question, "hook_event_name": "PostToolUse"},
            bash("PreToolUse", "cargo test", 2),
            {"hook_event_name": "PermissionRequest", "tool_name": "Bash",
             "tool_input": {"command": "cargo test"}}]},
        {"label": "the tests ran", "events": [
            bash("PostToolUse", "cargo test", 2),
            {"hook_event_name": "Stop", "last_assistant_message": "Done."}]},
    ],
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
import time

if sys.argv[1:2] == ["plugin"]:
    sys.exit(0)

CASE = sys.argv[1] if len(sys.argv) > 1 else "push"
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
    if step.get("auto"):
        time.sleep(step.get("after", 0))
    elif not read_line():
        break
    for event in step["events"]:
        if "sleep" in event:
            time.sleep(event["sleep"])
            continue
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

# A new terminal in the center, running the stand-in on case `$1` to its first step.
waiting_terminal() {
  palette "workspace: new terminal"
  settle 3
  type_text "claude $1"
  press "" Return
  settle 3
  press "" Return
  settle 3
}

# Today's rows of the question route, its outcomes, and whether they hold a text.
route_rows() {
  cat "$E2E_PROFILE"/system_one/calls-*.jsonl 2>/dev/null | grep '"row":"call"' |
    grep '"use":"question_route"' || true
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

  echo "== four agents wait: a forced push, a read, the tests, a question"
  waiting_terminal push
  waiting_terminal read
  waiting_terminal test
  waiting_terminal ask
  settle 10
  expect "the use off logs nothing" test "$(route_rows | wc -l)" -eq 0

  echo "== suggest: the rules' marks, and the readings with a question mark"
  route_mode suggest
  settle 3
  shot 570-01-marks
  route_rows | tee "$E2E_WORK/rows.txt"
  expect "two rules rows, for the push and the read" \
    test "$(grep -c '"provider":"rules"' "$E2E_WORK/rows.txt")" -eq 2
  expect "and two calls, for the tests and the question" \
    test "$(grep -c '"provider":"replay"' "$E2E_WORK/rows.txt")" -eq 2
  expect "the question's state carries its options" \
    bash -c "grep 'Which base branch' '$E2E_WORK/rows.txt' | grep -q 'options: main, release'"
  expect "no working directory in any state" bash -c "! grep -q 'cwd' '$E2E_WORK/rows.txt'"

  echo "== the tooltips: the reading on the tests, the rule on the push"
  pointer_to "$C_MARK_X" "$C_MARK_Y"
  settle 2
  shot 570-02a-tooltip
  pointer_to "$A_MARK_X" "$A_MARK_Y"
  settle 2
  shot 570-02b-tooltip

  echo "== the fleet listing carries the question's options"
  mcp_agent fleet | tee "$E2E_WORK/fleet.txt"
  expect "the question's options" holds "$E2E_WORK/fleet.txt" "options ['main', 'release']"
  expect "the hook's answer stays under the bound, the tool and preview kept" \
    bash -c "awk '{exit !(\$2 < 4096)}' '$E2E_WORK/bound.txt' && grep -q \"'preview', 'session_id', 'tool'\" '$E2E_WORK/bound.txt'"

  echo "== the question answered; the tests asked; act orders the level"
  click "$D_ENTRY_X" "$D_ENTRY_Y"
  settle 2
  press "" Return
  settle 4
  route_mode act
  settle 2
  shot 570-03-act-order
  expect "the cargo test was asked too" \
    test "$(route_rows | grep -c 'cargo test')" -eq 1

  echo "== off: no marks, and no call"
  local before
  before=$(route_rows | wc -l)
  route_mode off
  shot 570-04-off
  expect "off makes no call" test "$(route_rows | wc -l)" -eq "$before"

  echo "== act; the push answered in its terminal; the read goes on by itself"
  route_mode act
  click "$A_ENTRY_X" "$A_ENTRY_Y"
  settle 2
  press "" Return
  settle 4
  expect "the push's outcome names its owner" outcome_says 'owner after'
  local waited=0
  until outcome_says 'agent after' || ((waited >= 150)); do
    settle 5
    waited=$((waited + 5))
  done
  shot 570-05-answered
  expect "the read's outcome names the agent" outcome_says 'agent after'
  cat "$E2E_WORK/stdin.log"
  expect "the stand-ins read nothing but the scenario's Enters" python3 -c '
import json, sys
lines = [json.loads(line) for line in open(sys.argv[1])]
sys.exit(0 if lines and all(line == "\n" for line in lines) else 1)' "$E2E_WORK/stdin.log"

  echo "== the Marley settings page"
  palette "marley: open settings"
  settle 4
  pointer_to "$SETTINGS_X" "$SETTINGS_Y"
  settle 1
  scroll 44
  settle 2
  shot 570-06-setting
}
