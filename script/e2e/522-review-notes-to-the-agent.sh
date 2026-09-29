# shellcheck shell=bash
# #522's visual check, after the fact: review notes to a terminal agent. A scratch repository with
# a committed file and one line changed, not staged, and in its terminal a stand-in Claude Code
# (Marley's plugin's SessionStart makes it idle, #519) that echoes each line it reads. The project
# diff offers Add Review on the changed line (REQ-001); a note there, then Send Review to Agent,
# opens the picker with the agent ready (REQ-002); Enter pastes the note as `File:`, `Line:` and
# `User comment:` and presses Enter, and the stand-in echoes it (REQ-003); the note stays, marked
# Sent, and the button's count goes (REQ-004).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The changed line's gutter and the toolbar's Send Review button, from the first run's shots.
GUTTER_X=${GUTTER_X:-845}
GUTTER_Y=${GUTTER_Y:-220}
SEND_X=${SEND_X:-1260}
SEND_Y=${SEND_Y:-89}
# The diff's tab, to come back to it once the send showed the agent's terminal.
DIFF_TAB_X=${DIFF_TAB_X:-714}
DIFF_TAB_Y=${DIFF_TAB_Y:-51}

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
  git init -q -b main "$repo"
  printf 'one\ntwo\nthree\nfour\nfive\n' >"$repo/notes.txt"
  git -C "$repo" add -A
  git -C "$repo" -c core.hooksPath=/dev/null commit -q -m "Start"
  printf 'one\ntwo\nTHREE\nfour\nfive\n' >"$repo/notes.txt"
  write_fake_claude "$bin/claude"
  open_path "$repo"
}

# A fake `claude` that sends a SessionStart through Marley's plugin hook, so its seat is idle,
# and echoes each line it reads.
write_fake_claude() {
  sed -e "s|@HOOK@|$HOOK|" >"$1" <<'FAKE'
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
    print("got: " + line.rstrip("\n"), flush=True)
FAKE
  chmod +x "$1"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  type_text "claude"
  press "" Return
  settle 3
  shot 522-00-agent
  press "CTRL SHIFT" p
  settle 1
  type_text "git: diff"
  settle 1
  press "" Return
  settle 4
  shot 522-01-diff
  if [[ $GUTTER_X == 0 ]]; then
    echo "measure the gutter and the toolbar from 522-01-diff, then run again"
    return
  fi
  pointer_to "$GUTTER_X" "$GUTTER_Y"
  settle 1
  shot 522-02-add-review
  click "$GUTTER_X" "$GUTTER_Y"
  settle 1
  type_text "Say three in lower case"
  press "" Return
  settle 1
  shot 522-03-note
  if [[ $SEND_X == 0 ]]; then
    echo "measure the toolbar's Send Review from 522-03-note, then run again"
    return
  fi
  if [[ -n ${SEND_BY_PALETTE-} ]]; then
    press "CTRL SHIFT" p
    settle 1
    type_text "editor: send review to agent"
    settle 1
    press "" Return
  else
    # The pointer rests on the button first, as a hand's does: the toolbar redraws as the note's
    # editor loses the focus.
    press "" Escape
    settle 1
    # In the headless sway the first click on the diff's toolbar after typing in the note's editor
    # reaches no button (none of its listeners runs); a click on the toolbar's counts takes it.
    click "${COUNTS_X:-800}" "$SEND_Y"
    settle 1
    pointer_to "$SEND_X" "$SEND_Y"
    settle 1
    click "$SEND_X" "$SEND_Y"
  fi
  settle 1
  shot 522-04-picker
  press "" Return
  settle 2
  shot 522-05-sent
  mcp_agent terminal-read claude >"$E2E_WORK/agent.txt" || true
  cat "$E2E_WORK/agent.txt"
  expect "the agent got the file" holds "$E2E_WORK/agent.txt" "got: File: notes.txt"
  expect "and the line" holds "$E2E_WORK/agent.txt" "got: Line: 3"
  expect "and the note" holds "$E2E_WORK/agent.txt" 'got: User comment: "Say three in lower case"'
  click "$DIFF_TAB_X" "$DIFF_TAB_Y"
  settle 2
  shot 522-06-marked
  # The note again, under its line: marked Sent.
  pointer_to "$GUTTER_X" "$GUTTER_Y"
  settle 1
  click "$GUTTER_X" "$GUTTER_Y"
  settle 1
  shot 522-07-sent-mark
}
