# shellcheck shell=bash
# The visual check, after the fact, of #590, #591 and #589 on one repository (they shipped on
# 2026-09-29 before the visual check came back). #590: two worktree agents' terminals get PORT 3010
# and 3020 and MARLEY_PORT_OFFSET 10 and 20, and the main checkout's terminal none. #591 and #589:
# Remove on the first worktree's row asks, runs its `remove_worktree` task, removes the folder and
# its branch (merged: no commits of its own), and says so; a third worktree then takes the freed
# slot, PORT 3010 again.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The project's +, the steps to New Agent in Worktree and Claude Code in its submenu, as 510's.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-96}
WORKTREE_STEPS=${WORKTREE_STEPS:-7}
SUBMENU_CLAUDE_X=${SUBMENU_CLAUDE_X:-325}
SUBMENU_CLAUDE_Y=${SUBMENU_CLAUDE_Y:-326}
# The prompt's setup box and its editor, and the rows, from the first run's shots.
SETUP_BOX_X=${SETUP_BOX_X:-550}
SETUP_BOX_Y=${SETUP_BOX_Y:-205}
EDITOR_X=${EDITOR_X:-800}
EDITOR_Y=${EDITOR_Y:-143}
MAIN_ROW_X=${MAIN_ROW_X:-120}
MAIN_ROW_Y=${MAIN_ROW_Y:-136}
# The rail lists the worktrees by folder, each with its agent's row under it: the first agent's
# row, and the step to the next.
AGENT_ROW_Y=${AGENT_ROW_Y:-228}
WORKTREE_STEP=${WORKTREE_STEP:-92}

# The rail's worktree rows, each with its agent's row under it: the first worktree's row, the
# step to the next, and the menu's place at a row's right edge.
WORKTREE_ROW_Y=${WORKTREE_ROW_Y:-184}
MENU_X=${MENU_X:-248}

repo_git() {
  git -C "$E2E_WORK/repo" "$@"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config
  local repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$config/plugins" "$repo/.zed"
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
  cp "$E2E_WORK/gitconfig" "$home/.gitconfig"
  cat >"$config/plugins/installed_plugins.json" <<JSON
{"version": 2, "plugins": {"marley@marley": [{"scope": "user", "installPath": "$config/plugins/cache/marley/marley/$SHIPPED", "version": "$SHIPPED"}]}}
JSON
  export CLAUDE_CONFIG_DIR=$config
  git init -q -b main "$repo"
  printf 'the readme\n' >"$repo/README"
  # #591: a teardown task, committed so each worktree gets it.
  cat >"$repo/.zed/tasks.json" <<JSON
[
  {
    "label": "teardown",
    "command": "echo \\"torn down in \$ZED_WORKTREE_ROOT\\" >> $E2E_WORK/teardown.txt",
    "hooks": ["remove_worktree"]
  }
]
JSON
  repo_git add -A
  repo_git commit -q -m "Start"
  write_fake_claude "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude
  open_path "$repo"
}

# A fake `claude` that prints each argument it got in brackets, logs them with its folder, sends a
# SessionStart through Marley's plugin hook, and waits, as 510's does.
write_fake_claude() {
  sed -e "s|@HOOK@|$HOOK|" -e "s|@LOG@|$E2E_WORK/launches.log|" >"$1" <<'FAKE'
#!/usr/bin/env python3
import json
import os
import subprocess
import sys

if sys.argv[1:2] == ["plugin"]:
    sys.exit(0)
ARGS = sys.argv[1:]
with open("@LOG@", "a", encoding="utf-8") as log:
    log.write(json.dumps({"cwd": os.getcwd(), "args": ARGS,
                          "port": os.environ.get("PORT"),
                          "offset": os.environ.get("MARLEY_PORT_OFFSET")}) + "\n")
print("PORT=" + str(os.environ.get("PORT")) + " MARLEY_PORT_OFFSET="
      + str(os.environ.get("MARLEY_PORT_OFFSET")), flush=True)
print("started in " + os.path.basename(os.path.dirname(os.getcwd())) + ": "
      + " ".join("[" + argument + "]" for argument in ARGS), flush=True)
event = {"hook_event_name": "SessionStart", "source": "startup", "cwd": os.getcwd(),
         "session_id": "s-" + str(os.getpid()), "permission_mode": "default"}
answer = subprocess.run([sys.executable, "@HOOK@"], input=json.dumps(event),
                        capture_output=True, text=True, check=False).stdout
sequence = json.loads(answer or "{}").get("terminalSequence")
if sequence:
    sys.stdout.write(sequence)
    sys.stdout.flush()
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$1"
}

