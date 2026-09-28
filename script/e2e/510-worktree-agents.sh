# shellcheck shell=bash
# #510's e2e test: worktree agents. A scratch repository on `main` with a commit, a worktree made
# by hand at `$E2E_WORK/manual` on `manual`, and a Claude Code-style one at
# `.claude/worktrees/scratch`. The project's `+` lists New Agent in Worktree with the installed
# CLIs (REQ-001); the rail shows `manual` and not `scratch` (REQ-006, REQ-008). Claude Code under
# it opens the prompt naming `agent/<name> from main` (REQ-002); a prompt with quotes and an
# apostrophe makes the worktree on `agent/<name>` from `main` with no upstream and the base in the
# config (REQ-003, REQ-004), and the fake `claude` in the worktree's terminal gets the prompt as
# one argument, after the project's bypass flag (REQ-005), its row under the worktree's
# (REQ-006, REQ-007). A click on `manual` opens it (REQ-009); Left from the agent's row selects its
# worktree's (REQ-010); a worktrees folder made read-only refuses a second, saying why once, with
# no new row (REQ-011).
compositor sway

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The project's +, the rows and the fields, from the first run's shots.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-96}
# Down steps from New Terminal in the + menu: New Browser Tab, New Agent Thread, the CLIs (four on
# this box), then New Agent in Worktree.
WORKTREE_STEPS=${WORKTREE_STEPS:-7}
# Claude Code, first in New Agent in Worktree's submenu: Enter opens the submenu, and the keys stay
# with the menu above it, so the entry is clicked.
SUBMENU_CLAUDE_X=${SUBMENU_CLAUDE_X:-325}
SUBMENU_CLAUDE_Y=${SUBMENU_CLAUDE_Y:-326}
# New Agent in Worktree itself, where the pointer shows its submenu for the shot.
WORKTREE_ENTRY_X=${WORKTREE_ENTRY_X:-150}
WORKTREE_ENTRY_Y=${WORKTREE_ENTRY_Y:-322}
AGENT_ROW_X=${AGENT_ROW_X:-120}
AGENT_ROW_Y=${AGENT_ROW_Y:-230}
# The agent's row once `manual`'s workspace is open, with its terminal listed above.
AGENT_ROW_LATER_Y=${AGENT_ROW_LATER_Y:-320}
MAIN_ROW_X=${MAIN_ROW_X:-120}
MAIN_ROW_Y=${MAIN_ROW_Y:-136}
MANUAL_ROW_X=${MANUAL_ROW_X:-120}
MANUAL_ROW_Y=${MANUAL_ROW_Y:-180}

repo_git() {
  git -C "$E2E_WORK/repo" "$@"
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

  git init -q -b main "$repo"
  printf 'hello\n' >"$repo/src/lib.txt"
  printf '.claude/\n' >"$repo/.gitignore"
  repo_git add -A
  repo_git commit -q -m "Start"
  repo_git worktree add -q "$E2E_WORK/manual" -b manual
  repo_git worktree add -q "$repo/.claude/worktrees/scratch" -b worktree-scratch
  repo_git worktree list

  write_fake_claude "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude
  open_path "$repo"
}

teardown() {
  chmod -R u+w "$E2E_WORK/worktrees" 2>/dev/null || true
}

# A fake `claude` that prints each argument it got in brackets, writes them to a log, sends a
# SessionStart through Marley's plugin hook, and waits.
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
MODE = "bypassPermissions" if "--dangerously-skip-permissions" in ARGS else "default"
event = {"hook_event_name": "SessionStart", "source": "startup", "cwd": os.getcwd(),
         "session_id": "s-" + str(os.getpid()), "permission_mode": MODE}
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

# Merges the JSON object `$1` into `marley` in the profile copy's settings.
marley_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'PY'
import json, pathlib, re, sys

path, changes = pathlib.Path(sys.argv[1]), json.loads(sys.argv[2])
text = path.read_text() if path.exists() else "{}"
text = re.sub(r"^\s*//.*$", "", text, flags=re.MULTILINE)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
settings.setdefault("marley", {}).update(changes)
path.write_text(json.dumps(settings, indent=2) + "\n")
PY
  settle 3
}

