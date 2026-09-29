# shellcheck shell=bash
# #585's e2e test: a new worktree agent's worktree gets its environment. A scratch repository on
# `main` whose `.gitignore` ignores `.env`, `secrets/`, `big/`, `node_modules/` and `vendor/`, and
# whose `.worktreeinclude` names `.env`, `secrets/`, `big/`, `README` (tracked) and
# `vendor/**/local.json`; `big/` holds a 150 MB sparse file; a `package.json` and a
# `pnpm-lock.yaml`. The worktree prompt offers `pnpm install` with its box clear (REQ-004). The
# first worktree gets `.env`, `secrets/key.txt` and `vendor/a/local.json`, and not
# `vendor/a/other.json`, `node_modules/` or README's copy (REQ-001, REQ-002), and a toast names
# `big/` as past the budget (REQ-003). With the box checked, the second runs a fake `pnpm install`
# before the fake agent (REQ-005), the choice is kept in `marley.worktreeSetup`, and the third
# prompt starts checked (REQ-006). A committed `create_worktree` task takes the offer away
# (REQ-007), and its run in the fourth worktree sees `MARLEY_ROOT_PATH` and `MARLEY_WORKTREE_PATH`
# (REQ-008).
compositor sway

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

repo_git() {
  git -C "$E2E_WORK/repo" "$@"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin config=$E2E_WORK/claude-config
  local repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$config/plugins" "$repo"
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
  printf '.env\nsecrets/\nbig/\nnode_modules/\nvendor/\n' >"$repo/.gitignore"
  printf '.env\nsecrets/\nbig/\nREADME\nvendor/**/local.json\n' >"$repo/.worktreeinclude"
  printf '{ "name": "scenario" }\n' >"$repo/package.json"
  printf "lockfileVersion: '9.0'\n" >"$repo/pnpm-lock.yaml"
  repo_git add -A
  repo_git commit -q -m "Start"
  printf 'TOKEN=scenario\n' >"$repo/.env"
  mkdir -p "$repo/secrets" "$repo/big" "$repo/node_modules/x" "$repo/vendor/a"
  printf 'a key\n' >"$repo/secrets/key.txt"
  truncate -s 150M "$repo/big/blob"
  printf '{}\n' >"$repo/vendor/a/local.json"
  printf '{}\n' >"$repo/vendor/a/other.json"
  printf 'module.exports = 1;\n' >"$repo/node_modules/x/index.js"
  repo_git status --short --ignored

  write_fake_claude "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude
  cat >"$bin/pnpm" <<SH
#!/bin/sh
printf '%s %s\n' "\$PWD" "\$*" >>"$E2E_WORK/pnpm.log"
echo "Done in 0.1s"
SH
  chmod +x "$bin/pnpm"
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
    log.write(json.dumps({"cwd": os.getcwd(), "args": ARGS}) + "\n")
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

steps() {
  local first second fourth
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the prompt offers pnpm install, its box clear"
  open_prompt
  shot 585-01-prompt

  echo "== the first worktree: the included files copied, big/ past the budget"
  type_text "first"
  press "" Return
  settle 10
  first=$(launch_folder 1)
  echo "first worktree: $first"
  in_main "cd $first && ls -a && find secrets vendor -type f; cd - >/dev/null"
  settle 3
  shot 585-02-copied
  (cd "$first" && find . -path ./.git -prune -o -type f -print | sort) | tee "$E2E_WORK/first.txt"
  expect "the first worktree has .env" holds "$E2E_WORK/first.txt" "./.env"
  expect "and secrets/key.txt" holds "$E2E_WORK/first.txt" "./secrets/key.txt"
  expect "and vendor/a/local.json" holds "$E2E_WORK/first.txt" "./vendor/a/local.json"
  expect "but not vendor/a/other.json" bash -c "! grep -qx './vendor/a/other.json' '$E2E_WORK/first.txt'"
  expect "and no node_modules" test ! -e "$first/node_modules"
  expect "and no big/" test ! -e "$first/big"
  expect "README is as committed" test "$(git -C "$first" status --porcelain)" = ""
  expect "no install ran" test ! -s "$E2E_WORK/pnpm.log"

  echo "== the second worktree, with the box checked: the install first, then the agent"
  open_prompt
  click "$SETUP_BOX_X" "$SETUP_BOX_Y"
  settle 1
  shot 585-03-checked
  # The box took the focus; the prompt's text goes to its editor.
  click "$EDITOR_X" "$EDITOR_Y"
  settle 1
  type_text "second"
  press "" Return
  settle 12
  second=$(launch_folder 2)
  echo "second worktree: $second"
  if [[ $(printf '%s\n%s\n' "$first" "$second" | LC_ALL=C sort | sed -n 1p) == "$second" ]]; then
    click "$MAIN_ROW_X" "$AGENT_ROW_Y"
  else
    click "$MAIN_ROW_X" $((AGENT_ROW_Y + WORKTREE_STEP))
  fi
  settle 2
  shot 585-03-setup
  cat "$E2E_WORK/pnpm.log"
  expect "pnpm install ran in the second worktree" holds "$E2E_WORK/pnpm.log" "$second install"
  expect "the choice is kept" test "$(repo_git config --get marley.worktreeSetup)" = "pnpm install"

  echo "== the third prompt starts checked"
  open_prompt
  shot 585-04-remembered
  press "" Escape
  settle 1

  echo "== a create_worktree task: no offer, and the hook sees Marley's paths"
  mkdir -p "$E2E_WORK/repo/.zed"
  cat >"$E2E_WORK/repo/.zed/tasks.json" <<JSON
[
  {
    "label": "worktree paths",
    "command": "printf '%s\\\\n%s\\\\n' \"\$MARLEY_ROOT_PATH\" \"\$MARLEY_WORKTREE_PATH\" > $E2E_WORK/hook.txt",
    "hooks": ["create_worktree"]
  }
]
JSON
  repo_git add .zed/tasks.json
  repo_git commit -q -m "A create_worktree task"
  settle 4
  open_prompt
  shot 585-05-hook
  type_text "fourth"
  press "" Return
  settle 12
  fourth=$(launch_folder 3)
  echo "fourth worktree: $fourth"
  cat "$E2E_WORK/hook.txt" 2>/dev/null || echo "the hook wrote nothing"
  expect "the hook saw the main checkout" \
    test "$(realpath -m "$(sed -n 1p "$E2E_WORK/hook.txt" 2>/dev/null)")" = "$(realpath "$E2E_WORK/repo")"
  expect "and its worktree" \
    test "$(realpath -m "$(sed -n 2p "$E2E_WORK/hook.txt" 2>/dev/null)")" = "$(realpath "$fourth")"
}
