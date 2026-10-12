# shellcheck shell=bash
# #730's visual check: a tool's findings in the project diff. #522's set-up with a longer file: a
# scratch repository with `notes.txt` (twenty lines) committed and lines 10 and 12 changed, not
# staged, and in its terminal the stand-in Claude Code that sends Marley's plugin's SessionStart
# (so its seat is idle) and logs each line it reads. The stand-in MCP client (`mcp_agent`) calls
# `review_notes_add` with source `scenario-lint` and notes on lines 10, 12 and 1 (unchanged, outside
# the diff's context). Shots: the question (`730-01-asked`, REQ-001); after Allow, the diff with two
# proposals naming their source, with Keep and Drop, and no Send Review count (`730-02-proposed`,
# REQ-002 to REQ-004), the answer naming line 1; Keep on line 10's, Drop on line 12's
# (`730-03-kept`, REQ-005, REQ-006); the send, with the source in what the agent read
# (`730-04-sent`, REQ-007).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

HOOK=$PWD/crates/marley_workbench/claude_plugin/marley/hooks/event.py
SHIPPED=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' \
  crates/marley_workbench/claude_plugin/marley/.claude-plugin/plugin.json)
# The question's Allow for This Session, and the proposals' Keep and Drop, from the first run's
# shots; `STOP_AT=<shot>` ends the run after that shot.
ALLOW_X=${ALLOW_X:-1225}
ALLOW_Y=${ALLOW_Y:-933}
KEEP_X=${KEEP_X:-1525}
KEEP_Y=${KEEP_Y:-337}
DROP_X=${DROP_X:-1566}
DROP_Y=${DROP_Y:-529}
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
  export CLAUDE_CONFIG_DIR=$config
  cat >"$E2E_WORK/gitconfig" <<'CONFIG'
[user]
	name = Scenario
	email = scenario@example.invalid
CONFIG
  export GIT_CONFIG_GLOBAL=$E2E_WORK/gitconfig GIT_CONFIG_NOSYSTEM=1
  git init -q -b main "$repo"
  seq 1 20 | sed 's/^/line /' >"$repo/notes.txt"
  git -C "$repo" add -A
  git -C "$repo" -c core.hooksPath=/dev/null commit -q -m "Start"
  sed -i -e 's/^line 10$/LINE TEN/' -e 's/^line 12$/LINE TWELVE/' "$repo/notes.txt"
  write_fake_claude "$bin/claude"
  : >"$E2E_WORK/claude.log"
  open_path "$repo"
}

# #522's stand-in Claude Code: idle through Marley's plugin hook; each line it reads logged.
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

findings() {
  local file=$E2E_WORK/repo/notes.txt
  printf '{"source": "scenario-lint", "notes": [%s, %s, %s]}' \
    "{\"path\": \"$file\", \"first_line\": 10, \"comment\": \"Ten should stay in lower case\"}" \
    "{\"path\": \"$file\", \"first_line\": 12, \"comment\": \"Twelve may stay as it is\"}" \
    "{\"path\": \"$file\", \"first_line\": 1, \"comment\": \"One is not changed\"}"
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

  echo "== review_notes_add asks first"
  mcp_agent tool review_notes_add "$(findings)" >"$E2E_WORK/answer.txt" 2>&1 &
  asking=$!
  settle 4
  take 730-01-asked || return 0

  echo "== Allow: the proposals in the diff"
  click "$ALLOW_X" "$ALLOW_Y"
  wait "$asking"
  settle 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 730-02-proposed || return 0
  cat "$E2E_WORK/answer.txt"
  expect "two notes placed" python3 - "$E2E_WORK/answer.txt" <<'PY'
import json, sys
text = open(sys.argv[1], encoding="utf-8").read()
answer = json.loads(text.split("review_notes_add: ", 1)[1].splitlines()[0])
placed = sorted(note["first_line"] for note in answer["placed"])
refused = [(note["first_line"], note["reason"]) for note in answer["refused"]]
print("placed", placed, "refused", refused)
sys.exit(0 if placed == [10, 12] and [line for line, _ in refused] == [1] else 1)
PY
  if ((KEEP_X == 0)); then
    echo "Keep and Drop are not placed yet: 730-02 is the last shot"
    return 0
  fi

  echo "== Keep line 10's, Drop line 12's"
  click "$KEEP_X" "$KEEP_Y"
  settle 1
  click "$DROP_X" "$DROP_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  take 730-03-kept || return 0

  echo "== the kept note goes with its source"
  palette "editor: send review to agent"
  settle 2
  press "" Return
  settle 3
  take 730-04-sent || return 0
  cat "$E2E_WORK/claude.log"
  expect "the agent read line 10's note" holds "$E2E_WORK/claude.log" "got: Line: 10" 'got: User comment: "Ten should stay in lower case"'
  expect "with its source" holds "$E2E_WORK/claude.log" 'Source: "scenario-lint"'
  expect "and not line 12's" test "$(grep -c "Twelve may stay" "$E2E_WORK/claude.log")" = 0
}
