# shellcheck shell=bash
# #924's visual check: plan review and the diff review as the Review and DiffReview boards draw
# them, built from #893's. A scratch repository with `notes.txt` (twenty lines), `other.txt` and
# `brief.md` committed, then lines 10 and 12 of `notes.txt` and `other.txt`'s second line changed.
# In its terminal the stand-in Claude Code sends Marley's plugin's SessionStart (so it is idle),
# at "turn" runs a turn that writes `todo.txt`, and at "plan" asks for two plans, the second a
# revision with a question card.
# - `924-03-diff`: a finding kept as a note on notes.txt: each file header's +/− beside Viewed and
#   the noted line's gutter mark (REQ-006, REQ-008).
# - `924-04-guide`: the guide's Send to: the stand-in "ready" in green on a filled row, then Copy
#   notes · any agent (REQ-007).
# - `924-01-plan`: plan review, the question answered and a note taken: the plan's own heading,
#   Send Answers and Send Feedback (1) at the foot with the hint, YOUR NOTES, the mono quote,
#   Remove, the turn card with its pill and todo.txt's counts (REQ-001 to REQ-004).
# - `924-02-file`: `brief.md` under review with a note: Send Review to Agent (1) at the column's
#   foot (REQ-005).
# - `924-05-day`, `924-06-day-diff`: plan review, and the diff, in Marley Day.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh
HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# From the probe run's shots; `STOP_AT=<shot>` ends the run after that shot.
ALLOW_X=${ALLOW_X:-1460}
ALLOW_Y=${ALLOW_Y:-929}
KEEP_X=${KEEP_X:-1525}
KEEP_Y=${KEEP_Y:-384}
TERMINAL_X=${TERMINAL_X:-480}
TERMINAL_Y=${TERMINAL_Y:-50}
DIFF_X=${DIFF_X:-842}
DIFF_Y=${DIFF_Y:-50}
ACCEPT_X=${ACCEPT_X:-362}
ACCEPT_Y=${ACCEPT_Y:-440}
WORD_X=${WORD_X:-420}
WORD_Y=${WORD_Y:-160}
FILE_WORD_X=${FILE_WORD_X:-420}
FILE_WORD_Y=${FILE_WORD_Y:-160}
AWAY_X=${AWAY_X:-700}
AWAY_Y=${AWAY_Y:-900}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$config/plugins" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/$SHIPPED", "version": "$SHIPPED"}]}}
JSON
  export CLAUDE_CONFIG_DIR=$config MARLEY_CLAUDE=$bin/claude
  cat >"$E2E_WORK/gitconfig" <<'CONFIG'
[user]
	name = Scenario
	email = scenario@example.invalid
CONFIG
  export GIT_CONFIG_GLOBAL=$E2E_WORK/gitconfig GIT_CONFIG_NOSYSTEM=1
  git init -q -b main "$repo"
  seq 1 20 | sed 's/^/line /' >"$repo/notes.txt"
  printf 'other one\nother two\nother three\n' >"$repo/other.txt"
  printf '# The brief\n\nKeep the notes short and the lines in lower case.\n' >"$repo/brief.md"
  git -C "$repo" add -A
  git -C "$repo" -c core.hooksPath=/dev/null commit -q -m "Start"
  sed -i -e 's/^line 10$/LINE TEN/' -e 's/^line 12$/LINE TWELVE/' "$repo/notes.txt"
  sed -i 's/^other two$/OTHER TWO/' "$repo/other.txt"
  profile_setting marley.plan_review true
  profile_setting marley.tmux_hosts '[]'
  write_fake_claude "$bin/claude"
  open_path "$repo"
}

write_fake_claude() {
  sed -e "s|@HOOK@|$HOOK|" >"$1" <<'FAKE'
#!/usr/bin/env python3
import json
import os
import subprocess
import sys
import time

if sys.argv[1:2] == ["plugin"]:
    sys.exit(0)
BIN = os.environ.get("MARLEY_BIN", "")
COMMON = {"transcript_path": "/tmp/e2e-transcript.jsonl", "cwd": os.getcwd(),
          "permission_mode": "default", "session_id": "s-" + str(os.getpid())}
FIRST = ("# Tidy the notes\n\n"
         "1. Read `notes.txt`.\n"
         "2. Put lines 10 and 12 back in lower case.\n")
REVISED = ("# Tidy the notes\n\n"
           "1. Read `notes.txt` from the top.\n"
           "2. Put lines 10 and 12 back in lower case, and line 2 of `other.txt`.\n\n"
           ":::question\n"
           "Should line 12 stay in capitals?\n"
           "- [ ] Yes, it is a heading\n"
           "- [ ] No, lower it too\n"
           "Recommended: No, lower it too\n"
           ":::\n\n"
           "3. Say what changed.\n")


def hook(event):
    answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps({**COMMON, **event}),
                            capture_output=True, text=True, check=False).stdout
    sequence = json.loads(answer or "{}").get("terminalSequence")
    if sequence:
        sys.stdout.write(sequence)
        sys.stdout.flush()