# The project's +, then New Agent in Worktree's submenu, which Enter opens.
worktree_menu() {
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  local step
  for ((step = 0; step < WORKTREE_STEPS; step++)); do
    press "" Down
  done
  press "" Return
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the terminal's claude is the fake"
  type_text "command -v claude > $E2E_WORK/which.txt"
  press "" Return
  settle 2
  if ! grep -qx "$E2E_WORK/bin/claude" "$E2E_WORK/which.txt"; then
    echo "check the fake comes first on the terminal's PATH: FAIL (stopping before any launch)"
    return 1
  fi

  echo "== bypass for the project, so the worktree agent's mode shows"
  marley_setting "{\"agent_permissions_by_project\": {\"$E2E_WORK/repo\": {\"claude_code\": \"bypass\"}}}"

  echo "== the + lists New Agent in Worktree; the rail lists manual and not scratch"
  worktree_menu
  pointer_to "$WORKTREE_ENTRY_X" "$WORKTREE_ENTRY_Y"
  settle 2
  shot 510-01-menu

  echo "== Claude Code: the prompt names the branch and the base"
  click "$SUBMENU_CLAUDE_X" "$SUBMENU_CLAUDE_Y"
  settle 2
  shot 510-02-prompt
  type_text "fix the \"login\" form's label"
  press "" Return
  settle 10

  echo "== the agent in its worktree"
  click "$AGENT_ROW_X" "$AGENT_ROW_Y"
  settle 2
  shot 510-03-agent
  cat "$E2E_WORK/launches.log" 2>/dev/null || echo "no launch logged"
  expect "the agent got its bypass and the prompt as one argument" python3 - "$E2E_WORK/launches.log" <<'PY'
import json, sys
launches = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8")]
sys.exit(0 if len(launches) == 1
         and launches[0]["args"] == ["--dangerously-skip-permissions", "fix the \"login\" form's label"]
         and "/worktrees/repo/" in launches[0]["cwd"] else 1)
PY

  echo "== the branch, its base and no upstream"
  repo_git worktree list | tee "$E2E_WORK/worktrees.txt"
  repo_git branch -vv --list 'agent/*' | tee "$E2E_WORK/branches.txt"
  repo_git config --get-regexp 'branch\..*\.base' | tee "$E2E_WORK/bases.txt"
  expect "one agent/ branch, with no upstream" \
    bash -c "test \$(wc -l <'$E2E_WORK/branches.txt') -eq 1 && ! grep -q '\\[' '$E2E_WORK/branches.txt'"
  expect "its base is main" grep -qE '^branch\.agent/[a-z]+-[a-z]+\.base main$' "$E2E_WORK/bases.txt"
  expect "the worktree sits in Zed's layout" grep -qE "/worktrees/repo/[a-z]+-[a-z]+/repo " "$E2E_WORK/worktrees.txt"
  click "$MAIN_ROW_X" "$MAIN_ROW_Y"
  settle 2
  type_text "git branch -vv --list 'agent/*'; git config --get-regexp 'branch.*base'"
  press "" Return
  settle 2
  shot 510-04-branch

  echo "== a click on manual opens it"
  click "$MANUAL_ROW_X" "$MANUAL_ROW_Y"
  settle 5
  shot 510-05-opened

  echo "== Left from the agent's row selects its worktree's"
  click "$AGENT_ROW_X" "$AGENT_ROW_LATER_Y"
  settle 1
  press "CTRL ALT" semicolon
  settle 1
  press "" Left
  settle 1
  shot 510-06-keys

  echo "== a worktrees folder made read-only refuses a second, once"
  chmod a-w "$E2E_WORK/worktrees/repo"
  worktree_menu
  click "$SUBMENU_CLAUDE_X" "$SUBMENU_CLAUDE_Y"
  settle 2
  type_text "a second one"
  press "" Return
  settle 6
  shot 510-07-refused
  expect "no second agent/ branch" test "$(repo_git branch --list 'agent/*' | wc -l)" -eq 1
  expect "no second launch" test "$(wc -l <"$E2E_WORK/launches.log")" -eq 1
}
