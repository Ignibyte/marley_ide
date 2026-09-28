# shellcheck shell=bash
# #509's e2e test: per-turn diffs for Claude Code in a terminal. A scratch repository, committed
# with `src/lib.txt` and `src/util.txt`, holds a staged change of the user's to `src/util.txt`, a
# planted turn ref from 40 days back and one from 5 days back, under a git config of the
# scenario's own. #566's stand-in `claude`, with pseudo-events that write files and run programs
# between its hook events, runs five turns in one terminal, each by an Enter: "Add a README"
# writes `README.md` and runs `sed -i` on `src/lib.txt`, then Stop; "Fix the typo in util" edits
# `src/util.txt` and sends no Stop; "Explain the build" closes it, changes nothing, then Stop; a
# `<task-notification>` prompt edits `src/lib.txt`, then Stop; "Refactor the parser" edits
# `src/util.txt`, then StopFailure. The row's "Turns (4)" opens to the four that changed the tree,
# newest first, with their files and marks (REQ-001, REQ-004 to REQ-006); a click opens a turn in
# the commit view with its changes alone (REQ-002, REQ-003); the refs, their parents and the
# user's index and HEAD are read from git (REQ-007), and the old ref is gone while the recent one
# stays (REQ-009).
compositor sway

SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
# The turns' line and rows, from the first run's shots.
TURNS_X=${TURNS_X:-60}
TURNS_Y=${TURNS_Y:-228}
FIRST_TURN_X=${FIRST_TURN_X:-120}
FIRST_TURN_Y=${FIRST_TURN_Y:-324}
SECOND_TURN_X=${SECOND_TURN_X:-120}
SECOND_TURN_Y=${SECOND_TURN_Y:-300}

repo_git() {
  git -C "$E2E_WORK/repo" "$@"
}

# The user's side of the repository: what no turn may change.
user_state() {
  echo "HEAD $(repo_git rev-parse HEAD)"
  echo "index:"
  repo_git diff --cached
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config
  local repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$config/plugins" "$repo/src"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  # A git config of the scenario's own, which Marley's git and Zed's inherit: no signing, no
  # identity but this one.
  cat >"$E2E_WORK/gitconfig" <<'CONFIG'
[user]
	name = Scenario
	email = scenario@example.invalid
CONFIG
  export GIT_CONFIG_GLOBAL=$E2E_WORK/gitconfig GIT_CONFIG_NOSYSTEM=1
  # The plugin listed at the version Marley ships, so the agent bar offers no update.
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/$SHIPPED", "version": "$SHIPPED"}]}}
JSON
  export CLAUDE_CONFIG_DIR=$config

  git init -q -b turns "$repo"
  printf 'alpha\nbeta\n' >"$repo/src/lib.txt"
  printf 'the utl module\n' >"$repo/src/util.txt"
  repo_git add -A
  repo_git commit -q -m "Start"
  # The user's own change, staged before the session.
  printf 'the utl module\nstaged by the user\n' >"$repo/src/util.txt"
  repo_git add src/util.txt
  # Two turn refs of earlier sessions: one 40 days old, which the first pin prunes, and one 5
  # days old, which it keeps.
  local old recent
  old=$(GIT_COMMITTER_DATE="$(($(date +%s) - 40 * 86400)) +0000" \
    git -C "$repo" commit-tree 'HEAD^{tree}' -p HEAD -m "An old turn")
  recent=$(GIT_COMMITTER_DATE="$(($(date +%s) - 5 * 86400)) +0000" \
    git -C "$repo" commit-tree 'HEAD^{tree}' -p HEAD -m "A recent turn")
  repo_git update-ref refs/marley/turns/s-old/1 "$old"
  repo_git update-ref refs/marley/turns/s-recent/1 "$recent"
  echo "== before the session"
  user_state | tee "$E2E_WORK/before.txt"
  repo_git for-each-ref --format='%(refname)' refs/marley/turns/

  write_stand_in_claude "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude
  open_path "$repo"
}