def ask(call, plan):
    return subprocess.Popen(
        [BIN, "approve", "--source", "mod:claude-code", "--call", call, "--tool", "ExitPlanMode"],
        stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, text=True)


print("stand-in Claude Code: type turn, then plan", flush=True)
hook({"hook_event_name": "SessionStart", "source": "startup"})
for line in sys.stdin:
    word = line.strip()
    if word == "turn":
        hook({"hook_event_name": "UserPromptSubmit", "prompt": "Write the to-do list",
              "prompt_id": "p1"})
        time.sleep(2)
        with open("todo.txt", "w", encoding="utf-8") as out:
            out.write("- tidy the notes\n")
        time.sleep(1)
        hook({"hook_event_name": "Stop", "last_assistant_message": "Wrote it."})
        print("turn done", flush=True)
    elif word == "plan":
        for call, plan in (("toolu_1", FIRST), ("toolu_2", REVISED)):
            asking = ask(call, plan)
            asking.stdin.write(json.dumps({"plan": plan}))
            asking.stdin.close()
            time.sleep(2)
        print("plans asked", flush=True)
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

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

# A note on the word at `$1`,`$2`: a double-click selects it, c opens the note, `$3` is its text.
note_at() {
  click "$1" "$2"
  click "$1" "$2"
  settle 1
  press "" c
  settle 1
  type_text "$3"
  press "" Return
  settle 1
}

findings() {
  local file=$E2E_WORK/repo/notes.txt
  printf '{"source": "scenario-lint", "notes": [%s]}' \
    "{\"path\": \"$file\", \"first_line\": 12, \"comment\": \"Twelve may stay as it is\"}"
}

guide() {
  local repo=$E2E_WORK/repo
  cat <<JSON
{"title": "Shout two lines", "summary": "Two files changed.",
 "sections": [
  {"title": "Notes", "summary": "Lines 10 and 12 in capitals.",
   "files": [{"path": "$repo/notes.txt", "first_line": 10}]},
  {"title": "Other", "summary": "Its second line in capitals.",
   "files": [{"path": "$repo/other.txt", "first_line": 2}]}
 ]}
JSON
}

steps() {
  local asking
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  type_text "claude"
  press "" Return
  settle 3
  echo "== a turn"
  type_text "turn"
  press "" Return
  settle 6

  echo "== findings, allowed, kept as a note"
  mcp_agent tool review_notes_add "$(findings)" >"$E2E_WORK/answer.txt" 2>&1 &
  asking=$!
  settle 4
  click "$ALLOW_X" "$ALLOW_Y"
  wait "$asking"
  cat "$E2E_WORK/answer.txt"
  settle 3
  click "$KEEP_X" "$KEEP_Y"
  settle 2
  away
  take 924-03-diff || return 0

  echo "== the guide"
  mcp_agent tool review_guide_open "$(guide)" >"$E2E_WORK/guide.txt" 2>&1
  cat "$E2E_WORK/guide.txt"
  settle 3
  away
  take 924-04-guide || return 0

  echo "== two plans, the second with a question"
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 2
  type_text "plan"
  press "" Return
  settle 8
  click "$ACCEPT_X" "$ACCEPT_Y"
  settle 1
  note_at "$WORD_X" "$WORD_Y" "Start at line 9"
  away
  take 924-01-plan || return 0

  echo "== Marley Day, plan review"
  profile_setting theme '"Marley Day"'
  settle 4
  take 924-05-day || return 0
  click "$DIFF_X" "$DIFF_Y"
  settle 3
  away
  take 924-06-day-diff || return 0
  profile_setting theme '"Marley Night"'
  settle 3

  echo "== brief.md under review"
  press CTRL p
  settle 2
  type_text "brief.md"
  settle 2
  press "" Return
  settle 2
  palette "marley: review markdown"
  settle 3
  note_at "$FILE_WORD_X" "$FILE_WORD_Y" "Say why"
  away
  take 924-02-file || return 0
}
