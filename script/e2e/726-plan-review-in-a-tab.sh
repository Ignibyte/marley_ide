# shellcheck shell=bash
# #726's visual check: Claude Code's plan in a review tab, with the plugin's part played by a
# stand-in. `marley.plan_review` is on in the run's settings. In the project's terminal a stand-in
# `claude` prints `MARLEY_APPROVE_TOOLS`, reports idle through `"$MARLEY_BIN" report` as the plugin
# does, then runs `"$MARLEY_BIN" approve --tool ExitPlanMode` with a three-step plan on stdin and
# prints the answer and the exit code; after a deny it sends a revised plan. Shots: the variable
# (`726-01-env`, REQ-001); the Plan tab beside the terminal and Needs you's entry (`726-02-tab`,
# REQ-003, REQ-004); a passage dragged over, `c`, a comment, an overall note (`726-03-note`,
# REQ-005); Send Feedback, the deny and its reason in the terminal (`726-04-feedback`, REQ-006); the
# revised plan in the same tab, no notes (`726-05-revised`, REQ-007); Approve (`726-06-approved`,
# REQ-008); the switch off and a new terminal without the variable (`726-07-off`, REQ-002).
compositor sway

# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
DRAG_FROM_X=${DRAG_FROM_X:-304}
DRAG_FROM_Y=${DRAG_FROM_Y:-207}
DRAG_TO_X=${DRAG_TO_X:-566}
DRAG_TO_Y=${DRAG_TO_Y:-207}
OVERALL_X=${OVERALL_X:-1408}
OVERALL_Y=${OVERALL_Y:-292}
FEEDBACK_X=${FEEDBACK_X:-1350}
FEEDBACK_Y=${FEEDBACK_Y:-331}
APPROVE_X=${APPROVE_X:-1259}
APPROVE_Y=${APPROVE_Y:-270}
TERMINAL_TAB_X=${TERMINAL_TAB_X:-470}
TERMINAL_TAB_Y=${TERMINAL_TAB_Y:-51}
PLAN_TAB_X=${PLAN_TAB_X:-676}
PLAN_TAB_Y=${PLAN_TAB_Y:-51}
AWAY_X=${AWAY_X:-700}
AWAY_Y=${AWAY_Y:-900}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  profile_setting marley.plan_review true
  write_fake_claude "$bin/claude"
  : >"$E2E_WORK/answers.log"
  open_path "$repo"
}

# The plugin's part: a report, then an approve with the plan; a revised plan after a deny.
write_fake_claude() {
  sed -e "s|@LOG@|$E2E_WORK/answers.log|" >"$1" <<'FAKE'
#!/usr/bin/env python3
import json
import os
import subprocess
import sys

BIN = os.environ.get("MARLEY_BIN", "")
LOG = "@LOG@"
FIRST = ("# Fix the login form\n\n"
         "1. Read `login.rs` and find where the label is drawn.\n"
         "2. Change the label from \"Usrname\" to \"Username\".\n"
         "3. Run the tests and report what failed.\n")
REVISED = ("# Fix the login form, revised\n\n"
           "1. Read `login.rs`.\n"
           "2. Change the label, and its twin in `signup.rs`.\n"
           "3. Run only the form's tests.\n")

print("MARLEY_APPROVE_TOOLS=[" + os.environ.get("MARLEY_APPROVE_TOOLS", "") + "]", flush=True)
subprocess.run([BIN, "report", "--source", "mod:claude-code", "--seq", "1", "idle"], check=False)


def ask(call, plan):
    done = subprocess.run(
        [BIN, "approve", "--source", "mod:claude-code", "--call", call, "--tool", "ExitPlanMode"],
        input=json.dumps({"plan": plan}), capture_output=True, text=True, check=False)
    line = f"{call}: exit {done.returncode} {done.stdout.strip()} {done.stderr.strip()}"
    print(line, flush=True)
    with open(LOG, "a", encoding="utf-8") as log:
        log.write(line + "\n")
    return done


if '"deny"' in ask("toolu_1", FIRST).stdout:
    ask("toolu_2", REVISED)
for _ in sys.stdin:
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

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the stand-in Claude Code asks with its plan"
  type_text "claude"
  press "" Return
  settle 4
  take 726-02-tab || return 0
  if ((DRAG_FROM_X == 0)); then
    echo "the plan's points are not placed yet: 726-02 is the last shot"
    return 0
  fi

  echo "== the variable in the terminal"
  click "$TERMINAL_TAB_X" "$TERMINAL_TAB_Y"
  settle 2
  take 726-01-env || return 0
  click "$PLAN_TAB_X" "$PLAN_TAB_Y"
  settle 2

  echo "== a note on a passage, and an overall note"
  pointer_to "$DRAG_FROM_X" "$DRAG_FROM_Y"
  pointer_down
  pointer_to "$DRAG_TO_X" "$DRAG_TO_Y"
  pointer_up
  settle 1
  take 726-03a-dragged || return 0
  press "" c
  settle 1
  type_text "Say which tests"
  press "" Return
  settle 1
  click "$OVERALL_X" "$OVERALL_Y"
  type_text "Keep the change small."
  settle 1
  pointer_to "$AWAY_X" "$AWAY_Y"
  take 726-03-note || return 0

  echo "== Send Feedback: a deny with the notes"
  click "$FEEDBACK_X" "$FEEDBACK_Y"
  settle 4
  take 726-04-feedback || return 0
  cat "$E2E_WORK/answers.log"
  expect "the first plan was denied with the note and the overall note" \
    holds "$E2E_WORK/answers.log" 'toolu_1: exit 0 {"decision": "deny"' 'Comment: \"Say which tests\"' 'Overall: \"Keep the change small.\"'

  echo "== the revised plan in the same tab"
  take 726-05-revised || return 0

  echo "== Approve"
  click "$APPROVE_X" "$APPROVE_Y"
  settle 3
  take 726-06-approved || return 0
  cat "$E2E_WORK/answers.log"
  expect "the revised plan was approved" holds "$E2E_WORK/answers.log" 'toolu_2: exit 0 {"decision": "approve"}'

  echo "== the switch off: a new terminal has no tools named"
  profile_setting marley.plan_review false
  settle 3
  palette "workspace: new terminal"
  settle 3
  type_text "echo \"tools=[\$MARLEY_APPROVE_TOOLS]\""
  press "" Return
  settle 2
  take 726-07-off || return 0
}
