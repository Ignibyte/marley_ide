# shellcheck shell=bash
# #566's e2e test: what a stopped Claude Code turn needs. #547's stand-in `claude`, named through
# MARLEY_CLAUDE, acts out one turn per step through the plugin's real hook, each in a session of
# its own with a prompt id. The profile turns #565's layer on with the `replay` provider and the
# scratch repository listed, and the recorded answers, written before Marley starts, stand in for
# Jev. The stop kind's mode moves from step to step: `shadow` logs only (REQ-008), `act` shows the
# kind on the rail's row (REQ-001, REQ-002, REQ-006, REQ-012), `suggest` asks it (REQ-005), and
# `off` makes no call (REQ-009). The rules settle a question and a block with no call (REQ-003,
# REQ-004, REQ-013), a stop with no recorded answer stays `idle` (REQ-007), the user's next prompt
# clears the labels and logs the outcome (REQ-010), and a token in a message is logged masked
# (REQ-011). `mcp_agent fleet-labels` reads the seat's labels as agents see them.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The rail's agent row, where a click shows its terminal again after System One calls, from the first
# run's shots.
AGENT_ROW_X=${AGENT_ROW_X:-110}
AGENT_ROW_Y=${AGENT_ROW_Y:-140}
# A point in the Settings window's page to scroll at.
SETTINGS_X=${SETTINGS_X:-1200}
SETTINGS_Y=${SETTINGS_Y:-500}

# Merges the JSON object `$1` into `marley.system_one` in the profile copy's settings (565's).
system_one_setting() {
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
settings.setdefault("marley", {}).setdefault("system_one", {}).update(changes)
path.write_text(json.dumps(settings, indent=2) + "\n")
PY
}

# Sets the stop kind's mode.
stop_kind_mode() {
  system_one_setting "{\"uses\": {\"stop_kind\": \"$1\"}}"
  settle 2
}

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
  # A token assembled here, so no file of the repository holds one.
  local token
  token=$(printf 'gh%s_%s' p "$(printf 'Fake%.0s' {1..9})")
  local summary
  summary=$(printf 'I went through the crates one by one and wrote down what each of them does. %.0s' {1..8})
  python3 - "$E2E_WORK/steps.json" "$token" "$summary" <<'PY'
import json, sys

path, token, summary = sys.argv[1:]


def turn(label, session, prompt, *tools, stop=None, interrupt=None):
    events = [{"hook_event_name": "UserPromptSubmit", "prompt": prompt}]
    for number, (tool, target) in enumerate(tools, 1):
        key = "command" if tool == "Bash" else "file_path"
        call = {"tool_name": tool, "tool_input": {key: target}, "tool_use_id": f"{session}-{number}"}
        events.append({"hook_event_name": "PreToolUse", **call})
        events.append({"hook_event_name": "PostToolUse", **call})
    if interrupt:
        call = {"tool_name": "Bash", "tool_input": {"command": interrupt}, "tool_use_id": f"{session}-run"}
        events.append({"hook_event_name": "PreToolUse", **call})
        events.append({"hook_event_name": "PostToolUseFailure", **call, "is_interrupt": True})
    if stop is not None:
        events.append({"hook_event_name": "Stop", "last_assistant_message": stop})
    return {"label": label, "session": session, "prompt_id": f"{session}-prompt", "events": events}


blocked = turn("a permission never answered", "s5", "Write the config file",
               stop="I need permission to write the config file.")
write = {"tool_name": "Write", "tool_input": {"file_path": "config.toml"}, "tool_use_id": "s5-write"}
blocked["events"][1:1] = [{"hook_event_name": "PreToolUse", **write},
                          {"hook_event_name": "PermissionRequest", "tool_name": "Write",
                           "tool_input": {"file_path": "config.toml"}}]
