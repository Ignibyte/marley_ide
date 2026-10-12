# shellcheck shell=bash
# #727's visual check: plan versions and their diff, on #726's set-up (a stand-in `claude` plays the
# plugin's part). Version 1 is three steps; an overall note and Send Feedback; version 2 rewords step
# 2 and adds step 4. Shots: the header with "Version 2" and +2 −1 (`727-01-version`, REQ-001);
# Changes: the diff of version 1 against version 2 (`727-02-changes`, REQ-002); Plan: version 2 as
# Markdown again, with Approve and Send Feedback (`727-03-plan`, REQ-003). Version 1's header shows
# neither (`727-00-first`, REQ-004).
compositor sway

# Points from the first run's shots; `STOP_AT=<shot>` ends the run after that shot.
OVERALL_X=${OVERALL_X:-1408}
OVERALL_Y=${OVERALL_Y:-231}
FEEDBACK_X=${FEEDBACK_X:-1350}
FEEDBACK_Y=${FEEDBACK_Y:-270}
CHANGES_X=${CHANGES_X:-1359}
CHANGES_Y=${CHANGES_Y:-128}
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
         "1. Read `login.rs`.\n"
         "2. Change the label.\n"
         "3. Run the tests.\n")
REVISED = ("# Fix the login form\n\n"
           "1. Read `login.rs`.\n"
           "2. Change the label, and its twin in `signup.rs`.\n"
           "3. Run the tests.\n"
           "4. Report what failed.\n")

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

  echo "== version 1: no count, no Changes"
  type_text "claude"
  press "" Return
  settle 4
  take 727-00-first || return 0

  echo "== feedback, then version 2"
  click "$OVERALL_X" "$OVERALL_Y"
  type_text "Mind the signup form too."
  settle 1
  click "$FEEDBACK_X" "$FEEDBACK_Y"
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 727-01-version || return 0
  cat "$E2E_WORK/answers.log"
  expect "version 1 was denied" holds "$E2E_WORK/answers.log" 'toolu_1: exit 0 {"decision": "deny"'
  if ((CHANGES_X == 0)); then
    echo "Changes is not placed yet: 727-01 is the last shot"
    return 0
  fi

  echo "== Changes: the diff"
  click "$CHANGES_X" "$CHANGES_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 727-02-changes || return 0

  echo "== Plan: version 2 again"
  click "$CHANGES_X" "$CHANGES_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 727-03-plan || return 0
}