# The project's +, then New Agent in Worktree's submenu, then Claude Code in it: the prompt.
open_prompt() {
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  local step
  for ((step = 0; step < WORKTREE_STEPS; step++)); do
    press "" Down
  done
  press "" Return
  settle 1
  click "$SUBMENU_CLAUDE_X" "$SUBMENU_CLAUDE_Y"
  settle 3
}

# Types `$1` into the main checkout's terminal and runs it.
in_main() {
  click "$MAIN_ROW_X" "$MAIN_ROW_Y"
  settle 1
  type_text "$1"
  press "" Return
}

# The worktree the fake agent's `n`th launch ran in.
launch_folder() {
  python3 - "$E2E_WORK/launches.log" "$1" <<'PY'
import json, sys
launches = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8")]
print(launches[int(sys.argv[2]) - 1]["cwd"])
PY
}

# The worktree the fake agent's `n`th launch ran in, and the PORT it saw.
launch_field() {
  python3 - "$E2E_WORK/launches.log" "$1" "$2" <<'PY2'
import json, sys
launches = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8")]
print(launches[int(sys.argv[2]) - 1][sys.argv[3]])
PY2
}

new_agent() {
  open_prompt
  type_text "$1"
  press "" Return
  settle 10
}

steps() {
  local first second third first_branch
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== #590: two worktree agents, each with a port of its own"
  new_agent "first"
  first=$(launch_field 1 cwd)
  shot 589-01-first-agent
  new_agent "second"
  second=$(launch_field 2 cwd)
  shot 589-02-second-agent
  expect "the first agent saw PORT 3010" test "$(launch_field 1 port)" = 3010
  expect "and MARLEY_PORT_OFFSET 10" test "$(launch_field 1 offset)" = 10
  expect "the second saw PORT 3020" test "$(launch_field 2 port)" = 3020
  in_main "echo main-port=[\$PORT]"
  settle 2
  shot 589-03-main-no-port
  mcp_agent terminal-read "main-port" | tee "$E2E_WORK/main-port.txt"
  expect "the main checkout's terminal has no PORT" holds "$E2E_WORK/main-port.txt" "main-port=[]"

  echo "== #589 and #591: Remove the first worktree"
  first_branch=$(git -C "$first" rev-parse --abbrev-ref HEAD)
  local row=$WORKTREE_ROW_Y
  if [[ $(printf '%s\n%s\n' "$first" "$second" | LC_ALL=C sort | sed -n 1p) != "$first" ]]; then
    row=$((WORKTREE_ROW_Y + WORKTREE_STEP))
  fi
  click "$MENU_X" "$row" right
  settle 1
  shot 589-04-menu
  press "" End
  press "" Return
  settle 2
  shot 589-05-prompt
  press "" Return
  settle 8
  shot 589-06-removed
  cat "$E2E_WORK/teardown.txt" 2>/dev/null || echo "no teardown ran"
  expect "the teardown task ran in the worktree" holds "$E2E_WORK/teardown.txt" "torn down in"
  expect "the worktree's folder is gone" test ! -e "$first"
  expect "its merged branch is gone" test -z "$(repo_git branch --list "$first_branch")"
  expect "the other worktree stays" test -d "$second"

  echo "== #590: a third worktree takes the freed slot"
  new_agent "third"
  third=$(launch_field 3 cwd)
  echo "third worktree: $third"
  shot 589-07-third-agent
  expect "the third agent saw PORT 3010 again" test "$(launch_field 3 port)" = 3010
}
