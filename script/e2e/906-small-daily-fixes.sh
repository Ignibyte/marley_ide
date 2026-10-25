# shellcheck shell=bash
# #906's visual check: four small daily fixes. A fresh profile opens `repo`, with #836's two
# Claude Code CLI sessions in `CLAUDE_CONFIG_DIR` for the Threads page, and #827's stand-in Claude
# Code (driven through a FIFO, Marley's hooks on a private notification bus) on the terminals'
# PATH, never the box's own (PR-687, F-claude-903).
# - `906-01-open-summary`: the stand-in waits on the user; the open `repo` header reads "1
#   waiting" in pink (REQ-004).
# - `906-02-threads-full`: the Threads page full width, a card selected: filters, list and
#   preview side by side (REQ-001).
# - `906-03-threads-half`: the page as narrow as half the center (the output narrowed, since the
#   page does not split): titles readable, the preview under the list or beside it (REQ-001).
# - `906-04-panel-tabs`: the status bar's Changes and Outline clicked: tabs titled "Changes" and
#   "Outline" (REQ-002).
# - `906-05-primary`: a changed buffer closed: the prompt card's Save in the coat with dark ink
#   (REQ-003).
# - `906-06-home-start`: Home's Start in the coat (REQ-003).
# - `906-07-day`: Marley Day: the Threads page narrow, the open header's summary (REQ-001,
#   REQ-004).
# - `906-08-day-primary`: Marley Day: the prompt card's Save in `text.accent` with light ink
#   (REQ-005).
compositor sway

HOOKS=$PWD/crates/marley_workbench/claude_plugin/marley/hooks
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the probe run's shots.
FIRST_CARD_X=${FIRST_CARD_X:-700}
FIRST_CARD_Y=${FIRST_CARD_Y:-200}
CHANGES_X=${CHANGES_X:-1290}
OUTLINE_X=${OUTLINE_X:-1355}
STATUS_Y=${STATUS_Y:-985}

CLAUDE_ONE=11111111-9060-4906-8906-111111111111
CLAUDE_TWO=22222222-9060-4906-8906-222222222222

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

