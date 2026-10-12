# shellcheck shell=bash
# #770's visual check: one New Agent in Worktree prompt starts several agents. A scratch repository
# on `main` with a commit, and fakes for `claude` and `codex` first on the terminal's PATH that log
# their arguments and folder (#510's fake, without the hook). Claude Code's prompt shows "Also
# start:" with a clear Codex box (`770-01-boxes`, REQ-001); ticked, the line names
# `agent/<name>-claude, agent/<name>-codex from main` (`770-02-ticked`, REQ-002); Enter makes both
# worktrees from main and starts each agent in its own with the same prompt, each a row of its own
# (`770-03-two-rows`, REQ-003, REQ-004); a later prompt with nothing ticked makes one `agent/<name>`
# (`770-04-one`, REQ-005).
compositor sway

# The project's +, New Agent in Worktree's submenu and the prompt's Codex box, from the first
# run's shots; `STOP_AT=<shot>` ends the run after that shot.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-136}
# Down steps from New Terminal: New Browser Tab, New Agent…, Reopen Thread…, New Agent in Worktree.
WORKTREE_STEPS=${WORKTREE_STEPS:-4}
SUBMENU_CLAUDE_X=${SUBMENU_CLAUDE_X:-325}
SUBMENU_CLAUDE_Y=${SUBMENU_CLAUDE_Y:-260}
CODEX_BOX_X=${CODEX_BOX_X:-618}
CODEX_BOX_Y=${CODEX_BOX_Y:-178}
STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

repo_git() {
  git -C "$E2E_WORK/repo" "$@"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo name
  mkdir -p "$home" "$bin" "$repo/src"
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
  git init -q -b main "$repo"
  printf 'hello\n' >"$repo/src/lib.txt"
  repo_git add -A
  repo_git commit -q -m "Start"
  for name in claude codex; do
    sed -e "s|@NAME@|$name|" -e "s|@LOG@|$E2E_WORK/launches.log|" >"$bin/$name" <<'FAKE'
#!/usr/bin/env python3
# A fake agent CLI: its arguments and folder, logged and printed; then it waits.
import json
import os
import sys

if sys.argv[1:2] == ["plugin"]:
    sys.exit(0)
with open("@LOG@", "a", encoding="utf-8") as log:
    log.write(json.dumps({"agent": "@NAME@", "cwd": os.getcwd(), "args": sys.argv[1:]}) + "\n")
print("@NAME@ started in " + os.path.basename(os.path.dirname(os.getcwd())) + ": "
      + " ".join("[" + argument + "]" for argument in sys.argv[1:]), flush=True)
for _ in sys.stdin:
    pass
FAKE
    chmod +x "$bin/$name"
  done
  export MARLEY_CLAUDE=$bin/claude
  open_path "$repo"
}

# The project's +, then New Agent in Worktree's submenu, which Enter opens.
worktree_menu() {
  local step
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  # A menu opens with nothing chosen (Zed #64365); Home chooses New Terminal.
  press "" Home
  for ((step = 0; step < WORKTREE_STEPS; step++)); do
    press "" Down
  done
  press "" Return
  settle 1
}

# The launches as `<agent> <folder's last two parts> <arguments>`, one a line.
launches() {
  python3 - "$E2E_WORK/launches.log" <<'PY'
import json, os, sys
for line in open(sys.argv[1], encoding="utf-8"):
    launch = json.loads(line)
    folder = os.path.join(*launch["cwd"].split(os.sep)[-2:])
    print(launch["agent"], folder, json.dumps(launch["args"]))
PY
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the terminal's claude and codex are the fakes"
  type_text "command -v claude codex > $E2E_WORK/which.txt"
  press "" Return
  settle 2
  if ! grep -qx "$E2E_WORK/bin/codex" "$E2E_WORK/which.txt"; then
    echo "check the fakes come first on the terminal's PATH: FAIL (stopping before any launch)"
    return 1
  fi
  : >"$E2E_WORK/launches.log"

  echo "== the prompt offers the other agents, clear"
  worktree_menu
  take 770-00-menu || return 0
  if ((SUBMENU_CLAUDE_X == 0)); then
    echo "the submenu is not placed yet: 770-00 is the last shot"
    return 0
  fi
  click "$SUBMENU_CLAUDE_X" "$SUBMENU_CLAUDE_Y"
  settle 2
  take 770-01-boxes || return 0
  if ((CODEX_BOX_X == 0)); then
    echo "the Codex box is not placed yet: 770-01 is the last shot"
    return 0
  fi

  echo "== Codex ticked: the line names both branches"
  click "$CODEX_BOX_X" "$CODEX_BOX_Y"
  settle 1
  take 770-02-ticked || return 0

  echo "== Enter starts both, each in its own worktree"
  type_text "compare the \"login\" fix"
  press "" Return
  settle 15
  take 770-03-two-rows || return 0
  launches | tee "$E2E_WORK/launched.txt"
  repo_git branch --list 'agent/*' | tee "$E2E_WORK/branches.txt"
  repo_git config --get-regexp 'branch\..*\.base' | tee "$E2E_WORK/bases.txt"
  expect "two launches, Claude Code first" \
    bash -c "test \$(wc -l <'$E2E_WORK/launched.txt') -eq 2 && head -n 1 '$E2E_WORK/launched.txt' | grep -q '^claude '"
  expect "each in its own worktree, named as a set" python3 - "$E2E_WORK/launched.txt" <<'PY'
import re, sys
lines = open(sys.argv[1], encoding="utf-8").read().splitlines()
folders = {line.split(" ")[0]: line.split(" ")[1] for line in lines}
claude = re.fullmatch(r"([a-z]+-[a-z]+)-claude/repo", folders.get("claude", ""))
codex = re.fullmatch(r"([a-z]+-[a-z]+)-codex/repo", folders.get("codex", ""))
sys.exit(0 if claude and codex and claude.group(1) == codex.group(1) else 1)
PY
  expect "both got the same prompt" python3 - "$E2E_WORK/launches.log" <<'PY'
import json, sys
launches = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8")]
sys.exit(0 if sum('compare the "login" fix' in launch["args"] for launch in launches) == 2 else 1)
PY
  expect "two branches, -claude and -codex" \
    bash -c "grep -qE 'agent/[a-z]+-[a-z]+-claude$' '$E2E_WORK/branches.txt' && grep -qE 'agent/[a-z]+-[a-z]+-codex$' '$E2E_WORK/branches.txt'"
  expect "both from main" test "$(grep -c '\.base main$' "$E2E_WORK/bases.txt")" -eq 2

  echo "== nothing ticked: one worktree, as before"
  worktree_menu
  click "$SUBMENU_CLAUDE_X" "$SUBMENU_CLAUDE_Y"
  settle 2
  type_text "on its own"
  press "" Return
  settle 10
  take 770-04-one || return 0
  launches | tee "$E2E_WORK/launched.txt"
  repo_git branch --list 'agent/*' | tee "$E2E_WORK/branches.txt"
  expect "a third launch, Claude Code alone" \
    bash -c "test \$(wc -l <'$E2E_WORK/launched.txt') -eq 3 && tail -n 1 '$E2E_WORK/launched.txt' | grep -qE '^claude [a-z]+-[a-z]+/repo '"
  expect "its branch has no agent's name" grep -qE '^[ +*] agent/[a-z]+-[a-z]+$' "$E2E_WORK/branches.txt"
}