# #566's stand-in `claude`, its case its first argument. A step's events run at a line read; an
# event `sleep` waits, `write` writes a file and `run` runs a program in the working directory.
write_stand_in_claude() {
  python3 - "$E2E_WORK/steps.json" <<'PY'
import json, sys

def prompt(text, number):
    # The checkpoint of the turn's start is taken as the prompt's frame lands; the edits wait
    # for it, as a model's first reply does.
    return [{"hook_event_name": "UserPromptSubmit", "prompt": text, "prompt_id": f"p{number}"},
            {"sleep": 2}]

def tool(event, name, path, number):
    return {"hook_event_name": event, "tool_name": name, "tool_input": {"file_path": path},
            "tool_use_id": f"t{number}"}

def stop(message):
    return [{"sleep": 1}, {"hook_event_name": "Stop", "last_assistant_message": message}]

cases = {
    "turns": [
        {"label": "Add a README", "events": prompt("Add a README", 1) + [
            tool("PreToolUse", "Write", "README.md", 1),
            {"write": "README.md", "text": "# Turns\n\nA scratch repository.\n"},
            tool("PostToolUse", "Write", "README.md", 1),
            {"run": ["sed", "-i", "s/alpha/ALPHA/", "src/lib.txt"]},
        ] + stop("Added it.")},
        {"label": "Fix the typo in util, with no Stop", "events": prompt("Fix the typo in util", 2) + [
            tool("PreToolUse", "Edit", "src/util.txt", 2),
            {"run": ["sed", "-i", "s/utl/util/", "src/util.txt"]},
            tool("PostToolUse", "Edit", "src/util.txt", 2),
        ]},
        {"label": "Explain the build", "events": prompt("Explain the build", 3) + stop("It builds.")},
        {"label": "a task notification", "events": prompt(
            "<task-notification>\n<task-id>b1</task-id>\n<status>completed</status>\n"
            "<summary>The build finished</summary>\n</task-notification>", 4) + [
            tool("PreToolUse", "Edit", "src/lib.txt", 4),
            {"run": ["sed", "-i", "s/beta/beta\\ngamma/", "src/lib.txt"]},
            tool("PostToolUse", "Edit", "src/lib.txt", 4),
        ] + stop("Noted.")},
        {"label": "Refactor the parser, which fails", "events": prompt("Refactor the parser", 5) + [
            tool("PreToolUse", "Edit", "src/util.txt", 5),
            {"run": ["sed", "-i", "s/module/parser module/", "src/util.txt"]},
            tool("PostToolUse", "Edit", "src/util.txt", 5),
            {"sleep": 1},
            {"hook_event_name": "StopFailure", "error": "rate_limit"},
        ]},
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

CASE = sys.argv[1] if len(sys.argv) > 1 else "turns"
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
        if "sleep" in event:
            time.sleep(event["sleep"])
            continue
        if "write" in event:
            with open(event["write"], "w", encoding="utf-8") as out:
                out.write(event["text"])
            continue
        if "run" in event:
            subprocess.run(event["run"], check=False)
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

# The session's turn refs, oldest first.
session_refs() {
  repo_git for-each-ref --sort=refname --format='%(refname)' refs/marley/turns/s-turns/
}

# The files turn ref `$1` changed against its parent.
turn_files() {
  repo_git diff --name-only "$1^" "$1" | sort | tr '\n' ' ' | sed 's/ $//'
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== five turns in one terminal, each by an Enter"
  palette "workspace: new terminal"
  settle 3
  type_text "claude turns"
  press "" Return
  settle 3
  local turn
  for turn in 1 2 3 4 5; do
    echo "   turn $turn"
    press "" Return
    settle 6
  done
  settle 3
  shot 509-00-row

  echo "== the row's turns, opened"
  click "$TURNS_X" "$TURNS_Y"
  settle 2
  shot 509-01-turns

  echo "== the refs"
  repo_git for-each-ref --format='%(refname) %(committerdate:unix)' refs/marley/turns/ |
    tee "$E2E_WORK/refs.txt"
  expect "the session pinned four turns" test "$(session_refs | wc -l)" -eq 4
  expect "the 40-day-old ref is gone" bash -c "! grep -q 's-old/' '$E2E_WORK/refs.txt'"
  expect "the 5-day-old ref stays" grep -q 's-recent/1' "$E2E_WORK/refs.txt"
  local ref
  for ref in $(session_refs); do
    echo "$ref: $(repo_git log -1 --format='%an <%ae> | %s' "$ref") | parent: $(repo_git log -1 --format=%s "$ref^") | $(turn_files "$ref")"
    repo_git log -1 --format=%b "$ref"
  done | tee "$E2E_WORK/turns.txt"
  expect "each turn is Marley's, on a checkpoint" \
    test "$(grep -c '^refs/.*: Marley <marley@localhost> | .* | parent: Checkpoint |' "$E2E_WORK/turns.txt")" -eq 4
  expect "turn 1 holds the README and the sed" \
    test "$(turn_files refs/marley/turns/s-turns/1)" = "README.md src/lib.txt"
  expect "turn 2 holds the typo's fix alone" \
    test "$(turn_files refs/marley/turns/s-turns/2)" = "src/util.txt"
  expect "turn 3 is the task notification's" \
    bash -c "grep -q '^refs/marley/turns/s-turns/3: Marley <marley@localhost> | task notification |' '$E2E_WORK/turns.txt'"
  expect "turn 4 is the failed refactor's" \
    bash -c "grep -q '^refs/marley/turns/s-turns/4: Marley <marley@localhost> | Refactor the parser |' '$E2E_WORK/turns.txt'"
  expect "the trailers name the session and the turn" \
    bash -c "grep -q '^Marley-Session: s-turns$' '$E2E_WORK/turns.txt' && grep -q '^Marley-Turn: 4$' '$E2E_WORK/turns.txt'"

  echo "== the user's index and HEAD, untouched"
  user_state | tee "$E2E_WORK/after.txt"
  expect "HEAD and the index are as the user left them" cmp -s "$E2E_WORK/before.txt" "$E2E_WORK/after.txt"
  repo_git status --porcelain | tee "$E2E_WORK/status.txt"
  expect "the agent's edits are in the tree, unstaged" \
    bash -c "grep -qx '?? README.md' '$E2E_WORK/status.txt' && grep -qx ' M src/lib.txt' '$E2E_WORK/status.txt' && grep -qx 'MM src/util.txt' '$E2E_WORK/status.txt'"

  echo "== the first turn, in the commit view"
  click "$FIRST_TURN_X" "$FIRST_TURN_Y"
  settle 4
  shot 509-02-first-turn

  echo "== the turn a prompt closed, in the commit view"
  click "$SECOND_TURN_X" "$SECOND_TURN_Y"
  settle 4
  shot 509-03-closed-by-a-prompt
  expect "the stand-in read the five Enters and nothing else" test "$(wc -l <"$E2E_WORK/stdin.log")" -eq 5
}