# A Claude Code transcript, as #836's: `$1` the id, `$2` the folder, `$3` the prompt, `$4` the title.
claude_session() {
  local id=$1 folder=$2 prompt=$3 title=$4
  local dir=$E2E_WORK/claude/projects/${folder//\//-}
  mkdir -p "$dir"
  python3 - "$dir/$id.jsonl" "$id" "$folder" "$prompt" "$title" <<'PY'
import json, sys
path, ident, folder, prompt, title = sys.argv[1:]
base = {"sessionId": ident, "cwd": folder, "entrypoint": "cli", "isSidechain": False,
        "timestamp": "2026-10-11T10:00:00.000Z", "version": "2.1.296"}
records = [
    {**base, "type": "user", "message": {"role": "user", "content": prompt}},
    {**base, "type": "assistant", "message": {"role": "assistant",
                                               "content": [{"type": "text", "text": "On it."}]}},
    {"type": "ai-title", "sessionId": ident, "aiTitle": title},
]
with open(path, "w", encoding="utf-8") as file:
    for record in records:
        file.write(json.dumps(record) + "\n")
PY
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo/sub"
  # A terminal opened while the start mark is there runs the stand-in in place of the shell.
  sed -e "s|@BIN@|$bin|" -e "s|@WORK@|$E2E_WORK|" >"$home/.bashrc" <<'RC'
PS1='\$ '
export PATH="@BIN@:$PATH"
if [[ -e @WORK@/start-${PWD##*/} ]]; then
  rm -f "@WORK@/start-${PWD##*/}"
  exec claude "${PWD##*/}"
fi
RC
  terminal_env HOME "$home"
  dbus-daemon --session --fork --print-address=1 --print-pid=2 \
    >"$E2E_WORK/bus.address" 2>"$E2E_WORK/bus.pid"
  DBUS_SESSION_BUS_ADDRESS=$(head -1 "$E2E_WORK/bus.address")
  export DBUS_SESSION_BUS_ADDRESS
  write_stand_in "$bin/claude"
  mkfifo "$E2E_WORK/repo.fifo"
  git init -q -b main "$repo"
  printf 'fn main() {}\n' >"$repo/main.rs"
  export CLAUDE_CONFIG_DIR=$E2E_WORK/claude
  claude_session "$CLAUDE_ONE" "$repo" "fix the flaky parser test in tokenizer.rs" \
    "Fix the flaky tokenizer test"
  claude_session "$CLAUDE_TWO" "$repo/sub" "write the release notes" "Release notes for sub"
  touch -d '-1 minutes' "$CLAUDE_CONFIG_DIR/projects/"*"/$CLAUDE_ONE.jsonl"
  touch -d '-4 days' "$CLAUDE_CONFIG_DIR/projects/"*"/$CLAUDE_TWO.jsonl"
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

teardown() {
  [[ -f $E2E_WORK/bus.pid ]] && kill "$(head -1 "$E2E_WORK/bus.pid")" 2>/dev/null
  return 0
}

write_stand_in() {
  sed -e "s|@HOOKS@|$HOOKS|" -e "s|@WORK@|$E2E_WORK|" >"$1" <<'FAKE'
#!/usr/bin/env python3
# A stand-in Claude Code, as #827's: each line written to its FIFO names the step it acts out,
# the plugin's event.py run for every event and its answer's sequence written to the terminal.
import json
import os
import subprocess
import sys

name = sys.argv[1]
STEPS = {"waiting": [
    {"hook_event_name": "SessionStart", "source": "startup"},
    {"hook_event_name": "UserPromptSubmit", "prompt": "Write the README"},
    {"hook_event_name": "PreToolUse", "tool_name": "Write",
     "tool_input": {"file_path": "README.md", "content": "# repo"}, "tool_use_id": "t1"},
    {"hook_event_name": "PermissionRequest", "tool_name": "Write",
     "tool_input": {"file_path": "README.md", "content": "# repo"}},
]}
COMMON = {"session_id": f"e2e-{name}", "transcript_path": "/tmp/e2e-transcript.jsonl",
          "cwd": os.getcwd(), "permission_mode": "default"}

print(f"Claude Code (stand-in {name}): steps come through its FIFO", flush=True)
while True:
    with open(f"@WORK@/{name}.fifo", encoding="utf-8") as fifo:
        for line in fifo:
            step = line.strip()
            for event in STEPS.get(step, []):
                answer = subprocess.run([sys.executable, "@HOOKS@/event.py"],
                                        input=json.dumps({**COMMON, **event}),
                                        capture_output=True, text=True, check=False).stdout
                sequence = json.loads(answer or "{}").get("terminalSequence")
                if sequence:
                    sys.stdout.write(sequence)
                    sys.stdout.flush()
            print(f"{name}: {step}", flush=True)
FAKE
  chmod +x "$1"
}

step_of() {
  timeout 5 bash -c "echo $2 >'$E2E_WORK/$1.fifo'" ||
    echo "step_of: stand-in $1 is not reading" >&2
}

# A changed new buffer, closed: the prompt card asks to save it.
close_changed() {
  palette "workspace: new file" 2
  type_text "notes"
  settle 1
  palette "pane: close active item" 2
  away
}

steps() {
  settle 15
  # Trusts repo.
  press "" Return
  settle 3

  echo "== the stand-in waits on the user"
  touch "$E2E_WORK/start-repo"
  palette "workspace: new terminal" 4
  step_of repo waiting
  away
  settle 2
  shot 906-01-open-summary

  echo "== the Threads page, full width"
  palette "marley: open threads" 4
  click "$FIRST_CARD_X" "$FIRST_CARD_Y"
  away
  shot 906-02-threads-full
  [ -n "${STOP_AT_THREADS:-}" ] && return

  echo "== the Threads page as narrow as half the window"
  # The page does not split, so the output narrows to what half the center would leave it.
  sway_msg output HEADLESS-1 mode 930x1000 >/dev/null
  settle 3
  away
  shot 906-03-threads-half
  sway_msg output HEADLESS-1 mode "$SIZE" >/dev/null
  settle 3

  echo "== Changes and Outline as tabs"
  click "$CHANGES_X" "$STATUS_Y"
  settle 2
  click "$OUTLINE_X" "$STATUS_Y"
  settle 2
  away
  shot 906-04-panel-tabs

  echo "== a changed buffer closed"
  close_changed
  shot 906-05-primary
  press "" Escape
  settle 1

  echo "== Home's Start"
  launch_marley_with home
  settle 5
  away
  shot 906-06-home-start

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  palette "marley: open threads" 3
  sway_msg output HEADLESS-1 mode 930x1000 >/dev/null
  settle 3
  away
  shot 906-07-day
  sway_msg output HEADLESS-1 mode "$SIZE" >/dev/null
  settle 3
  close_changed
  shot 906-08-day-primary
  press "" Escape
}
