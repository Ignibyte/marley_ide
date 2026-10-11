# shellcheck shell=bash
# #714's visual check: Keep and Undo This Turn… on a turn row. A scratch repository committed with
# `src/lib.txt` (nine lines, `alpha` to `iota`) and `src/util.txt`, with a staged change of the
# user's to `src/util.txt`, under a git config of the scenario's own. #509's stand-in `claude`,
# case `turns`, runs four turns, each with a Stop: "Add a README" (writes `README.md`, line 1),
# "Shout iota" (line 9), "Fix the typo in util" (`src/util.txt`), "Shout gamma" (line 3). Then the
# scenario, as the user, writes `notes.txt` and changes lines 5 and 3. Each edit sits an unchanged
# line away from the others, since git merges adjacent lines as one change.
# - "Add a README" right-clicked (`menu`, REQ-001), Undo chosen (`confirm`, REQ-002), confirmed
#   (`undone`, REQ-003 to REQ-006): `README.md` gone, line 1 back, the later turns' and the user's
#   lines kept, `HEAD`, the index and the refs as before.
# - Undo on "Shout gamma", whose line the user changed since: refused (`conflict`, REQ-008).
# - Keep on "Fix the typo in util" (`kept`, REQ-009).
# - `src/util.txt` typed into, Undo on "Fix the typo in util": refused (`unsaved`, REQ-010).
# - A second terminal's `claude busy` working, Undo on "Shout iota": refused (`another-agent`,
#   REQ-011).
# - Undo on "Shout iota" with line 9 changed while the question is open: refused at the write
#   (`changed-meanwhile`, REQ-013).
compositor sway

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The turns' line and rows, newest first, as the first run's shots found them; `STOP_AT=<shot>`
# ends the run after that shot.
TURNS_X=${TURNS_X:-60}
TURNS_Y=${TURNS_Y:-268}
ROW_X=${ROW_X:-120}
GAMMA_Y=${GAMMA_Y:-292}
UTIL_Y=${UTIL_Y:-316}
IOTA_Y=${IOTA_Y:-340}
README_Y=${README_Y:-364}
STOP_AT=${STOP_AT:-}

repo_git() {
  git -C "$E2E_WORK/repo" "$@"
}

# What no undo may change, and the files: HEAD, the index, the turn refs, the status and each
# file's hash.
state() {
  echo "HEAD $(repo_git rev-parse HEAD)"
  echo "index:"
  repo_git diff --cached
  repo_git for-each-ref --format='%(refname) %(objectname)' refs/marley/turns/
  repo_git status --porcelain
  (cd "$E2E_WORK/repo" && find . -path ./.git -prune -o -type f -print | sort | xargs sha1sum)
}

line() { sed -n "${1}p" "$E2E_WORK/repo/src/lib.txt"; }

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config
  local repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$config/plugins" "$repo/src"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  cat >"$E2E_WORK/gitconfig" <<'CONFIG'
[user]
	name = Scenario
	email = scenario@example.invalid
CONFIG
  export GIT_CONFIG_GLOBAL=$E2E_WORK/gitconfig GIT_CONFIG_NOSYSTEM=1
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/$SHIPPED", "version": "$SHIPPED"}]}}
JSON
  export CLAUDE_CONFIG_DIR=$config
  git init -q -b turns "$repo"
  printf '%s\n' alpha beta gamma delta epsilon zeta eta theta iota >"$repo/src/lib.txt"
  printf 'the utl module\n' >"$repo/src/util.txt"
  repo_git add -A
  repo_git commit -q -m "Start"
  printf 'the utl module\nstaged by the user\n' >"$repo/src/util.txt"
  repo_git add src/util.txt
  write_stand_in_claude "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude
  open_path "$repo"
}

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
            {"write": "README.md", "text": "# Turns\n"},
            tool("PostToolUse", "Write", "README.md", 1),
            {"run": ["sed", "-i", "s/^alpha$/ALPHA/", "src/lib.txt"]},
        ] + stop("Added it.")},
        {"label": "Shout iota", "events": prompt("Shout iota", 2) + [
            {"run": ["sed", "-i", "s/^iota$/IOTA/", "src/lib.txt"]},
        ] + stop("Shouted.")},
        {"label": "Fix the typo in util", "events": prompt("Fix the typo in util", 3) + [
            {"run": ["sed", "-i", "s/utl/util/", "src/util.txt"]},
        ] + stop("Fixed.")},
        {"label": "Shout gamma", "events": prompt("Shout gamma", 4) + [
            {"run": ["sed", "-i", "s/^gamma$/GAMMA/", "src/lib.txt"]},
        ] + stop("Shouted.")},
    ],
    "busy": [
        {"label": "a long turn", "events": prompt("Think hard", 1) + [{"sleep": 25}]
         + stop("Thought.")},
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
  settle "${2:-2}"
}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

