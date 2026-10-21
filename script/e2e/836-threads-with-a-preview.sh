# shellcheck shell=bash
# #836's visual check: the Threads page with its filters, day groups and preview. #754's fixtures:
# two Claude Code CLI sessions in `CLAUDE_CONFIG_DIR`, one in the repository changed a minute ago
# and one in `sub` changed four days ago; a stand-in `claude` that logs its launch.
# - `marley: open threads`: WHO and FOLDER at the left, the two cards under TODAY and THIS WEEK,
#   "Pick a conversation" at the right (`836-01-night`, REQ-001, REQ-002).
# - The first card clicked: selected and previewed with Open and Open in a Split
#   (`836-02-selected`, REQ-003).
# - The `sub` folder: only its card (`836-03-folder`, REQ-001).
# - All folders, the first card, Open: the stand-in resumes it (`836-04-opened`, REQ-004).
# - Marley Day, the page with a card selected (`836-05-day`, REQ-001 to REQ-003).
compositor sway

# Points from the probe run's shots.
FIRST_CARD_X=${FIRST_CARD_X:-700}
FIRST_CARD_Y=${FIRST_CARD_Y:-200}
SUB_FOLDER_X=${SUB_FOLDER_X:-292}
SUB_FOLDER_Y=${SUB_FOLDER_Y:-539}
ALL_FOLDERS_X=${ALL_FOLDERS_X:-288}
ALL_FOLDERS_Y=${ALL_FOLDERS_Y:-347}
OPEN_X=${OPEN_X:-1301}
OPEN_Y=${OPEN_Y:-265}
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-900}

CLAUDE_ONE=11111111-8360-4836-8836-111111111111
CLAUDE_TWO=22222222-8360-4836-8836-222222222222

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

# A Claude Code transcript: `$1` the id, `$2` the folder, `$3` the entrypoint, `$4` the prompt,
# then `ai-title <text>` or `custom-title <text>`.
claude_session() {
  local id=$1 folder=$2 entry=$3 prompt=$4 kind=$5 title=$6
  local dir=$E2E_WORK/claude/projects/${folder//\//-}
  mkdir -p "$dir"
  python3 - "$dir/$id.jsonl" "$id" "$folder" "$entry" "$prompt" "$kind" "$title" <<'PY'
import json, sys
path, ident, folder, entry, prompt, kind, title = sys.argv[1:]
base = {"sessionId": ident, "cwd": folder, "entrypoint": entry, "isSidechain": False,
        "timestamp": "2026-10-11T10:00:00.000Z", "version": "2.1.296"}
records = [
    {**base, "type": "user", "message": {"role": "user", "content": prompt}},
    {**base, "type": "assistant", "message": {"role": "assistant",
                                               "content": [{"type": "text", "text": "On it."}]}},
    {**base, "type": "user", "isMeta": True, "message": {"role": "user",
                                                          "content": "Caveat: a meta record"}},
]
if kind == "ai-title":
    records.append({"type": "ai-title", "sessionId": ident, "aiTitle": title})
else:
    records.append({"type": "custom-title", "sessionId": ident, "customTitle": title})
with open(path, "w", encoding="utf-8") as file:
    for record in records:
        file.write(json.dumps(record) + "\n")
PY
}

write_stand_ins() {
  local bin=$1
  cat >"$bin/claude" <<FAKE
#!/usr/bin/env python3
import base64, json, os, sys
with open("$E2E_WORK/launches.log", "a", encoding="utf-8") as log:
    log.write("claude: " + " ".join(sys.argv[1:]) + " @ " + os.getcwd() + "\n")
print("Claude Code (stand-in): " + " ".join(sys.argv[1:]), flush=True)
if "--resume" in sys.argv:
    session = sys.argv[sys.argv.index("--resume") + 1]
    body = {"v": 1, "cwd": os.getcwd(), "event": "SessionStart", "session_id": session,
            "source": "resume"}
    encoded = base64.b64encode(json.dumps(body).encode()).decode()
    sys.stdout.write("\x1b]777;notify;marley-event;" + encoded + "\x07")
    sys.stdout.flush()
for _ in sys.stdin:
    pass
FAKE
  cat >"$bin/codex" <<FAKE
#!/usr/bin/env python3
import os, sys
with open("$E2E_WORK/launches.log", "a", encoding="utf-8") as log:
    log.write("codex: " + " ".join(sys.argv[1:]) + " @ " + os.getcwd() + "\n")
print("Codex (stand-in): " + " ".join(sys.argv[1:]), flush=True)
for _ in sys.stdin:
    pass
FAKE
  chmod +x "$bin/claude" "$bin/codex"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo/sub" "$CODEX_HOME"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  git init -q -b main "$repo"
  write_stand_ins "$bin"
  export CLAUDE_CONFIG_DIR=$E2E_WORK/claude
  claude_session "$CLAUDE_ONE" "$repo" cli "fix the flaky parser test in tokenizer.rs" \
    ai-title "Fix the flaky tokenizer test"
  claude_session "$CLAUDE_TWO" "$repo/sub" cli "write the release notes" \
    custom-title "Release notes for sub"
  touch -d '-1 minutes' "$CLAUDE_CONFIG_DIR/projects/"*"/$CLAUDE_ONE.jsonl"
  touch -d '-4 days' "$CLAUDE_CONFIG_DIR/projects/"*"/$CLAUDE_TWO.jsonl"
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

steps() {
  settle 15
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== the Threads page"
  palette "marley: open threads" 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 836-01-night

  echo "== a card selected"
  click "$FIRST_CARD_X" "$FIRST_CARD_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 836-02-selected

  echo "== the sub folder"
  click "$SUB_FOLDER_X" "$SUB_FOLDER_Y"
  settle 2
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 836-03-folder

  echo "== Open"
  click "$ALL_FOLDERS_X" "$ALL_FOLDERS_Y"
  settle 1
  click "$FIRST_CARD_X" "$FIRST_CARD_Y"
  settle 1
  click "$OPEN_X" "$OPEN_Y"
  settle 5
  echo "launches: $(cat "$E2E_WORK/launches.log" 2>/dev/null)"
  shot 836-04-opened

  echo "== Marley Day"
  palette "marley: open threads" 3
  click "$FIRST_CARD_X" "$FIRST_CARD_Y"
  settle 1
  profile_setting theme '"Marley Day"'
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 4
  shot 836-05-day
}
