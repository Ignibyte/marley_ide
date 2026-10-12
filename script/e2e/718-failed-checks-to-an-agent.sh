# shellcheck shell=bash
# #718's visual check: a pull request's failed checks to an agent. #531's set-up: a scratch
# repository whose `origin` is a GitHub URL, on `feature`, and a stand-in `gh` named by `MARLEY_GH`
# that answers in `gh` 2.102.0's shapes:
# - `pr list` for `feature`: #42 with a check run `test` (CI) failed, its link an Actions job (22),
#   a check run `lint` (CI) passed, and a commit status `ci/external` failed;
# - `pr list` for `green`: #43 with its one check passed;
# - `run view --job=22 --log-failed`: a fixture log that holds a GitHub-shaped token (built at run
#   time, so this file holds none) and ends in the failing line.
# In the repository's terminal, #522's stand-in Claude Code, idle through Marley's plugin, logs
# each line it reads. Shots: the chip with a red cross and its tooltip (`718-01-chip`, REQ-001,
# REQ-002); the row's menu with Send Failed Checks to Agent… (`718-02-menu`, REQ-003); the picker
# with the agent ready and Copy Prompt (`718-03-picker`, REQ-004); the prompt at the agent's
# prompt, unsent, the file written 0600 and redacted (`718-04-pasted`, REQ-005 to REQ-007); on
# `green`, #43 with a tick and no send item (`718-05-green`, `718-06-green-menu`, REQ-001, REQ-003).
compositor sway

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The project row's pull request chip, from the first run's shots; `STOP_AT=<shot>` ends the run
# after that shot.
CHIP_X=${CHIP_X:-195}
CHIP_Y=${CHIP_Y:-136}
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-500}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

repo_git() {
  git -C "$E2E_WORK/repo" "$@"
}

# A GitHub token of the kind the redactor knows, put together here so no file holds one.
fake_token() {
  printf 'gh%s_%s' p "$(printf 'x%.0s' {1..36})"
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
  export CLAUDE_CONFIG_DIR=$config
  cat >"$E2E_WORK/gitconfig" <<'CONFIG'
[user]
	name = Scenario
	email = scenario@example.invalid
CONFIG
  export GIT_CONFIG_GLOBAL=$E2E_WORK/gitconfig GIT_CONFIG_NOSYSTEM=1
  write_fixtures
  write_fake_gh "$bin/gh"
  # Named, not first on the PATH: Marley takes its PATH from the user's login shell.
  export MARLEY_GH=$bin/gh
  write_fake_claude "$bin/claude"
  : >"$E2E_WORK/claude.log"
  git init -q -b main "$repo"
  printf 'one\n' >"$repo/notes.txt"
  repo_git add -A
  repo_git commit -q -m "Start"
  repo_git remote add origin https://github.com/example/demo.git
  repo_git checkout -q -b feature
  open_path "$repo"
}

# The answers the stand-in gh prints, in gh 2.102.0's shapes.
write_fixtures() {
  local run=https://github.com/example/demo/actions/runs
  cat >"$E2E_WORK/feature.json" <<JSON
[{"number":42,"state":"OPEN","isDraft":false,"title":"Fix the login form","url":"https://github.com/example/demo/pull/42","baseRefName":"main","headRefOid":"1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b","statusCheckRollup":[
 {"__typename":"CheckRun","name":"test","workflowName":"CI","status":"COMPLETED","conclusion":"FAILURE","detailsUrl":"$run/11/job/22","startedAt":"2026-10-12T04:22:22Z","completedAt":"2026-10-12T04:22:29Z"},
 {"__typename":"CheckRun","name":"lint","workflowName":"CI","status":"COMPLETED","conclusion":"SUCCESS","detailsUrl":"$run/11/job/23","startedAt":"2026-10-12T04:22:22Z","completedAt":"2026-10-12T04:22:29Z"},
 {"__typename":"StatusContext","context":"ci/external","state":"FAILURE","targetUrl":"https://ci.example.invalid/build/7","startedAt":"2026-10-12T04:22:12Z"}]}]
JSON
  cat >"$E2E_WORK/green.json" <<JSON
[{"number":43,"state":"OPEN","isDraft":false,"title":"Tidy the notes","url":"https://github.com/example/demo/pull/43","baseRefName":"main","headRefOid":"0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b1a","statusCheckRollup":[
 {"__typename":"CheckRun","name":"test","workflowName":"CI","status":"COMPLETED","conclusion":"SUCCESS","detailsUrl":"$run/12/job/30","startedAt":"2026-10-12T04:22:22Z","completedAt":"2026-10-12T04:22:29Z"}]}]
JSON
  {
    printf 'test\tRun tests\t2026-10-12T04:22:25Z running 12 tests\n'
    printf 'test\tRun tests\t2026-10-12T04:22:26Z env: GITHUB_TOKEN=%s\n' "$(fake_token)"
    printf 'test\tRun tests\t2026-10-12T04:22:27Z test login::rejects_empty ... FAILED\n'
    printf 'test\tRun tests\t2026-10-12T04:22:28Z error: assertion failed: form.is_valid()\n'
  } >"$E2E_WORK/job-22.log"
}