# Opens the menu of the turn row at `$1` and picks its `$2`th entry.
menu_pick() {
  click "$ROW_X" "$1" right
  settle 1
  for _ in $(seq "$2"); do
    press "" Down
  done
  press "" Return
  settle 3
}

# Whether the state's kept part (HEAD, index, refs) matches the one saved as `$1`.
same_kept() {
  state >"$E2E_WORK/now.txt"
  diff <(sed -n '/^HEAD/,/^refs/p;/^refs/p' "$E2E_WORK/$1") <(sed -n '/^HEAD/,/^refs/p;/^refs/p' "$E2E_WORK/now.txt")
}

# Whether every file is as in the state saved as `$1`.
same_files() {
  state >"$E2E_WORK/now.txt"
  diff <(grep '  \./' "$E2E_WORK/$1") <(grep '  \./' "$E2E_WORK/now.txt")
}

undone_as_asked() {
  echo "lines: $(line 1) $(line 3) $(line 5) $(line 9)"
  [[ ! -e $E2E_WORK/repo/README.md && $(line 1) == alpha && $(line 3) == Gamma-user &&
    $(line 5) == epsilon-user && $(line 9) == IOTA && -e $E2E_WORK/repo/notes.txt ]]
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== four turns"
  palette "workspace: new terminal" 3
  type_text "claude turns"
  press "" Return
  settle 3
  for _ in 1 2 3 4; do
    press "" Return
    settle 6
  done
  echo "== the user's own edits"
  printf 'my notes\n' >"$E2E_WORK/repo/notes.txt"
  sed -i 's/^epsilon$/epsilon-user/; s/^GAMMA$/Gamma-user/' "$E2E_WORK/repo/src/lib.txt"
  settle 2
  state | tee "$E2E_WORK/before.txt"
  click "$TURNS_X" "$TURNS_Y"
  settle 2
  take 714-00-turns || return 0

  echo "== Undo This Turn on Add a README"
  click "$ROW_X" "$README_Y" right
  settle 1
  # Off the row, so its tooltip does not cover the menu.
  pointer_to 300 410
  settle 1
  take 714-01-menu || return 0
  press "" Down
  press "" Down
  press "" Return
  settle 3
  take 714-02-confirm || return 0
  press "" Return
  settle 3
  take 714-03-undone || return 0
  state | tee "$E2E_WORK/after-undo.txt"
  expect "the turn came out and everything later stayed" undone_as_asked
  expect "HEAD, the index and the refs are as they were" same_kept before.txt

  echo "== a conflict"
  menu_pick "$GAMMA_Y" 2
  take 714-04-conflict || return 0
  press "" Return
  settle 2
  expect "a refused undo changes no file" same_files after-undo.txt

  echo "== Keep"
  menu_pick "$UTIL_Y" 1
  take 714-05-kept || return 0

  echo "== unsaved edits"
  palette "file finder: toggle" 1
  type_text "util.txt"
  settle 1
  press "" Return
  settle 3
  type_text "unsaved "
  settle 1
  menu_pick "$UTIL_Y" 1
  take 714-06-unsaved || return 0
  press "" Return
  settle 1
  press "CTRL" z
  settle 1
  expect "a refused undo changes no file" same_files after-undo.txt

  echo "== another agent at work"
  palette "workspace: new terminal" 3
  type_text "claude busy"
  press "" Return
  settle 3
  press "" Return
  settle 3
  menu_pick "$IOTA_Y" 2
  take 714-07-another-agent || return 0
  press "" Return
  settle 2
  expect "a refused undo changes no file" same_files after-undo.txt

  echo "== a file changed while the question is open"
  settle 25
  menu_pick "$IOTA_Y" 2
  sed -i 's/^IOTA$/iota-late/' "$E2E_WORK/repo/src/lib.txt"
  settle 1
  press "" Return
  settle 3
  take 714-08-changed-meanwhile || return 0
  echo "line 9: $(line 9)"
  expect "the write refused and line 9 stays as changed" test "$(line 9)" = iota-late
}
