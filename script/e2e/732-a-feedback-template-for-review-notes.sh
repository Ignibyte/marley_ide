# shellcheck shell=bash
# #732's visual check: the message around review notes. #522's set-up: a scratch repository with a
# committed file and one line changed, not staged, and in its terminal a stand-in Claude Code
# (Marley's plugin's SessionStart makes it idle) that echoes and logs each line it reads. A note
# sent with no template arrives inside Marley's own message (`732-01-default`, REQ-001); with
# `marley.review_notes_template` set by the scenario, a second note arrives inside it and none of
# the default's lines come again (`732-02-custom`, REQ-002).
compositor sway

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The changed line's gutter in the diff, from the first run's shot; `STOP_AT=<shot>` ends the run
# after that shot.
GUTTER_X=${GUTTER_X:-960}
GUTTER_Y=${GUTTER_Y:-220}
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
  : >"$E2E_WORK/claude.log"
  open_path "$repo"
}

# #522's stand-in Claude Code: idle through Marley's plugin hook; each line it reads echoed and
# logged.
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

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Adds a note on the changed line, shot as `$2`, then sends the unsent notes to the ready agent.
note_and_send() {
  pointer_to "$GUTTER_X" "$GUTTER_Y"
  settle 1
  click "$GUTTER_X" "$GUTTER_Y"
  settle 1
  type_text "$1"
  press "" Return
  settle 1
  take "$2" || return 1
  press "" Escape
  settle 1
  palette "editor: send review to agent"
  settle 2
  press "" Return
  settle 3
}

# Sets `marley.review_notes_template` in the run's settings.
set_template() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'PY'
import json, pathlib, re, sys

path, template = pathlib.Path(sys.argv[1]), sys.argv[2]
text = path.read_text() if path.exists() else "{}"
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
settings.setdefault("marley", {})["review_notes_template"] = template
path.write_text(json.dumps(settings, indent=2) + "\n")
PY
  settle 3
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  type_text "claude"
  press "" Return
  settle 3
  palette "git: diff"
  settle 4
  take 732-00-diff || return 0
  if ((GUTTER_X == 0)); then
    echo "the gutter is not placed yet: 732-00 is the last shot"
    return 0
  fi

  echo "== no template: Marley's own message"
  note_and_send "Say three in lower case" 732-01a-note || return 0
  take 732-01-default || return 0
  cat "$E2E_WORK/claude.log"
  # The paste is bracketed, so the first line read starts with the paste's opening mark.
  expect "the default's first line came first" \
    bash -c "head -n 1 '$E2E_WORK/claude.log' | grep -q '^got: .*Review notes on your work follow. For each note, check it against the code'"
  expect "then the note" holds "$E2E_WORK/claude.log" 'got: User comment: "Say three in lower case"'

  echo "== a template of the user's"
  : >"$E2E_WORK/claude.log"
  set_template $'Scenario template.\n\n{notes}\n\nEnd of notes.'
  palette "git: diff"
  settle 3
  note_and_send "Keep five as it is" 732-02a-note || return 0
  take 732-02-custom || return 0
  cat "$E2E_WORK/claude.log"
  expect "the template's first line" \
    bash -c "head -n 1 '$E2E_WORK/claude.log' | grep -q '^got: .*Scenario template\.$'"
  expect "the note inside it" holds "$E2E_WORK/claude.log" 'got: User comment: "Keep five as it is"'
  expect "its last line" holds "$E2E_WORK/claude.log" "got: End of notes."
  expect "none of the default's" test "$(grep -c 'Review notes on your work follow' "$E2E_WORK/claude.log")" = 0
}