write_fake_gh() {
  cat >"$1" <<SH
#!/bin/sh
# The stand-in gh: #42 with failed checks for feature, #43 all passed for green, job 22's log.
printf '%s\n' "\$*" >>"$E2E_WORK/gh.log"
case " \$* " in
  *" pr list "*" --head=feature "*) cat "$E2E_WORK/feature.json" ;;
  *" pr list "*" --head=green "*) cat "$E2E_WORK/green.json" ;;
  *" pr list "*) printf '[]\n' ;;
  *" run view "*"--job=22 "*"--log-failed "*) cat "$E2E_WORK/job-22.log" ;;
  *) echo "unknown: \$*" >&2; exit 1 ;;
esac
SH
  chmod +x "$1"
}

# #522's stand-in Claude Code: idle through Marley's plugin hook, and each line it reads logged.
write_fake_claude() {
  sed -e "s|@HOOK@|$HOOK|" -e "s|@LOG@|$E2E_WORK/claude.log|" >"$1" <<'FAKE'
#!/usr/bin/env python3
import json
import os
import subprocess
import sys

if sys.argv[1:2] == ["plugin"]:
    sys.exit(0)
print("stand-in Claude Code, idle", flush=True)
event = {"hook_event_name": "SessionStart", "source": "startup", "cwd": os.getcwd(),
         "session_id": "s-" + str(os.getpid()), "permission_mode": "default"}
answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps(event),
                        capture_output=True, text=True, check=False).stdout
sequence = json.loads(answer or "{}").get("terminalSequence")
if sequence:
    sys.stdout.write(sequence)
    sys.stdout.flush()
for line in sys.stdin:
    with open("@LOG@", "a", encoding="utf-8") as log:
        log.write("got: " + line)
    print("got: " + line.rstrip("\n"), flush=True)
FAKE
  chmod +x "$1"
}

# The project row's menu, from a right-click on the chip.
chip_menu() {
  click "$CHIP_X" "$CHIP_Y" right
  settle 1
}

checks_file() {
  find "$E2E_PROFILE/checks" -name '*.log' -print -quit 2>/dev/null || true
}

steps() {
  local file
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 6
  type_text "claude"
  press "" Return
  settle 4
  take 718-00-row || return 0
  expect "gh was asked for the checks and the head" \
    holds "$E2E_WORK/gh.log" "--json=number,state,isDraft,title,url,baseRefName,headRefOid,statusCheckRollup"
  if ((CHIP_X == 0)); then
    echo "the chip is not placed yet: 718-00 is the last shot"
    return 0
  fi

  echo "== the chip marks the failed checks"
  pointer_to "$CHIP_X" "$CHIP_Y"
  settle 2
  take 718-01-chip || return 0

  echo "== the row's menu offers the send"
  chip_menu
  take 718-02-menu || return 0
  # Send Failed Checks to Agent… sits two above the menu's last item, Remove Project.
  press "" End
  press "" Up
  press "" Up
  press "" Return
  settle 3
  take 718-03-picker || return 0

  echo "== the agent gets the prompt, unsent"
  press "" Return
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 718-04-pasted || return 0
  cat "$E2E_WORK/gh.log"
  file=$(checks_file)
  echo "the file: $file"
  expect "the job's failed log was read, the job one argument" \
    holds "$E2E_WORK/gh.log" "run view --repo=example/demo --job=22 --log-failed"
  expect "the file is named for the pull request and its head" \
    test "$(basename "$file")" = "example-demo-pr42-1a2b3c4d.log"
  expect "the file is the user's alone" test "$(stat -c %a "$file")" = 600
  expect "it holds the failing line" grep -q "assertion failed: form.is_valid()" "$file"
  expect "the token is redacted" grep -q "GITHUB_TOKEN=\[redacted" "$file"
  expect "and never in the file" test "$(grep -c "$(fake_token)" "$file")" = 0
  expect "nothing reached the agent as a line" test ! -s "$E2E_WORK/claude.log"

  echo "== green: a tick, and no send"
  repo_git checkout -q -b green
  settle 10
  pointer_to "$CHIP_X" "$CHIP_Y"
  settle 2
  take 718-05-green || return 0
  chip_menu
  take 718-06-green-menu || return 0
  press "" Escape
}