steps = [
    turn("shadow: a README, edited and tested", "s1", "Add a README to the project",
         ("Edit", "README.md"), ("Bash", "cargo test"), stop="Done; the tests pass."),
    turn("act: the same turn", "s2", "Add a README to the project",
         ("Edit", "README.md"), ("Bash", "cargo test"), stop="Done; the tests pass."),
    {"label": "the next prompt", "session": "s2", "prompt_id": "s2-next",
     "events": [{"hook_event_name": "UserPromptSubmit", "prompt": "Now add a license"}]},
    turn("an edit and no check", "s3", "Fix the typo in the README",
         ("Edit", "README.md"), stop="Fixed the typo; the tests pass."),
    turn("a question", "s4", "Add a README", ("Edit", "README.md"),
         stop="Added the README. Should I also add a license?"),
    blocked,
    turn("suggest: going on", "s6", "Wire the tests", stop="Next I will wire the tests into CI."),
    turn("two parts, one done", "s7", "Add a README. Add a license.",
         ("Edit", "README.md"), ("Bash", "cargo test"), stop="Added the README."),
    turn("no recorded answer", "s8", "Tidy the imports", ("Edit", "src/lib.rs"),
         stop="Tidied the imports."),
    turn("off", "s9", "Rename the module", stop="Renamed it."),
    turn("an interrupt", "s10", "Run the slow tests", interrupt="cargo test -- --ignored"),
    turn("a long message", "s11", "Summarize the changes",
         stop=summary + "Should I also add a license?"),
    turn("a token in the message", "s12", "Set up the deploy",
         stop=f"I set GITHUB_TOKEN={token} in the deploy environment."),
]
json.dump(steps, open(path, "w"), indent=1)
PY
  # The recorded answers, in the order the steps ask. A row answers once unless it repeats; the
  # question's row repeats, so a stop the rules settle would read `done · claimed` were it asked.
  cat >"$E2E_PROFILE/system_one/replay.jsonl" <<'JSONL'
{"set": "stop_kind_1/1", "match": "message: Done; the tests pass.", "answers": {"kind": {"type": "choice", "choice": "done_checked", "confidence": 0.91, "probabilities": {"done_checked": 0.91}}, "part_1_done": {"type": "noul", "noul": 0.95}}}
{"set": "stop_kind_1/1", "match": "message: Done; the tests pass.", "answers": {"kind": {"type": "choice", "choice": "done_checked", "confidence": 0.91, "probabilities": {"done_checked": 0.91}}, "part_1_done": {"type": "noul", "noul": 0.95}}}
{"set": "stop_kind_1/1", "match": "message: Fixed the typo", "answers": {"kind": {"type": "choice", "choice": "done_checked", "confidence": 0.93, "probabilities": {"done_checked": 0.93}}}}
{"set": "stop_kind_1/1", "match": "Should I also add a license?", "repeat": true, "answers": {"kind": {"type": "choice", "choice": "done_claimed", "confidence": 0.8, "probabilities": {"done_claimed": 0.8}}}}
{"set": "stop_kind_1/1", "match": "message: Next I will wire", "answers": {"kind": {"type": "choice", "choice": "still_going", "confidence": 0.88, "probabilities": {"still_going": 0.88}}}}
{"set": "stop_kind_2/1", "match": "part 2: Add a license.", "answers": {"kind": {"type": "choice", "choice": "done_checked", "confidence": 0.9, "probabilities": {"done_checked": 0.9}}, "part_1_done": {"type": "noul", "noul": 0.94}, "part_2_done": {"type": "noul", "noul": 0.05}}}
JSONL
  sed -e "s|@HOOK@|$HOOK|" -e "s|@STEPS@|$E2E_WORK/steps.json|" -e "s|@LOG@|$E2E_WORK/plugin.log|" \
    >"$bin/claude" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code (#547's): `claude plugin ...` logs its arguments. Otherwise, at each line
# it reads, it runs the plugin's hook with the next step's payloads, in the step's session and
# prompt, as Claude Code runs a hook, and writes each answer's sequence to its terminal.
import json
import os
import subprocess
import sys

if sys.argv[1:2] == ["plugin"]:
    with open("@LOG@", "a", encoding="utf-8") as log:
        log.write(" ".join(sys.argv[1:]) + "\n")
    sys.exit(0)

STEPS = json.load(open("@STEPS@", encoding="utf-8"))
COMMON = {"transcript_path": "/tmp/e2e-transcript.jsonl", "cwd": os.getcwd(),
          "permission_mode": "default"}

print("Claude Code (stand-in): press Enter for each step", flush=True)
for number, step in enumerate(STEPS, 1):
    if not sys.stdin.readline():
        break
    for event in step["events"]:
        payload = {**COMMON, "session_id": step["session"], "prompt_id": step["prompt_id"], **event}
        answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps(payload),
                                capture_output=True, text=True, check=False).stdout
        sequence = json.loads(answer or "{}").get("terminalSequence")
        if sequence:
            sys.stdout.write(sequence)
            sys.stdout.flush()
    print(f"step {number}: {step['label']}", flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$bin/claude"
  # The `claude` Marley runs for the plugin's commands: the stand-in, never the real one.
  export MARLEY_CLAUDE=$bin/claude
  git init -q -b main "$E2E_WORK/repo"
  system_one_setting "{\"enabled\": true, \"provider\": \"replay\", \"projects\": [\"$E2E_WORK/repo\"], \"uses\": {\"stop_kind\": \"shadow\"}}"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# The next step of the stand-in, and time for its answer to land.
next_step() {
  press "" Return
  settle 3
}

# The stand-in agent's reading of the seats' labels, kept as $E2E_WORK/<label>.txt.
seat_labels() {
  mcp_agent fleet-labels | tee "$E2E_WORK/$1.txt"
}

# Today's call rows.
calls() {
  cat "$E2E_PROFILE"/system_one/calls-*.jsonl 2>/dev/null | grep '"row":"call"' || true
}

# The last call row, kept as $E2E_WORK/<label>.json, with its provider, reading and state printed.
last_call() {
  calls | tail -n 1 >"$E2E_WORK/$1.json"
  python3 - "$E2E_WORK/$1.json" <<'PY'
import json, sys

row = json.load(open(sys.argv[1]))
print(f"  {row['use']} {row['mode']} {row['provider']} {row['set']}: {row['reading']}")
for line in (row.get("state") or "").splitlines():
    print(f"    {line}")
PY
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "claude"
  press "" Return
  settle 3

  echo "== shadow"
  next_step
  shot 566-01a-shadow-row
  seat_labels shadow
  expect "shadow puts no kind on the seat" bash -c "! grep -q stop_kind '$E2E_WORK/shadow.txt'"
  last_call shadow-call
  expect "shadow logged the reading it would have shown" holds "$E2E_WORK/shadow-call.json" \
    '"mode":"shadow"' 'kind: done_checked 0.91' '"set":"stop_kind_1/1"'
  palette "marley: open system one calls"
  settle 3
  shot 566-01-shadow
  click "$AGENT_ROW_X" "$AGENT_ROW_Y"
  settle 2

  echo "== act"
  stop_kind_mode act
  next_step
  shot 566-02-done-checked
  seat_labels act
  expect "the seat carries the kind, its source and its confidence" holds "$E2E_WORK/act.txt" \
    "stop_kind 'done_checked'" "stop_kind_source 'model'" "stop_kind_confidence '0.91'"
  next_step
  seat_labels next-prompt
  expect "the next prompt clears the kind" bash -c "! grep -q stop_kind '$E2E_WORK/next-prompt.txt'"
  expect "and logs the stop's outcome" bash -c \
    "grep '\"row\":\"outcome\"' '$E2E_PROFILE'/system_one/calls-*.jsonl | grep -q 'next prompt after'"

  echo "== an edit and no check"
  next_step
  shot 566-03-done-claimed
  seat_labels claimed
  expect "a claim with no check after the edit reads done_claimed" holds "$E2E_WORK/claimed.txt" \
    "stop_kind 'done_claimed'"

  echo "== a question, by rule"
  next_step
  shot 566-04-asks-you-by-rule
  last_call question
  expect "the rules read the question, and nothing was asked" holds "$E2E_WORK/question.json" \
    '"provider":"rules"' 'kind: asks_you (rules)'

  echo "== a permission never answered, by rule"
  next_step
  shot 566-05-blocked-by-rule
  last_call blocked
  expect "the rules read the block" holds "$E2E_WORK/blocked.json" '"provider":"rules"' \
    'kind: blocked (rules)'

  echo "== suggest"
  stop_kind_mode suggest
  next_step
  shot 566-06-suggest

  echo "== a part not covered"
  stop_kind_mode act
  next_step
  shot 566-07-part-missing
  seat_labels parts
  expect "the part the message leaves out is on the seat" holds "$E2E_WORK/parts.txt" \
    'Add a license.'

  echo "== no recorded answer"
  next_step
  shot 566-08-no-row
  last_call no-row
  expect "the log says there was no signal" holds "$E2E_WORK/no-row.json" \
    'kind: no signal (no replay row)'

  echo "== off"
  stop_kind_mode off
  local before
  before=$(calls | wc -l)
  next_step
  shot 566-09-off
  expect "off makes no call and writes no row" test "$(calls | wc -l)" -eq "$before"

  echo "== an interrupt, by rule"
  stop_kind_mode act
  next_step
  shot 566-10-interrupted
  last_call interrupted
  expect "the interrupt is the rules' reading" holds "$E2E_WORK/interrupted.json" \
    '"provider":"rules"' 'kind: interrupted (rules)'

  echo "== a long message's end"
  next_step
  shot 566-11-long-message
  last_call long
  expect "the message kept its start, then its last sentences and the question" \
    holds "$E2E_WORK/long.json" 'message: I went through the crates' ' … I went through' \
    'does. Should I also add a license?' 'kind: asks_you (rules)'

  echo "== a token in the message"
  next_step
  last_call token
  expect "the token was logged masked" holds "$E2E_WORK/token.json" '[redacted: secret]'
  expect "and not as it was written" bash -c "! grep -q 'FakeFake' '$E2E_WORK/token.json'"

  echo "== the Stop Kind item"
  palette "marley: open settings"
  settle 4
  pointer_to "$SETTINGS_X" "$SETTINGS_Y"
  settle 1
  scroll 20
  settle 2
  shot 566-12-settings
}
